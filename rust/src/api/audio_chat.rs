use std::mem;
pub use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherCoreWrapper, StreamCipherSeek};
use aes::Aes256;
use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, Stream};
use ctr::{Ctr128BE, CtrCore};
use hex_literal::hex;
use hkdf::Hkdf;
use kanal::{bounded, bounded_async, AsyncReceiver, AsyncSender, Receiver, Sender};
use log::{debug, error};
use nnnoiseless::{DenoiseState, FRAME_SIZE};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use sha2::Sha256;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::select;
use tokio::sync::{Mutex, Notify};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

use flutter_rust_bridge::{frb, spawn, spawn_blocking_with};

use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::items::{Hello, InputConfig};
use crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER;

type Result<T> = std::result::Result<T, Error>;
type TransferBuffer = [u8; TRANSFER_BUFFER_SIZE];
type AesCipher = StreamCipherCoreWrapper<CtrCore<Aes256, ctr::flavors::Ctr128BE>>;

/// The number of audio bytes in a single UDP packet
const TRANSFER_BUFFER_SIZE: usize = FRAME_SIZE * 2;
/// The number of samples in a single packet
const CHUNK_SIZE: usize = TRANSFER_BUFFER_SIZE / mem::size_of::<f32>();
const FLOAT_SILENCE: [f32; FRAME_SIZE] = [0_f32; FRAME_SIZE];
const BYTE_SILENCE: TransferBuffer = [0; TRANSFER_BUFFER_SIZE];
const RESAMPLER_PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: WindowFunction::BlackmanHarris2,
};
const SALT: [u8; 32] = hex!("04acee810b938239a6d2a09c109af6e3eaedc961fc66b9b6935a441c2690e336");

#[frb(opaque)]
#[derive(Clone)]
pub struct AudioChat {
    /// The port to listen for incoming TCP connections
    listen_port: u16,

    /// The port to which the UDP socket binds to
    receive_port: u16,

    /// The contacts that this chat knows about
    contacts: Arc<Mutex<Vec<SocketAddr>>>,

    /// The audio host
    host: Arc<Host>,

    /// Controls the threshold for silence detection
    rms_threshold: Arc<AtomicF32>,

    /// The factor to adjust the input volume by
    input_volume: Arc<AtomicF32>,

    /// The factor to adjust the output volume by
    output_volume: Arc<AtomicF32>,

    /// Notifies the call to end
    end_call: Arc<Notify>,

    /// Manually set the input device
    input_device: Arc<Mutex<Option<String>>>,

    /// Manually set the output device
    output_device: Arc<Mutex<Option<String>>>,
}

impl AudioChat {
    // this function must be async to use `spawn`
    pub async fn new(listen_port: u16, receive_port: u16) -> AudioChat {
        let chat = Self {
            listen_port,
            receive_port,
            contacts: Default::default(),
            host: Arc::new(cpal::default_host()),
            rms_threshold: Arc::new(AtomicF32::new(0.002)),
            input_volume: Arc::new(AtomicF32::new(1.0)),
            output_volume: Arc::new(AtomicF32::new(1.0)),
            end_call: Default::default(),
            input_device: Default::default(),
            output_device: Default::default(),
        };

        let chat_clone = chat.clone();

        spawn(async move {
            chat_clone.listener().await.unwrap();
        });

        chat
    }

    /// The public say_hello function
    pub async fn say_hello(&self, address: String) -> std::result::Result<(), DartError> {
        self._say_hello(address).await.map_err(DartError::from)
    }

    /// The public add_contact function
    pub async fn add_contact(&self, contact: String) -> std::result::Result<(), DartError> {
        self._add_contact(contact).await.map_err(DartError::from)
    }

    /// Ends the call (if there is one)
    pub async fn end_call(&self) {
        self.end_call.notify_waiters();
    }

    /// Initiate a call to the given address
    async fn _say_hello(&self, address: String) -> Result<()> {
        let address: SocketAddr = address.parse()?;
        let mut stream = TcpStream::connect(address).await?;
        self.handshake(&mut stream, address.ip(), true).await
    }

    /// Adds an address to the list of known contacts
    async fn _add_contact(&self, contact: String) -> Result<()> {
        let contact: SocketAddr = contact.parse()?;
        let mut contacts = self.contacts.lock().await;
        contacts.push(contact);
        Ok(())
    }

