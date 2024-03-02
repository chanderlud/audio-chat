use std::collections::HashMap;
use std::mem;
pub use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU16};
use std::sync::Arc;
use std::time::{Duration, Instant};

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherCoreWrapper, StreamCipherSeek};
use aes::Aes256;
use atomic_float::AtomicF32;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use compressor::Compressor;
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
use rsntp::AsyncSntpClient;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use sha2::Sha256;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Mutex, Notify};
use tokio::time::{sleep, timeout};
use tokio::{io, select};
use uuid::Uuid;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::items::{message, AudioHeader, Hello, Identity, Message, Ports};
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
const RECEIVE_TIMEOUT: Duration = Duration::from_secs(2);
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

    /// Enables rnnoise denoising
    denoise: Arc<AtomicBool>,

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

    /// Keeps track of and controls the controllers
    controller_states: Arc<Mutex<HashMap<String, Arc<ControllerState>>>>,

    /// Used for producing validation timestamps
    time: Arc<Mutex<SyncedTime>>,

    /// Prompts the user to accept a call
    accept_call: Arc<Mutex<dyn Fn(String) -> DartFnFuture<bool> + Send>>,

    /// Alerts the UI that a call has ended
    call_ended: Arc<Mutex<dyn Fn(String) -> DartFnFuture<()> + Send>>,

    /// Fetches a contact from the front end
    get_contact: Arc<Mutex<dyn Fn(String) -> DartFnFuture<Option<Contact>> + Send>>,

    /// Alerts the UI that the call has connected
    connected: Arc<Mutex<dyn Fn() -> DartFnFuture<()> + Send>>,

    /// Notifies the frontend that the call has disconnected or reconnected
    call_state: Arc<Mutex<dyn Fn(bool) -> DartFnFuture<()> + Send>>,

    /// Alerts the UI when a contact comes online or goes offline
    contact_status: Arc<Mutex<dyn Fn(String, bool) -> DartFnFuture<()> + Send>>,

    /// Starts a controller for each of the UI's contacts
    start_controllers: Arc<Mutex<dyn Fn(AudioChat) -> DartFnFuture<()> + Send>>,
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
        denoise: bool,
        accept_call: impl Fn(String) -> DartFnFuture<bool> + Send + 'static,
        call_ended: impl Fn(String) -> DartFnFuture<()> + Send + 'static,
        get_contact: impl Fn(String) -> DartFnFuture<Option<Contact>> + Send + 'static,
        connected: impl Fn() -> DartFnFuture<()> + Send + 'static,
        call_state: impl Fn(bool) -> DartFnFuture<()> + Send + 'static,
        contact_status: impl Fn(String, bool) -> DartFnFuture<()> + Send + 'static,
        start_controllers: impl Fn(AudioChat) -> DartFnFuture<()> + Send + 'static,
    ) -> AudioChat {
        let key_bytes: [u8; 32] = signing_key.try_into().unwrap();

        let host = cpal::default_host();

        let chat = Self {
            listen_port: Arc::new(AtomicU16::new(listen_port)),
            receive_port: Arc::new(AtomicU16::new(receive_port)),
            host: Arc::new(host),
            rms_threshold: Arc::new(AtomicF32::new(db_to_multiplier(rms_threshold))),
            input_volume: Arc::new(AtomicF32::new(db_to_multiplier(input_volume))),
            output_volume: Arc::new(AtomicF32::new(db_to_multiplier(output_volume))),
            denoise: Arc::new(AtomicBool::new(denoise)),
            end_call: Default::default(),
            stop_listener: Default::default(),
            input_device: Default::default(),
            output_device: Default::default(),
            secret_key: SecretKey::from(key_bytes),
            in_call: Default::default(),
            deafened: Default::default(),
            muted: Default::default(),
            controller_states: Default::default(),
            time: Default::default(),
            accept_call: Arc::new(Mutex::new(accept_call)),
            call_ended: Arc::new(Mutex::new(call_ended)),
            get_contact: Arc::new(Mutex::new(get_contact)),
            connected: Arc::new(Mutex::new(connected)),
            call_state: Arc::new(Mutex::new(call_state)),
            contact_status: Arc::new(Mutex::new(contact_status)),
            start_controllers: Arc::new(Mutex::new(start_controllers)),
        };

        // start the time synchronization background thread
        spawn(synchronize(chat.time.clone()));

        // start the listener
        let chat_clone = chat.clone();
        spawn(async move { chat_clone.listener().await });

        // start the controllers
        (chat.start_controllers.lock().await)(chat.clone()).await;

        chat
    }

    /// The public say_hello function
    pub async fn say_hello(&self, contact: &Contact) -> std::result::Result<(), DartError> {
        if let Some(state) = self.controller_states.lock().await.get(&contact.id) {
            state.start.notify_one();
            Ok(())
        } else {
            Err(String::from("No controller active for contact").into())
        }
    }

    /// Tries to start a controller for a contact
    pub async fn connect(&self, contact: &Contact) {
        if let Err(error) = self._connect(contact).await {
            error!("Error connecting to {}: {}", contact.nickname, error);
        }
    }

    /// Ends the call (if there is one)
    #[frb(sync)]
    pub fn end_call(&self) {
        self.end_call.notify_waiters();
    }

    /// Restarts the controllers
    pub async fn restart_controllers(&self) {
        for state in self.controller_states.lock().await.values() {
            if !state.in_call.load(Relaxed) {
                state.stop.notify_one();
            }
        }

        (self.start_controllers.lock().await)(self.clone()).await;
    }

    /// Restarts the listener
    pub async fn restart_listener(&self) -> std::result::Result<(), DartError> {
        if self.in_call.load(Relaxed) {
            return Err(Error::in_call().into());
        }

        self.restart_controllers().await;
        self.stop_listener.notify_one();

        let chat_clone = self.clone();
        spawn(async move { chat_clone.listener().await });

        Ok(())
    }

    /// Stop a specific controller
    pub async fn stop_controller(&self, contact: &Contact) {
        if let Some(state) = self.controller_states.lock().await.remove(&contact.id) {
            state.stop.notify_one();
        }
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

    /// Changing the denoise flag will not affect the current call
    #[frb(sync)]
    pub fn set_denoise(&self, denoise: bool) {
        self.denoise.store(denoise, Relaxed);
    }

    /// Internal connect function
    async fn _connect(&self, contact: &Contact) -> Result<()> {
        let address = contact.address;

        let mut stream = timeout(HELLO_TIMEOUT, TcpStream::connect(address)).await??;
        stream.write_u16(self.listen_port.load(Relaxed)).await?;

        self.start_controller(stream, contact.clone(), false).await;
        Ok(())
    }

    /// Listens for incoming connections and starts new controllers
    async fn listener(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.listen_port.load(Relaxed))).await?;

        let future = async {
            while let Ok((mut stream, mut address)) = listener.accept().await {
                // a connecting client will send its listen port first
                match stream.read_u16().await {
                    Ok(port) => {
                        // set the port for the address to correctly identify the contact
                        address.set_port(port);
                        let address_str = address.to_string();

                        if let Some(contact) = (self.get_contact.lock().await)(address_str).await {
                            self.start_controller(stream, contact, true).await;
                        } else {
                            error!("connection from unknown contact: {}", address);
                        }
                    }
                    Err(error) => error!("Error reading ports from {} {}", address, error),
                }
            }
        };

        select! {
            _ = future => {},
            _ = self.stop_listener.notified() => debug!("Listener stopped"),
        }

        Ok(())
    }

    /// A wrapper which starts the controller and registers it in the controller states
    async fn start_controller(&self, stream: TcpStream, contact: Contact, listener: bool) {
        // alert the UI that this contact is now online
        (self.contact_status.lock().await)(contact.id.clone(), true).await;

        // create the state and a clone of it for the controller
        let state = Default::default();
        let state_clone = Arc::clone(&state);

        // handle situations where a controller is already running for the contact
        if let Some(controller) = self
            .controller_states
            .lock()
            .await
            .insert(contact.id.clone(), state)
        {
            if !controller.in_call.load(Relaxed) {
                debug!("stopping existing controller for {}", contact.nickname);
                controller.stop.notify_one();
            } else {
                warn!("{} is already in a call", contact.nickname);
            }
        }

        let chat_clone = self.clone();
        spawn(async move {
            let id = contact.id.clone();
            let nickname = contact.nickname.clone();

            if let Err(error) = chat_clone
                .controller(stream, contact, state_clone, listener)
                .await
            {
                error!("Controller error: {}", error);
            } else {
                debug!("Controller for {} ended", nickname);
            }

            // cleanup
            chat_clone.controller_states.lock().await.remove(&id);
            (chat_clone.contact_status.lock().await)(id, false).await;
        });
    }

    /// The controller for each online contact
    async fn controller(
        &self,
        mut stream: TcpStream,
        contact: Contact,
        state: Arc<ControllerState>,
        listener: bool,
    ) -> Result<()> {
        // perform the key exchange
        let shared_secret = key_exchange(&mut stream).await?;

        // HKDF for the key derivation
        let hk = Hkdf::<Sha256>::new(Some(&SALT), shared_secret.as_bytes());

        // stream send cipher
        let mut ss_cipher = cipher_factory(&hk, b"ss-key", b"ss-iv")?;
        // stream read cipher
        let mut sr_cipher = cipher_factory(&hk, b"sr-key", b"sr-iv")?;

        // one client always has the ciphers reversed
        if listener {
            mem::swap(&mut ss_cipher, &mut sr_cipher);
        }

        // create a random nonce
        let mut nonce = [0; 128];
        OsRng.fill(&mut nonce[16..]);

        // adds the current timestamp to the nonce
        {
            let time = self.time.lock().await;
            let timestamp = time.current_timestamp();
            nonce[0..16].copy_from_slice(&timestamp.to_be_bytes());
        }

        // sign the nonce
        let signing_key = SigningKey::from(self.secret_key);
        let signature = signing_key.sign(&nonce);

        // create the identity message
        let message = Identity::new(nonce, signature);
        write_message(&mut stream, &message, &mut ss_cipher).await?;

        // receive the identity message
        let identity: Identity = read_message(&mut stream, &mut sr_cipher).await?;

        let timestamp = u128::from_be_bytes(identity.nonce[0..16].try_into().unwrap());

        let delta = {
            let time = self.time.lock().await;
            let current_timestamp = time.current_timestamp();

            if current_timestamp > timestamp {
                current_timestamp - timestamp
            } else {
                timestamp - current_timestamp
            }
        };

        // a max delta of 60 seconds should prevent replay attacks
        if delta > 60_000_000 {
            warn!("Rejecting handshake due to high delta of {}ms", delta / 1_000);
            return Ok(());
        } else {
            debug!("delta: {}", delta);
        }

        // verify the signature
        let verifying_key = VerifyingKey::from_bytes(&contact.verifying_key)?;
        let signature = Signature::from_slice(&identity.signature)?;
        verifying_key.verify_strict(&identity.nonce, &signature)?;

        // seeds the IV so subsequent calls in the same session are unique
        let mut i = 0;

        loop {
            let future = async {
                debug!("[{}] controller waiting for event", contact.nickname);

                select! {
                    result = read_message(&mut stream, &mut sr_cipher) => {
                        result?; // ensure that a valid Hello message was received
                        state.in_call.store(true, Relaxed); // blocks the controller from being restarted

                        if self.in_call.load(Relaxed) {
                            // do not accept another call if already in one
                            let busy = Message::busy();
                            write_message(&mut stream, &busy, &mut ss_cipher).await
                        } else if (self.accept_call.lock().await)(contact.id.clone()).await {
                            // respond with hello if the call is accepted
                            let hello = Message::hello();
                            write_message(&mut stream, &hello, &mut ss_cipher).await?;

                            i += 1;
                            self.handshake(&mut stream, contact.address.ip(), false, &hk, (&mut ss_cipher, &mut sr_cipher), i).await
                        } else {
                            // reject the call if not accepted
                            let reject = Message::reject();
                            write_message(&mut stream, &reject, &mut ss_cipher).await
                        }
                    }
                    _ = state.start.notified() => {
                        state.in_call.store(true, Relaxed); // blocks the controller from being restarted

                        // queries the other client for a call
                        let hello = Hello::new();
                        write_message(&mut stream, &hello, &mut ss_cipher).await?;

                        // handles a variety of messages sent in response to Hello
                        match read_message(&mut stream, &mut sr_cipher).await? {
                            Message { message: Some(message::Message::Hello(_)) } => {
                                i += 1;
                                self.handshake(&mut stream, contact.address.ip(), true, &hk, (&mut ss_cipher, &mut sr_cipher), i).await?;
                            }
                            Message { message: Some(message::Message::Reject(_)) } => {
                                (self.call_ended.lock().await)(format!("{} did not accept the call", contact.nickname)).await;
                            },
                            Message { message: Some(message::Message::Busy(_)) } => {
                                (self.call_ended.lock().await)(format!("{} is busy", contact.nickname)).await;
                            }
                            _ => warn!("received unexpected message from {}", contact.nickname),
                        }

                        Ok(())
                    }
                }
            };

            select! {
                _ = state.stop.notified() => break Ok(()),
                result = future => {
                    if let Err(error) = result {
                        if state.in_call.load(Relaxed) {
                            (self.call_ended.lock().await)(error.to_string()).await;
                        }

                        match error.kind {
                            ErrorKind::Io(error) => match error.kind() {
                                // these errors indicate that the stream is closed
                                io::ErrorKind::ConnectionReset | io::ErrorKind::UnexpectedEof => break Ok(()),
                                _ => error!("Controller io error: {}", error),
                            }
                            _ => error!("Controller error: {}", error),
                        }
                    }
                }
            }

            // the controller is now safe to restart
            state.in_call.store(false, Relaxed);
        }
    }

    /// Set up the cryptography and negotiate the audio ports
    async fn handshake(
        &self,
        stream: &mut TcpStream,
        remote_address: IpAddr,
        caller: bool,
        hk: &Hkdf<Sha256>,
        stream_cipher: (&mut AesCipher, &mut AesCipher),
        iv_seed: i64,
    ) -> Result<()> {
        // create the send and receive ciphers
        let mut send_cipher = cipher_factory(hk, b"send-key", &iv_seed.to_be_bytes())?;
        let mut receive_cipher = cipher_factory(hk, b"receive-key", &(iv_seed + 1).to_be_bytes())?;

        let receive_port = self.receive_port.load(Relaxed);

        // build the ports message
        let message = Ports::new(receive_port);
        // send the ports message
        write_message(stream, &message, stream_cipher.0).await?;
        // receive the ports message
        let ports: Ports = read_message(stream, stream_cipher.1).await?;

        // create the UDP socket
        let socket = Arc::new(UdpSocket::bind(("0.0.0.0", receive_port)).await?);
        // connect to the remote address on the hello port
        socket.connect((remote_address, ports.port as u16)).await?;

        if caller {
            // the caller always has the ciphers reversed
            mem::swap(&mut send_cipher, &mut receive_cipher);
        }

        // alert the UI that the call has connected
        (self.connected.lock().await)().await;
        self.in_call.store(true, Relaxed);

        let result = self
            .call(
                Arc::clone(&socket),
                stream,
                stream_cipher,
                receive_cipher,
                send_cipher,
            )
            .await;

        // the call has ended
        self.in_call.store(false, Relaxed);

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
        stream: &mut TcpStream,
        stream_cipher: (&mut AesCipher, &mut AesCipher),
        send_cipher: AesCipher,
        receive_cipher: AesCipher,
    ) -> Result<()> {
        // the denoise flag is constant for the entire call
        let denoise = self.denoise.load(Relaxed);

        // get the output device and its default configuration
        let output_device = get_output_device(&self.output_device, &self.host).await?;
        let output_config = output_device.default_output_config()?;
        debug!("output_device: {:?}", output_device.name());

        // get the input device and its default configuration
        let input_device = self.get_input_device().await?;
        let input_config = input_device.default_input_config()?;
        debug!("input_device: {:?}", input_device.name());

        let mut audio_header = AudioHeader::from(&input_config);

        // rnnoise requires a 48kHz sample rate
        if denoise {
            audio_header.sample_rate = 48_000;
        }

        // send the audio header
        write_message(stream, &audio_header, stream_cipher.0).await?;
        let remote_input_config: AudioHeader = read_message(stream, stream_cipher.1).await?;

        // the number of samples to hold in a channel
        let size = 2_400;

        // sends messages from the input processor to the sending socket
        let (processed_input_sender, processed_input_receiver) =
            bounded_async::<ProcessorToOutput>(size / 480);
        // sends raw data from the receiving socket to the output processor
        let (output_sender, output_receiver) = bounded_async::<OutputToProcessor>(size / 480);
        // sends samples from the output processor to the output stream
        let (processed_output_sender, processed_output_receiver) = bounded::<f32>(size);
        // sends samples from the input to the input processor
        let (input_sender, input_receiver) = bounded::<f32>(size);

        // get a reference to input volume for the processor
        let input_volume = Arc::clone(&self.input_volume);
        // get a reference to the rms threshold for the processor
        let rms_threshold = Arc::clone(&self.rms_threshold);
        // get a reference to the muted flag for the processor
        let muted = Arc::clone(&self.muted);
        // get a sync version of the processed input sender
        let processed_input_sender = processed_input_sender.to_sync();
        // the input processor needs the sample rate
        let sample_rate = input_config.sample_rate().0 as f64;

        // spawn the input processor thread
        let input_processor_handle = std::thread::spawn(move || {
            input_processor(
                input_receiver,
                processed_input_sender,
                sample_rate,
                input_volume,
                rms_threshold,
                muted,
                denoise,
            )
        });

        // the ratio of the output sample rate to the remote input sample rate
        let ratio = output_config.sample_rate().0 as f64 / remote_input_config.sample_rate as f64;
        // get a reference to output volume for the processor
        let output_volume = Arc::clone(&self.output_volume);

        // spawn the output processor thread
        let output_processor_handle = std::thread::spawn(move || {
            output_processor(
                output_receiver.to_sync(),
                processed_output_sender,
                ratio,
                output_volume,
            )
        });

        // sends single samples to the socket
        let input_channels = input_config.channels() as usize;

        let input_stream = SendStream {
            stream: input_device.build_input_stream(
                &input_config.into(),
                move |input, _: &_| {
                    for frame in input.chunks(input_channels) {
                        _ = input_sender.try_send(frame[0]);
                    }
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

        let output_stream = SendStream {
            stream: output_device.build_output_stream(
                &output_config.into(),
                move |output: &mut [f32], _: &_| {
                    if deafened.load(Relaxed) {
                        output.fill(0_f32);
                        return;
                    }

                    for frame in output.chunks_mut(output_channels) {
                        let sample = processed_output_receiver.recv().unwrap_or(0_f32);

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
            processed_input_receiver,
            Arc::clone(&socket),
            send_cipher,
            Arc::clone(&self.end_call),
        ));

        let output_handle = spawn(socket_output(
            output_sender,
            Arc::clone(&socket),
            receive_cipher,
            Arc::clone(&self.end_call),
            Arc::clone(&self.call_state),
        ));

        let input_processor_future = spawn_blocking_with(
            move || input_processor_handle.join(),
            FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
        );

        let output_processor_future = spawn_blocking_with(
            move || output_processor_handle.join(),
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
            result = output_processor_future => result?.unwrap()?,
            // this unwrap is safe because the processor thread will not panic
            result = input_processor_future => result?.unwrap()?,
            _ = self.end_call.notified() => (self.call_ended.lock().await)(String::new()).await,
        }

        // no matter what, send the end signal
        debug!("Sending end call signal");
        socket.send(&[1]).await?;

        Ok(())
    }

    /// Returns either the default or the user specified device
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

#[derive(Clone, Debug)]
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

/// A message from the receiving socket to the output processor
enum OutputToProcessor {
    Data(Box<TransferBuffer>),
    Silence,
}

/// A message from the input processor to the sending socket
enum ProcessorToOutput {
    Data(Box<[f32; 480]>),
    Silence,
}

/// Keeps track of the active controllers
#[derive(Default)]
struct ControllerState {
    /// Signals the controller to initiate a call
    start: Notify,

    /// Stops the controller
    stop: Notify,

    /// If the controller is in a call
    in_call: AtomicBool,
}

/// Produces current timestamps with high precision
struct SyncedTime {
    datetime: DateTime<Utc>,
    instant: Instant,
}

impl Default for SyncedTime {
    fn default() -> Self {
        Self::new(Utc::now())
    }
}

impl SyncedTime {
    fn new(datetime: DateTime<Utc>) -> Self {
        Self {
            datetime,
            instant: Instant::now(),
        }
    }

    /// Returns the current timestamp in microseconds
    fn current_timestamp(&self) -> u128 {
        self.datetime.timestamp_micros() as u128 + self.instant.elapsed().as_micros()
    }
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

/// Receives frames of audio data from the input processor and sends them to the socket
async fn socket_input<C: StreamCipher>(
    input_receiver: AsyncReceiver<ProcessorToOutput>,
    socket: Arc<UdpSocket>,
    mut cipher: C,
    notify: Arc<Notify>,
) -> Result<()> {
    let mut byte_buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut int_buffer = [0; FRAME_SIZE];

    let mut sequence_number = 0_u64;

    let future = async {
        while let Ok(message) = input_receiver.recv().await {
            match message {
                ProcessorToOutput::Silence => {
                    // send the silence signal
                    socket.send(&[0]).await?;
                }
                ProcessorToOutput::Data(frame) => {
                    // cast the f32 samples to i16
                    int_buffer = frame.map(|x| x as i16);

                    // convert the i16 samples to bytes
                    let bytes = unsafe {
                        std::slice::from_raw_parts(
                            int_buffer.as_ptr() as *const u8,
                            int_buffer.len() * mem::size_of::<i16>(),
                        )
                    };

                    // encrypt the audio data (unwrap is safe because we know the buffer is the correct length)
                    cipher
                        .apply_keystream_b2b(bytes, &mut byte_buffer[8..])
                        .unwrap();

                    // add the sequence number to the buffer
                    byte_buffer[..8].copy_from_slice(&sequence_number.to_be_bytes());
                    sequence_number += TRANSFER_BUFFER_SIZE as u64; // increment the sequence number

                    socket.send(&byte_buffer).await?;
                }
            }
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
    disconnected_callback: Arc<Mutex<dyn Fn(bool) -> DartFnFuture<()> + Send>>,
) -> EndReason {
    let mut in_buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut out_buffer = [0; TRANSFER_BUFFER_SIZE];
    let mut disconnected = false;

    let future = async {
        loop {
            match timeout(RECEIVE_TIMEOUT, socket.recv(&mut in_buffer)).await {
                // normal chunk of audio data
                Ok(Ok(len)) if len == TRANSFER_BUFFER_SIZE + 8 => {
                    if disconnected {
                        disconnected = false;
                        (disconnected_callback.lock().await)(disconnected).await;
                    }

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

                    _ = sender.try_send(OutputToProcessor::Data(Box::new(out_buffer)));
                }
                // control signals
                Ok(Ok(1)) => {
                    if disconnected {
                        disconnected = false;
                        (disconnected_callback.lock().await)(disconnected).await;
                    }

                    match in_buffer[0] {
                        0 => _ = sender.try_send(OutputToProcessor::Silence), // silence
                        1 => break EndReason::Ended,                          // end of call
                        2 => break EndReason::MissingDevice, // the other party is missing a device
                        3 => break EndReason::Error,         // the other party encountered an error
                        _ => error!("received unknown control signal {}", in_buffer[0]),
                    }
                }
                // malformed packets (never happens)
                Ok(Ok(len)) => error!("Received {} < {} data", len, TRANSFER_BUFFER_SIZE + 8),
                // timeouts
                Err(_) => {
                    if !disconnected {
                        disconnected = true;
                        (disconnected_callback.lock().await)(disconnected).await;
                    }
                }
                // socket errors
                Ok(Err(error)) => match error.kind() {
                    io::ErrorKind::ConnectionReset => {
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

// TODO consider using a SincFixedOut resampler here
/// Processes the audio input and sends it to the sending socket
fn input_processor(
    receiver: Receiver<f32>,
    sender: Sender<ProcessorToOutput>,
    sample_rate: f64,
    input_factor: Arc<AtomicF32>,
    rms_threshold: Arc<AtomicF32>,
    muted: Arc<AtomicBool>,
    denoise: bool,
) -> Result<()> {
    // the maximum and minimum values for i16 as f32
    let max_i16_f32 = i16::MAX as f32;
    let min_i16_f32 = i16::MIN as f32;

    let ratio = if denoise {
        // rnnoise requires a 48kHz sample rate
        48_000.0 / sample_rate
    } else {
        // do not resample if not using rnnoise
        1_f64
    };

    // rubato requires 10 extra spaces in the output buffer as a safety margin
    let post_len = (FRAME_SIZE as f64 + 10_f64) as usize;
    let in_len = (FRAME_SIZE as f64 / ratio).ceil() as usize;

    let mut resampler = resampler_factory(ratio, 1, in_len)?;
    let mut denoiser = denoise.then_some(DenoiseState::new());
    let mut compressor = Compressor::peak_min(1_000_f64, 1_000_f64, sample_rate, 0.1, 100_f32);

    // the input for the resampler
    let mut pre_buf = [vec![0_f32; in_len]];
    // the output for the resampler
    let mut post_buf = [vec![0_f32; post_len]];
    // the output for rnnoise
    let mut out_buf = [0_f32; FRAME_SIZE];

    // the position in pre_buf
    let mut position = 0;
    // a counter user for short silence detection
    let mut silence_length = 0_u8;
    // avoids checking the muted state in every iteration
    let mut is_muted = muted.load(Relaxed);

    while let Ok(mut sample) = receiver.recv() {
        // check if the muted state has changed every 100 iterations
        if position % 100 == 0 {
            let muted = muted.load(Relaxed);

            // muted -> unmuted transition
            if !muted && is_muted {
                silence_length = 0;
                position = 0;
            }

            is_muted = muted;
        }

        // sends a silence signal for every FRAME_SIZE samples if the input is muted
        if is_muted {
            if position > FRAME_SIZE {
                position = 0;
                _ = sender.try_send(ProcessorToOutput::Silence);
            } else {
                position += 1;
            }

            continue;
        }

        sample = compressor.next_frame([sample])[0];
        pre_buf[0][position] = sample;
        position += 1;

        if position < in_len {
            continue;
        } else {
            position = 0;
        }

        let (target_buffer, len) = if let Some(resampler) = &mut resampler {
            // resample the data
            let processed = resampler.process_into_buffer(&pre_buf, &mut post_buf, None)?;
            (&mut post_buf[0], processed.1)
        } else {
            (&mut pre_buf[0], FRAME_SIZE)
        };

        // the first frame may be smaller than FRAME_SIZE
        if len != FRAME_SIZE {
            warn!("input_processor: len != FRAME_SIZE: {}", len);
            continue;
        }

        // apply the input volume & scale the samples to -32768.0 to 32767.0
        let factor = max_i16_f32 * input_factor.load(Relaxed);

        // rescale the samples to -32768.0 to 32767.0 for rnnoise
        target_buffer.par_iter_mut().for_each(|x| {
            *x *= factor;
            *x = x.trunc().clamp(min_i16_f32, max_i16_f32);
        });

        if let Some(ref mut denoiser) = denoiser {
            // denoise the frame
            denoiser.process_frame(&mut out_buf, &target_buffer[..len]);
        } else {
            out_buf = target_buffer[..len].try_into().unwrap();
        };

        // check if the frame is below the rms threshold
        if calculate_rms(&out_buf) < rms_threshold.load(Relaxed) {
            if silence_length < 80 {
                silence_length += 1; // short silences are ignored
            } else {
                _ = sender.try_send(ProcessorToOutput::Silence);
            }
        } else {
            silence_length = 0;
            _ = sender.try_send(ProcessorToOutput::Data(Box::new(out_buf)));
        }
    }

    debug!("Input processor ended");
    Ok(())
}

/// Processes the audio data and sends it to the output stream
fn output_processor(
    receiver: Receiver<OutputToProcessor>,
    sender: Sender<f32>,
    ratio: f64,
    output_volume: Arc<AtomicF32>,
) -> Result<()> {
    let max_i16_f32 = i16::MAX as f32;

    let mut resampler = resampler_factory(ratio, 1, FRAME_SIZE)?;

    // rubato requires 10 extra spaces in the output buffer as a safety margin
    let post_len = (FRAME_SIZE as f64 * ratio + 10_f64) as usize;

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

                // convert the frame to f32s
                ints.iter()
                    .enumerate()
                    .for_each(|(i, &x)| pre_buf[0][i] = x as f32 / max_i16_f32);

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

    debug!("Output processor ended");
    Ok(())
}

/// Reads a public key from the stream
async fn read_public(stream: &mut TcpStream) -> Result<PublicKey> {
    let mut buffer = [0; 32];
    stream.read_exact(&mut buffer).await?;
    Ok(PublicKey::from(buffer))
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
pub(crate) fn mul(frame: &mut [f32], factor: f32) {
    frame.par_iter_mut().for_each(|p| {
        *p *= factor;
        *p = p.clamp(-1_f32, 1_f32);
    })
}

/// Converts a decibel value to a multiplier
pub(crate) fn db_to_multiplier(db: f32) -> f32 {
    10_f32.powf(db / 20.0)
}

/// Produces a resampler if needed
pub(crate) fn resampler_factory(
    ratio: f64,
    channels: usize,
    size: usize,
) -> Result<Option<SincFixedIn<f32>>> {
    if ratio == 1_f64 {
        Ok(None)
    } else {
        // create the resampler if needed
        Ok(Some(SincFixedIn::<f32>::new(
            ratio,
            2.0,
            RESAMPLER_PARAMETERS,
            size,
            channels,
        )?))
    }
}

/// Gets the output device
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

/// Performs the key exchange
async fn key_exchange(stream: &mut TcpStream) -> Result<SharedSecret> {
    let secret = EphemeralSecret::random();

    // send our public key
    let our_public = PublicKey::from(&secret);
    stream.write_all(our_public.as_bytes()).await?;

    // receive their public key
    let their_public = timeout(HELLO_TIMEOUT, read_public(stream)).await??;
    let shared_secret = secret.diffie_hellman(&their_public);

    Ok(shared_secret)
}

/// A background thread which produces synchronized datetime objects
async fn synchronize(time: Arc<Mutex<SyncedTime>>) {
    let client = AsyncSntpClient::new();

    loop {
        match client.synchronize("pool.ntp.org").await {
            Ok(result) => {
                let local = DateTime::from(result.datetime().into_chrono_datetime().unwrap());
                *time.lock().await = SyncedTime::new(local);
                debug!("time synchronized: {}", local.to_rfc2822());

                // re-sync every 5 minutes
                sleep(Duration::from_secs(60 * 5)).await;
            }
            Err(error) => {
                error!("Failed to synchronize time: {}", error);
                // retry after 20 seconds
                sleep(Duration::from_secs(20)).await;
            }
        }
    }
}