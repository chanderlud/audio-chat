use std::collections::{HashMap, HashSet, VecDeque};
use std::mem;
use std::net::Ipv4Addr;
pub use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::Arc;
use std::time::Duration;

use async_throttle::RateLimiter;
use atomic_float::AtomicF32;
use chrono::{DateTime, Local};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use cpal::Host;
use cpal::{Device, Stream as CpalStream};
use flutter_rust_bridge::for_generated::futures::stream::{SplitSink, SplitStream};
use flutter_rust_bridge::for_generated::futures::{Sink, SinkExt};
use flutter_rust_bridge::{frb, spawn, DartFnFuture};
pub use kanal::AsyncReceiver;
use kanal::{bounded, bounded_async, unbounded_async, AsyncSender, Receiver, Sender};
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
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::lookup_host;
use tokio::select;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::{interval, sleep_until, timeout, Instant, Interval};
use tokio_util::bytes::Bytes;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};

use crate::api::contact::Contact;
use crate::api::error::{DartError, Error, ErrorKind};
use crate::api::overlay::overlay::Overlay;
use crate::api::overlay::{CONNECTED, LATENCY, LOSS};
use crate::api::screenshare;
use crate::api::screenshare::{Decoder, Encoder};
use crate::{Behaviour, BehaviourEvent};
use crate::api::constants::*;
use messages::{message, Attachment, AudioHeader, Message, ScreenshareHeader};

type Result<T> = std::result::Result<T, Error>;
pub(crate) type DeviceName = Arc<Mutex<Option<String>>>;
type TransportStream = Compat<Stream>;
pub type Transport<T> = Framed<T, LengthDelimitedCodec>;
type StartScreenshare = (PeerId, Option<ScreenshareHeader>);

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

    /// Signals the session manager to start a screenshare
    start_screenshare: AsyncSender<StartScreenshare>,

    /// Restarts the session manager when needed
    restart_manager: Arc<Notify>,

    /// Network configuration for p2p connections
    network_config: NetworkConfig,

    /// Configuration for the screenshare functionality
    screenshare_config: ScreenshareConfig,

    /// A reference to the object that controls the call overlay
    overlay: Overlay,

    /// A list of PeerIds which are chat rooms
    chat_rooms: Arc<RwLock<HashSet<PeerId>>>,

    /// Prompts the user to accept a call
    accept_call:
        Arc<Mutex<dyn Fn(String, Option<Vec<u8>>, DartNotify) -> DartFnFuture<bool> + Send>>,

    /// Alerts the UI that a call has ended
    call_ended: Arc<Mutex<dyn Fn(String, bool) -> DartFnFuture<()> + Send>>,

    /// Fetches a contact from the front end
    get_contact: Arc<Mutex<dyn Fn(Vec<u8>) -> DartFnFuture<Option<Contact>> + Send>>,

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
    message_received: Arc<Mutex<dyn Fn(ChatMessage) -> DartFnFuture<()> + Send>>,

    /// Alerts the UI when the manager is active and restartable
    manager_active: Arc<Mutex<dyn Fn(bool, bool) -> DartFnFuture<()> + Send>>,

    /// Called when a screenshare starts
    screenshare_started: Arc<Mutex<dyn Fn(DartNotify, bool) -> DartFnFuture<()> + Send>>,
}

