use std::mem;
pub use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherCoreWrapper, StreamCipherSeek};
use aes::Aes256;
use atomic_float::AtomicF32;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, Stream};
use ctr::{Ctr128BE, CtrCore};
use ed25519_dalek::{SecretKey, Signature, Signer, SigningKey, VerifyingKey};
use flutter_rust_bridge::{frb, spawn, spawn_blocking_with, DartFnFuture};
use hex_literal::hex;
use hkdf::Hkdf;
use itertools::Itertools;
use kanal::{bounded, bounded_async, AsyncReceiver, AsyncSender, Receiver, Sender};
use log::{debug, error};
use nnnoiseless::{DenoiseState, FRAME_SIZE};
use rand::rngs::OsRng;
use rand::Rng;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU16};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::select;
use tokio::sync::{Mutex, Notify};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::items::{Hello, Identity, InputConfig};
use crate::api::logger;
use crate::frb_generated::{StreamSink, FLUTTER_RUST_BRIDGE_HANDLER};

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
    listen_port: Arc<AtomicU16>,

    /// The port to which the UDP socket binds to
    receive_port: Arc<AtomicU16>,

    /// The contacts that this chat knows about
    contacts: Arc<Mutex<HashMap<String, Contact>>>,

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

    /// Notifies the listener to stop
    stop_listener: Arc<Notify>,

    /// Manually set the input device
    input_device: Arc<Mutex<Option<String>>>,

    /// Manually set the output device
    output_device: Arc<Mutex<Option<String>>>,

    secret_key: SecretKey,

    in_call: Arc<AtomicBool>,

    /// Prompts the user to accept or reject the call
    accept_call: Arc<Mutex<dyn Fn(Contact) -> DartFnFuture<bool> + Send>>,
}

impl AudioChat {
    // this function must be async to use `spawn`
    pub async fn new(
        listen_port: u16,
        receive_port: u16,
        signing_key: Vec<u8>,
        accept_call: impl Fn(Contact) -> DartFnFuture<bool> + Send + 'static,
    ) -> AudioChat {
        let chat = Self {
            listen_port: Arc::new(AtomicU16::new(listen_port)),
            receive_port: Arc::new(AtomicU16::new(receive_port)),
            contacts: Default::default(),
            host: Arc::new(cpal::default_host()),
            rms_threshold: Arc::new(AtomicF32::new(0.002)),
            input_volume: Arc::new(AtomicF32::new(1.0)),
            output_volume: Arc::new(AtomicF32::new(1.0)),
            end_call: Default::default(),
            stop_listener: Default::default(),
            input_device: Default::default(),
            output_device: Default::default(),
            secret_key: SecretKey::from(
                <Vec<u8> as TryInto<[u8; 32]>>::try_into(signing_key).unwrap(),
            ),
            in_call: Default::default(),
            accept_call: Arc::new(Mutex::new(accept_call)),
        };

        let chat_clone = chat.clone();

        spawn(async move {
            if let Err(error) = chat_clone.listener().await {
                error!("Listener failed: {:?}", error);
            }
        });

        chat
    }

    /// The public say_hello function
    pub async fn say_hello(&self, contact: &Contact) -> std::result::Result<(), DartError> {
        debug!("say hello called for {}", contact.nickname);
        self._say_hello(contact).await.map_err(DartError::from)
    }

    /// Adds add a contact to the known contacts
    pub async fn add_contact(&self, contact: &Contact) {
        let mut contacts = self.contacts.lock().await;
        contacts.insert(contact.nickname.clone(), contact.clone());
    }

    /// Ends the call (if there is one)
    pub async fn end_call(&self) {
        self.end_call.notify_waiters();
    }

    /// Restarts the listener
    pub async fn restart_listener(&self) -> std::result::Result<(), DartError> {
        if self.in_call.load(Relaxed) {
            return Err(Error::in_call().into());
        }

        self.stop_listener.notify_waiters();

        let chat_clone = self.clone();
        spawn(async move {
            if let Err(error) = chat_clone.listener().await {
                error!("Listener failed: {:?}", error);
            }
        });
        Ok(())
    }

    #[frb(sync)]
    pub fn set_listen_port(&self, port: u16) {
        self.listen_port.store(port, Relaxed);
    }

    #[frb(sync)]
    pub fn set_receive_port(&self, port: u16) {
        self.receive_port.store(port, Relaxed);
    }

