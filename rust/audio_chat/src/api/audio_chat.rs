use std::collections::{HashMap, VecDeque};
use std::mem;
use std::net::Ipv4Addr;
pub use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::time::Duration;

use async_throttle::RateLimiter;
use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use cpal::Host;
use cpal::{Device, Stream as CpalStream};
use flutter_rust_bridge::for_generated::futures::stream::{SplitSink, SplitStream};
use flutter_rust_bridge::for_generated::futures::{Sink, SinkExt};
use flutter_rust_bridge::{frb, spawn, spawn_blocking_with, DartFnFuture};
use kanal::{
    bounded, bounded_async, unbounded_async, AsyncReceiver, AsyncSender, Receiver, Sender,
};
use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::{ConnectionId, SwarmEvent};
use libp2p::{
    autonat, dcutr, identify, noise, ping, tcp, yamux, Multiaddr, PeerId, Stream, StreamProtocol,
};
use libp2p_stream::Control;
use log::{debug, error, info, warn};
use nnnoiseless::{DenoiseState, RnnModel, FRAME_SIZE};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::lookup_host;
use tokio::select;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::{interval, sleep_until, timeout, Instant};
use tokio_util::bytes::Bytes;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};

use crate::api::contact::Contact;
use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::items::{message, AudioHeader, Message};
use crate::api::overlay::overlay::Overlay;
use crate::api::overlay::{CONNECTED, LATENCY, LOSS};
use crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER;
use crate::{Behaviour, BehaviourEvent};

type Result<T> = std::result::Result<T, Error>;
pub(crate) type DeviceName = Arc<Mutex<Option<String>>>;
type TransportStream = Compat<Stream>;
pub type Transport<T> = Framed<T, LengthDelimitedCodec>;

/// The number of bytes in a single network audio frame
const TRANSFER_BUFFER_SIZE: usize = FRAME_SIZE * mem::size_of::<i16>();
/// Parameters used for resampling throughout the application
const RESAMPLER_PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: WindowFunction::BlackmanHarris2,
};
/// A timeout used when initializing the call
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
/// A timeout used to detect temporary network issues
const TIMEOUT_DURATION: Duration = Duration::from_millis(100);
/// the number of frames to hold in a channel
const CHANNEL_SIZE: usize = 2_400;
/// the protocol identifier for audio chat
const PROTOCOL: StreamProtocol = StreamProtocol::new("/audio-chat/0.0.1");

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
    identity: Arc<RwLock<Keypair>>,

    /// Keeps track of whether the user is in a call
    in_call: Arc<AtomicBool>,

    /// Disables the output stream
    deafened: Arc<AtomicBool>,

    /// Disables the input stream
    muted: Arc<AtomicBool>,

    /// Disables the playback of custom ringtones
    play_custom_ringtones: Arc<AtomicBool>,

    /// Keeps track of and controls the sessions
    session_states: Arc<RwLock<HashMap<PeerId, Arc<SessionState>>>>,

    /// Signals the session manager to start a new session
    start_session: AsyncSender<PeerId>,

    /// Restarts the session manager when needed
    restart_manager: Arc<Notify>,

    /// Network configuration for p2p connections
    network_config: NetworkConfig,

    /// A reference to the object that controls the call overlay
    overlay: Overlay,

    /// Prompts the user to accept a call
    accept_call:
        Arc<Mutex<dyn Fn(String, Option<Vec<u8>>, DartNotify) -> DartFnFuture<bool> + Send>>,

    /// Alerts the UI that a call has ended
    call_ended: Arc<Mutex<dyn Fn(String, bool) -> DartFnFuture<()> + Send>>,

    /// Fetches a contact from the front end
    get_contact: Arc<Mutex<dyn Fn(Vec<u8>) -> DartFnFuture<Option<Contact>> + Send>>,

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

    /// Called when the backend is starting a call on its own
    call_started: Arc<Mutex<dyn Fn(Contact) -> DartFnFuture<()> + Send>>,
}

