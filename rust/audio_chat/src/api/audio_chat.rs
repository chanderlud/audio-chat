use std::collections::HashMap;
use std::mem;
use std::net::ToSocketAddrs;
pub use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::time::Duration;

use async_throttle::RateLimiter;
use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use cpal::Host;
use cpal::{Device, Stream};
use ed25519_dalek::{SecretKey, SigningKey};
use flutter_rust_bridge::{frb, spawn, spawn_blocking_with, DartFnFuture};
use kanal::{
    bounded, bounded_async, unbounded_async, AsyncReceiver, AsyncSender, Receiver, Sender,
};
use log::{debug, error, info, warn};
use nnnoiseless::{DenoiseState, RnnModel, FRAME_SIZE};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, Notify, RwLock};
use tokio::time::{interval, sleep, timeout, Instant};
use tokio::{io, select};
use tokio_util::bytes::Bytes;
use tokio_util::codec::LengthDelimitedCodec;
use webrtc_ice::agent::agent_config::AgentConfig;
use webrtc_ice::agent::Agent;
use webrtc_ice::candidate::candidate_base::unmarshal_candidate;
use webrtc_ice::network_type::NetworkType;
use webrtc_ice::state::ConnectionState;
use webrtc_ice::udp_network::UDPNetwork;
use webrtc_ice::url::Url;
use webrtc_sctp::association::{Association, Config};
use webrtc_sctp::chunk::chunk_payload_data::PayloadProtocolIdentifier;
use webrtc_sctp::stream::{PollStream, ReliabilityType};
use webrtc_util::Conn;

use common::crypto::{identity_factory, key_exchange, verify_identity, PairedCipher};
use common::items::{Candidate, Identity, RequestOutcome, RequestSession};
use common::items::{EndSession, Message as CommonMessage};
use common::time::synchronize;
use common::{
    read_message, write_message, Aes256, AesCipher, Ctr128BE, Hkdf, KeyIvInit, Sha256,
    StreamCipher, StreamCipherSeek, Time, Transport,
};

use crate::api::contact::Contact;
use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::items::{message, AudioHeader, Message};
use crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER;

type Result<T> = std::result::Result<T, Error>;
pub(crate) type DeviceName = Arc<Mutex<Option<String>>>;
type TransferBuffer = [u8; TRANSFER_BUFFER_SIZE];

/// The number of bytes in a single UDP packet
const TRANSFER_BUFFER_SIZE: usize = FRAME_SIZE * mem::size_of::<i16>();
const PACKET_SIZE: usize = TRANSFER_BUFFER_SIZE + 16;
/// Parameters used for resampling throughout the application
const RESAMPLER_PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: WindowFunction::BlackmanHarris2,
};
/// The timeout for receiving the next audio frame
const RECEIVE_TIMEOUT: Duration = Duration::from_secs(2);
/// A timeout used when initializing the call
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
/// Times out a session if no pings are received
const SESSION_TIMEOUT: Duration = Duration::from_secs(30);
/// Controls the frequency of application level pings
const PING_INTERVAL: Duration = Duration::from_secs(1);
/// the number of frames to hold in a channel
const CHANNEL_SIZE: usize = 2_400;

#[frb(opaque)]
#[derive(Clone)]
pub struct AudioChat {
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

    /// The rnnoise model
    denoise_model: Arc<RwLock<RnnModel>>,

    /// Notifies the call to end
    end_call: Arc<Notify>,

    /// Manually set the input device
    input_device: DeviceName,

    /// Manually set the output device
    output_device: DeviceName,

    /// Private key for signing the handshake
    signing_key: Arc<RwLock<SigningKey>>,

    /// Keeps track of whether the user is in a call
    in_call: Arc<AtomicBool>,

    /// Disables the output stream
    deafened: Arc<AtomicBool>,

    /// Disables the input stream
    muted: Arc<AtomicBool>,

    /// Disables the playback of custom ringtones
    play_custom_ringtones: Arc<AtomicBool>,

    /// Keeps track of and controls the sessions
    session_states: Arc<RwLock<HashMap<String, Arc<SessionState>>>>,

    /// Used for producing validation timestamps
    time: Time,

    /// Signals the session manager to start a new session
    start_session: AsyncSender<[u8; 32]>,

    /// Restarts the session manager when needed
    restart_manager: Arc<Notify>,

    /// Network configuration for p2p connections
    network_config: NetworkConfig,

    /// Prompts the user to accept a call
    accept_call: Arc<Mutex<dyn Fn(String, Option<Vec<u8>>) -> DartFnFuture<bool> + Send>>,

    /// Alerts the UI that a call has ended
    call_ended: Arc<Mutex<dyn Fn(String, bool) -> DartFnFuture<()> + Send>>,

    /// Fetches a contact from the front end
    get_contact: Arc<Mutex<dyn Fn([u8; 32]) -> DartFnFuture<Option<Contact>> + Send>>,

    /// Alerts the UI that the call has connected
    connected: Arc<Mutex<dyn Fn() -> DartFnFuture<()> + Send>>,

    /// Notifies the frontend that the call has disconnected or reconnected
    call_state: Arc<Mutex<dyn Fn(bool) -> DartFnFuture<()> + Send>>,

    /// Alerts the UI when the status of a session changes
    session_status: Arc<Mutex<dyn Fn(String, String) -> DartFnFuture<()> + Send>>,

    /// Starts a session for each of the UI's contacts
    start_sessions: Arc<Mutex<dyn Fn(AudioChat) -> DartFnFuture<()> + Send>>,

    /// Used to load custom ringtones
    load_ringtone: Arc<Mutex<dyn Fn() -> DartFnFuture<Option<Vec<u8>>> + Send>>,

    /// Used to report statistics to the frontend
    statistics: Arc<Mutex<dyn Fn(Statistics) -> DartFnFuture<()> + Send>>,

    /// Used to send chat messages to the frontend
    message_received: Arc<Mutex<dyn Fn(String) -> DartFnFuture<()> + Send>>,

    /// Alerts the UI when the manager is active and restartable
    manager_active: Arc<Mutex<dyn Fn(bool, bool) -> DartFnFuture<()> + Send>>,
}