    #[frb(sync)]
    pub fn set_rms_threshold(&self, decimal: f32) {
        let threshold = db_to_multiplier(decimal);
        self.rms_threshold.store(threshold, Relaxed);
    }

    #[frb(sync)]
    pub fn set_input_volume(&self, decibel: f32) {
        let multiplier = db_to_multiplier(decibel);
        self.input_volume.store(multiplier, Relaxed);
    }

    #[frb(sync)]
    pub fn set_output_volume(&self, decibel: f32) {
        let multiplier = db_to_multiplier(decibel);
        self.output_volume.store(multiplier, Relaxed);
    }

    /// Lists the input and output devices
    #[frb(sync)]
    pub fn list_devices(&self) -> std::result::Result<(Vec<String>, Vec<String>), DartError> {
        let input_devices = self.host.input_devices().map_err(Error::from)?;
        let output_devices = self.host.output_devices().map_err(Error::from)?;

        let input_devices = input_devices
            .filter_map(|device| device.name().ok())
            .collect();

        let output_devices = output_devices
            .filter_map(|device| device.name().ok())
            .collect();

        Ok((input_devices, output_devices))
    }

    /// Initiate a call to the given address
    async fn _say_hello(&self, contact: &Contact) -> Result<()> {
        let mut stream = TcpStream::connect(contact.address).await?;
        // this signal lets the callee know that the caller wants to start a handshake
        stream.write(&[255]).await?;

        let chat_clone = self.clone();
        let contact = contact.clone();
        self.in_call.store(true, Relaxed);

        spawn(async move {
            if let Err(error) = chat_clone
                .handshake(stream, contact.address.ip(), true, contact)
                .await
            {
                error!("Call failed [caller]: {:?}", error);
            }
            chat_clone.in_call.store(false, Relaxed);
        });

        Ok(())
    }

