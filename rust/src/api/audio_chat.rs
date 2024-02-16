use std::collections::VecDeque;
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
use cpal::{Device, Host, Stream};
use ctr::{Ctr128BE, CtrCore};
use ed25519_dalek::{SecretKey, Signature, Signer, SigningKey, VerifyingKey};
use flutter_rust_bridge::{frb, spawn, spawn_blocking_with, DartFnFuture};
use hex_literal::hex;
use hkdf::Hkdf;
use itertools::Itertools;
use kanal::{bounded, bounded_async, AsyncReceiver, AsyncSender, Receiver, Sender};
use log::{debug, error, warn};
use nnnoiseless::{DenoiseState, FRAME_SIZE};
use rand::rngs::OsRng;
use rand::Rng;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use sha2::Sha256;
use std::sync::atomic::{AtomicBool, AtomicU16};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::select;
use tokio::sync::{Mutex, Notify};
use tokio::time::timeout;
use uuid::Uuid;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::items::{Hello, Identity};
use crate::api::logger;
use crate::frb_generated::{StreamSink, FLUTTER_RUST_BRIDGE_HANDLER};

type Result<T> = std::result::Result<T, Error>;
pub(crate) type DeviceName = Arc<Mutex<Option<String>>>;
type TransferBuffer = [u8; TRANSFER_BUFFER_SIZE];
type AesCipher = StreamCipherCoreWrapper<CtrCore<Aes256, ctr::flavors::Ctr128BE>>;

/// The number of bytes in a single UDP packet
const TRANSFER_BUFFER_SIZE: usize = FRAME_SIZE * mem::size_of::<i16>();
const RESAMPLER_PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: WindowFunction::BlackmanHarris2,
};
const RECEIVE_TIMEOUT: Duration = Duration::from_secs(1);
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
const SALT: [u8; 32] = hex!("04acee810b938239a6d2a09c109af6e3eaedc961fc66b9b6935a441c2690e336");

#[frb(opaque)]
#[derive(Clone)]
pub struct AudioChat {
    /// The port to listen for incoming TCP connections
    listen_port: Arc<AtomicU16>,

    /// The port to which the UDP socket binds to
    receive_port: Arc<AtomicU16>,

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
    input_device: DeviceName,

    /// Manually set the output device
    output_device: DeviceName,

    /// Private key for signing the handshake
    secret_key: SecretKey,

    /// Keeps track of whether the user is in a call
    in_call: Arc<AtomicBool>,

    /// Disables the output stream
    deafened: Arc<AtomicBool>,

    /// Disables the input stream
    muted: Arc<AtomicBool>,

    /// Prompts the user to accept or reject the call
    accept_call: Arc<Mutex<dyn Fn(Contact) -> DartFnFuture<bool> + Send>>,

    /// Alerts the UI that a call has ended
    call_ended: Arc<Mutex<dyn Fn(String) -> DartFnFuture<()> + Send>>,

    /// Fetches a contact from the front end
    get_contact: Arc<Mutex<dyn Fn(String) -> DartFnFuture<Option<Contact>> + Send>>,

    /// Alerts the UI that the call has connected
    connected: Arc<Mutex<dyn Fn() -> DartFnFuture<()> + Send>>,
}