impl AudioChat {
    // this function must be async to use `spawn`
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        signing_key: Vec<u8>,
        host: Arc<Host>,
        network_config: &NetworkConfig,
        accept_call: impl Fn(String, Option<Vec<u8>>) -> DartFnFuture<bool> + Send + 'static,
        call_ended: impl Fn(String, bool) -> DartFnFuture<()> + Send + 'static,
        get_contact: impl Fn([u8; 32]) -> DartFnFuture<Option<Contact>> + Send + 'static,
        connected: impl Fn() -> DartFnFuture<()> + Send + 'static,
        call_state: impl Fn(bool) -> DartFnFuture<()> + Send + 'static,
        session_status: impl Fn(String, String) -> DartFnFuture<()> + Send + 'static,
        start_sessions: impl Fn(AudioChat) -> DartFnFuture<()> + Send + 'static,
        load_ringtone: impl Fn() -> DartFnFuture<Option<Vec<u8>>> + Send + 'static,
        statistics: impl Fn(Statistics) -> DartFnFuture<()> + Send + 'static,
        message_received: impl Fn(String) -> DartFnFuture<()> + Send + 'static,
        manager_active: impl Fn(bool, bool) -> DartFnFuture<()> + Send + 'static,
    ) -> AudioChat {
        let key_bytes: [u8; 32] = signing_key.try_into().unwrap();

        let (start_session, start) = unbounded_async::<[u8; 32]>();

        let chat = Self {
            host,
            rms_threshold: Default::default(),
            input_volume: Default::default(),
            output_volume: Default::default(),
            denoise: Default::default(),
            denoise_model: Default::default(),
            end_call: Default::default(),
            input_device: Default::default(),
            output_device: Default::default(),
            signing_key: Arc::new(RwLock::new(SigningKey::from(SecretKey::from(key_bytes)))),
            in_call: Default::default(),
            deafened: Default::default(),
            muted: Default::default(),
            play_custom_ringtones: Default::default(),
            session_states: Default::default(),
            time: Default::default(),
            start_session,
            restart_manager: Default::default(),
            network_config: network_config.clone(),
            accept_call: Arc::new(Mutex::new(accept_call)),
            call_ended: Arc::new(Mutex::new(call_ended)),
            get_contact: Arc::new(Mutex::new(get_contact)),
            connected: Arc::new(Mutex::new(connected)),
            call_state: Arc::new(Mutex::new(call_state)),
            session_status: Arc::new(Mutex::new(session_status)),
            start_sessions: Arc::new(Mutex::new(start_sessions)),
            load_ringtone: Arc::new(Mutex::new(load_ringtone)),
            statistics: Arc::new(Mutex::new(statistics)),
            message_received: Arc::new(Mutex::new(message_received)),
            manager_active: Arc::new(Mutex::new(manager_active)),
        };

        // start the time synchronization background thread
        spawn(synchronize(chat.time.clone()));

        // start the session manager
        let chat_clone = chat.clone();
        spawn(async move {
            // retry the session manager if it fails, but not too fast
            let rate_limiter = RateLimiter::new(Duration::from_millis(100));

            loop {
                while let Err(error) = rate_limiter
                    .throttle(|| async { chat_clone.session_manager(&start).await })
                    .await
                {
                    (chat_clone.manager_active.lock().await)(false, false).await;
                    error!("Session manager failed: {}", error);
                }

                debug!("Session manager waiting for restart signal");
                (chat_clone.manager_active.lock().await)(false, true).await;
                chat_clone.restart_manager.notified().await;
            }
        });

        // start the sessions
        (chat.start_sessions.lock().await)(chat.clone()).await;

        chat
    }

    /// Tries to start a session for a contact
    pub async fn start_session(&self, contact: &Contact) {
        debug!(
            "start_session called for {:?}",
            &contact.verifying_key[0..5]
        );

        (self.session_status.lock().await)(contact.id.clone(), "Connecting".to_string()).await;

        if self
            .start_session
            .send(contact.verifying_key)
            .await
            .is_err()
        {
            error!("start_session channel is closed");
        }
    }

    /// Attempts to start a call through an existing session
    pub async fn say_hello(&self, contact: &Contact) -> std::result::Result<(), DartError> {
        if let Some(state) = self.session_states.read().await.get(&contact.id) {
            state.start.notify_one();
            Ok(())
        } else {
            Err(String::from("No session found for contact").into())
        }
    }

    /// Ends the call (if there is one)
    #[frb(sync)]
    pub fn end_call(&self) {
        self.end_call.notify_one();
    }

    /// Restarts the session manager
    pub async fn restart_manager(&self) -> std::result::Result<(), DartError> {
        if self.in_call.load(Relaxed) {
            Err(ErrorKind::InCall.into())
        } else {
            self.restart_manager.notify_one();
            (self.start_sessions.lock().await)(self.clone()).await;
            Ok(())
        }
    }

    /// Sets the signing key (called when the profile changes)
    pub async fn set_signing_key(&self, key: Vec<u8>) -> std::result::Result<(), DartError> {
        let key: [u8; 32] = key.try_into().map_err(|_| ErrorKind::InvalidSigningKey)?;
        *self.signing_key.write().await = SigningKey::from(SecretKey::from(key));
        Ok(())
    }

    /// Stops a specific session (called when a contact is deleted)
    pub async fn stop_session(&self, contact: &Contact) {
        if let Some(state) = self.session_states.write().await.remove(&contact.id) {
            state.stop.notify_one();
        }
    }

    /// Blocks while an audio test is running
    pub async fn audio_test(&self) -> std::result::Result<(), DartError> {
        self.in_call.store(true, Relaxed);

        let result = self
            .call(None, None, None, None, None)
            .await
            .map_err(Error::from)
            .map_err(Into::into);

        self.in_call.store(false, Relaxed);

        result
    }

    /// Sends a chat message
    pub async fn send_chat(
        &self,
        message: String,
        id: String,
    ) -> std::result::Result<(), DartError> {
        if let Some(state) = self.session_states.read().await.get(&id) {
            let message = Message::chat(message);

            state.channel.0.send(message).await.map_err(Error::from)?;
        }

        Ok(())
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

    /// Changing the denoise flag will not affect the current call
    #[frb(sync)]
    pub fn set_denoise(&self, denoise: bool) {
        self.denoise.store(denoise, Relaxed);
    }

    #[frb(sync)]
    pub fn set_play_custom_ringtones(&self, play: bool) {
        self.play_custom_ringtones.store(play, Relaxed);
    }

    pub async fn set_input_device(&self, device: Option<String>) {
        *self.input_device.lock().await = device;
    }

    pub async fn set_output_device(&self, device: Option<String>) {
        *self.output_device.lock().await = device;
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

    pub async fn set_model(&self, model: Vec<u8>) -> std::result::Result<(), DartError> {
        let model = if !model.is_empty() {
            RnnModel::from_bytes(&model).ok_or(String::from("invalid model"))?
        } else {
            RnnModel::default()
        };

        *self.denoise_model.write().await = model;
        Ok(())
    }

    /// Starts new sessions and communicates with the matchmaker
    async fn session_manager(&self, start: &AsyncReceiver<[u8; 32]>) -> Result<()> {
        // the local verifying key
        let verifying_key = self.signing_key.read().await.verifying_key().to_bytes();

        let mut stream = TcpStream::connect(*self.network_config.match_maker.read().await).await?;
        let shared_secret = key_exchange(&mut stream).await?;

        // HKDF for the key derivation
        let hk = Hkdf::<Sha256>::new(Some(&common::SALT), shared_secret.as_bytes());

        // note; the client uses the reverse of the server's ciphers
        // stream send cipher
        let mut sr_cipher = common::crypto::cipher_factory(&hk, b"ss-key", b"ss-iv")?;
        // stream read cipher
        let mut ss_cipher = common::crypto::cipher_factory(&hk, b"sr-key", b"sr-iv")?;

        let mut transport = LengthDelimitedCodec::builder().new_framed(stream);

        // send the identity message, the server will allow further messages if the identity is accepted
        let identity = identity_factory(&self.time, &*self.signing_key.read().await).await?;
        write_message(&mut transport, identity, &mut ss_cipher).await?;

        // the server's identity is not verified here because we do not trust it

        // alerts the UI that the manager is active
        (self.manager_active.lock().await)(true, true).await;

        // sends messages from sessions to the session manager for dispatch to the matchmaker
        let (sender, receiver) = unbounded_async::<CommonMessage>();
        // maps messages from the session manager to the sessions
        let mut sender_map: HashMap<[u8; 32], AsyncSender<CommonMessage>> = Default::default();

        let mut ice_agents: HashMap<[u8; 32], Arc<Agent>> = HashMap::new();

        // controls the application level pings
        let mut interval = interval(Duration::from_secs(15));

        loop {
            let result = select! {
                _ = interval.tick() => {
                    let ping = CommonMessage::ping();
                    write_message(&mut transport, ping, &mut ss_cipher).await?;
                    continue;
                }
                _ = self.restart_manager.notified() => {
                    break Err(ErrorKind::ManagerRestarted.into());
                }
                result = receiver.recv() => {
                    write_message(&mut transport, result?, &mut ss_cipher).await?;
                    continue;
                }
                result = start.recv() => {
                    let public = result?;

                    if (self.get_contact.lock().await)(public).await.is_some() {
                        let agent = ice_factory(&self.network_config.stun_servers.read().await).await?;
                        let (local_ufrag, local_pwd) = agent.get_local_user_credentials().await;

                        let request = RequestSession::new(&local_ufrag, &local_pwd);
                        let message = CommonMessage::new(request.into(), &public, &verifying_key);
                        write_message(&mut transport, message, &mut ss_cipher).await?;

                        ice_agents.insert(public, agent);
                    } else {
                        warn!("No contact found for {:?}", &public[0..5]);
                    }

                    continue;
                }
                result = read_message::<CommonMessage, _, _>(&mut transport, &mut sr_cipher) => {
                    result
                }
            };

            match result {
                Ok(message) => {
                    let from: [u8; 32] = if let Ok(from) = message.from.clone().try_into() {
                        from
                    } else {
                        error!("Failed to parse public key");
                        continue;
                    };

                    match &message.message {
                        Some(common::items::message::Message::RequestSession(request)) => {
                            info!("Received request session from {:?}", &from[0..5]);

                            let contact = match (self.get_contact.lock().await)(from).await {
                                Some(contact) => contact,
                                None => {
                                    warn!("[RequestSession] No contact found for {:?}", from);
                                    continue;
                                }
                            };

                            let channel = unbounded_async::<CommonMessage>();

                            let agent =
                                ice_factory(&self.network_config.stun_servers.read().await).await?;
                            let (local_ufrag, local_pwd) = agent.get_local_user_credentials().await;
                            let contact_id = contact.id.clone();

                            self._start_session(
                                contact,
                                (sender.clone(), channel.1),
                                agent,
                                false,
                                (request.ufrag.clone(), request.pwd.clone()),
                                verifying_key,
                            )
                            .await;

                            // alert the UI that this session is connecting
                            (self.session_status.lock().await)(
                                contact_id,
                                "Connecting".to_string(),
                            )
                            .await;
                            // only update the sender map if the session starts
                            sender_map.insert(from, channel.0);

                            let outcome = RequestOutcome::success(&local_ufrag, &local_pwd);
                            let message = CommonMessage::new(outcome.into(), &from, &verifying_key);
                            write_message(&mut transport, message, &mut ss_cipher).await?;
                        }
                        Some(common::items::message::Message::RequestOutcome(outcome)) => {
                            info!("Received request outcome {:?}", message);

                            let contact;

                            if ice_agents.get(&from).is_none() {
                                warn!("No agent found for {:?}", &from[0..5]);
                                continue;
                            } else if let Some(x) = (self.get_contact.lock().await)(from).await {
                                contact = x;
                            } else {
                                warn!("[RequestOutcome] No contact found for {:?}", &from[0..5]);

                                continue;
                            }

                            if !outcome.success {
                                warn!(
                                    "RequestSession for {:?} failed because {:?}",
                                    &from[0..5],
                                    outcome.reason
                                );

                                (self.session_status.lock().await)(
                                    contact.id.clone(),
                                    "Inactive".to_string(),
                                )
                                .await;
                                continue;
                            }

                            let channel = unbounded_async::<CommonMessage>();
                            let agent = ice_agents.remove(&from).ok_or(ErrorKind::AgentNotFound)?;

                            self._start_session(
                                contact,
                                (sender.clone(), channel.1),
                                agent,
                                true,
                                (
                                    outcome.ufrag.clone().ok_or(ErrorKind::MissingCredentials)?,
                                    outcome.pwd.clone().ok_or(ErrorKind::MissingCredentials)?,
                                ),
                                verifying_key,
                            )
                            .await;

                            sender_map.insert(from, channel.0);
                        }
                        Some(common::items::message::Message::ServerError(error)) => {
                            info!("Received server error from {:?}", &from[0..5]);

                            match error.message.as_ref() {
                                "Session already exists" => break Ok(()),
                                _ => warn!("Server error: {:?}", error.message),
                            }
                        }
                        // ping messages are ignored
                        Some(common::items::message::Message::Ping(_)) => {}
                        // all other messages are intended for the sessions
                        _ => {
                            info!(
                                "Received session message {:?} from {:?}",
                                message,
                                &from[0..5]
                            );

                            if let Some(sender) = sender_map.get(&from) {
                                let warning = format!(
                                    "Failed to forward message {:?} to session {:?}",
                                    message,
                                    &from[0..5]
                                );

                                if sender.send(message).await.is_err() {
                                    warn!("{}", warning);
                                } else {
                                    debug!("Forwarded message to session {:?}", &from[0..5]);
                                }
                            } else {
                                warn!("No session found for {:?}", &from[0..5]);
                            }
                        }
                    }
                }
                Err(error) => match error.kind {
                    common::error::ErrorKind::TransportRecv => break Err(error.into()),
                    _ => error!("error receiving message {:?}", error),
                },
            }
        }
    }

    /// A wrapper which starts the session and registers it in the session states
    async fn _start_session(
        &self,
        contact: Contact,
        session_message_channel: (AsyncSender<CommonMessage>, AsyncReceiver<CommonMessage>),
        agent: Arc<Agent>,
        controlling: bool,
        remote_credentials: (String, String),
        local_verifying_key: [u8; 32],
    ) {
        let message_channel = unbounded_async::<Message>();

        // create the state and a clone of it for the session
        let state = Arc::new(SessionState::new(&message_channel));
        let state_clone = Arc::clone(&state);

        let mut states = self.session_states.write().await;

        // we assume here that any existing session is invalid if the remote wants a new session
        if let Some(state) = states.remove(&contact.id) {
            warn!("{} already has a session", contact.nickname);

            if state.in_call.load(Relaxed) {
                warn!("{} is in a call, terminating", contact.nickname);
                self.end_call.notify_one();
                self.in_call.store(false, Relaxed);
                (self.call_ended.lock().await)("Call terminated".to_string(), false).await;
            }

            state.terminate.notify_waiters();
        }

        states.insert(contact.id.clone(), state.clone());

        let chat_clone = self.clone();
        spawn(async move {
            let contact_clone = contact.clone();

            let session_future = chat_clone.session(
                contact,
                session_message_channel.clone(),
                message_channel,
                agent,
                controlling,
                remote_credentials,
                &state_clone,
            );

            let reason = select! {
                result = session_future => {
                    if let Err(error) = result {
                        error!("Session error for {}: {}", contact_clone.nickname, error);
                        error.to_string()
                    } else {
                        warn!("Session for {} ended", contact_clone.nickname);
                        "session ended".to_string()
                    }
                }
                _ = state_clone.terminate.notified() => {
                    warn!("Session for {} terminated", contact_clone.nickname);
                    return; // terminated sessions do not need cleanup
                }
            };

            let end_session = EndSession::new(&reason);
            let message = CommonMessage::new(
                end_session.into(),
                &contact_clone.verifying_key,
                &local_verifying_key,
            );

            // the error is ignored because the state must be cleaned up still
            _ = session_message_channel.0.send(message).await;

            // TODO ask the match maker to reconnect this session if an error occurred

            // cleanup
            chat_clone
                .session_states
                .write()
                .await
                .remove(&contact_clone.id);

            (chat_clone.session_status.lock().await)(contact_clone.id, "Inactive".to_string())
                .await;

            info!("Session for {} cleaned up", contact_clone.nickname);
        });
    }

    /// A session with a contact
    async fn session(
        &self,
        contact: Contact,
        // carries messages to and from the matchmaker
        session_channel: (AsyncSender<CommonMessage>, AsyncReceiver<CommonMessage>),
        // carries messages to and from the remote
        call_message_channel: (AsyncSender<Message>, AsyncReceiver<Message>),
        agent: Arc<Agent>,
        controlling: bool,
        credentials: (String, String),
        state: &Arc<SessionState>,
    ) -> Result<()> {
        let (message_sender, message_receiver) = session_channel;
        let stop: Arc<Notify> = Default::default();
        let (cancel_tx, cancel_rx) = mpsc::channel(1);

        let remote_public_key = contact.verifying_key;
        let local_public_key = self.signing_key.read().await.verifying_key().to_bytes();

        agent.on_candidate(Box::new(move |c| {
            let message_sender = message_sender.clone();

            Box::pin(async move {
                if let Some(candidate) = c {
                    info!("local candidate {}", candidate);

                    let candidate: Candidate = candidate.marshal().into();
                    let message =
                        CommonMessage::new(candidate.into(), &remote_public_key, &local_public_key);
                    _ = message_sender.send(message).await
                }
            })
        }));

        // this monitors the matchmaker connection for messages
        let agent_clone = agent.clone();
        let stop_clone = stop.clone();
        spawn(async move {
            while let Ok(message) = message_receiver.recv().await {
                match message.message {
                    Some(common::items::message::Message::Candidate(candidate)) => {
                        info!("remote candidate {:?}", candidate.candidate);

                        if let Ok(candidate) = unmarshal_candidate(&candidate.candidate) {
                            let c: Arc<dyn webrtc_ice::candidate::Candidate + Send + Sync> =
                                Arc::new(candidate);
                            if let Err(error) = agent_clone.add_remote_candidate(&c) {
                                error!("Failed to add remote candidate {:?}", error);
                            }
                        }
                    }
                    Some(common::items::message::Message::ServerError(error)) => {
                        warn!("received server error: {:?}", error);
                        break;
                    }
                    Some(common::items::message::Message::EndSession(_)) => {
                        info!("received end session message in session");
                        break;
                    }
                    _ => error!("session received unexpected message: {:?}", message),
                }
            }

            info!("session message receiver closed");
            stop_clone.notify_one();
        });

        let stop_clone = stop.clone();
        agent.on_connection_state_change(Box::new(move |c| {
            if c == ConnectionState::Failed {
                stop_clone.notify_one();
            }

            Box::pin(async move {})
        }));

        agent.on_selected_candidate_pair_change(Box::new(move |a, b| {
            warn!("selected candidate pair changed: {}:{}", a, b);
            Box::pin(async move {})
        }));

        agent.gather_candidates()?;

        let conn: Arc<dyn Conn + Send + Sync> = if controlling {
            agent.dial(cancel_rx, credentials.0, credentials.1).await?
        } else {
            agent
                .accept(cancel_rx, credentials.0, credentials.1)
                .await?
        };

        let config = Config {
            net_conn: conn,
            max_receive_buffer_size: 0,
            max_message_size: 0,
            name: String::from("audio-chat"),
        };

        let association = if controlling {
            Association::client(config).await?
        } else {
            Association::server(config).await?
        };

        info!("session associated successfully");

        let stream = if controlling {
            association
                .open_stream(0, PayloadProtocolIdentifier::Binary)
                .await?
        } else {
            association
                .accept_stream()
                .await
                .ok_or(ErrorKind::AcceptStream)?
        };

        let mut poll_stream = SendPollStream {
            poll_stream: PollStream::new(stream),
        };

        // perform the key exchange
        let shared_secret = key_exchange(&mut poll_stream).await?;

        let mut transport = LengthDelimitedCodec::builder().new_framed(poll_stream);

        // HKDF for the key derivation
        let hk = Hkdf::<Sha256>::new(Some(&common::SALT), shared_secret.as_bytes());

        // stream send cipher
        let ss_cipher = common::crypto::cipher_factory(&hk, b"ss-key", b"ss-iv")?;
        // stream read cipher
        let sr_cipher = common::crypto::cipher_factory(&hk, b"sr-key", b"sr-iv")?;
        // construct the stream cipher pair
        let mut stream_cipher = PairedCipher::new(ss_cipher, sr_cipher);

        // one client always has the ciphers reversed
        if controlling {
            stream_cipher.swap();
        }

        // send the identity message
        let identity = identity_factory(&self.time, &*self.signing_key.read().await).await?;
        write_message(&mut transport, identity, &mut stream_cipher.send_cipher).await?;

        // receive the identity message
        let identity: Identity =
            read_message(&mut transport, &mut stream_cipher.receive_cipher).await?;

        verify_identity(&self.time, &identity).await?;

        // alert the UI that this session is now connected
        (self.session_status.lock().await)(contact.id.clone(), "Connected".to_string()).await;

        // seeds the IV so subsequent calls in the same session are unique
        let mut i = 0;

        // controls application level pings
        let mut interval = interval(PING_INTERVAL);
        let mut last_ping = Instant::now();

        let result = loop {
            let future = async {
                debug!("[{}] session waiting for event", contact.nickname);

                select! {
                    _ = interval.tick() => {
                        if last_ping.elapsed() > SESSION_TIMEOUT {
                            return Err(ErrorKind::SessionTimeout.into());
                        }

                        let ping = Message::ping();
                        write_message(&mut transport, ping, &mut stream_cipher.send_cipher).await.map_err(Error::from)
                    }
                    result = read_message::<Message, _, _>(&mut transport, &mut stream_cipher.receive_cipher) => {
                        info!("read_message result: {:?}", result);

                        let message = result?;
                        let mut ringtone = None;

                        match message {
                            Message { message: Some(message::Message::Hello(message)) } => {
                                if !message.ringtone.is_empty() && self.play_custom_ringtones.load(Relaxed) {
                                    ringtone = Some(message.ringtone);
                                }
                            }
                            Message { message: Some(message::Message::Ping(_)) } => {
                                last_ping = Instant::now();
                                return Ok(());
                            }
                            _ => {
                                warn!("received unexpected {:?} from {}", message, contact.nickname);

                                // let the other client know we are not in a call
                                let goodbye = Message::goodbye_reason("Not in call".to_string());
                                write_message(&mut transport, goodbye, &mut stream_cipher.send_cipher).await.map_err(Error::from)?;

                                return Ok(());
                            },
                        }

                        state.in_call.store(true, Relaxed); // blocks the session from being restarted

                        if self.in_call.load(Relaxed) {
                            // do not accept another call if already in one
                            let busy = Message::busy();
                            write_message(&mut transport, busy, &mut stream_cipher.send_cipher).await.map_err(Error::from)?;
                        } else if (self.accept_call.lock().await)(contact.id.clone(), ringtone).await {
                            // respond with hello if the call is accepted
                            let hello = Message::hello(None);
                            write_message(&mut transport, hello, &mut stream_cipher.send_cipher).await?;

                            // start the handshake
                            self.handshake(&mut transport, controlling, &hk, &mut stream_cipher, &mut i, &call_message_channel, &association).await?;
                        } else {
                            // reject the call if not accepted
                            let reject = Message::reject();
                            write_message(&mut transport, reject, &mut stream_cipher.send_cipher).await.map_err(Error::from)?;
                        }

                        last_ping = Instant::now();
                        Ok(())
                    }
                    _ = state.start.notified() => {
                        state.in_call.store(true, Relaxed); // blocks the session from being restarted

                        // queries the other client for a call
                        let ringtone = (self.load_ringtone.lock().await)().await;
                        let hello = Message::hello(ringtone);
                        write_message(&mut transport, hello, &mut stream_cipher.send_cipher).await?;

                        let mut retries = 0;

                        while retries < 4 {
                            retries += 1;

                            let message = timeout(HELLO_TIMEOUT, read_message(&mut transport, &mut stream_cipher.receive_cipher)).await??;

                            // handles a variety of messages sent in response to Hello
                            match message {
                                Message { message: Some(message::Message::Hello(_)) } => {
                                    self.handshake(&mut transport, controlling, &hk, &mut stream_cipher, &mut i, &call_message_channel, &association).await?;
                                    break;
                                }
                                Message { message: Some(message::Message::Reject(_)) } => {
                                    (self.call_ended.lock().await)(format!("{} did not accept the call", contact.nickname), true).await;
                                    break;
                                },
                                Message { message: Some(message::Message::Busy(_)) } => {
                                    (self.call_ended.lock().await)(format!("{} is busy", contact.nickname), true).await;
                                    break;
                                }
                                // if we receive one of these messages we retry as there may still be a hello message
                                Message { message: Some(message::Message::Ping(_)) } | Message { message: Some(message::Message::LatencyTest(_)) } => (),
                                _ => {
                                    (self.call_ended.lock().await)(format!("Received an unexpected message from {}", contact.nickname), true).await;
                                    warn!("received unexpected {:?} from {} [stopped call process]", message, contact.nickname);
                                },
                            }
                        }

                        // prevent the session from timing out
                        last_ping = Instant::now();

                        Ok(())
                    }
                }
            };

            select! {
                // state will never notify while a call is active
                _ = state.stop.notified() => {
                    info!("session state stop notified for {}", contact.nickname);
                    break Ok(());
                },
                // if stop notifies while a call is active then the call fails
                _ = stop.notified() => {
                    let in_call = state.in_call.load(Relaxed);
                    info!("session local stop notified for {} in_call={}", contact.nickname, in_call);

                    if in_call {
                        (self.call_ended.lock().await)(String::from("Session failed"), false).await;
                    }

                    break Ok(());
                }
                result = future => {
                    if let Err(error) = result {
                        // prevent errors from causing the session to timeout unexpectedly
                        last_ping = Instant::now();

                        if state.in_call.load(Relaxed) {
                            (self.call_ended.lock().await)(error.to_string(), false).await;
                        }

                        match error.kind {
                            ErrorKind::KanalReceive(_) => break Err(error),
                            ErrorKind::Common(ref kind) => match kind {
                                // these errors indicate the transport has failed
                                common::error::ErrorKind::TransportSend | common::error::ErrorKind::TransportRecv => break Err(error),
                                _ => error!("Session error: {:?}", error),
                            }
                            ErrorKind::SessionTimeout => break Err(error),
                            _ => error!("Session error: {:?}", error),
                        }
                    }
                }
            }

            // the session is now safe to restart
            state.in_call.store(false, Relaxed);
        };

        // cleanup the ice agent
        _ = cancel_tx.send(()).await;

        if let Err(error) = agent.close().await {
            error!("Failed to close agent: {:?}", error);
        }

        info!("session for {} returning", contact.nickname);
        result
    }

    /// Gets everything ready for the call
    async fn handshake(
        &self,
        transport: &mut Transport<SendPollStream>,
        controlling: bool,
        hk: &Hkdf<Sha256>,
        stream_cipher: &mut PairedCipher<AesCipher>,
        iv_seed: &mut i64,
        message_channel: &(AsyncSender<Message>, AsyncReceiver<Message>),
        association: &Association,
    ) -> Result<()> {
        debug!("handshake running");

        // create the send and receive ciphers
        let send_cipher = cipher_factory(hk, b"send-key", &iv_seed.to_be_bytes())?;
        *iv_seed += 1;
        let receive_cipher = cipher_factory(hk, b"receive-key", &iv_seed.to_be_bytes())?;
        *iv_seed += 1;

        // create the cipher pair for data
        let mut data_cipher = PairedCipher::new(send_cipher, receive_cipher);

        if controlling {
            // the controlling peer always has the ciphers reversed
            data_cipher.swap();
        }

        let stream = if controlling {
            association
                .open_stream(*iv_seed as u16, PayloadProtocolIdentifier::Binary)
                .await?
        } else {
            association
                .accept_stream()
                .await
                .ok_or(ErrorKind::AcceptStream)?
        };

        // sending a byte through the stream here causes it to open correctly
        if controlling {
            stream.write(&Bytes::copy_from_slice(&[0])).await?;
        } else {
            let mut buffer = [0; 1];
            stream.read(&mut buffer).await?;
        }

        stream.set_reliability_params(true, ReliabilityType::Rexmit, 0);

        spawn(send_timestamps(
            message_channel.0.clone(),
            Arc::clone(&self.time),
        ));

        // alert the UI that the call has connected
        (self.connected.lock().await)().await;
        self.in_call.store(true, Relaxed);

        let result = self
            .call(
                Some(Arc::clone(&stream)),
                Some(transport),
                Some(stream_cipher),
                Some(data_cipher),
                Some(message_channel.1.clone()),
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
                    let message = Message::goodbye_reason("Audio device error".to_string());
                    write_message(transport, message, &mut stream_cipher.send_cipher).await?;
                    Err(error)
                }
                _ => {
                    let message = Message::goodbye_reason(error.to_string());
                    write_message(transport, message, &mut stream_cipher.send_cipher).await?;
                    Err(error)
                }
            },
        }
    }

    /// The bulk of the call logic
    async fn call(
        &self,
        stream: Option<Arc<webrtc_sctp::stream::Stream>>,
        mut transport: Option<&mut Transport<SendPollStream>>,
        mut stream_cipher: Option<&mut PairedCipher<AesCipher>>,
        data_cipher: Option<PairedCipher<AesCipher>>,
        message_receiver: Option<AsyncReceiver<Message>>,
    ) -> Result<()> {
        // if any of the values required for a normal call is missing, the call is an audio test
        let audio_test = transport.is_none()
            || stream.is_none()
            || message_receiver.is_none()
            || data_cipher.is_none()
            || stream_cipher.is_none();

        // the denoise flag is constant for the entire call
        let denoise = self.denoise.load(Relaxed);

        // the number of frames to hold in a channel
        let framed_size = CHANNEL_SIZE / FRAME_SIZE;

        // sends messages from the input processor to the sending socket
        let (processed_input_sender, processed_input_receiver) =
            bounded_async::<ProcessorMessage>(framed_size);
        // sends raw data from the receiving socket to the output processor
        let (output_sender, output_receiver) = bounded_async::<ProcessorMessage>(framed_size);
        // sends samples from the output processor to the output stream
        let (processed_output_sender, processed_output_receiver) = bounded::<f32>(CHANNEL_SIZE);
        // sends samples from the input to the input processor
        let (input_sender, input_receiver) = bounded::<f32>(CHANNEL_SIZE);

        // channels used for moving values to the statistics collector
        let (input_rms_sender, input_rms_receiver) = unbounded_async::<f32>();
        let (output_rms_sender, output_rms_receiver) = unbounded_async::<f32>();
        let latency: Arc<AtomicUsize> = Default::default();
        let upload_bandwidth: Arc<AtomicUsize> = Default::default();
        let download_bandwidth: Arc<AtomicUsize> = Default::default();

        // get the output device and its default configuration
        let output_device = get_output_device(&self.output_device, &self.host).await?;
        let output_config = output_device.default_output_config()?;
        info!("output device: {:?}", output_device.name());

        // get the input device and its default configuration
        let input_device = self.get_input_device().await?;
        let input_config = input_device.default_input_config()?;
        info!("input_device: {:?}", input_device.name());

        let mut audio_header = AudioHeader::from(&input_config);

        // rnnoise requires a 48kHz sample rate
        if denoise {
            audio_header.sample_rate = 48_000;
        }

        let remote_input_config: AudioHeader = if audio_test {
            // the client will be receiving its own audio
            audio_header
        } else {
            let (send_cipher, receive_cipher) = stream_cipher.as_mut().unwrap().mut_parts();
            let transport = transport.as_mut().unwrap();

            info!("sending audio header");
            write_message(transport, audio_header, send_cipher).await?;
            read_message(transport, receive_cipher).await?
        };

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
        // the rnnoise denoiser
        let denoiser = denoise.then_some(DenoiseState::from_model(
            self.denoise_model.read().await.clone(),
        ));

        // spawn the input processor thread
        let input_processor_handle = std::thread::spawn(move || {
            input_processor(
                input_receiver,
                processed_input_sender,
                sample_rate,
                input_volume,
                rms_threshold,
                muted,
                denoiser,
                input_rms_sender.to_sync(),
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
                output_rms_sender.to_sync(),
            )
        });

        let input_channels = input_config.channels() as usize;
        let end_call = Arc::clone(&self.end_call);

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
                    end_call.notify_one();
                },
                None,
            )?,
        };

        // get the output channels for chunking the output
        let output_channels = output_config.channels() as usize;

        // a cached reference to the flag for use in the output callback
        let mut deafened = CachedAtomicFlag::new(&self.deafened);
        let end_call = Arc::clone(&self.end_call);

        let output_stream = SendStream {
            stream: output_device.build_output_stream(
                &output_config.into(),
                move |output: &mut [f32], _: &_| {
                    if deafened.load() {
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
                    end_call.notify_one();
                },
                None,
            )?,
        };

        // play the streams
        input_stream.stream.play()?;
        output_stream.stream.play()?;

        let stop_io = Arc::new(Notify::new());

        // shared values used in the call controller
        let end_call = Arc::clone(&self.end_call);
        let time = Arc::clone(&self.time);

        let input_processor_future = spawn_blocking_with(
            move || input_processor_handle.join(),
            FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
        );

        let output_processor_future = spawn_blocking_with(
            move || output_processor_handle.join(),
            FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
        );

        let statistics_handle = spawn(statistics_collector(
            input_rms_receiver,
            output_rms_receiver,
            Arc::clone(&latency),
            Arc::clone(&upload_bandwidth),
            Arc::clone(&download_bandwidth),
            Arc::clone(&self.statistics),
            Arc::clone(&stop_io),
        ));

        let result = if audio_test {
            let loopback_handle = spawn(loopback(
                processed_input_receiver,
                output_sender,
                Arc::clone(&stop_io),
            ));

            select! {
                _ = loopback_handle => Ok(()),
                // this unwrap is safe because the processor thread will not panic
                result = output_processor_future => result?.unwrap(),
                // this unwrap is safe because the processor thread will not panic
                result = input_processor_future => result?.unwrap(),
                _ = self.end_call.notified() => Ok(()),
                result = statistics_handle => result?,
            }
        } else {
            let (send_cipher, receive_cipher) = data_cipher.unwrap().into_parts();
            let stream = stream.unwrap();

            let input_handle = spawn(socket_input(
                processed_input_receiver,
                Arc::clone(&stream),
                send_cipher,
                Arc::clone(&stop_io),
                Arc::clone(&upload_bandwidth),
            ));

            let output_handle = spawn(socket_output(
                output_sender,
                Arc::clone(&stream),
                receive_cipher,
                Arc::clone(&stop_io),
                Arc::clone(&download_bandwidth),
            ));

            let cipher = stream_cipher.unwrap();
            let transport = transport.as_mut().unwrap();
            let message_receiver = message_receiver.unwrap();

            let message_received = Arc::clone(&self.message_received);
            let call_state = Arc::clone(&self.call_state);

            select! {
                result = input_handle => result?,
                _ = output_handle => Ok(()),
                // this unwrap is safe because the processor thread will not panic
                result = output_processor_future => result?.unwrap(),
                // this unwrap is safe because the processor thread will not panic
                result = input_processor_future => result?.unwrap(),
                result = statistics_handle => result?,
                message = call_controller(transport, cipher, message_receiver, end_call, time, latency, message_received, call_state) => {
                    debug!("controller exited: {:?}", message);

                    match message {
                        Ok(message) => (self.call_ended.lock().await)(message, true).await,
                        Err(error) => (self.call_ended.lock().await)(error.to_string(), false).await,
                    }

                    Ok(())
                },
            }
        };

        // ensure that the input and output streams are stopped
        stop_io.notify_waiters();

        result
    }

    /// Returns either the default or the user specified device
    async fn get_input_device(&self) -> Result<Device> {
        match *self.input_device.lock().await {
            Some(ref name) => Ok(self
                .host
                .input_devices()?
                .find(|device| {
                    if let Ok(ref device_name) = device.name() {
                        name == device_name
                    } else {
                        false
                    }
                })
                .unwrap_or(
                    self.host
                        .default_input_device()
                        .ok_or(ErrorKind::NoInputDevice)?,
                )),
            None => self
                .host
                .default_input_device()
                .ok_or(ErrorKind::NoInputDevice.into()),
        }
    }
}