impl AudioChat {
    // this function must be async to use `spawn`
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        identity: Vec<u8>,
        host: Arc<Host>,
        network_config: &NetworkConfig,
        overlay: &Overlay,
        accept_call: impl Fn(String, Option<Vec<u8>>, DartNotify) -> DartFnFuture<bool> + Send + 'static,
        call_ended: impl Fn(String, bool) -> DartFnFuture<()> + Send + 'static,
        get_contact: impl Fn(Vec<u8>) -> DartFnFuture<Option<Contact>> + Send + 'static,
        connected: impl Fn() -> DartFnFuture<()> + Send + 'static,
        call_state: impl Fn(bool) -> DartFnFuture<()> + Send + 'static,
        session_status: impl Fn(String, String) -> DartFnFuture<()> + Send + 'static,
        start_sessions: impl Fn(AudioChat) -> DartFnFuture<()> + Send + 'static,
        load_ringtone: impl Fn() -> DartFnFuture<Option<Vec<u8>>> + Send + 'static,
        statistics: impl Fn(Statistics) -> DartFnFuture<()> + Send + 'static,
        message_received: impl Fn(String) -> DartFnFuture<()> + Send + 'static,
        manager_active: impl Fn(bool, bool) -> DartFnFuture<()> + Send + 'static,
        call_started: impl Fn(Contact) -> DartFnFuture<()> + Send + 'static,
    ) -> AudioChat {
        let (start_session, start) = unbounded_async::<PeerId>();

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
            identity: Arc::new(RwLock::new(
                Keypair::from_protobuf_encoding(&identity).unwrap(),
            )),
            in_call: Default::default(),
            deafened: Default::default(),
            muted: Default::default(),
            play_custom_ringtones: Default::default(),
            session_states: Default::default(),
            start_session,
            restart_manager: Default::default(),
            network_config: network_config.clone(),
            overlay: overlay.clone(),
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
            call_started: Arc::new(Mutex::new(call_started)),
        };

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
        debug!("start_session called for {}", contact.peer_id);

        if self.start_session.send(contact.peer_id).await.is_err() {
            error!("start_session channel is closed");
        }
    }

    /// Attempts to start a call through an existing session
    pub async fn say_hello(&self, contact: &Contact) -> std::result::Result<(), DartError> {
        if let Some(state) = self.session_states.read().await.get(&contact.peer_id) {
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
    pub async fn set_identity(&self, key: Vec<u8>) -> std::result::Result<(), DartError> {
        *self.identity.write().await =
            Keypair::from_protobuf_encoding(&key).map_err(Error::from)?;
        Ok(())
    }

    /// Stops a specific session (called when a contact is deleted)
    pub async fn stop_session(&self, contact: &Contact) {
        if let Some(state) = self.session_states.write().await.remove(&contact.peer_id) {
            state.stop.notify_one();

            // clean up the session state
            self.session_states.write().await.remove(&contact.peer_id);
        }
    }

    /// Blocks while an audio test is running
    pub async fn audio_test(&self) -> std::result::Result<(), DartError> {
        self.in_call.store(true, Relaxed);

        let result = self
            .call(None, None, None, None, false)
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
        contact: &Contact,
    ) -> std::result::Result<(), DartError> {
        if let Some(state) = self.session_states.read().await.get(&contact.peer_id) {
            let message = Message::chat(message);

            state
                .message_sender
                .send(message)
                .await
                .map_err(Error::from)?;
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

    /// Starts new sessions
    async fn session_manager(&self, start: &AsyncReceiver<PeerId>) -> Result<()> {
        let mut swarm =
            libp2p::SwarmBuilder::with_existing_identity(self.identity.read().await.clone())
                .with_tokio()
                .with_tcp(
                    tcp::Config::default().port_reuse(true).nodelay(true),
                    noise::Config::new,
                    yamux::Config::default,
                )
                .map_err(|_| ErrorKind::SwarmBuild)?
                .with_quic()
                .with_relay_client(noise::Config::new, yamux::Config::default)
                .map_err(|_| ErrorKind::SwarmBuild)?
                .with_behaviour(|keypair, relay_behaviour| Behaviour {
                    relay_client: relay_behaviour,
                    ping: ping::Behaviour::new(ping::Config::new()),
                    identify: identify::Behaviour::new(identify::Config::new(
                        "/audio-chat/0.0.1".to_string(),
                        keypair.public(),
                    )),
                    dcutr: dcutr::Behaviour::new(keypair.public().to_peer_id()),
                    stream: libp2p_stream::Behaviour::new(),
                    auto_nat: autonat::Behaviour::new(
                        keypair.public().to_peer_id(),
                        autonat::Config {
                            ..Default::default()
                        },
                    ),
                })
                .map_err(|_| ErrorKind::SwarmBuild)?
                .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
                .build();

        let listen_addr_quic = Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Udp(*self.network_config.listen_port.read().await))
            .with(Protocol::QuicV1);

        swarm.listen_on(listen_addr_quic)?;

        let listen_addr_tcp = Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(*self.network_config.listen_port.read().await));

        swarm.listen_on(listen_addr_tcp)?;

        let socket_address = *self.network_config.relay_address.read().await;
        let relay_identity = *self.network_config.relay_id.read().await;

        let relay_address_udp = Multiaddr::empty()
            .with(Protocol::from(socket_address.ip()))
            .with(Protocol::Udp(socket_address.port()))
            .with(Protocol::QuicV1)
            .with_p2p(relay_identity)
            .map_err(|_| ErrorKind::SwarmBuild)?;

        let relay_address_tcp = Multiaddr::empty()
            .with(Protocol::from(socket_address.ip()))
            .with(Protocol::Tcp(socket_address.port()))
            .with_p2p(relay_identity)
            .map_err(|_| ErrorKind::SwarmBuild)?;

        let relay_address;

        if swarm.dial(relay_address_udp.clone()).is_err() {
            if let Err(error) = swarm.dial(relay_address_tcp.clone()) {
                return Err(error.into());
            } else {
                info!("connected to relay with tcp");
                relay_address = relay_address_tcp.with(Protocol::P2pCircuit);
            }
        } else {
            info!("connected to relay with udp");
            relay_address = relay_address_udp.with(Protocol::P2pCircuit);
        }

        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match swarm.next().await.ok_or(ErrorKind::SwarmEnded)? {
                SwarmEvent::NewListenAddr { .. } => (),
                SwarmEvent::Dialing { .. } => (),
                SwarmEvent::ConnectionEstablished { .. } => (),
                SwarmEvent::Behaviour(BehaviourEvent::Ping(_)) => (),
                SwarmEvent::NewExternalAddrCandidate { .. } => (),
                SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent {
                    ..
                })) => {
                    info!("Told relay its public address");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                    info: identify::Info { .. },
                    ..
                })) => {
                    info!("Relay told us our observed address");
                    learned_observed_addr = true;
                }
                // no other event occurs during a successful initialization
                event => {
                    error!("Unexpected event during initialization {:?}", event);
                    return Err(ErrorKind::UnexpectedSwarmEvent.into());
                }
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }

        swarm.listen_on(relay_address.clone())?;

        // alerts the UI that the manager is active
        (self.manager_active.lock().await)(true, true).await;

        // handle incoming streams
        spawn({
            let self_clone = self.clone();
            let mut control = swarm.behaviour().stream.new_control();

            async move {
                while let Ok(mut incoming_streams) = control.accept(PROTOCOL) {
                    while let Some((peer, stream)) = incoming_streams.next().await {
                        let contact_option =
                            (self_clone.get_contact.lock().await)(peer.to_bytes()).await;

                        if let Some(contact) = contact_option {
                            let state_option =
                                self_clone.session_states.read().await.get(&peer).cloned();

                            if let Some(state) = state_option {
                                if state.wants_audio.load(Relaxed) {
                                    info!("audio stream accepted for {}", peer);

                                    if let Err(error) = state.stream_sender.send(stream).await {
                                        error!("error sending audio stream to {}: {}", peer, error);
                                    }
                                } else {
                                    warn!("received a stream while {} did not want audio, starting new session", peer);
                                    self_clone._start_session(contact, None, stream).await;
                                }
                            } else {
                                info!("stream accepted for new session with {}", peer);
                                self_clone._start_session(contact, None, stream).await;
                            }
                        } else {
                            warn!("Received a stream from an unknown peer: {:?}", peer);
                        }
                    }

                    info!("incoming streams ended, trying to restart");
                }

                warn!("stopped accepting incoming streams; restarting controller");
                self_clone.restart_manager.notify_one();
            }
        });

        // handles the state needed for negotiating sessions
        let mut peer_states: HashMap<PeerId, PeerState> = HashMap::new();

        loop {
            select! {
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                            // ignore the relay connection
                            if peer_id == *self.network_config.relay_id.read().await {
                                continue;
                            }

                            let relayed = endpoint.is_relayed();
                            let listener = endpoint.is_listener();

                            info!("connection {} established with {} endpoint={:?} relayed={}", connection_id, peer_id, endpoint, relayed);

                            if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                                // insert the new connection
                                peer_state.connections.insert(connection_id, ConnectionState::new(relayed));
                                continue; // if the state already exists, the remaining logic is unnecessary
                            } else {
                                // insert the new state and new connection
                                peer_states.insert(peer_id, PeerState::new(!listener, connection_id, relayed));
                            }

                            if let Some(contact) = (self.get_contact.lock().await)(peer_id.to_bytes()).await {
                                if listener {
                                    // a stream will be established by the other client
                                    // the dialer already has the connecting status set
                                    (self.session_status.lock().await)(contact.peer_id(), "Connecting".to_string()).await;
                                }
                            } else {
                                warn!("received a connection from an unknown peer: {:?}", peer_id);

                                if swarm.disconnect_peer_id(peer_id).is_err() {
                                    error!("error disconnecting from unknown peer");
                                }

                                peer_states.remove(&peer_id);
                            }
                        }
                        SwarmEvent::OutgoingConnectionError { peer_id, error, connection_id } => {
                            if let Some(peer_id) = peer_id {
                                if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                                    warn!("outgoing connection failed for {} because {}", peer_id, error);
                                    peer_state.connections.remove(&connection_id);
                                } else {
                                    // if an outgoing error occurs when no connection is active, the session initialization failed
                                    error!("session initialization failed for {} because {}", peer_id, error);
                                    (self.session_status.lock().await)(peer_id.to_string(), "Inactive".to_string()).await;
                                }
                            }
                        },
                        SwarmEvent::ConnectionClosed { peer_id, cause, connection_id, .. } => {
                            warn!("connection {} closed with {} cause={:?}", connection_id, peer_id, cause);

                            if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                                peer_state.connections.remove(&connection_id);
                            }
                        },
                        SwarmEvent::Behaviour(BehaviourEvent::Ping(event)) => {
                            let latency = event.result.map(|duration| duration.as_millis()).ok();

                            // update the latency for the peer's session
                            if let Some(state) = self.session_states.read().await.get(&event.peer) {
                                state.latency.store(latency.unwrap_or(0) as usize, Relaxed);
                                continue; // the other logic is not needed while a session is active
                            }

                            // if the session isn't active yet, process the connections
                            if let Some(peer_state) = peer_states.get_mut(&event.peer) {
                                // update the latency for the peer's connections
                                if let Some(connection_latency) = peer_state.connections.get_mut(&event.connection) {
                                    connection_latency.latency = latency;
                                }

                                // only the dialer needs to proceed
                                if !peer_state.dialer {
                                    continue;
                                }

                                info!("connection states: {:?}", peer_state.connections);

                                if peer_state.connections.iter().any(|(_, state)| state.latency.is_none()) {
                                    // only start a session if all connections have latency
                                    debug!("not trying to establish a session with {} because not all connections have latency", event.peer);
                                    continue;
                                } else if peer_state.connections.iter().all(|(_, state)| state.relay) {
                                    // TODO what happens if no other connection is ever initiated?
                                    debug!("not trying to establish a session with {} because all connections are relayed", event.peer);
                                    continue;
                                }

                                // choose the connection with the lowest latency, prioritizing non-relay connections
                                let connection = peer_state
                                    .connections
                                    .iter()
                                    .min_by(|a, b| {
                                        match (a.1.relay, b.1.relay) {
                                            (false, true) => std::cmp::Ordering::Less, // prioritize non-relay connections
                                            (true, false) => std::cmp::Ordering::Greater, // prioritize non-relay connections
                                            _ => a.1.latency.cmp(&b.1.latency), // compare latencies if both have the same relay status
                                        }
                                    })
                                    .map(|(id, _)| id);

                                if let Some(connection_id) = connection {
                                    info!("using connection id={} for {}", connection_id, event.peer);

                                    // close the other connections
                                    peer_state.connections
                                        .iter()
                                        .filter(|(id, _)| *id != connection_id)
                                        .for_each(|(id, _)| { swarm.close_connection(*id); });

                                    let mut control = swarm.behaviour().stream.new_control();

                                    if let Some(contact) = (self.get_contact.lock().await)(event.peer.to_bytes()).await {
                                        // it may take multiple tries to open the stream because the of the RNG in the stream handler
                                        loop {
                                            if let Ok(stream) = control.open_stream(event.peer, PROTOCOL).await {
                                                info!("opened stream with {} on connection {}, starting new session", event.peer, connection_id);
                                                self._start_session(contact, Some(control), stream).await;
                                                break;
                                            }
                                        }
                                    } else {
                                        warn!("peer in peer states with no contact {}", event.peer);
                                    }
                                } else {
                                    warn!("no connection available for {}", event.peer);
                                }
                            }
                        },
                        SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received { peer_id, info })) => {
                            if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                                if peer_state.dialed || !peer_state.dialer {
                                    continue;
                                } else {
                                    peer_state.dialed = true;
                                }
                            } else {
                                // the relay server sends identity events which will be caught here
                                continue;
                            }

                            info!("Received first identify event from {}", peer_id);

                            for address in info.listen_addrs {
                                // checks for relayed addresses which are not useful
                                if address.ends_with(&Protocol::P2p(peer_id).into()) {
                                    continue;
                                }

                                // dials the non-relayed addresses to attempt direct connections
                                if let Err(error) = swarm.dial(address) {
                                    error!("Error dialing {}: {}", peer_id, error);
                                }
                            }
                        },
                        event => {
                            debug!("other swarm event: {:?}", event);
                        },
                    }
                }
                result = start.recv() => {
                    let peer_id = result?;

                    // prevents dialing a peer who is already connected
                    if swarm.is_connected(&peer_id) {
                        warn!("{} is already connected", peer_id);
                        continue;
                    }

                    debug!("initial dial for {}", peer_id);

                    // dial the peer through the relay
                    if let Err(error) = swarm.dial(relay_address.clone().with(Protocol::P2p(peer_id))) {
                        error!("dial error for {}: {}", peer_id, error);
                        (self.session_status.lock().await)(peer_id.to_string(), "Inactive".to_string()).await;
                    } else {
                        (self.session_status.lock().await)(peer_id.to_string(), "Connecting".to_string()).await;
                    }
                }
                _ = self.restart_manager.notified() => {
                    break Err(ErrorKind::ManagerRestarted.into());
                }
            }
        }
    }

    /// A wrapper which starts the session, registers it in the session states, and cleans up after it
    async fn _start_session(&self, contact: Contact, control: Option<Control>, stream: Stream) {
        let message_channel = unbounded_async::<Message>();
        let (stream_sender, stream_receiver) = unbounded_async();

        // create the state and a clone of it for the session
        let state = Arc::new(SessionState::new(&message_channel.0, stream_sender));
        let state_clone = Arc::clone(&state);

        let mut states = self.session_states.write().await;

        if let Some(state) = states.get(&contact.peer_id) {
            warn!("{} already has a session", contact.nickname);

            // stop the session
            state.stop.notify_one();

            if state.in_call.load(Relaxed) {
                // if the session was in a call, end it so the session can end
                self.end_call.notify_one();
                // alerts the UI that a new call is starting
                (self.call_started.lock().await)(contact.clone()).await;
                // signals the new session to restart the call
                state_clone.start.notify_one();
            }
        }

        // insert the new state
        states.insert(contact.peer_id, state.clone());

        let chat_clone = self.clone();
        spawn(async move {
            let contact_clone = contact.clone();

            let result = chat_clone
                .session(
                    contact,
                    control,
                    stream.compat(),
                    &state_clone,
                    message_channel,
                    stream_receiver,
                )
                .await;

            if let Err(error) = result {
                error!("Session error for {}: {}", contact_clone.nickname, error);
            } else {
                warn!("Session for {} stopped", contact_clone.nickname);
                return; // the session has already been cleaned up
            }

            debug!("removing session state for {}", contact_clone.nickname);

            // cleanup
            chat_clone
                .session_states
                .write()
                .await
                .remove(&contact_clone.peer_id);

            debug!(
                "session state removed for {} | updating session status",
                contact_clone.nickname
            );

            (chat_clone.session_status.lock().await)(
                contact_clone.peer_id(),
                "Inactive".to_string(),
            )
            .await;

            info!("Session for {} cleaned up", contact_clone.nickname);
        });
    }

    /// A session with a contact
    async fn session(
        &self,
        contact: Contact,
        mut control: Option<Control>,
        stream: TransportStream,
        state: &Arc<SessionState>,
        message_channel: (AsyncSender<Message>, AsyncReceiver<Message>),
        stream_receiver: AsyncReceiver<Stream>,
    ) -> Result<()> {
        // alert the UI that this session is now connected
        (self.session_status.lock().await)(contact.peer_id(), "Connected".to_string()).await;

        let mut transport = LengthDelimitedCodec::builder().new_framed(stream);
        let mut keep_alive = interval(Duration::from_secs(10));

        let result = loop {
            // an async block is used to capture errors and handle them internally
            let future = async {
                info!("[{}] session waiting for event", contact.nickname);

                select! {
                    result = read_message::<Message, _>(&mut transport) => {
                        let mut ringtone = None;

                        match result? {
                            Message { message: Some(message::Message::Hello(message)) } => {
                                if !message.ringtone.is_empty() && self.play_custom_ringtones.load(Relaxed) {
                                    ringtone = Some(message.ringtone);
                                }
                            },
                            Message { message: Some(message::Message::KeepAlive(_)) } => {
                                return Ok::<(), Error>(());
                            },
                            message => {
                                warn!("received unexpected {:?} from {}", message, contact.nickname);
                                return Ok::<(), Error>(());
                            }
                        }

                        if self.in_call.load(Relaxed) {
                            // do not accept another call if already in one
                            let busy = Message::busy();
                            write_message(&mut transport, busy).await?;
                            return Ok(());
                        }

                        state.in_call.store(true, Relaxed); // blocks the session from being restarted
                        let cancel_prompt = Arc::new(Notify::new());
                        let dart_cancel = DartNotify { inner: Arc::clone(&cancel_prompt) };

                        let accept_call_clone = Arc::clone(&self.accept_call);
                        let contact_id = contact.id.clone();
                        let accept_handle = spawn(async move {
                            (accept_call_clone.lock().await)(contact_id, ringtone, dart_cancel).await
                        });

                        select! {
                            accepted = accept_handle => {
                                if accepted? {
                                    // respond with hello ack if the call is accepted
                                    let hello_ack = Message::hello_ack();
                                    write_message(&mut transport, hello_ack).await?;

                                    // start the handshake
                                    self.handshake(&mut transport, control.as_mut(), contact.peer_id, &message_channel, &stream_receiver, state).await?;
                                } else {
                                    // reject the call if not accepted
                                    let reject = Message::reject();
                                    write_message(&mut transport, reject).await?;
                                }
                            }
                            result = read_message::<Message, _>(&mut transport) => {
                                info!("received message while accept call was pending");

                                match result {
                                    Ok(Message { message: Some(message::Message::Goodbye(_)) }) => {
                                        info!("received goodbye from {} while prompting for call", contact.nickname);
                                        cancel_prompt.notify_one();
                                    }
                                    Ok(message) => {
                                        warn!("received unexpected {:?} from {} while prompting for call", message, contact.nickname);
                                    }
                                    Err(error) => {
                                        error!("Error reading message while prompting for call from {}: {}", contact.nickname, error);
                                    }
                                }
                            }
                        }

                        Ok(())
                    }
                    _ = state.start.notified() => {
                        state.in_call.store(true, Relaxed); // blocks the session from being restarted

                        // queries the other client for a call
                        let ringtone = (self.load_ringtone.lock().await)().await;
                        let hello = Message::hello(ringtone);
                        write_message(&mut transport, hello).await?;

                        select! {
                            result = timeout(HELLO_TIMEOUT, read_message(&mut transport)) => {
                                // handles a variety of outcomes in response to Hello
                                match result?? {
                                    Message { message: Some(message::Message::HelloAck(_)) } => {
                                        self.handshake(&mut transport, control.as_mut(), contact.peer_id, &message_channel, &stream_receiver, state).await?;
                                    }
                                    Message { message: Some(message::Message::Reject(_)) } => {
                                        (self.call_ended.lock().await)(format!("{} did not accept the call", contact.nickname), true).await;
                                    },
                                    Message { message: Some(message::Message::Busy(_)) } => {
                                        (self.call_ended.lock().await)(format!("{} is busy", contact.nickname), true).await;
                                    }
                                    message => {
                                        (self.call_ended.lock().await)(format!("Received an unexpected message from {}", contact.nickname), true).await;
                                        warn!("received unexpected {:?} from {} [stopped call process]", message, contact.nickname);
                                    }
                                }
                            }
                            _ = self.end_call.notified() => {
                                info!("end call notified while waiting for hello ack");
                                let goodbye = Message::goodbye();
                                write_message(&mut transport, goodbye).await?;
                            }
                        }

                        Ok(())
                    }
                    // state will never notify while a call is active
                    _ = state.stop.notified() => {
                        info!("session state stop notified for {}", contact.nickname);
                        Err(ErrorKind::SessionStoped.into())
                    },
                    _ = keep_alive.tick() => {
                        let keep_alive = Message::keep_alive();
                        write_message(&mut transport, keep_alive).await?;
                        Ok(())
                    },
                }
            };

            if let Err(error) = future.await {
                if state.in_call.load(Relaxed) {
                    info!("session error while call active, alerting ui");
                    (self.call_ended.lock().await)(error.to_string(), false).await;
                }

                match error.kind {
                    ErrorKind::KanalReceive(_)
                    | ErrorKind::TransportRecv
                    | ErrorKind::TransportSend => break Err(error),
                    ErrorKind::SessionStoped => break Ok(()),
                    _ => error!("Session error: {:?}", error),
                }
            }

            // the session is now safe to restart
            state.in_call.store(false, Relaxed);
        };

        info!("session for {} returning", contact.nickname);
        result
    }

    /// Gets everything ready for the call
    async fn handshake(
        &self,
        transport: &mut Transport<TransportStream>,
        control: Option<&mut Control>,
        peer: PeerId,
        message_channel: &(AsyncSender<Message>, AsyncReceiver<Message>),
        stream_receiver: &AsyncReceiver<Stream>,
        state: &Arc<SessionState>,
    ) -> Result<()> {
        debug!("handshake running");

        let dialer = control.is_some();

        // change the session state to accept incoming audio streams
        state.wants_audio.store(true, Relaxed);

        let future = async {
            let stream = if let Some(control) = control {
                // if dialer open stream
                control.open_stream(peer, PROTOCOL).await?
            } else {
                // if listener receive stream
                stream_receiver.recv().await?
            };

            Ok::<_, Error>(stream)
        };

        let stream = select! {
            stream = future => stream?,
            // handle unexpected messages while waiting for the audio stream
            result = read_message::<Message, _>(transport) => {
                warn!("received unexpected message while waiting for audio stream: {:?}", result);
                return Ok(());
            }
        };

        // alert the UI that the call has connected
        (self.connected.lock().await)().await;
        // change the app call state
        self.in_call.store(true, Relaxed);
        // change the session state
        state.wants_audio.store(false, Relaxed);

        self.overlay.show();

        let result = self
            .call(
                Some(stream),
                Some(transport),
                Some(message_channel.1.clone()),
                Some(state),
                dialer,
            )
            .await;

        info!("call ended in receiver");

        // the call has ended
        self.in_call.store(false, Relaxed);

        self.overlay.hide();
        info!("overlay hidden");

        match result {
            Ok(()) => Ok(()),
            Err(error) => match error.kind {
                ErrorKind::NoInputDevice
                | ErrorKind::NoOutputDevice
                | ErrorKind::BuildStream(_)
                | ErrorKind::StreamConfig(_) => {
                    let message = Message::goodbye_reason("Audio device error".to_string());
                    write_message(transport, message).await?;
                    Err(error)
                }
                _ => {
                    let message = Message::goodbye_reason(error.to_string());
                    write_message(transport, message).await?;
                    Err(error)
                }
            },
        }
    }

    /// The bulk of the call logic
    async fn call(
        &self,
        stream: Option<Stream>,
        mut transport: Option<&mut Transport<TransportStream>>,
        message_receiver: Option<AsyncReceiver<Message>>,
        state: Option<&Arc<SessionState>>,
        dialer: bool,
    ) -> Result<()> {
        // if any of the values required for a normal call is missing, the call is an audio test
        let audio_test = transport.is_none()
            || stream.is_none()
            || message_receiver.is_none()
            || state.is_none();

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
        let upload_bandwidth: Arc<AtomicUsize> = Default::default();
        let download_bandwidth: Arc<AtomicUsize> = Default::default();

        let (receiving_sender, receiving_receiver) = unbounded_async::<bool>();

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
            let transport = transport.as_mut().unwrap();

            // the dialer reads the message first because its stream opens instantly
            if dialer {
                let message = read_message(transport).await?;
                info!("sending audio header");
                write_message(transport, audio_header).await?;
                message
            } else {
                info!("sending audio header");
                write_message(transport, audio_header).await?;
                read_message(transport).await?
            }
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

        let latency = if let Some(state) = state {
            Arc::clone(&state.latency)
        } else {
            Default::default()
        };

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
            latency,
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
            let audio_transport = LengthDelimitedCodec::builder()
                .max_frame_length(TRANSFER_BUFFER_SIZE)
                .length_field_type::<u16>()
                .new_framed(stream.unwrap().compat());

            let (write, read) = audio_transport.split();

            let input_handle = spawn(socket_input(
                processed_input_receiver,
                write,
                Arc::clone(&stop_io),
                upload_bandwidth,
            ));

            let output_handle = spawn(socket_output(
                output_sender,
                read,
                Arc::clone(&stop_io),
                download_bandwidth,
                receiving_sender,
            ));

            let transport = transport.as_mut().unwrap();
            let message_receiver = message_receiver.unwrap();

            let message_received = Arc::clone(&self.message_received);
            let call_state = Arc::clone(&self.call_state);

            select! {
                result = input_handle => result?,
                result = output_handle => result?,
                // this unwrap is safe because the processor thread will not panic
                result = output_processor_future => result?.unwrap(),
                // this unwrap is safe because the processor thread will not panic
                result = input_processor_future => result?.unwrap(),
                result = statistics_handle => result?,
                message = call_controller(transport, message_receiver, end_call, receiving_receiver, message_received, call_state) => {
                    debug!("controller exited: {:?}", message);

                    match message {
                        Ok(message) => (self.call_ended.lock().await)(message, true).await,
                        Err(error) => match error.kind {
                            ErrorKind::CallEnded => (), // when the call is ended externally, no UI notification is needed
                            _ => (self.call_ended.lock().await)(error.to_string(), false).await,
                        },
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

#[derive(Debug)]
struct PeerState {
    /// when true the peer's identity addresses will not be dialed
    dialed: bool,

    dialer: bool,

    /// a map of connections and their latencies
    connections: HashMap<ConnectionId, ConnectionState>,
}

impl PeerState {
    fn new(dialer: bool, connection_id: ConnectionId, relayed: bool) -> Self {
        let mut connections = HashMap::new();
        connections.insert(connection_id, ConnectionState::new(relayed));

        Self {
            dialed: false,
            dialer,
            connections,
        }
    }
}

#[derive(Debug)]
struct ConnectionState {
    latency: Option<u128>,
    relay: bool,
}

impl ConnectionState {
    fn new(relay: bool) -> Self {
        Self {
            latency: None,
            relay,
        }
    }
}

/// Wraps a cpal stream to unsafely make it send
pub(crate) struct SendStream {
    pub(crate) stream: CpalStream,
}

unsafe impl Send for SendStream {}

/// A message containing either a frame of audio or silence
enum ProcessorMessage {
    Data(Bytes),
    Silence,
}

// common message constructors
impl ProcessorMessage {
    fn data(bytes: &'static [u8]) -> Result<Self> {
        Ok(Self::Data(Bytes::from(bytes)))
    }

    fn silence() -> Self {
        Self::Silence
    }

    fn frame(frame: Bytes) -> Self {
        Self::Data(frame)
    }
}

/// Shared values for a single session
struct SessionState {
    /// Signals the session to initiate a call
    start: Notify,

    /// Stops the session normally
    stop: Notify,

    /// If the session is in a call
    in_call: AtomicBool,

    /// A reusable sender for messages while a call is active
    message_sender: AsyncSender<Message>,

    /// Forwards sub-streams to the session
    stream_sender: AsyncSender<Stream>,

    /// A shared latency value for the session from libp2p ping
    latency: Arc<AtomicUsize>,

    /// Whether the session wants an audio stream
    wants_audio: Arc<AtomicBool>,
}

impl SessionState {
    fn new(message_sender: &AsyncSender<Message>, stream_sender: AsyncSender<Stream>) -> Self {
        Self {
            start: Notify::new(),
            stop: Notify::new(),
            in_call: AtomicBool::new(false),
            message_sender: message_sender.clone(),
            stream_sender,
            latency: Default::default(),
            wants_audio: Default::default(),
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

    /// a value between 0 and 1 representing the percent of audio lost in a sliding window
    pub loss: f64,
}

#[frb(opaque)]
#[derive(Clone)]
pub struct NetworkConfig {
    /// The match-maker server to use for signaling
    relay_address: Arc<RwLock<SocketAddr>>,

    /// The relay's peer id
    relay_id: Arc<RwLock<PeerId>>,

    /// The libp2p port for the swarm
    listen_port: Arc<RwLock<u16>>,
}

impl NetworkConfig {
    #[frb(sync)]
    pub fn new(relay_address: String, relay_id: String) -> std::result::Result<Self, DartError> {
        Ok(Self {
            relay_address: Arc::new(RwLock::new(relay_address.parse().map_err(Error::from)?)),
            relay_id: Arc::new(RwLock::new(
                PeerId::from_str(&relay_id).map_err(Error::from)?,
            )),
            listen_port: Arc::new(RwLock::new(0)),
        })
    }

    // TODO test whether domain resolution works
    pub async fn set_relay_address(
        &self,
        relay_address: String,
    ) -> std::result::Result<(), DartError> {
        if let Some(address) = lookup_host(&relay_address)
            .await
            .map_err(Error::from)?
            .next()
        {
            *self.relay_address.write().await = address;
            Ok(())
        } else {
            Err("Failed to resolve address".to_string().into())
        }
    }

    pub async fn get_relay_address(&self) -> String {
        self.relay_address.read().await.to_string()
    }

    pub async fn set_relay_id(&self, relay_id: String) -> std::result::Result<(), DartError> {
        *self.relay_id.write().await = PeerId::from_str(&relay_id).map_err(Error::from)?;
        Ok(())
    }

    pub async fn get_relay_id(&self) -> String {
        self.relay_id.read().await.to_string()
    }
}

/// A notify that can be passed to dart code
#[frb(opaque)]
pub struct DartNotify {
    inner: Arc<Notify>,
}

impl DartNotify {
    pub async fn notified(&self) {
        self.inner.notified().await;
    }
}

async fn call_controller(
    transport: &mut Transport<TransportStream>,
    receiver: AsyncReceiver<Message>,
    end_call: Arc<Notify>,
    receiving: AsyncReceiver<bool>,
    message_received: Arc<Mutex<dyn Fn(String) -> DartFnFuture<()> + Send>>,
    call_state: Arc<Mutex<dyn Fn(bool) -> DartFnFuture<()> + Send>>,
) -> Result<String> {
    // whether the session is currently receiving audio
    let mut is_receiving = false;
    // whether the remote peer is currently receiving audio
    let mut remote_is_receiving = true;

    // the instant the UI will be notified that the session is not receiving audio
    let mut notify_ui = Instant::now() + Duration::from_secs(2);

    // the instant the session stopped receiving audio
    let mut disconnected_at = Instant::now();

    // the instant the disconnect started and the duration of the disconnect
    let mut disconnect_durations: VecDeque<(Instant, Duration)> = VecDeque::new();
    // ticks to update the connection quality and remove old entries from `disconnect_durations`
    let mut update_durations = interval(Duration::from_secs(1));

    // constant durations used in the connection quality algorithm
    let window_duration = Duration::from_secs(10);
    let disconnect_duration = Duration::from_secs(2);

    let (state_sender, state_receiver) = unbounded_async::<bool>();

    loop {
        select! {
            // receives and handles messages from the callee
            result = read_message(transport) => {
                info!("call controller read_message result: {:?}", result);

                let message: Message = result?;

                match message.message {
                    Some(message::Message::Goodbye(message)) => {
                        debug!("received goodbye, reason = {:?}", message.reason);
                        break Ok(message.reason);
                    },
                    Some(message::Message::Chat(message)) => {
                        (message_received.lock().await)(message.message).await;
                    }
                    Some(message::Message::ConnectionInterrupted(_)) => {
                        info!("received connection interrupted message r={} rr={}", is_receiving, remote_is_receiving);

                        let receiving = is_receiving && remote_is_receiving;
                        remote_is_receiving = false;
                        state_sender.send(receiving).await?;
                    }
                    Some(message::Message::ConnectionRestored(_)) => {
                        info!("received connection restored message r={} rr={}", is_receiving, remote_is_receiving);

                        if remote_is_receiving {
                            warn!("received connection restored message while already receiving");
                            continue;
                        }

                        remote_is_receiving = true;
                        state_sender.send(false).await?;
                    }
                    _ => error!("received unexpected message: {:?}", message),
                }
            },
            // sends messages to the callee
            result = receiver.recv() => {
                if let Ok(message) = result {
                    write_message(transport, message).await?;
                } else {
                    // if the channel closes, the call has ended
                    break Ok(String::new());
                }
            },
            // ends the call
            _ = end_call.notified() => {
                let message = Message::goodbye();
                write_message(transport, message).await?;
                break Err(ErrorKind::CallEnded.into());
            },
            receiving = state_receiver.recv() => {
                if receiving? {
                    info!("state switched to not receiving r={} rr={}", is_receiving, remote_is_receiving);

                    // the instant the disconnect began
                    disconnected_at = Instant::now();
                    // notify the ui in 2 seconds if the disconnect hasn't ended
                    notify_ui = disconnected_at + disconnect_duration;
                } else if is_receiving && remote_is_receiving {
                    let elapsed = disconnected_at.elapsed();
                    info!("reconnected after {}ms interruption", elapsed.as_millis());

                    // update the call state in the UI
                    (call_state.lock().await)(false).await;
                    // record the disconnect
                    disconnect_durations.push_back((disconnected_at, elapsed));
                    // prevents any notification to the ui as audio is being received
                    notify_ui = Instant::now() + Duration::from_secs(86400 * 365 * 30);
                    // set the overlay to connected
                    CONNECTED.store(true, Relaxed);
                } else if is_receiving ^ remote_is_receiving {
                    info!("partial reconnect r={} rr={}", is_receiving, remote_is_receiving);
                } else {
                    info!("full disconnect r={} rr={}", is_receiving, remote_is_receiving)
                }
            },
            // receives when the receiving state changes
            Ok(receiving) = receiving.recv() => {
                info!("received receiving state: {} | r={} rr={}", receiving, is_receiving, remote_is_receiving);

                if receiving != is_receiving {
                    state_sender.send(is_receiving && remote_is_receiving).await?;

                    is_receiving = receiving;

                    let message = if is_receiving {
                        Message::connection_restored()
                    } else {
                        Message::connection_interrupted()
                    };

                    if let Err(error) = write_message(transport, message).await {
                        error!("Error sending connection notification message: {}", error);
                    }
                } else {
                    warn!("received duplicate receiving state: {}", receiving);
                }
            },
            // if the session doesn't reconnect within the time limit, notify the UI
            _ = sleep_until(notify_ui) => {
                (call_state.lock().await)(true).await;
                // the UI does not need to be notified until the session reconnects
                notify_ui = Instant::now() + Duration::from_secs(86400 * 365 * 30);
                // set the overlay to disconnected
                CONNECTED.store(false, Relaxed);
            },
            _ = update_durations.tick() => {
                let now = Instant::now();

                // check for disconnects outside the 10-second window
                while let Some((start, _)) = disconnect_durations.front() {
                    if now - *start > window_duration {
                        disconnect_durations.pop_front();
                    } else {
                        break;
                    }
                }

                let mut total_disconnect = disconnect_durations.iter().fold(Duration::default(), |acc, (_, duration)| acc + *duration).as_millis();

                // if not receiving, add the current disconnect duration
                if !is_receiving || !remote_is_receiving {
                    total_disconnect += disconnected_at.elapsed().as_millis();
                }

                LOSS.store(total_disconnect as f64 / 10_000_f64, Relaxed);
            }
        }
    }
}

/// Receives frames of audio data from the input processor and sends them to the socket
async fn socket_input(
    input_receiver: AsyncReceiver<ProcessorMessage>,
    mut socket: SplitSink<Framed<Compat<Stream>, LengthDelimitedCodec>, Bytes>,
    stop: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
) -> Result<()> {
    let silence_byte = &[0];

    let future = async {
        while let Ok(message) = input_receiver.recv().await {
            match message {
                ProcessorMessage::Silence => {
                    bandwidth.fetch_add(1, Relaxed);
                    // send the silence signal
                    socket.send(Bytes::from_static(silence_byte)).await?;
                }
                ProcessorMessage::Data(bytes) => {
                    bandwidth.fetch_add(bytes.len(), Relaxed);
                    // send the bytes to the socket
                    socket.send(bytes).await?;
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
async fn socket_output(
    sender: AsyncSender<ProcessorMessage>,
    mut socket: SplitStream<Framed<Compat<Stream>, LengthDelimitedCodec>>,
    notify: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
    receiving: AsyncSender<bool>,
) -> Result<()> {
    let mut is_receiving = false;

    let future = async {
        loop {
            match timeout(TIMEOUT_DURATION, socket.next()).await {
                Ok(Some(Ok(message))) => {
                    if !is_receiving {
                        _ = receiving.send(true).await;
                        is_receiving = true;
                    }

                    let len = message.len();
                    bandwidth.fetch_add(len, Relaxed);

                    match len {
                        TRANSFER_BUFFER_SIZE => {
                            _ = sender.try_send(ProcessorMessage::frame(message.freeze()));
                        }
                        1 => match message[0] {
                            0 => _ = sender.try_send(ProcessorMessage::silence()), // silence
                            _ => error!("received unknown control signal {}", message[0]),
                        },
                        // this should be impossible
                        len => error!("received {} < {} data", len, TRANSFER_BUFFER_SIZE),
                    }
                }
                Ok(Some(Err(error))) => {
                    error!("Socket output error: {}", error);
                    break Err(error.into());
                }
                Ok(None) => {
                    debug!("Socket output ended");
                    break Ok(());
                }
                Err(_) => {
                    if is_receiving {
                        _ = receiving.send(false).await;
                        is_receiving = false;
                    }
                }
            }
        }
    };

    select! {
        result = future => result,
        _ = notify.notified() => {
            debug!("Socket output ended");
            Ok(())
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

    let result = loop {
        select! {
            _ = interval.tick() => {
                let statistics = Statistics {
                    input_level: level_from_window(&input_receiver, &mut input_max).await,
                    output_level: level_from_window(&output_receiver, &mut output_max).await,
                    latency: latency.load(Relaxed),
                    upload_bandwidth: upload_bandwidth.load(Relaxed),
                    download_bandwidth: download_bandwidth.load(Relaxed),
                    loss: LOSS.load(Relaxed),
                };

                LATENCY.store(statistics.latency, Relaxed);
                (callback.lock().await)(statistics).await;
            }
            _ = notify.notified() => {
                debug!("Statistics collector ended");
                break Ok(());
            }
        }
    };

    // zero out the statistics when the collector ends
    let statistics = Statistics::default();
    (callback.lock().await)(statistics).await;

    LATENCY.store(0, Relaxed);
    LOSS.store(0_f64, Relaxed);
    CONNECTED.store(false, Relaxed);

    result
}

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
    let i16_size = mem::size_of::<i16>();

    let ratio = if denoiser.is_some() {
        // rnnoise requires a 48kHz sample rate
        48_000_f64 / sample_rate
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
                int_buffer.len() * i16_size,
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
    let i16_size = mem::size_of::<i16>();

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
                    std::slice::from_raw_parts(bytes.as_ptr() as *const i16, bytes.len() / i16_size)
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

/// Writes a protobuf message to the stream
async fn write_message<M: prost::Message, W>(transport: &mut Transport<W>, message: M) -> Result<()>
where
    W: AsyncWrite + Unpin,
    Transport<W>: Sink<Bytes> + Unpin,
{
    let len = message.encoded_len(); // get the length of the message
    let mut buffer = Vec::with_capacity(len);

    message.encode(&mut buffer).unwrap(); // encode the message into the buffer (infallible)

    transport
        .send(Bytes::from(buffer))
        .await
        .map_err(|_| ErrorKind::TransportSend)
        .map_err(Into::into)
}

/// Reads a protobuf message from the stream
async fn read_message<M: prost::Message + Default, R: AsyncRead + Unpin>(
    transport: &mut Transport<R>,
) -> Result<M> {
    if let Some(Ok(buffer)) = transport.next().await {
        let message = M::decode(&buffer[..])?; // decode the message
        Ok(message)
    } else {
        Err(ErrorKind::TransportRecv.into())
    }
}