    /// Lists the input and output devices
    pub fn list_devices(&self) -> (Vec<String>, Vec<String>) {
        let input_devices = self.host.input_devices().unwrap();
        let output_devices = self.host.output_devices().unwrap();

        let input_devices = input_devices
            .filter_map(|device| device.name().ok())
            .collect();

        let output_devices = output_devices
            .filter_map(|device| device.name().ok())
            .collect();

        (input_devices, output_devices)
    }

    async fn listener(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.listen_port)).await?;

        while let Ok((mut stream, address)) = listener.accept().await {
            let remote_address = address.ip();

            if !self
                .contacts
                .lock()
                .await
                .iter()
                .any(|contact| contact.ip() == remote_address)
            {
                error!("Received call from unknown contact: {}", address);
                continue;
            }

            if let Err(error) = self.handshake(&mut stream, remote_address, false).await {
                error!("Call failed: {:?}", error);
            }
        }

        Ok(())
    }

    /// Set up the cryptography and negotiate the audio ports
    async fn handshake(
        &self,
        stream: &mut TcpStream,
        remote_address: IpAddr,
        caller: bool,
    ) -> Result<()> {
        // perform the key exchange
        let shared_secret = key_exchange(stream).await?;
        // HKDF for the key derivation
        let hk = Hkdf::<Sha256>::new(Some(&SALT), shared_secret.as_bytes());

        // create the stream, send, and receive ciphers
        let mut stream_cipher = cipher_factory(&hk, b"stream-key", b"stream-iv")?;
        let mut send_cipher = cipher_factory(&hk, b"send-key", b"send-iv")?;
        let mut receive_cipher = cipher_factory(&hk, b"receive-key", b"receive-iv")?;

        // create the UDP socket
        let socket = Arc::new(UdpSocket::bind(("0.0.0.0", self.receive_port)).await?);

        // build the hello message
        let message = Hello::new(self.receive_port);

        let hello = if caller {
            // the caller sends the hello message first
            write_message(stream, &message, &mut stream_cipher).await?;
            // then receives the hello message from the callee
            read_message::<Hello, _>(stream, &mut stream_cipher).await?
        } else {
            // callee receives the hello message from the caller
            let hello = read_message::<Hello, _>(stream, &mut stream_cipher).await?;
            // then sends the hello message
            write_message(stream, &message, &mut stream_cipher).await?;
            hello
        };

        // connect to the remote address on the hello port
        socket.connect((remote_address, hello.port as u16)).await?;

        if caller {
            // the caller always has the ciphers reversed
            mem::swap(&mut send_cipher, &mut receive_cipher);
        }

        let result = self
            .call(
                Arc::clone(&socket),
                stream,
                stream_cipher,
                receive_cipher,
                send_cipher,
                caller,
            )
            .await;

        match result {
            Ok(()) => Ok(()),
            Err(error) => match error.kind {
                ErrorKind::NoInputDevice | ErrorKind::NoOutputDevice => {
                    socket.send(&[2]).await?;
                    Ok(())
                }
                _ => {
                    socket.send(&[3]).await?;
                    Err(error)
                }
            },
        }
    }

    /// The bulk of the call logic
    async fn call(
        &self,
        socket: Arc<UdpSocket>,
        stream: &mut TcpStream,
        mut stream_cipher: AesCipher,
        send_cipher: AesCipher,
        receive_cipher: AesCipher,
        caller: bool,
    ) -> Result<()> {
        let output_device = match *self.output_device.lock().await {
            Some(ref name) => self
                .host
                .output_devices()?
                .find(|device| {
                    if let Ok(ref device_name) = device.name() {
                        name == device_name
                    } else {
                        false
                    }
                })
                .ok_or(Error::no_output_device())?,
            None => self
                .host
                .default_output_device()
                .ok_or(Error::no_output_device())?,
        };

        debug!("output_device: {:?}", output_device.name());

        let input_device = match *self.input_device.lock().await {
            Some(ref name) => self
                .host
                .input_devices()?
                .find(|device| {
                    if let Ok(ref device_name) = device.name() {
                        name == device_name
                    } else {
                        false
                    }
                })
                .ok_or(Error::no_input_device())?,
            None => self
                .host
                .default_input_device()
                .ok_or(Error::no_input_device())?,
        };

        debug!("input_device: {:?}", input_device.name());

        let output_config = output_device.default_output_config()?;
        let input_config = input_device.default_input_config()?;
        let message = InputConfig::from(&input_config);

        let remote_input_config = if caller {
            // caller sends the input config first
            write_message(stream, &message, &mut stream_cipher).await?;
            // then receive the input config from the callee
            read_message::<InputConfig, _>(stream, &mut stream_cipher).await?
        } else {
            // callee receives the input config from the caller
            let remote_input_config =
                read_message::<InputConfig, _>(stream, &mut stream_cipher).await?;
            // then sends the input config
            write_message(stream, &message, &mut stream_cipher).await?;
            remote_input_config
        };

        // sends denoised data to the output socket
        let (input_sender, input_receiver) = bounded_async::<[f32; FRAME_SIZE]>(1_000);
        // sends raw data from the socket to the processor
        let (output_sender, output_receiver) = bounded_async::<TransferBuffer>(1_000);
        // sends the resampled data to the output stream
        let (processed_sender, processed_receiver) = bounded::<f32>(1_000);

        // the ratio of the output sample rate to the input sample rate
        let ratio = output_config.sample_rate().0 as f64 / remote_input_config.sample_rate as f64;
        // get a reference to output volume for the processor
        let output_volume = Arc::clone(&self.output_volume);

        // spawn the processor thread
        let processor_handle = std::thread::spawn(move || {
            processor(
                output_receiver.to_sync(),
                processed_sender,
                ratio,
                output_volume,
            )
        });

        // sends single samples to the socket
        let input_channels = input_config.channels() as usize;
        // get a reference to the threshold for input stream
        let rms_threshold = Arc::clone(&self.rms_threshold);
        // get a reference to the input factor for the input stream
        let input_volume = Arc::clone(&self.input_volume);
        // create a sync sender for the input stream
        let sync_input_sender = input_sender.to_sync();
        // create a denoise state
        let mut denoise_state = DenoiseState::new();
        // create a buffer to store samples left over from the previous frame
        let mut remaining_samples = Vec::new();
        // create a buffer to store the output of the denoising process
        let mut out_buf = [0_f32; FRAME_SIZE];
        // short silences are ignored to prevent popping
        let mut silence_length = 0;

        let input_stream = SendStream {
            stream: input_device.build_input_stream(
                &input_config.into(),
                move |input, _: &_| {
                    // add the samples for the first channel for this frame to the remaining samples
                    for frame in input.chunks(input_channels) {
                        remaining_samples.push(frame[0]);
                    }

                    // process all the samples and store the remaining samples for the next frame
                    remaining_samples = process_input_data(
                        &remaining_samples,
                        &sync_input_sender,
                        &mut denoise_state,
                        &mut out_buf,
                        &mut silence_length,
                        rms_threshold.load(Relaxed), // load the threshold for each frame
                        input_volume.load(Relaxed),  // load the input factor for each frame
                    );
                },
                move |err| {
                    error!("Error in input stream: {}", err);
                },
                None,
            )?,
        };

        // get the output channels for chunking the output
        let output_channels = output_config.channels() as usize;

        let output_stream = SendStream {
            stream: output_device.build_output_stream(
                &output_config.into(),
                move |output: &mut [f32], _: &_| {
                    for frame in output.chunks_mut(output_channels) {
                        // get the next sample from the processor
                        let sample = match processed_receiver.try_recv() {
                            Ok(Some(sample)) => sample,
                            Ok(None) | Err(_) => 0_f32, // if there is no sample, use silence
                        };

                        // write the sample to all the channels
                        for channel in frame.iter_mut() {
                            *channel = sample;
                        }
                    }
                },
                move |err| {
                    error!("Error in output stream: {}", err);
                },
                None,
            )?,
        };

        input_stream.stream.play()?;
        output_stream.stream.play()?;

        let input_handle = spawn(input_to_socket(
            input_receiver,
            Arc::clone(&socket),
            send_cipher,
        ));

        let output_handle = spawn(socket_output(
            output_sender,
            Arc::clone(&socket),
            receive_cipher,
        ));

        let processor_future = spawn_blocking_with(move || processor_handle.join(), FLUTTER_RUST_BRIDGE_HANDLER.thread_pool());

        select! {
            result = input_handle => result??,
            reason = output_handle => match reason? {
                // TODO UI prompts
                EndReason::Ended => (),
                EndReason::MissingDevice => error!("The other party is missing a device"),
                EndReason::Error => error!("The other party encountered an error"),
            },
            // this unwrap is safe because the processor thread will not panic
            result = processor_future => result?.unwrap()?,
            _ = self.end_call.notified() => (),
        }

        // send the end signal
        socket.send(&[1]).await?;

        Ok(())
    }
}