/// Wraps a cpal stream to unsafely make it send
pub(crate) struct SendStream {
    pub(crate) stream: Stream,
}

unsafe impl Send for SendStream {}

// TODO yikers
struct SendPollStream {
    poll_stream: PollStream,
}

unsafe impl Send for SendPollStream {}

impl AsyncRead for SendPollStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        Pin::new(&mut self.poll_stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for SendPollStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        Pin::new(&mut self.poll_stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        Pin::new(&mut self.poll_stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        Pin::new(&mut self.poll_stream).poll_shutdown(cx)
    }
}

/// A message containing either a frame of audio or silence
enum ProcessorMessage {
    Data(Box<TransferBuffer>),
    Silence,
}

// common message constructors
impl ProcessorMessage {
    fn data(bytes: &[u8]) -> Result<Self> {
        Ok(Self::Data(Box::new(TransferBuffer::try_from(bytes)?)))
    }

    fn silence() -> Self {
        Self::Silence
    }

    fn frame(frame: TransferBuffer) -> Self {
        Self::Data(Box::new(frame))
    }
}

/// Keeps track of the active sessions
struct SessionState {
    /// Signals the session to initiate a call
    start: Notify,

    /// Stops the session normally
    stop: Notify,

    /// Stops the session when it is being replaced
    terminate: Notify,

    /// If the session is in a call
    in_call: AtomicBool,

    /// A reusable channel used to send and receive messages while a call is active
    channel: (AsyncSender<Message>, AsyncReceiver<Message>),
}

impl SessionState {
    fn new(message_channel: &(AsyncSender<Message>, AsyncReceiver<Message>)) -> Self {
        Self {
            start: Notify::new(),
            stop: Notify::new(),
            terminate: Notify::new(),
            in_call: AtomicBool::new(false),
            channel: message_channel.clone(),
        }
    }
}

/// An AtomicBool Flag which is not loaded from the atomic every time
struct CachedAtomicFlag {
    counter: i32,
    cache: bool,
    atomic: Arc<AtomicBool>,
}

impl CachedAtomicFlag {
    fn new(atomic: &Arc<AtomicBool>) -> Self {
        Self {
            counter: 0,
            cache: atomic.load(Relaxed),
            atomic: Arc::clone(atomic),
        }
    }

    fn load(&mut self) -> bool {
        if self.counter % 100 == 0 {
            self.cache = self.atomic.load(Relaxed);
        }

        self.counter += 1;
        self.cache
    }
}

struct CachedAtomicFloat {
    counter: i32,
    cache: f32,
    atomic: Arc<AtomicF32>,
}

impl CachedAtomicFloat {
    fn new(atomic: &Arc<AtomicF32>) -> Self {
        Self {
            counter: 0,
            cache: atomic.load(Relaxed),
            atomic: Arc::clone(atomic),
        }
    }

    fn load(&mut self) -> f32 {
        if self.counter % 100 == 0 {
            self.cache = self.atomic.load(Relaxed);
        }

        self.counter += 1;
        self.cache
    }
}

/// Processed statistics for the frontend
#[derive(Default)]
pub struct Statistics {
    /// a percentage of the max input volume in the window
    pub input_level: f32,

    /// a percentage of the max output volume in the window
    pub output_level: f32,

    /// the current call latency
    pub latency: usize,

    /// the approximate upload bandwidth used by the current call
    pub upload_bandwidth: usize,

    /// the approximate download bandwidth used by the current call
    pub download_bandwidth: usize,
}

#[frb(opaque)]
#[derive(Clone)]
pub struct NetworkConfig {
    /// The STUN servers to use for NAT traversal
    stun_servers: Arc<RwLock<Vec<Url>>>,

    /// The match-maker server to use for signaling
    match_maker: Arc<RwLock<SocketAddr>>,
}

impl NetworkConfig {
    #[frb(sync)]
    pub fn new(
        stun_servers: Vec<String>,
        match_maker: String,
    ) -> std::result::Result<Self, DartError> {
        Ok(Self {
            stun_servers: Arc::new(RwLock::new(
                stun_servers
                    .into_iter()
                    .filter_map(|server| Url::parse_url(&server).ok())
                    .collect(),
            )),
            match_maker: Arc::new(RwLock::new(
                match_maker
                    .to_socket_addrs()
                    .map_err(|_| String::from("invalid address"))?
                    .next()
                    .ok_or(String::from("no address found"))?,
            )),
        })
    }

    // pub fn add_url(&mut self, url: String) -> std::result::Result<(), DartError> {
    //     self.stun_servers.push(Url::parse_url(&url).map_err(|_| String::from("invalid url"))?);
    //     Ok(())
    // }
    //
    // pub fn remove_url(&mut self, url: String) -> std::result::Result<(), DartError> {
    //     self.stun_servers.retain(|server| server.to_string() != url);
    //     Ok(())
    // }
}

async fn call_controller<C: StreamCipher + StreamCipherSeek>(
    transport: &mut Transport<SendPollStream>,
    cipher: &mut PairedCipher<C>,
    receiver: AsyncReceiver<Message>,
    end_call: Arc<Notify>,
    time: Time,
    latency: Arc<AtomicUsize>,
    message_received: Arc<Mutex<dyn Fn(String) -> DartFnFuture<()> + Send>>,
    disconnected_callback: Arc<Mutex<dyn Fn(bool) -> DartFnFuture<()> + Send>>,
) -> Result<String> {
    let mut ping_interval = interval(PING_INTERVAL);
    let mut last_ping = Instant::now();
    let mut next_ping = Instant::now() + PING_INTERVAL + RECEIVE_TIMEOUT;

    let mut disconnected = false;

    loop {
        select! {
            // receives and handles messages from the callee
            result = read_message(transport, &mut cipher.receive_cipher) => {
                info!("call controller read_message result: {:?}", result);

                let message: Message = result?;

                if disconnected {
                    disconnected = false;
                    (disconnected_callback.lock().await)(disconnected).await;
                }

                match message.message {
                    Some(message::Message::Goodbye(message)) => {
                        debug!("received goodbye, reason = {:?}", message.reason);
                        break Ok(message.reason);
                    },
                    Some(message::Message::LatencyTest(message)) => {
                        // get the current timestamp and remote
                        let current_timestamp = time.lock().await.current_timestamp();

                        if current_timestamp > message.timestamp {
                            // calculate the latency
                            let delta_ms = current_timestamp - message.timestamp;
                            latency.store(delta_ms as usize, Relaxed);
                        } else {
                            warn!("received invalid latency test (ping from future)");
                        }
                    }
                    Some(message::Message::Chat(message)) => {
                        (message_received.lock().await)(message.message).await;
                    }
                    Some(message::Message::Ping(_)) => {
                        let delta = Instant::now() - last_ping;

                        last_ping = Instant::now();
                        next_ping = last_ping + delta + RECEIVE_TIMEOUT;
                    },
                    _ => error!("received unexpected message: {:?}", message),
                }
            },
            // sends messages to the callee
            result = receiver.recv() => {
                if let Ok(message) = result {
                    write_message(transport, message, &mut cipher.send_cipher).await?;
                } else {
                    // if the channel closes, the call has ended
                    break Ok(String::new());
                }
            },
            // ends the call
            _ = end_call.notified() => {
                let message = Message::goodbye();
                write_message(transport, message, &mut cipher.send_cipher).await?;
                break Ok(String::new());
            },
            // sends normal pings and checks for timeouts
            _ = ping_interval.tick() => {
                if last_ping.elapsed() > SESSION_TIMEOUT {
                    warn!("ping timeout in call controller");
                    break Ok(String::from("Ping timeout"));
                }

                let message = Message::ping();
                write_message(transport, message, &mut cipher.send_cipher).await?;
            },
            // handles temporary disconnects
            _ = sleep(next_ping - Instant::now()) => {
                warn!("ping timeout in call controller");

                // prevent an infinite loop of ping timeouts
                next_ping = Instant::now() + PING_INTERVAL + RECEIVE_TIMEOUT;

                if !disconnected {
                    disconnected = true;
                    (disconnected_callback.lock().await)(disconnected).await;
                }
            },
        }
    }
}

/// Receives frames of audio data from the input processor and sends them to the socket
async fn socket_input<C: StreamCipher>(
    input_receiver: AsyncReceiver<ProcessorMessage>,
    socket: Arc<webrtc_sctp::stream::Stream>,
    mut cipher: C,
    stop: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
) -> Result<()> {
    let mut byte_buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut sequence_number = 0_u64;

    let silence = Bytes::from_static(&[0]);

    let future = async {
        while let Ok(message) = input_receiver.recv().await {
            match message {
                ProcessorMessage::Silence => {
                    // send the silence signal
                    socket.write(&silence).await?;
                    bandwidth.fetch_add(9, Relaxed); // udp header + 1 byte
                }
                ProcessorMessage::Data(bytes) => {
                    // encrypt the audio data (unwrap is safe because we know the buffer is the correct length)
                    cipher
                        .apply_keystream_b2b(bytes.as_slice(), &mut byte_buffer[8..])
                        .unwrap();

                    // add the sequence number to the buffer
                    byte_buffer[..8].copy_from_slice(&sequence_number.to_be_bytes());
                    sequence_number += TRANSFER_BUFFER_SIZE as u64; // increment the sequence number

                    // send the bytes to the socket
                    socket.write(&Bytes::copy_from_slice(&byte_buffer)).await?;
                    bandwidth.fetch_add(PACKET_SIZE, Relaxed); // udp header + sequence number + TRANSFER_BUFFER_SIZE
                }
            }
        }

        Ok::<(), Error>(())
    };

    select! {
        result = future => result,
        _ = stop.notified() => {
            debug!("Input to socket ended");
            Ok(())
        },
    }
}

/// Receives audio data from the socket and sends it to the output processor
async fn socket_output<C: StreamCipher + StreamCipherSeek>(
    sender: AsyncSender<ProcessorMessage>,
    socket: Arc<webrtc_sctp::stream::Stream>,
    mut cipher: C,
    notify: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
) {
    let mut in_buffer = [0; TRANSFER_BUFFER_SIZE + 8];
    let mut out_buffer = [0; TRANSFER_BUFFER_SIZE];

    let future = async {
        loop {
            match socket.read(&mut in_buffer).await {
                // normal chunk of audio data
                Ok(len) if len == TRANSFER_BUFFER_SIZE + 8 => {
                    bandwidth.fetch_add(PACKET_SIZE, Relaxed); // udp header + sequence number + TRANSFER_BUFFER_SIZE

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

                    _ = sender.try_send(ProcessorMessage::frame(out_buffer));
                }
                // control signals
                Ok(1) => {
                    bandwidth.fetch_add(9, Relaxed); // udp header + 1 byte

                    match in_buffer[0] {
                        0 => _ = sender.try_send(ProcessorMessage::silence()), // silence
                        _ => error!("received unknown control signal {}", in_buffer[0]),
                    }
                }
                // malformed packets (never happens)
                Ok(len) => error!("Received {} < {} data", len, TRANSFER_BUFFER_SIZE + 8),
                // socket errors
                Err(error) => {
                    error!("error receiving: {}", error);
                }
            }
        }
    };

    select! {
        _ = future => {},
        _ = notify.notified() => {
            debug!("Socket output ended");
        },
    }
}

/// Used for audio tests, plays the input into the output
async fn loopback(
    input_receiver: AsyncReceiver<ProcessorMessage>,
    output_sender: AsyncSender<ProcessorMessage>,
    notify: Arc<Notify>,
) {
    let loopback_future = async {
        while let Ok(message) = input_receiver.recv().await {
            _ = output_sender.try_send(message);
        }
    };

    select! {
        result = loopback_future => result,
        _ = notify.notified() => {
            debug!("Loopback ended");
        },
    }
}

/// Sends a timestamp to the other client every 10 seconds for latency testing
async fn send_timestamps(sender: AsyncSender<Message>, time: Time) {
    // a message is produced every 10 seconds
    let mut interval = interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        let timestamp = time.lock().await.current_timestamp();
        let message = Message::latency_test(timestamp);

        // stop sending when the channel closes
        if sender.send(message).await.is_err() {
            debug!("timestamp sender closing");
            break;
        }
    }
}