impl AudioChat {
    // this function must be async to use `spawn`
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        identity: Vec<u8>,
        host: Arc<Host>,
        network_config: &NetworkConfig,
        screenshare_config: &ScreenshareConfig,
        overlay: &Overlay,
        accept_call: impl Fn(String, Option<Vec<u8>>, DartNotify) -> DartFnFuture<bool> + Send + 'static,
        call_ended: impl Fn(String, bool) -> DartFnFuture<()> + Send + 'static,
        get_contact: impl Fn(Vec<u8>) -> DartFnFuture<Option<Contact>> + Send + 'static,
        call_state: impl Fn(bool) -> DartFnFuture<()> + Send + 'static,
        session_status: impl Fn(String, String) -> DartFnFuture<()> + Send + 'static,
        start_sessions: impl Fn(AudioChat) -> DartFnFuture<()> + Send + 'static,
        load_ringtone: impl Fn() -> DartFnFuture<Option<Vec<u8>>> + Send + 'static,
        statistics: impl Fn(Statistics) -> DartFnFuture<()> + Send + 'static,
        message_received: impl Fn(ChatMessage) -> DartFnFuture<()> + Send + 'static,
        manager_active: impl Fn(bool, bool) -> DartFnFuture<()> + Send + 'static,
        screenshare_started: impl Fn(DartNotify, bool) -> DartFnFuture<()> + Send + 'static,
    ) -> AudioChat {
        let (start_session, session) = unbounded_async::<PeerId>();
        let (start_screenshare, screenshare) = unbounded_async::<StartScreenshare>();

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
            start_screenshare,
            restart_manager: Default::default(),
            network_config: network_config.clone(),
            screenshare_config: screenshare_config.clone(),
            overlay: overlay.clone(),
            chat_rooms: Default::default(),
            accept_call: Arc::new(Mutex::new(accept_call)),
            call_ended: Arc::new(Mutex::new(call_ended)),
            get_contact: Arc::new(Mutex::new(get_contact)),
            call_state: Arc::new(Mutex::new(call_state)),
            session_status: Arc::new(Mutex::new(session_status)),
            start_sessions: Arc::new(Mutex::new(start_sessions)),
            load_ringtone: Arc::new(Mutex::new(load_ringtone)),
            statistics: Arc::new(Mutex::new(statistics)),
            message_received: Arc::new(Mutex::new(message_received)),
            manager_active: Arc::new(Mutex::new(manager_active)),
            screenshare_started: Arc::new(Mutex::new(screenshare_started)),
        };

        // start the session manager
        let chat_clone = chat.clone();
        spawn(async move {
            // retry the session manager if it fails, but not too fast
            let rate_limiter = RateLimiter::new(Duration::from_millis(100));

            loop {
                while let Err(error) = rate_limiter
                    .throttle(|| async { chat_clone.session_manager(&session, &screenshare).await })
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
            state.start_call.notify_one();
            Ok(())
        } else {
            Err(String::from("No session found for contact").into())
        }
    }

    /// Join a chat room
    pub async fn join_room(&self, contact: &Contact) {
        // dials the room through the relay using the swarm
        if self.start_session.send(contact.peer_id).await.is_err() {
            error!("start_session channel is closed in join_room");
        }

        // the session manager will know to treat this connection as a chat room
        self.chat_rooms.write().await.insert(contact.peer_id);
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
            state.stop_session.notify_one();

            // clean up the session state
            self.session_states.write().await.remove(&contact.peer_id);
        }
    }

    /// Blocks while an audio test is running
    pub async fn audio_test(&self) -> std::result::Result<(), DartError> {
        self.in_call.store(true, Relaxed);

        let stop_io = Arc::new(Notify::new());

        let result = self
            .call(None, None, None, None, None, &stop_io)
            .await
            .map_err(Error::from)
            .map_err(Into::into);

        stop_io.notify_waiters();
        self.in_call.store(false, Relaxed);

        result
    }

    #[frb(sync)]
    pub fn build_chat(
        &self,
        contact: &Contact,
        text: String,
        attachments: Vec<(String, Vec<u8>)>,
    ) -> ChatMessage {
        ChatMessage {
            text,
            receiver: contact.peer_id,
            timestamp: Local::now(),
            attachments: attachments
                .into_iter()
                .map(|(name, data)| Attachment { name, data })
                .collect(),
        }
    }

    /// Sends a chat message
    pub async fn send_chat(&self, message: &ChatMessage) -> std::result::Result<(), DartError> {
        if let Some(state) = self.session_states.read().await.get(&message.receiver) {
            // TODO the attachments are cloned here, can cause issues when large
            let message = Message::chat(message.text.clone(), message.attachments.clone());

            state
                .message_sender
                .send(message)
                .await
                .map_err(Error::from)?;
        }

        Ok(())
    }

    pub async fn start_screenshare(&self, contact: &Contact) -> std::result::Result<(), DartError> {
        self.start_screenshare
            .send((contact.peer_id, None))
            .await
            .map_err(Error::from)?;
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

    pub async fn set_model(&self, model: Option<Vec<u8>>) -> std::result::Result<(), DartError> {
        let model = if let Some(mode_bytes) = model {
            RnnModel::from_bytes(&mode_bytes).ok_or(String::from("invalid model"))?
        } else {
            RnnModel::default()
        };

        *self.denoise_model.write().await = model;
        Ok(())
    }

    /// Starts new sessions
    async fn session_manager(
        &self,
        start: &AsyncReceiver<PeerId>,
        screenshare: &AsyncReceiver<StartScreenshare>,
    ) -> Result<()> {
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
                SwarmEvent::NewExternalAddrOfPeer { .. } => (),
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
                while let Ok(mut incoming_streams) = control.accept(CHAT_PROTOCOL) {
                    while let Some((peer, stream)) = incoming_streams.next().await {
                        let state_option =
                            self_clone.session_states.read().await.get(&peer).cloned();

                        if let Some(state) = state_option {
                            if state.wants_stream.load(Relaxed) {
                                info!("sub-stream accepted for {}", peer);

                                if let Err(error) = state.stream_sender.send(stream).await {
                                    error!("error sending sub-stream to {}: {}", peer, error);
                                }

                                continue;
                            } else {
                                warn!("received a stream while {} did not want sub-stream, starting new session", peer);
                            }
                        } else {
                            info!("stream accepted for new session with {}", peer);
                        }

                        if let Err(error) = self_clone._start_session(peer, None, stream).await {
                            error!("error starting session with {}: {}", peer, error);
                        }
                    }

                    info!("incoming streams ended, trying to restart");
                }

                warn!("stopped accepting incoming streams; restarting controller");
                self_clone.restart_manager.notify_one();
            }
        });

        // handles the state needed for negotiating sessions
        // it is cleared each time a peer successfully connects
        let mut peer_states: HashMap<PeerId, PeerState> = HashMap::new();

        loop {
            let event = select! {
                // events are handled outside the select to help with spagetification
                event = swarm.select_next_some() => event,
                // restart the manager
                _ = self.restart_manager.notified() => {
                    break Err(ErrorKind::ManagerRestarted.into());
                }
                // start a new session
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

                    continue;
                }
                // starts a stream for outgoing screen shares
                result = screenshare.recv() => {
                    let (peer_id, header_option) = result?;
                    info!("starting screenshare for {} {:?}", peer_id, header_option);

                    if let Some(state) = self.session_states.read().await.get(&peer_id) {
                        let stop = Arc::new(Notify::new());
                        let dart_stop = DartNotify { inner: stop.clone() };

                        if let Some(header) = header_option {
                            if let Ok(stream) = swarm
                                .behaviour()
                                .stream
                                .new_control()
                                .open_stream(peer_id, CHAT_PROTOCOL)
                                .await {
                                let width = self.screenshare_config.width.load(Relaxed);
                                let height = self.screenshare_config.height.load(Relaxed);
                                spawn(screenshare::playback(stream, stop, state.download_bandwidth.clone(), header.encoder, width, height));
                                (self.screenshare_started.lock().await)(dart_stop, false).await;
                            }
                        } else {
                            if let Some(config) = self.screenshare_config.recording_config.read().await.clone() {
                                let message = Message::screenshare(config.encoder.into());

                                state
                                    .message_sender
                                    .send(message)
                                    .await
                                    .map_err(Error::from)?;

                                state.wants_stream.store(true, Relaxed);

                                if let Ok(stream) = state.stream_receiver.recv().await {
                                    spawn(screenshare::record(stream, stop, state.upload_bandwidth.clone(), config));
                                }

                                state.wants_stream.store(false, Relaxed);
                                (self.screenshare_started.lock().await)(dart_stop, true).await;
                            } else {
                                // TODO this should be blocked from occurring via the frontend i think
                                warn!("screenshare started without recording configuration");
                            }
                        }
                    } else {
                        warn!("screenshare started for a peer without a session: {}", peer_id);
                    }

                    continue;
                }
            };

            match event {
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    endpoint,
                    connection_id,
                    ..
                } => {
                    if peer_id == *self.network_config.relay_id.read().await {
                        // ignore the relay connection
                        continue;
                    } else if self.session_states.read().await.contains_key(&peer_id) {
                        // ignore connections with peers who have a session
                        continue;
                    }

                    let relayed = endpoint.is_relayed();
                    let listener = endpoint.is_listener();

                    info!(
                        "connection {} established with {} endpoint={:?} relayed={}",
                        connection_id, peer_id, endpoint, relayed
                    );

                    if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                        // insert the new connection
                        peer_state
                            .connections
                            .insert(connection_id, ConnectionState::new(relayed));
                        continue; // if the state already exists, the remaining logic is unnecessary
                    } else {
                        // insert the new state and new connection
                        peer_states
                            .insert(peer_id, PeerState::new(!listener, connection_id, relayed));
                    }

                    if let Some(contact) = (self.get_contact.lock().await)(peer_id.to_bytes()).await
                    {
                        if listener {
                            // a stream will be established by the other client
                            // the dialer already has the connecting status set
                            (self.session_status.lock().await)(
                                contact.peer_id(),
                                "Connecting".to_string(),
                            )
                            .await;
                        }
                    } else {
                        warn!("received a connection from an unknown peer: {:?}", peer_id);

                        if swarm.disconnect_peer_id(peer_id).is_err() {
                            error!("error disconnecting from unknown peer");
                        }

                        peer_states.remove(&peer_id);
                    }
                }
                SwarmEvent::OutgoingConnectionError {
                    peer_id,
                    error,
                    connection_id,
                } => {
                    if let Some(peer_id) = peer_id {
                        if self.session_states.read().await.contains_key(&peer_id) {
                            warn!(
                                "outgoing connection failed for {} because {}",
                                peer_id, error
                            );
                        } else if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                            warn!(
                                "outgoing connection failed for {} because {}",
                                peer_id, error
                            );
                            peer_state.connections.remove(&connection_id);
                        } else {
                            // if an outgoing error occurs when no connection is active, the session initialization failed
                            error!(
                                "session initialization failed for {} because {}",
                                peer_id, error
                            );
                            (self.session_status.lock().await)(
                                peer_id.to_string(),
                                "Inactive".to_string(),
                            )
                            .await;
                        }
                    }
                }
                SwarmEvent::ConnectionClosed {
                    peer_id,
                    cause,
                    connection_id,
                    ..
                } => {
                    warn!(
                        "connection {} closed with {} cause={:?}",
                        connection_id, peer_id, cause
                    );

                    if let Some(peer_state) = peer_states.get_mut(&peer_id) {
                        peer_state.connections.remove(&connection_id);
                    }
                }
                SwarmEvent::Behaviour(BehaviourEvent::Ping(event)) => {
                    let latency = event.result.map(|duration| duration.as_millis()).ok();

                    // update the latency for the peer's session
                    if let Some(state) = self.session_states.read().await.get(&event.peer) {
                        state.latency.store(latency.unwrap_or(0) as usize, Relaxed);
                        continue; // the remaining logic is not needed while a session is active
                    }

                    // if the session is still connecting, update the latency and try to choose a connection
                    if let Some(peer_state) = peer_states.get_mut(&event.peer) {
                        // the dialer chooses the connection
                        if !peer_state.dialer {
                            continue;
                        }

                        // update the latency for the peer's connections
                        if let Some(connection_latency) =
                            peer_state.connections.get_mut(&event.connection)
                        {
                            connection_latency.latency = latency;
                        } else {
                            warn!(
                                "received a ping for an unknown connection id={}",
                                event.connection
                            );
                        }

                        info!("connection states: {:?}", peer_state.connections);

                        if peer_state.latencies_missing() {
                            // only start a session if all connections have latency
                            debug!("not trying to establish a session with {} because not all connections have latency", event.peer);
                            continue;
                        } else if peer_state.relayed_only() {
                            // only start a session if there is a non-relayed connection
                            debug!("not trying to establish a session with {} because all connections are relayed", event.peer);
                            continue;
                        }

                        // choose the connection with the lowest latency, prioritizing non-relay connections
                        let connection = peer_state
                            .connections
                            .iter()
                            .min_by(|a, b| {
                                match (a.1.relayed, b.1.relayed) {
                                    (false, true) => std::cmp::Ordering::Less, // prioritize non-relay connections
                                    (true, false) => std::cmp::Ordering::Greater, // prioritize non-relay connections
                                    _ => a.1.latency.cmp(&b.1.latency), // compare latencies if both have the same relay status
                                }
                            })
                            .map(|(id, _)| id);

                        if let Some(connection_id) = connection {
                            info!("using connection id={} for {}", connection_id, event.peer);

                            // close the other connections
                            peer_state
                                .connections
                                .iter()
                                .filter(|(id, _)| *id != connection_id)
                                .for_each(|(id, _)| {
                                    swarm.close_connection(*id);
                                });

                            let control = swarm.behaviour().stream.new_control();

                            if self.chat_rooms.read().await.contains(&event.peer) {
                                // open the room streams and start necessary threads
                                self._join_room(event.peer, control).await;
                                peer_states.remove(&event.peer);
                            } else {
                                // open a session control stream and start the session controller
                                self.open_stream(event.peer, control, &mut peer_states)
                                    .await;
                            }
                        } else {
                            warn!("no connection available for {}", event.peer);
                        }
                    }
                }
                SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                    peer_id,
                    info,
                    ..
                })) => {
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
                }
                // TODO validate that this logic successfully handles cases where the relay is the only available connection
                SwarmEvent::Behaviour(BehaviourEvent::Dcutr(dcutr::Event {
                    remote_peer_id,
                    result,
                })) => {
                    debug!("ductr event with {}: {:?}", remote_peer_id, result);

                    if let Some(peer_state) = peer_states.get(&remote_peer_id) {
                        if peer_state.relayed_only() && result.is_err() {
                            info!("ductr failed while relayed_only, falling back to relay");
                            let control = swarm.behaviour().stream.new_control();
                            self.open_stream(remote_peer_id, control, &mut peer_states)
                                .await;
                        }
                    }
                }
                event => {
                    debug!("other swarm event: {:?}", event);
                }
            }
        }
    }

    /// Called by the dialer to open a stream and session
    async fn open_stream(
        &self,
        peer_id: PeerId,
        mut control: Control,
        peer_states: &mut HashMap<PeerId, PeerState>,
    ) {
        // it may take multiple tries to open the stream because the of the (dumb) RNG in the stream handler
        loop {
            if let Ok(stream) = control.open_stream(peer_id, CHAT_PROTOCOL).await {
                info!("opened stream with {}, starting new session", peer_id);

                if let Err(error) = self._start_session(peer_id, Some(control), stream).await {
                    error!("error starting session with {}: {}", peer_id, error);
                }

                // the peer state is no longer needed
                peer_states.remove(&peer_id);
                break;
            }
        }
    }

    async fn _join_room(&self, peer: PeerId, mut control: Control) {
        // it may take multiple tries to open the stream because the of the (dumb) RNG in the stream handler
        let control_stream = loop {
            let control_stream = control.open_stream(peer, ROOM_PROTOCOL).await;

            if let Ok(control_stream) = control_stream {
                break control_stream;
            }
        };

        let audio_stream = loop {
            let audio_stream = control.open_stream(peer, ROOM_PROTOCOL).await;

            if let Ok(audio_stream) = audio_stream {
                break audio_stream;
            }
        };

        let mut control_transport = LengthDelimitedCodec::builder()
            .max_frame_length(usize::MAX)
            .length_field_type::<u64>()
            .new_framed(control_stream.compat());

        let (message_sender, message_receiver) = unbounded_async::<Message>();

        // create the state and a clone of it for the chat room
        let state = Arc::new(SessionState::new(&message_sender));
        let state_clone = Arc::clone(&state);

        self.session_states.write().await.insert(peer, state);

        // alert the UI that this chat room is now connected
        (self.session_status.lock().await)(peer.to_string(), "Connected".to_string()).await;

        let self_clone = self.clone();
        spawn(async move {
            let stop_io = Arc::new(Notify::new());

            let result = self_clone.call(
                Some(audio_stream),
                Some(&mut control_transport),
                Some(message_receiver),
                Some(&state_clone),
                Some(peer),
                &stop_io,
            ).await;

            warn!("chat room call ended: {:?}", result);

            stop_io.notify_waiters();

            // cleanup
            self_clone
                .session_states
                .write()
                .await
                .remove(&peer);

            (self_clone.session_status.lock().await)(
                peer.to_string(),
                "Inactive".to_string(),
            )
                .await;

            info!("chat room {} cleaned up", peer);
        });
    }

    /// A wrapper which manages the session throughout its lifetime
    async fn _start_session(
        &self,
        peer_id: PeerId,
        mut control: Option<Control>,
        stream: Stream,
    ) -> Result<()> {
        let contact = (self.get_contact.lock().await)(peer_id.to_bytes())
            .await
            .ok_or(ErrorKind::MissingContact)?;

        let message_channel = unbounded_async::<Message>();

        // create the state and a clone of it for the session
        let state = Arc::new(SessionState::new(&message_channel.0));
        let state_clone = Arc::clone(&state);

        let mut states = self.session_states.write().await;

        if let Some(state) = states.get(&peer_id) {
            warn!("{} already has a session", contact.nickname);

            if state.in_call.load(Relaxed) {
                // if the session was in a call, end it so the session can end
                self.end_call.notify_one();
            }

            // stop the session
            state.stop_session.notify_one();
        }

        // insert the new state
        states.insert(contact.peer_id, state.clone());

        // alert the UI that this session is now connected
        (self.session_status.lock().await)(contact.peer_id(), "Connected".to_string()).await;

        let self_clone = self.clone();
        spawn(async move {
            let contact_clone = contact.clone();

            // the length delimited transport used for the session
            let mut transport = LengthDelimitedCodec::builder()
                .max_frame_length(usize::MAX)
                .length_field_type::<u64>()
                .new_framed(stream.compat());

            // controls keep alive messages
            let mut keep_alive = interval(Duration::from_secs(10));

            let result = loop {
                let future = self_clone.session(
                    &contact,
                    control.as_mut(),
                    &mut transport,
                    &state_clone,
                    &message_channel,
                    &mut keep_alive,
                );

                if let Err(error) = future.await {
                    if state.in_call.load(Relaxed) {
                        info!("session error while call active, alerting ui");
                        (self_clone.call_ended.lock().await)(error.to_string(), false).await;
                    }

                    match error.kind {
                        ErrorKind::KanalReceive(_)
                        | ErrorKind::TransportRecv
                        | ErrorKind::TransportSend => break Err(error),
                        ErrorKind::SessionStopped => break Ok(()),
                        _ => error!("Session error: {:?}", error),
                    }
                }

                // the session is now safe to restart
                state.in_call.store(false, Relaxed);
            };

            // result is *only* Ok when the session has been stopped
            if let Err(error) = result {
                error!("Session error for {}: {}", contact_clone.nickname, error);
            } else {
                warn!("Session for {} stopped", contact_clone.nickname);
                return; // the session has already been cleaned up
            }

            // cleanup
            self_clone
                .session_states
                .write()
                .await
                .remove(&contact_clone.peer_id);

            (self_clone.session_status.lock().await)(
                contact_clone.peer_id(),
                "Inactive".to_string(),
            )
            .await;

            info!("Session for {} cleaned up", contact_clone.nickname);
        });

        Ok(())
    }

    /// The session logic
    async fn session(
        &self,
        contact: &Contact,
        control: Option<&mut Control>,
        transport: &mut Transport<TransportStream>,
        state: &Arc<SessionState>,
        message_channel: &(AsyncSender<Message>, AsyncReceiver<Message>),
        keep_alive: &mut Interval,
    ) -> Result<()> {
        info!("[{}] session waiting for event", contact.nickname);

        select! {
            result = read_message::<Message, _>(transport) => {
                let mut ringtone = None;

                info!("received {:?} from {}", result, contact.nickname);

                match result? {
                    Message { message: Some(message::Message::Hello(message)) } => {
                        if !message.ringtone.is_empty() && self.play_custom_ringtones.load(Relaxed) {
                            ringtone = Some(message.ringtone);
                        }
                    },
                    Message { message: Some(message::Message::KeepAlive(_) | message::Message::ConnectionInterrupted(_) | message::Message::ConnectionRestored(_)) } => {
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
                    write_message(transport, busy).await?;
                    return Ok(());
                }

                state.in_call.store(true, Relaxed); // blocks the session from being restarted

                let cancel_prompt = Arc::new(Notify::new());
                // a cancel Notify that can be used in the frontend
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
                            write_message(transport, hello_ack).await?;

                            // start the handshake
                            self.handshake(transport, control, contact.peer_id, message_channel, &state.stream_receiver, state).await?;
                            keep_alive.reset(); // start sending normal keep alive messages
                        } else {
                            // reject the call if not accepted
                            let reject = Message::reject();
                            write_message(transport, reject).await?;
                        }
                    }
                    result = read_message::<Message, _>(transport) => {
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
            _ = state.start_call.notified() => {
                state.in_call.store(true, Relaxed); // blocks the session from being restarted

                // queries the other client for a call
                let ringtone = (self.load_ringtone.lock().await)().await;
                let hello = Message::hello(ringtone);
                write_message(transport, hello).await?;

                loop {
                    select! {
                        result = timeout(HELLO_TIMEOUT, read_message(transport)) => {
                            // handles a variety of outcomes in response to Hello
                            match result?? {
                                Message { message: Some(message::Message::HelloAck(_)) } => {
                                    self.handshake(transport, control, contact.peer_id, message_channel, &state.stream_receiver, state).await?;
                                    keep_alive.reset(); // start sending normal keep alive messages
                                }
                                Message { message: Some(message::Message::Reject(_)) } => {
                                    (self.call_ended.lock().await)(format!("{} did not accept the call", contact.nickname), true).await;
                                },
                                Message { message: Some(message::Message::Busy(_)) } => {
                                    (self.call_ended.lock().await)(format!("{} is busy", contact.nickname), true).await;
                                }
                                // keep alive messages are sometimes received here
                                Message { message: Some(message::Message::KeepAlive(_)) } => continue,
                                message => {
                                    // the front end needs to know that the call ended here
                                    (self.call_ended.lock().await)(format!("Received an unexpected message from {}", contact.nickname), true).await;
                                    warn!("received unexpected {:?} from {} [stopped call process]", message, contact.nickname);
                                }
                            }

                            break;
                        }
                        _ = self.end_call.notified() => {
                            info!("end call notified while waiting for hello ack");
                            let goodbye = Message::goodbye();
                            write_message(transport, goodbye).await?;
                        }
                    }
                }

                Ok(())
            }
            // state will never notify while a call is active
            _ = state.stop_session.notified() => {
                info!("session state stop notified for {}", contact.nickname);
                Err(ErrorKind::SessionStopped.into())
            },
            _ = keep_alive.tick() => {
                debug!("sending keep alive to {}", contact.nickname);
                let keep_alive = Message::keep_alive();
                write_message(transport, keep_alive).await?;
                Ok(())
            },
        }
    }

    /// Gets everything ready for the call
    async fn handshake(
        &self,
        transport: &mut Transport<TransportStream>,
        mut control: Option<&mut Control>,
        peer: PeerId,
        message_channel: &(AsyncSender<Message>, AsyncReceiver<Message>),
        stream_receiver: &AsyncReceiver<Stream>,
        state: &Arc<SessionState>,
    ) -> Result<()> {
        // change the session state to accept incoming audio streams
        state.wants_stream.store(true, Relaxed);

        // TODO evaluate this loop's performance in handling unexpected messages
        let stream = loop {
            let future = async {
                let stream = if let Some(control) = control.as_mut() {
                    // if dialer, open stream
                    control.open_stream(peer, CHAT_PROTOCOL).await?
                } else {
                    // if listener, receive stream
                    stream_receiver.recv().await?
                };

                Ok::<_, Error>(stream)
            };

            select! {
                stream = future => break stream?,
                // handle unexpected messages while waiting for the audio stream
                // these messages tend to be from previous calls close together
                result = read_message::<Message, _>(transport) => {
                    warn!("received unexpected message while waiting for audio stream: {:?}", result);
                    // return Err(ErrorKind::UnexpectedMessage.into());
                }
            }
        };

        // change the app call state
        self.in_call.store(true, Relaxed);
        // change the session state
        state.wants_stream.store(false, Relaxed);
        // show the overlay
        self.overlay.show();

        // stop_io must notify when the call ends, so it is external to the call function
        let stop_io = Arc::new(Notify::new());

        let result = self
            .call(
                Some(stream),
                Some(transport),
                Some(message_channel.1.clone()),
                Some(state),
                Some(peer),
                &stop_io,
            )
            .await;

        info!("call ended in handshake");

        // ensure that all background i/o threads are stopped
        stop_io.notify_waiters();
        info!("notified stop_io");

        // the call has ended
        self.in_call.store(false, Relaxed);
        // hide the overlay
        self.overlay.hide();

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
        peer: Option<PeerId>,
        stop_io: &Arc<Notify>,
    ) -> Result<()> {
        // if any of the values required for a normal call is missing, the call is an audio test
        let audio_test = transport.is_none()
            || stream.is_none()
            || message_receiver.is_none()
            || state.is_none()
            || peer.is_none();

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

        // shared values for various statistics
        let latency = state
            .map(|state| Arc::clone(&state.latency))
            .unwrap_or_default();
        let upload_bandwidth = state
            .map(|state| Arc::clone(&state.upload_bandwidth))
            .unwrap_or_default();
        let download_bandwidth = state
            .map(|state| Arc::clone(&state.download_bandwidth))
            .unwrap_or_default();

        let (receiving_sender, receiving_receiver) = unbounded_async::<bool>();

        // get the output device and its default configuration
        let output_device = get_output_device(&self.output_device, &self.host).await?;
        let output_config = output_device.default_output_config()?;
        info!("output device: {:?}", output_device.name());

        // get the input device and its default configuration
        let input_device = self.get_input_device().await?;
        let input_config = input_device.default_input_config()?;
        info!("input_device: {:?}", input_device.name());

        let mut audio_header = AudioHeader {
            channels: input_config.channels() as u32,
            sample_rate: input_config.sample_rate().0,
            sample_format: input_config.sample_format().to_string(),
        };

        // rnnoise requires a 48kHz sample rate
        if denoise {
            audio_header.sample_rate = 48_000;
        }

        let mut audio_transport = stream.map(|stream| {
            LengthDelimitedCodec::builder()
                .max_frame_length(TRANSFER_BUFFER_SIZE)
                .length_field_type::<u16>()
                .new_framed(stream.compat())
        });

        let remote_input_config: AudioHeader = if audio_test {
            // the client will be receiving its own audio
            audio_header
        } else {
            // exchange audio headers using the audio transport
            let transport = audio_transport.as_mut().unwrap();
            write_message(transport, audio_header).await?;
            read_message(transport).await?
        };

        info!("exchanged audio headers");

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
        std::thread::spawn(move || {
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
        std::thread::spawn(move || {
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

        spawn(statistics_collector(
            input_rms_receiver,
            output_rms_receiver,
            latency,
            Arc::clone(&upload_bandwidth),
            Arc::clone(&download_bandwidth),
            Arc::clone(&self.statistics),
            Arc::clone(stop_io),
        ));

        if audio_test {
            spawn(loopback(
                processed_input_receiver,
                output_sender,
                Arc::clone(stop_io),
            ));

            // select! {
            //     _ = loopback_handle => (),
            //     // this unwrap is safe because the processor thread will not panic
            //     // result = output_processor_future => result?.unwrap()?,
            //     // this unwrap is safe because the processor thread will not panic
            //     // result = input_processor_future => result?.unwrap()?,
            //     _ = self.end_call.notified() => (),
            //     // result = statistics_handle => result??,
            // }

            self.end_call.notified().await;
            return Ok(());
        }

        let (write, read) = audio_transport.unwrap().split();

        spawn(audio_input(
            processed_input_receiver,
            write,
            Arc::clone(stop_io),
            upload_bandwidth,
        ));

        spawn(audio_output(
            output_sender,
            read,
            Arc::clone(stop_io),
            download_bandwidth,
            receiving_sender,
        ));

        let transport = transport.as_mut().unwrap();
        let message_receiver = message_receiver.unwrap();

        // shared values used in the call controller
        let end_call = Arc::clone(&self.end_call);
        let message_received = Arc::clone(&self.message_received);
        let call_state = Arc::clone(&self.call_state);

        let controller_future = call_controller(
            transport,
            message_receiver,
            end_call,
            receiving_receiver,
            message_received,
            call_state,
            self.start_screenshare.clone(),
            peer.unwrap(),
            self.identity.read().await.public().to_peer_id(),
        );

        info!("call controller starting");

        match controller_future.await {
            Ok(message) => (self.call_ended.lock().await)(message, true).await,
            Err(error) => match error.kind {
                ErrorKind::CallEnded => (), // when the call is ended externally, no UI notification is needed
                _ => (self.call_ended.lock().await)(error.to_string(), false).await,
            },
        }

        stop_io.notify_waiters();
        info!("call controller returned and was handled, call returning");
        Ok(())
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

/// a state used for session negotiation
#[derive(Debug)]
struct PeerState {
    /// when true the peer's identity addresses will not be dialed
    dialed: bool,

    /// when true the peer is the dialer
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

    fn relayed_only(&self) -> bool {
        self.connections.iter().all(|(_, state)| state.relayed)
    }

    fn latencies_missing(&self) -> bool {
        self.connections
            .iter()
            .any(|(_, state)| state.latency.is_none())
    }
}

/// the state of a single connection during session negotiation
#[derive(Debug)]
struct ConnectionState {
    /// the latency is ms when available
    latency: Option<u128>,

    /// whether the connection is relayed
    relayed: bool,
}

impl ConnectionState {
    fn new(relayed: bool) -> Self {
        Self {
            latency: None,
            relayed,
        }
    }
}

/// wraps a cpal stream to unsafely make it send
pub(crate) struct SendStream {
    pub(crate) stream: CpalStream,
}

unsafe impl Send for SendStream {}

/// a message containing either a frame of audio or silence
enum ProcessorMessage {
    Data(Bytes),
    Silence,
}

/// common processor message constructors
impl ProcessorMessage {
    fn slice(bytes: &'static [u8]) -> Self {
        Self::Data(Bytes::from(bytes))
    }

    fn silence() -> Self {
        Self::Silence
    }

    fn bytes(frame: Bytes) -> Self {
        Self::Data(frame)
    }
}

/// shared values for a single session
struct SessionState {
    /// signals the session to initiate a call
    start_call: Notify,

    /// stops the session normally
    stop_session: Notify,

    /// if the session is in a call
    in_call: AtomicBool,

    /// a reusable sender for messages while a call is active
    message_sender: AsyncSender<Message>,

    /// forwards sub-streams to the session
    stream_sender: AsyncSender<Stream>,

    /// receives sub-streams for the session
    stream_receiver: AsyncReceiver<Stream>,

    /// a shared latency value for the session from libp2p ping
    latency: Arc<AtomicUsize>,

    /// a shared upload bandwidth value for the session
    upload_bandwidth: Arc<AtomicUsize>,

    /// a shared download bandwidth value for the session
    download_bandwidth: Arc<AtomicUsize>,

    /// whether the session wants a sub-stream
    wants_stream: Arc<AtomicBool>,
}

impl SessionState {
    fn new(
        message_sender: &AsyncSender<Message>,
    ) -> Self {
        let stream_channel = unbounded_async();

        Self {
            start_call: Notify::new(),
            stop_session: Notify::new(),
            in_call: AtomicBool::new(false),
            message_sender: message_sender.clone(),
            stream_sender: stream_channel.0,
            stream_receiver: stream_channel.1,
            latency: Default::default(),
            upload_bandwidth: Default::default(),
            download_bandwidth: Default::default(),
            wants_stream: Default::default(),
        }
    }
}

/// an AtomicBool Flag which is not loaded from the atomic every time
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

/// processed statistics for the frontend
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
    /// the relay server's address
    relay_address: Arc<RwLock<SocketAddr>>,

    /// the relay server's peer id
    relay_id: Arc<RwLock<PeerId>>,

    /// the libp2p port for the swarm
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

#[frb(opaque)]
#[derive(Clone, Serialize, Deserialize)]
pub struct ScreenshareConfig {
    /// the screenshare capabilities. default until loaded
    #[serde(skip)]
    capabilities: Arc<RwLock<Capabilities>>,

    /// a validated recording configuration
    #[serde(with = "rwlock_option_recording_config")]
    recording_config: Arc<RwLock<Option<RecordingConfig>>>,

    /// the default width of the playback window
    #[serde(
        serialize_with = "atomic_u32_serialize",
        deserialize_with = "atomic_u32_deserialize"
    )]
    width: Arc<AtomicU32>,

    /// the default height of the playback window
    #[serde(
        serialize_with = "atomic_u32_serialize",
        deserialize_with = "atomic_u32_deserialize"
    )]
    height: Arc<AtomicU32>,
}

impl Default for ScreenshareConfig {
    fn default() -> Self {
        Self {
            capabilities: Default::default(),
            recording_config: Default::default(),
            width: Arc::new(AtomicU32::new(1280)),
            height: Arc::new(AtomicU32::new(720)),
        }
    }
}

impl ScreenshareConfig {
    // this function must be async to use spawn
    pub async fn new(config_str: String) -> ScreenshareConfig {
        let config: ScreenshareConfig = serde_json::from_str(&config_str).unwrap_or_default();

        let capabilities_clone = Arc::clone(&config.capabilities);
        spawn(async move {
            let now = Instant::now();
            let c = Capabilities::new().await;
            *capabilities_clone.write().await = c;
            info!("Capabilities loaded in {:?}", now.elapsed());
        });

        config
    }

    pub async fn capabilities(&self) -> Capabilities {
        self.capabilities.read().await.clone()
    }

    pub async fn recording_config(&self) -> Option<RecordingConfig> {
        self.recording_config.read().await.clone()
    }

    pub async fn update_recording_config(
        &self,
        encoder: String,
        device: String,
        bitrate: u32,
        framerate: u32,
        height: Option<u32>,
    ) -> std::result::Result<(), DartError> {
        let encoder = Encoder::from_str(&encoder).map_err(|_| ErrorKind::InvalidEncoder)?;

        let recording_config = RecordingConfig {
            encoder,
            // TODO do something with the device names idk
            device: screenshare::Device::DesktopDuplication,
            bitrate,
            framerate,
            height,
        };

        if let Ok(status) = recording_config.test_config().await {
            if status.success() {
                *self.recording_config.write().await = Some(recording_config);
                return Ok(());
            }
        }

        Err("Invalid configuration".to_string().into())
    }

    #[frb(sync)]
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// capabilities for ffmpeg and ffplay supported by this client
#[derive(Default, Debug, Clone)]
#[frb(opaque)]
pub struct Capabilities {
    pub(crate) available: bool,

    pub(crate) encoders: Vec<Encoder>,

    pub(crate) decoders: Vec<Decoder>,

    pub(crate) devices: Vec<screenshare::Device>,
}

impl Capabilities {
    #[frb(sync)]
    pub fn encoders(&self) -> Vec<String> {
        self.encoders.iter().map(|e| e.to_string()).collect()
    }

    #[frb(sync)]
    pub fn devices(&self) -> Vec<String> {
        self.devices.iter().map(|d| d.to_string()).collect()
    }
}

/// recording config for screenshare
#[derive(Debug, Clone, Serialize, Deserialize)]
#[frb(opaque)]
pub struct RecordingConfig {
    pub(crate) encoder: Encoder,

    pub(crate) device: screenshare::Device,

    pub(crate) bitrate: u32,

    pub(crate) framerate: u32,

    /// the height for the video output
    pub(crate) height: Option<u32>,
}

impl RecordingConfig {
    #[frb(sync)]
    pub fn encoder(&self) -> String {
        let encoder_str: &str = self.encoder.into();
        encoder_str.to_string()
    }

    #[frb(sync)]
    pub fn device(&self) -> String {
        self.device.to_string()
    }

    #[frb(sync)]
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }

    #[frb(sync)]
    pub fn framerate(&self) -> u32 {
        self.framerate
    }

    #[frb(sync)]
    pub fn height(&self) -> Option<u32> {
        self.height
    }
}

mod rwlock_option_recording_config {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use crate::api::audio_chat::RecordingConfig;

    pub fn serialize<S>(
        value: &Arc<RwLock<Option<RecordingConfig>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let lock = value.blocking_read();
        lock.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Arc<RwLock<Option<RecordingConfig>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Option::<RecordingConfig>::deserialize(deserializer)?;
        Ok(Arc::new(RwLock::new(inner)))
    }
}

fn atomic_u32_serialize<S>(
    value: &Arc<AtomicU32>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let value = value.load(Relaxed);
    serializer.serialize_u32(value)
}

fn atomic_u32_deserialize<'de, D>(deserializer: D) -> std::result::Result<Arc<AtomicU32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    Ok(Arc::new(AtomicU32::new(value)))
}

/// a shared notifier that can be passed to dart code
#[frb(opaque)]
pub struct DartNotify {
    inner: Arc<Notify>,
}

impl DartNotify {
    /// public notified function for dart
    pub async fn notified(&self) {
        self.inner.notified().await;
    }

    /// notifies one waiter
    #[frb(sync)]
    pub fn notify(&self) {
        self.inner.notify_waiters();
    }
}

pub struct ChatMessage {
    pub text: String,

    receiver: PeerId,

    timestamp: DateTime<Local>,

    pub(crate) attachments: Vec<Attachment>,
}

impl ChatMessage {
    #[frb(sync)]
    pub fn is_sender(&self, identity: String) -> bool {
        self.receiver.to_string() != identity
    }

    #[frb(sync)]
    pub fn time(&self) -> String {
        self.timestamp.format("%l:%M %p").to_string()
    }

    #[frb(sync)]
    pub fn attachments(&self) -> Vec<(String, Vec<u8>)> {
        self.attachments
            .iter()
            .map(|a| (a.name.clone(), a.data.clone()))
            .collect()
    }

    #[frb(sync)]
    pub fn clear_attachments(&mut self) {
        for attachment in self.attachments.iter_mut() {
            attachment.data.truncate(0);
        }
    }
}

/// the call controller
async fn call_controller(
    transport: &mut Transport<TransportStream>,
    receiver: AsyncReceiver<Message>,
    end_call: Arc<Notify>,
    receiving: AsyncReceiver<bool>,
    message_received: Arc<Mutex<dyn Fn(ChatMessage) -> DartFnFuture<()> + Send>>,
    call_state: Arc<Mutex<dyn Fn(bool) -> DartFnFuture<()> + Send>>,
    start_screenshare: AsyncSender<StartScreenshare>,
    peer: PeerId,
    identity: PeerId,
) -> Result<String> {
    // in case this value has been used in a previous call, reset to false
    CONNECTED.store(false, Relaxed);

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
    let disconnect_duration = Duration::from_millis(1_500);

    let (state_sender, state_receiver) = unbounded_async::<bool>();

    loop {
        select! {
            // receives and handles messages from the callee
            result = read_message(transport) => {
                let message: Message = result?;

                match message.message {
                    Some(message::Message::Goodbye(message)) => {
                        debug!("received goodbye, reason = {:?}", message.reason);
                        break Ok(message.reason);
                    },
                    Some(message::Message::Chat(message)) => {
                        (message_received.lock().await)(ChatMessage {
                            text: message.text,
                            receiver: identity,
                            timestamp: Local::now(),
                            attachments: message.attachments,
                        }).await;
                    }
                    Some(message::Message::ConnectionInterrupted(_)) => {
                        // info!("received connection interrupted message r={} rr={}", is_receiving, remote_is_receiving);

                        let receiving = is_receiving && remote_is_receiving;
                        remote_is_receiving = false;
                        state_sender.send(receiving).await?;
                    }
                    Some(message::Message::ConnectionRestored(_)) => {
                        // info!("received connection restored message r={} rr={}", is_receiving, remote_is_receiving);

                        if remote_is_receiving {
                            warn!("received connection restored message while already receiving");
                            continue;
                        }

                        remote_is_receiving = true;
                        state_sender.send(false).await?;
                    }
                    Some(message::Message::ScreenshareHeader(header)) => {
                        info!("received screenshare header {:?}", header);
                        start_screenshare.send((peer, Some(header))).await?;
                    }
                    Some(message::Message::RoomWelcome(welcome)) => {
                        warn!("received room welcome message {:?}", welcome);
                    }
                    Some(message::Message::RoomJoin(join)) => {
                        warn!("received room join message {:?}", join);
                    }
                    Some(message::Message::RoomLeave(leave)) => {
                        warn!("received room leave message {:?}", leave);
                    }
                    _ => error!("call controller unexpected message: {:?}", message),
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
                    // info!("state switched to not receiving r={} rr={}", is_receiving, remote_is_receiving);

                    // the instant the disconnect began
                    disconnected_at = Instant::now();
                    // notify the ui in 2 seconds if the disconnect hasn't ended
                    notify_ui = disconnected_at + disconnect_duration;
                } else if is_receiving && remote_is_receiving {
                    let elapsed = disconnected_at.elapsed();
                    // info!("reconnected after {}ms interruption", elapsed.as_millis());

                    // update the overlay to connected
                    if !CONNECTED.swap(true, Relaxed) {
                        // update the call state in the UI
                        (call_state.lock().await)(false).await;
                    }

                    // record the disconnect
                    disconnect_durations.push_back((disconnected_at, elapsed));
                    // prevents any notification to the ui as audio is being received
                    notify_ui = Instant::now() + Duration::from_secs(86400 * 365 * 30);
                }
                // else if is_receiving ^ remote_is_receiving {
                //     info!("partial reconnect r={} rr={}", is_receiving, remote_is_receiving);
                // } else {
                //     info!("full disconnect r={} rr={}", is_receiving, remote_is_receiving)
                // }
            },
            // receives when the receiving state changes
            Ok(receiving) = receiving.recv() => {
                // info!("received receiving state: {} | r={} rr={}", receiving, is_receiving, remote_is_receiving);

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
                    // TODO it appears that this is happening occasionally
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
async fn audio_input(
    input_receiver: AsyncReceiver<ProcessorMessage>,
    mut socket: SplitSink<Transport<TransportStream>, Bytes>,
    stop_io: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
) -> Result<()> {
    // a static byte used as the silence signal
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
        // this future should never complete
        result = future => result,
        _ = stop_io.notified() => {
            debug!("Input to socket ended");
            Ok(())
        },
    }
}

/// Receives audio data from the socket and sends it to the output processor
async fn audio_output(
    sender: AsyncSender<ProcessorMessage>,
    mut socket: SplitStream<Transport<TransportStream>>,
    stop_io: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
    receiving: AsyncSender<bool>,
) -> Result<()> {
    let mut is_receiving = false;

    let future = async {
        loop {
            match timeout(TIMEOUT_DURATION, socket.next()).await {
                Ok(Some(Ok(message))) => {
                    if !is_receiving {
                        is_receiving = true;
                        _ = receiving.send(is_receiving).await;
                    }

                    let len = message.len();
                    bandwidth.fetch_add(len, Relaxed);

                    match len {
                        TRANSFER_BUFFER_SIZE => {
                            _ = sender.try_send(ProcessorMessage::bytes(message.freeze()));
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
                    debug!("Socket output ended with None");
                    break Ok(());
                }
                Err(_) if is_receiving => {
                    is_receiving = false;
                    _ = receiving.send(is_receiving).await;
                }
                Err(_) => (),
            }
        }
    };

    select! {
        result = future => result,
        _ = stop_io.notified() => {
            debug!("Socket output ended from stop_io");
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
    // the interval for statistics updates
    let mut update_interval = interval(Duration::from_millis(100));
    // the interval for the input_max and output_max to decrease
    let mut reset_interval = interval(Duration::from_secs(5));

    let stop = Arc::new(AtomicBool::new(false));

    let mut input_max = 0_f32;
    let mut output_max = 0_f32;

    let stop_clone = Arc::clone(&stop);
    spawn(async move {
        notify.notified().await;
        stop_clone.store(true, Relaxed);
        info!("statistics collector set stop=true");
    });

    while !stop.load(Relaxed) {
        select! {
            _ = update_interval.tick() => {
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
            _ = reset_interval.tick() => {
                input_max /= 2_f32;
                output_max /= 2_f32;
            }
        }
    }

    // zero out the statistics when the collector ends
    let statistics = Statistics::default();
    (callback.lock().await)(statistics).await;

    LATENCY.store(0, Relaxed);
    LOSS.store(0_f64, Relaxed);
    CONNECTED.store(false, Relaxed);

    info!("statistics collector returning");
    Ok(())
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

        _ = sender.try_send(ProcessorMessage::slice(bytes));
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
        let local_max = window.into_iter().reduce(f32::max).unwrap_or(0_f32);
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