// TODO the whole SendStream shenanigans seems like a bad idea
struct SendStream {
    stream: Stream,
}

unsafe impl Send for SendStream {}

enum EndReason {
    Ended,
    MissingDevice,
    Error,
}

/// Writes a protobuf message to the stream
async fn write_message<M: prost::Message, C: StreamCipher>(
    stream: &mut TcpStream,
    message: &M,
    cipher: &mut C,
) -> Result<()> {
    let len = message.encoded_len(); // get the length of the message
    stream.write_u32(len as u32).await?; // write the length of the message

    let mut buffer = Vec::with_capacity(len); // create a buffer to write the message into
    message.encode(&mut buffer).unwrap(); // encode the message into the buffer (infallible)
    cipher.apply_keystream(&mut buffer); // apply the keystream to the buffer

    stream.write_all(&buffer).await?; // write the message to the writer

    Ok(())
}

/// Reads a protobuf message from the stream
async fn read_message<M: prost::Message + Default, C: StreamCipher>(
    stream: &mut TcpStream,
    cipher: &mut C,
) -> Result<M> {
    let len = stream.read_u32().await? as usize; // read the length of the message

    let mut buffer = vec![0; len]; // create a buffer to read the message into
    stream.read_exact(&mut buffer).await?; // read the message into the buffer
    cipher.apply_keystream(&mut buffer); // apply the keystream to the buffer

    let message = M::decode(&buffer[..])?; // decode the message

    Ok(message)
}