/// Collects statistics from throughout the application, processes them, and provides them to the frontend
async fn statistics_collector(
    input_receiver: AsyncReceiver<f32>,
    output_receiver: AsyncReceiver<f32>,
    latency: Arc<AtomicUsize>,
    upload_bandwidth: Arc<AtomicUsize>,
    download_bandwidth: Arc<AtomicUsize>,
    callback: Arc<Mutex<dyn Fn(Statistics) -> DartFnFuture<()> + Send>>,
    notify: Arc<Notify>,
) -> Result<()> {
    let mut interval = interval(Duration::from_millis(100));
    let mut input_max = 0_f32;
    let mut output_max = 0_f32;

    let future = async {
        loop {
            interval.tick().await;

            let statistics = Statistics {
                input_level: level_from_window(&input_receiver, &mut input_max).await,
                output_level: level_from_window(&output_receiver, &mut output_max).await,
                latency: latency.load(Relaxed),
                upload_bandwidth: upload_bandwidth.load(Relaxed),
                download_bandwidth: download_bandwidth.load(Relaxed),
            };

            (callback.lock().await)(statistics).await;
        }
    };

    let result = select! {
        result = future => result,
        _ = notify.notified() => {
            debug!("Statistics collector ended");
            Ok(())
        }
    };

    // zero out the statistics when the collector ends
    let statistics = Statistics::default();
    (callback.lock().await)(statistics).await;

    result
}