    async fn listener(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.listen_port.load(Relaxed))).await?;

        let listener_loop = async {
            while let Ok((mut stream, address)) = listener.accept().await {
                // TODO this signaling thing is maybe not super ideal
                let mut buf = [0; 1];

                if stream.read(&mut buf).await.is_err() {
                    debug!("Stream closed before handshake");
                    continue;
                } else if buf != [255] {
                    debug!("Unknown signal: {:?}", buf);
                    continue;
                }

                let remote_address = address.ip();

                let contact = {
                    self.contacts
                        .lock()
                        .await
                        .iter()
                        .find(|(_, contact)| contact.ip() == remote_address)
                        .map(|(_, contact)| contact.clone())
                };

                if let Some(contact) = contact {
                    debug!("Prompting for call from {}", contact.nickname);
                    if !(self.accept_call.lock().await)(contact.clone()).await {
                        continue;
                    }

                    self.in_call.store(true, Relaxed);
                    if let Err(error) = self.handshake(stream, remote_address, false, contact).await
                    {
                        error!("Call failed [callee]: {:?}", error);
                    }
                    self.in_call.store(false, Relaxed);
                } else {
                    error!("Unknown contact: {}", remote_address);
                }
            }
        };

        select! {
            result = listener_loop => Ok(result),
            _ = self.stop_listener.notified() => {
                debug!("Listener stopped");
                Ok(())
            }
        }
    }

    /// Set up the cryptography and negotiate the audio ports
    async fn handshake(
        &self,
        mut stream: TcpStream,
        remote_address: IpAddr,
        caller: bool,
        contact: Contact,
    ) -> Result<()> {
        // perform the key exchange
        let shared_secret = key_exchange(&mut stream).await?;
        // HKDF for the key derivation
        let hk = Hkdf::<Sha256>::new(Some(&SALT), shared_secret.as_bytes());

        // create the stream, send, and receive ciphers
        let mut stream_cipher = cipher_factory(&hk, b"stream-key", b"stream-iv")?;
        let mut send_cipher = cipher_factory(&hk, b"send-key", b"send-iv")?;
        let mut receive_cipher = cipher_factory(&hk, b"receive-key", b"receive-iv")?;

        // create a random nonce
        let mut nonce = [0; 1024];
        OsRng.fill(&mut nonce);

        // sign the nonce
        let signing_key = SigningKey::from(self.secret_key);
        let signature = signing_key.sign(&nonce);

        // create the identity message
        let message = Identity::new(nonce, signature);

        let identity = exchange_messages(&mut stream, &message, &mut stream_cipher, caller).await?;

        // verify the signature
        let verifying_key = VerifyingKey::from_bytes(&contact.verifying_key)?;
        let signature = Signature::from_slice(&identity.signature)?;
        verifying_key.verify_strict(&identity.nonce, &signature)?;

        let receive_port = self.receive_port.load(Relaxed);

        // create the UDP socket
        let socket = Arc::new(UdpSocket::bind(("0.0.0.0", receive_port)).await?);

        // build the hello message
        let message = Hello::new(receive_port);

        let hello = exchange_messages(&mut stream, &message, &mut stream_cipher, caller).await?;

        // connect to the remote address on the hello port
        socket.connect((remote_address, hello.port as u16)).await?;

        if caller {
            // the caller always has the ciphers reversed
            mem::swap(&mut send_cipher, &mut receive_cipher);
        }

        let result = self
            .call(
                Arc::clone(&socket),
                &mut stream,
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

        let remote_input_config =
            exchange_messages(stream, &message, &mut stream_cipher, caller).await?;

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

        let processor_future = spawn_blocking_with(
            move || processor_handle.join(),
            FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
        );

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

#[derive(Clone)]
#[frb(opaque)]
pub struct Contact {
    nickname: String,
    verifying_key: [u8; 32],
    address: SocketAddr,
}

impl FromStr for Contact {
    type Err = DartError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: (&str, &str, &str) = s
            .splitn(3, ',')
            .collect_tuple()
            .ok_or(Error::invalid_contact_format())?;

        let nickname = parts.0.to_string();
        let verifying_key = BASE64_STANDARD
            .decode(parts.1.as_bytes())
            .map_err(Error::from)?;
        let address = parts.2.parse().map_err(Error::from)?;

        Ok(Self {
            nickname,
            verifying_key: verifying_key
                .try_into()
                .map_err(|_| Error::invalid_contact_format())?,
            address,
        })
    }
}

impl Contact {
    #[frb(sync)]
    pub fn new(
        nickname: String,
        verifying_key: String,
        address: String,
    ) -> std::result::Result<Contact, DartError> {
        let key = BASE64_STANDARD
            .decode(verifying_key.as_bytes())
            .map_err(Error::from)?;

        Ok(Self {
            nickname,
            verifying_key: key
                .try_into()
                .map_err(|_| Error::invalid_contact_format())?,
            address: address.parse().map_err(Error::from)?,
        })
    }

    #[frb(sync)]
    pub fn parse(s: String) -> std::result::Result<Contact, DartError> {
        Self::from_str(&s)
    }

    #[frb(sync)]
    pub fn verifying_key(&self) -> Vec<u8> {
        self.verifying_key.to_vec()
    }

    #[frb(sync)]
    pub fn verifying_key_str(&self) -> String {
        BASE64_STANDARD.encode(&self.verifying_key)
    }

    #[frb(sync)]
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }

    #[frb(sync)]
    pub fn address_str(&self) -> String {
        self.address.to_string()
    }

    #[frb(sync)]
    pub fn store(&self) -> String {
        format!(
            "{},{},{}",
            self.nickname,
            self.verifying_key_str(),
            self.address
        )
    }

    fn ip(&self) -> IpAddr {
        self.address.ip()
    }
}

#[frb(sync)]
pub fn create_log_stream(s: StreamSink<String>) {
    logger::SendToDartLogger::set_stream_sink(s);
    error!("Logger set");
}

#[frb(sync)]
pub fn rust_set_up() {
    logger::init_logger();
}

#[frb(sync)]
pub fn generate_keys() -> [u8; 64] {
    let signing_key: SigningKey = SigningKey::generate(&mut OsRng);
    signing_key.to_keypair_bytes()
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

/// A common message exchange pattern
async fn exchange_messages<M: prost::Message + Default, C: StreamCipher>(
    stream: &mut TcpStream,
    message: &M,
    cipher: &mut C,
    caller: bool,
) -> Result<M> {
    if caller {
        // the caller sends the message first
        write_message(stream, message, cipher).await?;
        // then receives the message from the callee
        read_message(stream, cipher).await
    } else {
        // callee receives the message from the caller
        let remote_identity = read_message(stream, cipher).await;
        // then sends the message
        write_message(stream, message, cipher).await?;
        remote_identity
    }
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
                if *silence_length < 80 {
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

/// Converts a decibel value to a multiplier
fn db_to_multiplier(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}