/// Processes the input data and sends it to the socket
fn process_input_data(
    input: &[f32],
    sender: &Sender<[f32; FRAME_SIZE]>,
    denoise_state: &mut DenoiseState,
    out_buffer: &mut [f32; FRAME_SIZE],
    silence_length: &mut usize,
    rms_threshold: f32,
    input_factor: f32,
) -> Vec<f32> {
    for frame in input.chunks(FRAME_SIZE) {
        if frame.len() == FRAME_SIZE {
            denoise_state.process_frame(out_buffer, frame);

            if calculate_rms(out_buffer) < rms_threshold {
                if *silence_length < 40 {
                    *silence_length += 1;
                } else {
                    _ = sender.try_send(FLOAT_SILENCE);
                }
            } else {
                *silence_length = 0;
                // this unwrap is safe because we know the length of the frame is FRAME_SIZE
                mul(out_buffer, input_factor);
                _ = sender.send(out_buffer.as_slice().try_into().unwrap());
            }
        } else {
            return frame.to_vec();
        }
    }

    Vec::new()
}

/// Receives frames of audio data from the input receiver and sends them to the socket
async fn input_to_socket<C: StreamCipher>(
    input_receiver: AsyncReceiver<[f32; FRAME_SIZE]>,
    socket: Arc<UdpSocket>,
    mut cipher: C,
) -> Result<()> {
    let mut buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut sequence_number = 0_u64;

    while let Ok(frame) = input_receiver.recv().await {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                frame.as_ptr() as *const u8,
                frame.len() * mem::size_of::<f32>(),
            )
        };

        for chunk in bytes.chunks(TRANSFER_BUFFER_SIZE) {
            if chunk == BYTE_SILENCE {
                socket.send(&[0]).await?;
            } else {
                cipher.apply_keystream_b2b(chunk, &mut buffer[8..]).unwrap();

                buffer[..8].copy_from_slice(&sequence_number.to_be_bytes());
                sequence_number += TRANSFER_BUFFER_SIZE as u64; // advance the sequence number

                socket.send(&buffer).await?;
            }
        }
    }

    Ok(())
}