impl AudioChat {
    // this function must be async to use `spawn`
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        listen_port: u16,
        receive_port: u16,
        signing_key: Vec<u8>,
        rms_threshold: f32,
        input_volume: f32,
        output_volume: f32,
        accept_call: impl Fn(Contact) -> DartFnFuture<bool> + Send + 'static,
        call_ended: impl Fn(String) -> DartFnFuture<()> + Send + 'static,
        get_contact: impl Fn(String) -> DartFnFuture<Option<Contact>> + Send + 'static,
        connected: impl Fn() -> DartFnFuture<()> + Send + 'static,
    ) -> AudioChat {
        let chat = Self {
            listen_port: Arc::new(AtomicU16::new(listen_port)),
            receive_port: Arc::new(AtomicU16::new(receive_port)),
            host: Arc::new(cpal::default_host()),
            rms_threshold: Arc::new(AtomicF32::new(db_to_multiplier(rms_threshold))),
            input_volume: Arc::new(AtomicF32::new(db_to_multiplier(input_volume))),
            output_volume: Arc::new(AtomicF32::new(db_to_multiplier(output_volume))),
            end_call: Default::default(),
            stop_listener: Default::default(),
            input_device: Default::default(),
            output_device: Default::default(),
            secret_key: SecretKey::from(
                <Vec<u8> as TryInto<[u8; 32]>>::try_into(signing_key).unwrap(),
            ),
            in_call: Default::default(),
            deafened: Default::default(),
            muted: Default::default(),
            accept_call: Arc::new(Mutex::new(accept_call)),
            call_ended: Arc::new(Mutex::new(call_ended)),
            get_contact: Arc::new(Mutex::new(get_contact)),
            connected: Arc::new(Mutex::new(connected)),
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
    pub async fn say_hello(&self, contact: &Contact) -> std::result::Result<bool, DartError> {
        debug!("say hello called for {}", contact.nickname);
        self._say_hello(contact).await.map_err(DartError::from)
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

    #[frb(sync)]
    pub fn set_deafened(&self, deafened: bool) {
        self.deafened.store(deafened, Relaxed);
    }

    #[frb(sync)]
    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Relaxed);
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

    /// Initiate a call with the contact
    async fn _say_hello(&self, contact: &Contact) -> Result<bool> {
        let mut buf = [255; 1];
        let mut stream = timeout(HELLO_TIMEOUT, TcpStream::connect(contact.address)).await??;
        // this signal lets the callee know that the caller wants to start a handshake
        stream.write(&buf).await?;
        // this signal lets the caller know that the callee wants to start a handshake
        stream.read(&mut buf).await?;

        if buf != [0] {
            return Ok(false);
        }

        let chat_clone = self.clone();
        let contact = contact.clone();

        spawn(async move {
            if let Err(error) = chat_clone
                .handshake(stream, contact.address.ip(), true, contact)
                .await
            {
                error!("Call failed [caller]: {:?}", error);
            }
            chat_clone.in_call.store(false, Relaxed);
        });

        Ok(true)
    }

    async fn listener(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.listen_port.load(Relaxed))).await?;

        let listener_loop = async {
            while let Ok((mut stream, address)) = listener.accept().await {
                let mut buf = [0; 1];

                if stream.read(&mut buf).await.is_err() {
                    error!("Stream closed before handshake");
                    continue;
                } else if buf != [255] {
                    continue;
                }

                let remote_address = address.ip();
                let contact = (self.get_contact.lock().await)(remote_address.to_string()).await;

                if let Some(contact) = contact {
                    debug!("Prompting for call from {}", contact.nickname);

                    if !(self.accept_call.lock().await)(contact.clone()).await {
                        debug!("Call rejected");
                        continue;
                    } else if let Err(error) = stream.write(&[0]).await {
                        error!("Error writing to stream: {}", error);
                        continue;
                    }

                    if let Err(error) = self.handshake(stream, remote_address, false, contact).await
                    {
                        error!("Call failed [callee]: {:?}", error);
                    }
                    self.in_call.store(false, Relaxed);
                } else {
                    warn!("Unknown contact: {}", remote_address);
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
        self.in_call.store(true, Relaxed);

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

        // create the UDP socket
        let receive_port = self.receive_port.load(Relaxed);
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

        (self.connected.lock().await)().await;

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
                ErrorKind::NoInputDevice
                | ErrorKind::NoOutputDevice
                | ErrorKind::BuildStream(_)
                | ErrorKind::StreamConfig(_) => {
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
        _stream: &mut TcpStream,
        _stream_cipher: AesCipher,
        send_cipher: AesCipher,
        receive_cipher: AesCipher,
        _caller: bool,
    ) -> Result<()> {
        // get the output device and its default configuration
        let output_device = get_output_device(&self.output_device, &self.host).await?;
        let output_config = output_device.default_output_config()?;
        debug!("output_device: {:?}", output_device.name());

        // get the input device and its default configuration
        let input_device = self.get_input_device().await?;
        let input_config = input_device.default_input_config()?;
        debug!("input_device: {:?}", input_device.name());

        // the number of samples to hold in a channel
        let size = 4_800;

        // sends messages from the input stream to the sending socket
        let (input_sender, input_receiver) = bounded_async::<[f32; 480]>(size / 480);
        // sends raw data from the receiving socket to the processor
        let (output_sender, output_receiver) = bounded_async::<OutputToProcessor>(size / 480);
        // sends the resampled data to the output stream
        let (processed_sender, processed_receiver) = bounded::<f32>(size);

        // the ratio of the output sample rate to the standard sample rate
        let ratio = output_config.sample_rate().0 as f64 / 48_000_f64;
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
        // create a sync sender for the input stream
        let sync_input_sender = input_sender.to_sync();
        // create a buffer to store samples left over from the previous frame
        let mut remaining_samples = Vec::new();

        let input_stream = SendStream {
            stream: input_device.build_input_stream(
                &input_config.into(),
                move |input, _: &_| {
                    // add the samples for the first channel for this frame to the remaining samples
                    for frame in input.chunks(input_channels) {
                        remaining_samples.push(frame[0]);
                    }

                    // process all the samples and store the remaining samples for the next frame
                    remaining_samples = process_input_data(&remaining_samples, &sync_input_sender);
                },
                move |err| {
                    error!("Error in input stream: {}", err);
                },
                None,
            )?,
        };

        // get the output channels for chunking the output
        let output_channels = output_config.channels() as usize;
        let deafened = Arc::clone(&self.deafened);
        let mut sample_generator = SlidingWindow::new(FRAME_SIZE * 4);

        let output_stream = SendStream {
            stream: output_device.build_output_stream(
                &output_config.into(),
                move |output: &mut [f32], _: &_| {
                    if deafened.load(Relaxed) {
                        output.fill(0_f32);
                        return;
                    }

                    for frame in output.chunks_mut(output_channels) {
                        // get the next sample from the processor
                        let sample = match processed_receiver.try_recv() {
                            // if the processor has a sample, process it through the sliding window
                            Ok(Some(sample)) => sample_generator.process_sample(sample),
                            // if there are no frames available, generate a sample
                            Ok(None) | Err(_) => sample_generator.generate_sample(),
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

        // play the streams
        input_stream.stream.play()?;
        output_stream.stream.play()?;

        let input_handle = spawn(socket_input(
            input_receiver,
            Arc::clone(&socket),
            send_cipher,
            Arc::clone(&self.end_call),
            Arc::clone(&self.input_volume),
            Arc::clone(&self.rms_threshold),
            Arc::clone(&self.muted),
        ));

        let output_handle = spawn(socket_output(
            output_sender,
            Arc::clone(&socket),
            receive_cipher,
            Arc::clone(&self.end_call),
        ));

        let processor_future = spawn_blocking_with(
            move || processor_handle.join(),
            FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
        );

        select! {
            result = input_handle => result??,
            reason = output_handle => {
                let message = match reason? {
                    EndReason::Ended => String::new(),
                    EndReason::MissingDevice => "The other party is missing an audio device".to_string(),
                    EndReason::Error => "The other party encountered an error".to_string(),
                };

                (self.call_ended.lock().await)(message).await
            },
            // this unwrap is safe because the processor thread will not panic
            result = processor_future => result?.unwrap()?,
            _ = self.end_call.notified() => (self.call_ended.lock().await)(String::new()).await,
        }

        // no matter what, send the end signal
        debug!("Sending end call signal");
        socket.send(&[1]).await?;

        Ok(())
    }

    async fn get_input_device(&self) -> Result<Device> {
        match *self.input_device.lock().await {
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
                .ok_or(Error::no_input_device()),
            None => self
                .host
                .default_input_device()
                .ok_or(Error::no_input_device()),
        }
    }
}

/// Wraps a cpal stream to unsafely make it send
pub(crate) struct SendStream {
    pub(crate) stream: Stream,
}

unsafe impl Send for SendStream {}

#[derive(Debug)]
enum EndReason {
    Ended,
    MissingDevice,
    Error,
}

#[derive(Clone, PartialEq)]
#[frb(opaque)]
pub struct Contact {
    /// A random ID to identify the contact
    id: String,

    /// The nickname of the contact
    nickname: String,

    /// The public/verifying key for the contact
    verifying_key: [u8; 32],

    /// The address of the contact
    address: SocketAddr,
}

impl FromStr for Contact {
    type Err = DartError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: (&str, &str, &str, &str) = s
            .splitn(4, ',')
            .collect_tuple()
            .ok_or(Error::invalid_contact_format())?;

        let id = parts.0.to_string();
        let nickname = parts.1.to_string();
        let verifying_key = BASE64_STANDARD
            .decode(parts.2.as_bytes())
            .map_err(Error::from)?;
        let address = parts.3.parse().map_err(Error::from)?;

        Ok(Self {
            id,
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
            id: Uuid::new_v4().to_string(),
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
    pub fn ip_str(&self) -> String {
        self.address.ip().to_string()
    }

    #[frb(sync)]
    pub fn store(&self) -> String {
        format!(
            "{},{},{},{}",
            self.id,
            self.nickname,
            self.verifying_key_str(),
            self.address
        )
    }

    #[frb(sync)]
    pub fn equals(&self, other: &Contact) -> bool {
        self == other
    }

    #[frb(sync)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[frb(sync)]
    pub fn set_address(&mut self, address: String) -> std::result::Result<(), DartError> {
        self.address = address.parse().map_err(Error::from)?;
        Ok(())
    }

    #[frb(sync)]
    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = nickname;
    }

    #[frb(sync)]
    pub fn pub_clone(&self) -> Contact {
        self.clone()
    }
}

struct SlidingWindow {
    window: VecDeque<f32>,
    capacity: usize,
    generated_count: usize,
}

impl SlidingWindow {
    fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            capacity,
            generated_count: 0,
        }
    }

    /// Processes a real sample
    fn process_sample(&mut self, mut sample: f32) -> f32 {
        if self.generated_count > 0 {
            self.generated_count -= 1;

            if let Some(last) = self.window.back() {
                let difference = sample - last;
                sample = *last + difference * (self.generated_count as f32 / self.capacity as f32);
            }
        }

        self._add_sample(sample);
        sample
    }

    /// Generates a sample
    fn generate_sample(&mut self) -> f32 {
        if self.window.is_empty() {
            return 0_f32;
        }

        let mut sample = self.interpolate_sample();
        sample -= sample * (self.generated_count as f32 / self.capacity as f32);

        self._add_sample(sample);
        self.generated_count += 1;

        sample
    }

    // TODO a more sophisticated interpolation method
    fn interpolate_sample(&self) -> f32 {
        let sum: f32 = self.window.iter().sum();
        sum / self.window.len() as f32
    }

    /// Adds a sample to the window and removes the oldest sample if the window is full
    fn _add_sample(&mut self, sample: f32) {
        if self.window.len() == self.capacity {
            self.window.pop_front();
        }

        self.window.push_back(sample);
    }
}

/// A message from the receiving socket to the processor
enum OutputToProcessor {
    Data(TransferBuffer),
    Silence,
}

#[frb(sync)]
pub fn create_log_stream(s: StreamSink<String>) {
    logger::SendToDartLogger::set_stream_sink(s);
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
fn process_input_data(input: &[f32], sender: &Sender<[f32; 480]>) -> Vec<f32> {
    for frame in input.chunks(FRAME_SIZE) {
        if frame.len() == FRAME_SIZE {
            _ = sender.try_send(frame.try_into().unwrap());
        } else {
            // if input.len is not a multiple of FRAME_SIZE, the last frame is incomplete
            return frame.to_vec();
        }
    }

    Vec::new()
}

/// Receives frames of audio data from the input stream and sends them to the socket
async fn socket_input<C: StreamCipher>(
    input_receiver: AsyncReceiver<[f32; 480]>,
    socket: Arc<UdpSocket>,
    mut cipher: C,
    notify: Arc<Notify>,
    input_factor: Arc<AtomicF32>,
    rms_threshold: Arc<AtomicF32>,
    muted: Arc<AtomicBool>,
) -> Result<()> {
    let mut byte_buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut int_buffer = [0; FRAME_SIZE];
    let mut output_buffer = [0_f32; FRAME_SIZE];

    let mut sequence_number = 0_u64;
    let mut silence_length = 0; // short silence detection
    let mut denoiser = DenoiseState::new();

    // the maximum and minimum values for i16 as f32
    let max_i16_f32 = i16::MAX as f32;
    let min_i16_f32 = i16::MIN as f32;

    let future = async {
        while let Ok(mut frame) = input_receiver.recv().await {
            if muted.load(Relaxed) {
                socket.send(&[0]).await?;
                continue;
            }

            let factor = max_i16_f32 * input_factor.load(Relaxed);

            // rescale the samples to -32768.0 to 32767.0 for rnnoise
            frame.par_iter_mut().for_each(|x| {
                *x *= factor;
                *x = x.trunc().clamp(min_i16_f32, max_i16_f32);
            });

            // denoise the frame
            denoiser.process_frame(&mut output_buffer, &frame);

            // check if the frame is below the rms threshold
            if calculate_rms(&output_buffer) < rms_threshold.load(Relaxed) {
                if silence_length < 80 {
                    silence_length += 1; // short silences are ignored
                } else {
                    socket.send(&[0]).await?;
                    continue;
                }
            } else {
                silence_length = 0;
            }

            // cast the f32 samples to i16
            int_buffer = output_buffer.map(|x| x as i16);

            // convert the i16 samples to bytes
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    int_buffer.as_ptr() as *const u8,
                    int_buffer.len() * mem::size_of::<i16>(),
                )
            };

            // encrypt the audio data
            cipher
                .apply_keystream_b2b(bytes, &mut byte_buffer[8..])
                .unwrap();

            // add the sequence number to the buffer
            byte_buffer[..8].copy_from_slice(&sequence_number.to_be_bytes());
            sequence_number += TRANSFER_BUFFER_SIZE as u64; // increment the sequence number

            socket.send(&byte_buffer).await?;
        }

        Ok::<(), Error>(())
    };

    select! {
        result = future => result,
        _ = notify.notified() => {
            debug!("Input to socket ended");
            Ok(())
        },
    }
}

/// Receives audio data from the socket and sends it to the output processor
async fn socket_output<C: StreamCipher + StreamCipherSeek>(
    sender: AsyncSender<OutputToProcessor>,
    socket: Arc<UdpSocket>,
    mut cipher: C,
    notify: Arc<Notify>,
    // rate: f64,
) -> EndReason {
    let mut in_buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut out_buffer = [0; TRANSFER_BUFFER_SIZE];

    let future = async {
        loop {
            match timeout(RECEIVE_TIMEOUT, socket.recv(&mut in_buffer)).await {
                // normal chunk of audio data
                Ok(Ok(len)) if len == TRANSFER_BUFFER_SIZE + 8 => {
                    // unwrap is safe because we know the buffer is the correct length
                    let sequence_number = u64::from_be_bytes(in_buffer[..8].try_into().unwrap());
                    let position = cipher.current_pos::<u64>();

                    if position != sequence_number {
                        if position > sequence_number {
                            debug!(
                                "[cipher] seeking backward by {}",
                                position - sequence_number
                            );
                        } else {
                            debug!("[cipher] seeking forward by {}", sequence_number - position);
                        }

                        cipher.seek(sequence_number);
                    }

                    // unwrap is safe because we know the buffer is the correct length
                    cipher
                        .apply_keystream_b2b(&in_buffer[8..], &mut out_buffer)
                        .unwrap();

                    _ = sender.try_send(OutputToProcessor::Data(out_buffer));
                }
                // control signals
                Ok(Ok(1)) => match in_buffer[0] {
                    0 => _ = sender.try_send(OutputToProcessor::Silence), // silence
                    1 => break EndReason::Ended,                          // end of call
                    2 => break EndReason::MissingDevice, // the other party is missing a device
                    3 => break EndReason::Error,         // the other party encountered an error
                    _ => error!("received unknown control signal {}", in_buffer[0]),
                },
                Ok(Ok(len)) => error!("Received {} < {} data", len, TRANSFER_BUFFER_SIZE + 8),
                Err(_) => error!("Receiver timed out"),
                Ok(Err(error)) => match error.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        error!("connection reset");
                        break EndReason::Error;
                    }
                    _ => error!("error receiving: {}", error.kind()),
                },
            }
        }
    };

    select! {
        result = future => result,
        _ = notify.notified() => {
            debug!("Socket output ended");
            EndReason::Ended
        },
    }
}

/// Processes the audio data and sends it to the output stream
fn processor(
    receiver: Receiver<OutputToProcessor>,
    sender: Sender<f32>,
    ratio: f64,
    output_volume: Arc<AtomicF32>,
) -> Result<()> {
    // all audio is mono
    let mut resampler = resampler_factory(ratio, 1)?;

    // rubato requires 10 extra bytes in the output buffer as a safety margin
    let post_len = (FRAME_SIZE as f64 * ratio + 10.0) as usize;

    // the input for the resampler
    let mut pre_buf = [&mut [0_f32; FRAME_SIZE]];
    // the output for the resampler
    let mut post_buf = [vec![0_f32; post_len]];

    while let Ok(message) = receiver.recv() {
        match message {
            OutputToProcessor::Silence => {
                for _ in 0..FRAME_SIZE {
                    sender.try_send(0_f32)?;
                }
            }
            OutputToProcessor::Data(bytes) => {
                // convert the bytes to 16-bit integers
                let ints = unsafe {
                    std::slice::from_raw_parts(
                        bytes.as_ptr() as *const i16,
                        bytes.len() / mem::size_of::<i16>(),
                    )
                };

                // convert the chunk to f32s
                ints.iter()
                    .enumerate()
                    .for_each(|(i, &x)| pre_buf[0][i] = x as f32 / i16::MAX as f32);

                // apply the output volume
                let factor = output_volume.load(Relaxed);
                mul(pre_buf[0], factor);

                if let Some(resampler) = &mut resampler {
                    // resample the data
                    let processed = resampler.process_into_buffer(&pre_buf, &mut post_buf, None)?;

                    // send the resampled data to the output stream
                    for sample in &post_buf[0][..processed.1] {
                        sender.try_send(*sample)?;
                    }
                } else {
                    // if no resampling is needed, send the data to the output stream
                    for sample in *pre_buf[0] {
                        sender.try_send(sample)?;
                    }
                }
            }
        }
    }

    debug!("Processor ended");
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
pub(crate) fn mul(vec: &mut [f32], factor: f32) {
    vec.par_iter_mut().for_each(|p| {
        *p *= factor;
        *p = p.clamp(-1_f32, 1_f32);
    })
}

/// Converts a decibel value to a multiplier
pub(crate) fn db_to_multiplier(db: f32) -> f32 {
    10_f32.powf(db / 20.0)
}

pub(crate) fn resampler_factory(ratio: f64, channels: usize) -> Result<Option<SincFixedIn<f32>>> {
    if ratio == 1_f64 {
        debug!("No resampling needed");
        Ok(None)
    } else {
        debug!("Resampling at {:.2}%", ratio * 100_f64);

        // create the resampler if needed
        Ok(Some(SincFixedIn::<f32>::new(
            ratio,
            2.0,
            RESAMPLER_PARAMETERS,
            FRAME_SIZE,
            channels,
        )?))
    }
}

pub(crate) async fn get_output_device(
    output_device: &DeviceName,
    host: &Arc<Host>,
) -> Result<Device> {
    match *output_device.lock().await {
        Some(ref name) => host
            .output_devices()?
            .find(|device| {
                if let Ok(ref device_name) = device.name() {
                    name == device_name
                } else {
                    false
                }
            })
            .ok_or(Error::no_output_device()),
        None => host
            .default_output_device()
            .ok_or(Error::no_output_device()),
    }
}