// TODO consider using a SincFixedOut resampler here
/// Processes the audio input and sends it to the sending socket
fn input_processor(
    receiver: Receiver<f32>,
    sender: Sender<ProcessorMessage>,
    sample_rate: f64,
    input_factor: Arc<AtomicF32>,
    rms_threshold: Arc<AtomicF32>,
    muted: Arc<AtomicBool>,
    mut denoiser: Option<Box<DenoiseState>>,
    rms_sender: Sender<f32>,
) -> Result<()> {
    // the maximum and minimum values for i16 as f32
    let max_i16_f32 = i16::MAX as f32;
    let min_i16_f32 = i16::MIN as f32;

    let ratio = if denoiser.is_some() {
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

    // the input for the resampler
    let mut pre_buf = [vec![0_f32; in_len]];
    // the output for the resampler
    let mut post_buf = [vec![0_f32; post_len]];
    // the output for rnnoise
    let mut out_buf = [0_f32; FRAME_SIZE];

    // output for 16 bit samples. the compiler does not recognize that it is used
    #[allow(unused_assignments)]
    let mut int_buffer = [0; FRAME_SIZE];

    // the position in pre_buf
    let mut position = 0;
    // a counter user for short silence detection
    let mut silence_length = 0_u8;

    let mut muted = CachedAtomicFlag::new(&muted);
    let mut rms_threshold = CachedAtomicFloat::new(&rms_threshold);
    let mut input_factor = CachedAtomicFloat::new(&input_factor);

    while let Ok(sample) = receiver.recv() {
        // sends a silence signal for every FRAME_SIZE samples if the input is muted
        if muted.load() {
            if position > FRAME_SIZE {
                position = 0;
                _ = sender.try_send(ProcessorMessage::silence());
            } else {
                position += 1;
            }

            continue;
        }

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
        let factor = max_i16_f32 * input_factor.load();

        // rescale the samples to -32768.0 to 32767.0 for rnnoise
        target_buffer.par_iter_mut().for_each(|x| {
            *x *= factor;
            *x = x.trunc().clamp(min_i16_f32, max_i16_f32);
        });

        if let Some(ref mut denoiser) = denoiser {
            // denoise the frame
            denoiser.process_frame(&mut out_buf, &target_buffer[..len]);
        } else {
            out_buf = target_buffer[..len].try_into()?;
        };

        // calculate the rms
        let rms = calculate_rms(&out_buf);
        rms_sender.send(rms)?; // send the rms to the statistics collector

        // check if the frame is below the rms threshold
        if rms < rms_threshold.load() {
            if silence_length < 80 {
                silence_length += 1; // short silences are ignored
            } else {
                _ = sender.try_send(ProcessorMessage::silence());
                continue;
            }
        } else {
            silence_length = 0;
        }

        // cast the f32 samples to i16
        int_buffer = out_buf.map(|x| x as i16);

        // convert the i16 samples to bytes
        let bytes = unsafe {
            std::slice::from_raw_parts(
                int_buffer.as_ptr() as *const u8,
                int_buffer.len() * mem::size_of::<i16>(),
            )
        };

        _ = sender.try_send(ProcessorMessage::data(bytes)?);
    }

    debug!("Input processor ended");
    Ok(())
}

/// Processes the audio data and sends it to the output stream
fn output_processor(
    receiver: Receiver<ProcessorMessage>,
    sender: Sender<f32>,
    ratio: f64,
    output_volume: Arc<AtomicF32>,
    rms_sender: Sender<f32>,
) -> Result<()> {
    let max_i16_f32 = i16::MAX as f32;

    let mut resampler = resampler_factory(ratio, 1, FRAME_SIZE)?;

    // rubato requires 10 extra spaces in the output buffer as a safety margin
    let post_len = (FRAME_SIZE as f64 * ratio + 10_f64) as usize;

    // the input for the resampler
    let mut pre_buf = [&mut [0_f32; FRAME_SIZE]];
    // the output for the resampler
    let mut post_buf = [vec![0_f32; post_len]];

    // avoids checking the state in every iteration
    let mut output_volume = CachedAtomicFloat::new(&output_volume);

    while let Ok(message) = receiver.recv() {
        match message {
            ProcessorMessage::Silence => {
                for _ in 0..FRAME_SIZE {
                    sender.try_send(0_f32)?;
                }
            }
            ProcessorMessage::Data(bytes) => {
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
                mul(pre_buf[0], output_volume.load());

                let rms = calculate_rms(pre_buf[0]);
                rms_sender.send(rms)?;

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
    10_f32.powf(db / 20_f32)
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
        Some(ref name) => Ok(host
            .output_devices()?
            .find(|device| {
                if let Ok(ref device_name) = device.name() {
                    name == device_name
                } else {
                    false
                }
            })
            .unwrap_or(
                host.default_output_device()
                    .ok_or(ErrorKind::NoOutputDevice)?,
            )),
        None => host
            .default_output_device()
            .ok_or(ErrorKind::NoOutputDevice.into()),
    }
}

/// Creates an ICE agent
async fn ice_factory(urls: &[Url]) -> Result<Arc<Agent>> {
    let udp_network = UDPNetwork::Ephemeral(Default::default());

    Ok(Arc::new(
        Agent::new(AgentConfig {
            urls: urls.to_vec(),
            network_types: vec![NetworkType::Udp4, NetworkType::Udp6],
            udp_network,
            ..Default::default()
        })
        .await?,
    ))
}

/// Returns the percentage of the max input volume in the window compared to the max volume
async fn level_from_window(receiver: &AsyncReceiver<f32>, max: &mut f32) -> f32 {
    let mut window = Vec::new();

    while let Ok(Some(rms)) = receiver.try_recv() {
        window.push(rms);
    }

    let level = if window.is_empty() {
        0_f32
    } else {
        let local_max = window.iter().cloned().fold(0_f32, f32::max);
        *max = max.max(local_max);

        if *max != 0_f32 {
            local_max / *max
        } else {
            0_f32
        }
    };

    if level < 0.01 {
        0_f32
    } else {
        level
    }
}