/// Receives audio data from the socket and sends it to the output processor
async fn socket_output<C: StreamCipher + StreamCipherSeek>(
    output_sender: AsyncSender<TransferBuffer>,
    socket: Arc<UdpSocket>,
    mut cipher: C,
) -> EndReason {
    let mut buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut out = [0; TRANSFER_BUFFER_SIZE];

    loop {
        match socket.recv(&mut buffer).await {
            // normal chunk of audio data
            Ok(len) if len == TRANSFER_BUFFER_SIZE + 8 => {
                // unwrap is safe because we know the buffer is the correct length
                let sequence_number = u64::from_be_bytes(buffer[..8].try_into().unwrap());

                // seek the cipher if needed
                if cipher.current_pos::<u64>() != sequence_number {
                    cipher.seek(sequence_number);
                }

                // unwrap is safe because we know the buffer is the correct length
                cipher.apply_keystream_b2b(&buffer[8..], &mut out).unwrap();
                _ = output_sender.try_send(out);
            }
            // control signals
            Ok(1) => match buffer[0] {
                0 => _ = output_sender.try_send(BYTE_SILENCE), // silence
                1 => break EndReason::Ended,                   // end of call
                2 => break EndReason::MissingDevice, // the other party is missing a device
                3 => break EndReason::Error,         // the other party encountered an error
                _ => unreachable!("received unknown control signal {}", buffer[0]),
            },
            Ok(len) => error!("Received {} < {} data", len, TRANSFER_BUFFER_SIZE + 8),
            Err(error) => error!("Error receiving {}", error),
        }
    }
}

/// Processes the audio data and sends it to the output stream
fn processor(
    receiver: Receiver<TransferBuffer>,
    sender: Sender<f32>,
    ratio: f64,
    output_volume: Arc<AtomicF32>,
) -> Result<()> {
    let mut resampler = if ratio == 1_f64 {
        debug!("No resampling needed");
        None
    } else {
        debug!("Resampling at {:.2}%", ratio * 100_f64);

        // create the resampler if needed
        Some(SincFixedIn::<f32>::new(
            ratio,
            2.0,
            RESAMPLER_PARAMETERS,
            CHUNK_SIZE,
            1,
        )?)
    };

    // rubato requires 10 extra bytes in the output buffer as a safety margin
    let post_len = (CHUNK_SIZE as f64 * ratio + 10.0) as usize;

    // allocate the buffers for the resampler
    let mut post_buf = [vec![0_f32; post_len]];
    let mut pre_buf = [&[0_f32; CHUNK_SIZE]];

    while let Ok(bytes) = receiver.recv() {
        // convert the bytes to floats
        let floats = unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr() as *const f32,
                bytes.len() / mem::size_of::<f32>(),
            )
        };

        let factor = output_volume.load(Relaxed);

        if let Some(resampler) = &mut resampler {
            pre_buf[0] = floats.try_into().unwrap(); // infallible

            // resample the data
            let processed = resampler.process_into_buffer(&pre_buf, &mut post_buf, None)?;

            mul(&mut post_buf[0][..processed.1], factor);

            // send the resampled data to the output stream
            for sample in &post_buf[0][..processed.1] {
                _ = sender.try_send(*sample);
            }
        } else {
            // send the raw data to the output stream
            for sample in floats {
                _ = sender.send(sample * factor);
            }
        }
    }

    Ok(())
}

/// Performs the key exchange
async fn key_exchange(stream: &mut TcpStream) -> Result<SharedSecret> {
    let secret = EphemeralSecret::random();
    let our_public = PublicKey::from(&secret);

    stream.write_all(our_public.as_bytes()).await?;

    let mut buffer = [0; 32];
    stream.read_exact(&mut buffer).await?;
    let their_public = PublicKey::from(buffer);

    Ok(secret.diffie_hellman(&their_public))
}

/// Creates a cipher from the HKDF
fn cipher_factory(hk: &Hkdf<Sha256>, key_info: &[u8], iv_info: &[u8]) -> Result<AesCipher> {
    let mut key = [0_u8; 32];
    hk.expand(key_info, &mut key)?;

    let mut iv = [0_u8; 16];
    hk.expand(iv_info, &mut iv)?;

    Ok(Ctr128BE::<Aes256>::new(
        key.as_slice().into(),
        iv.as_slice().into(),
    ))
}

/// Calculates the RMS of the data
fn calculate_rms(data: &[f32]) -> f32 {
    let sum_of_squares: f32 = data.iter().map(|&x| x * x).sum();
    let mean_of_squares = sum_of_squares / data.len() as f32;
    mean_of_squares.sqrt()
}

/// Multiplies each element in the slice by the factor
fn mul(vec: &mut [f32], factor: f32) {
    vec.par_iter_mut().for_each(|p| *p *= factor);
}