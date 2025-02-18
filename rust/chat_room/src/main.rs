mod behaviour;
mod error;

use std::collections::HashMap;
use std::mem;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::behaviour::{Behaviour, BehaviourEvent};
use crate::error::{Error, ErrorKind};
use kanal::{bounded_async, unbounded_async, AsyncReceiver, AsyncSender};
use libp2p::bytes::Bytes;
use libp2p::futures::stream::{SplitSink, SplitStream};
use libp2p::futures::{Sink, SinkExt, StreamExt};
use libp2p::identity::Keypair;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    autonat, dcutr, identify, noise, ping, tcp, yamux, Multiaddr, PeerId, Stream, StreamProtocol,
};
use messages::{AudioHeader, Message};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tokio::{join, select};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};

type Subscribers = Arc<RwLock<HashMap<PeerId, HashMap<PeerId, Option<AsyncSender<Bytes>>>>>>;
type TransportStream = Compat<Stream>;
type Transport<T> = Framed<T, LengthDelimitedCodec>;
type Result<T> = std::result::Result<T, Error>;

const CHANNEL_SIZE: usize = 2_400;
const FRAME_SIZE: usize = 480;
const BUFFER_SIZE: usize = FRAME_SIZE * mem::size_of::<i16>();
const FRAME_DURATION: Duration = Duration::from_millis(1_000 / (48_800 / FRAME_SIZE as u64));
const PROTOCOL: StreamProtocol = StreamProtocol::new("/audio-chat-room/0.0.1");

#[tokio::main]
async fn main() {
    chat_room().await.expect("chat room failed");
}

/// main function to run a chat room server
async fn chat_room() -> Result<()> {
    let subscribers: Subscribers = Default::default();

    let identity = generate_ed25519(15);
    println!("identity: {}", identity.public().to_peer_id().to_string());

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity)
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
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
                "/audio-chat-room/0.0.1".to_string(),
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

    // Listen on all interfaces
    let listen_addr_tcp = Multiaddr::empty()
        .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Tcp(40142));

    swarm.listen_on(listen_addr_tcp)?;

    let listen_addr_quic = Multiaddr::empty()
        .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Udp(40142))
        .with(Protocol::QuicV1);

    swarm.listen_on(listen_addr_quic)?;

    let socket_address = SocketAddr::from_str("5.78.76.47:40142").unwrap();
    let relay_identity =
        PeerId::from_str("12D3KooWMpeKAbMK4BTPsQY3rG7XwtdstseHGcq7kffY8LToYYKK").unwrap();

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
            println!("connected to relay with tcp");
            relay_address = relay_address_tcp.with(Protocol::P2pCircuit);
        }
    } else {
        println!("connected to relay with udp");
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
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent { .. })) => {
                println!("Told relay its public address");
                told_relay_observed_addr = true;
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                info: identify::Info { .. },
                ..
            })) => {
                println!("Relay told us our observed address");
                learned_observed_addr = true;
            }
            // no other event occurs during a successful initialization
            event => {
                println!("Unexpected event during initialization {:?}", event);
                return Err(ErrorKind::UnexpectedSwarmEvent.into());
            }
        }

        if learned_observed_addr && told_relay_observed_addr {
            break;
        }
    }

    swarm.listen_on(relay_address)?;

    let (stream_sender, stream_receiver) = unbounded_async();

    tokio::spawn(controller(stream_receiver));

    tokio::spawn(simulated_client(1));

    let mut control = swarm.behaviour().stream.new_control();
    tokio::spawn(async move {
        while let Ok(mut incoming_streams) = control.accept(PROTOCOL) {
            while let Some((peer, stream)) = incoming_streams.next().await {
                if subscribers.read().await.contains_key(&peer) {
                    println!("incoming audio stream from {}", peer);

                    let transport = LengthDelimitedCodec::builder()
                        .max_frame_length(BUFFER_SIZE)
                        .length_field_type::<u16>()
                        .new_framed(stream.compat());

                    let (mut write, mut read) = transport.split();

                    let audio_header = AudioHeader {
                        channels: 1,
                        sample_rate: 48_000,
                        sample_format: "f32".to_string(),
                    };

                    // TODO i do not like blocking this loop to do this here
                    println!("sending audio header to {}", peer);
                    write_message(&mut write, audio_header).await.unwrap();
                    let header: AudioHeader = read_message(&mut read).await.unwrap();
                    println!("received audio header from {}: {:?}", peer, header);

                    tokio::spawn(audio_sender(peer, subscribers.clone(), write));
                    tokio::spawn(audio_receiver(peer, subscribers.clone(), read));
                } else {
                    println!("incoming control stream");
                    subscribers.write().await.insert(peer, HashMap::new());
                    if stream_sender.send((peer, stream)).await.is_err() {
                        println!("failed to send stream to controller");
                    }
                }
            }

            println!("incoming streams ended, trying to restart");
        }
    });

    println!("listening on 40142");
    loop {
        match swarm.select_next_some().await {
            event => {
                // println!("{:?}", event);
            }
        }
    }
}

/// controls messaging between clients
async fn controller(stream_receiver: AsyncReceiver<(PeerId, Stream)>) {
    let (sender, receiver) = unbounded_async();
    let mut writer_map = HashMap::new();

    loop {
        select! {
            result = stream_receiver.recv() => {
                match result {
                    Ok((peer, stream)) => {
                        println!("controller received stream from {}", peer);

                        let transport = LengthDelimitedCodec::builder()
                            .max_frame_length(usize::MAX)
                            .length_field_type::<u64>()
                            .new_framed(stream.compat());

                        let (mut write, read) = transport.split();

                        tokio::spawn(message_receiver(peer, sender.clone(), read));

                        for writer in writer_map.values_mut() {
                            let message = Message::room_join(peer.to_bytes());

                            if write_message(writer, message).await.is_err() {
                                println!("failed to send room join message to {}", peer);
                            }
                        }

                        let peers = writer_map.keys().map(|id: &PeerId| id.to_bytes()).collect();
                        let message = Message::room_welcome(peers);

                        if write_message(&mut write, message).await.is_err() {
                            println!("failed to send room welcome message to {}", peer);
                        }

                        writer_map.insert(peer, write);
                    }
                    Err(_) => {
                        println!("controller ended [stream receiver]");
                        break;
                    }
                }
            }
            result = receiver.recv() => {
                match result {
                    Ok((peer, message)) => {
                        for (identity, writer) in writer_map.iter_mut() {
                            // don't send a client's message back to itself
                            if identity == &peer {
                                continue;
                            }

                            if write_message(writer, message.clone()).await.is_err() {
                                println!("failed to send message to {}", identity);
                            }
                        }
                    }
                    Err(_) => {
                        println!("controller ended [message receiver]");
                        break;
                    }
                }
            }
        }
    }
}

/// receives audio from a single client and sends it to all other clients
async fn audio_receiver(
    identity: PeerId,
    subscribers: Subscribers,
    mut socket: SplitStream<Transport<TransportStream>>,
) {
    loop {
        let buffer = match socket.next().await {
            Some(Ok(buffer)) => {
                // the chat room ignores silence control signals
                if buffer.len() == 1 {
                    continue;
                }

                buffer.freeze()
            }
            result => {
                println!("audio receiver error: {:?}", result);
                break;
            }
        };

        let mut create_subscriptions = Vec::new();

        for (peer_id, subscriber) in subscribers.read().await.iter() {
            if peer_id != &identity {
                match subscriber.get(&identity) {
                    Some(sender) => {
                        match sender {
                            Some(sender) => {
                                let _sent = sender.try_send(buffer.clone());
                                // println!("Sent={} for {}", sent, peer_id);
                            }
                            None => {
                                println!("channel not created yet");
                            }
                        }
                    }
                    None => {
                        create_subscriptions.push(*peer_id);
                    }
                }
            }
        }

        let mut subscriptions = subscribers.write().await;

        for peer_id in create_subscriptions {
            if let Some(subscription) = subscriptions.get_mut(&peer_id) {
                subscription.insert(identity, None);
            }
        }
    }

    for subscription in subscribers.write().await.values_mut() {
        subscription.remove(&identity);
    }
}

/// outputs mixed audio to a single client
async fn audio_sender(
    identity: PeerId,
    subscribers: Subscribers,
    mut socket: SplitSink<Transport<TransportStream>, Bytes>,
) {
    let mut receiver_map: HashMap<PeerId, AsyncReceiver<Bytes>> = HashMap::new();

    let mut interval = interval(FRAME_DURATION);

    loop {
        interval.tick().await;

        let mut subscribers = subscribers.write().await;
        let subscription = subscribers.get_mut(&identity).unwrap();

        for (peer_id, subscription_sender) in subscription.iter_mut() {
            if subscription_sender.is_none() {
                let (sender, receiver) = bounded_async(CHANNEL_SIZE);
                *subscription_sender = Some(sender);
                receiver_map.insert(*peer_id, receiver);
            }
        }

        drop(subscribers);

        let mut to_remove = Vec::new();

        let frames: Vec<&[i16]> = receiver_map
            .iter()
            .filter_map(|(peer_id, receiver)| match receiver.try_recv() {
                Ok(bytes) => bytes,
                Err(_) => {
                    to_remove.push(*peer_id);
                    None
                }
            })
            .map(|bytes| unsafe {
                std::slice::from_raw_parts(bytes.as_ptr() as *const i16, FRAME_SIZE)
            })
            .collect();

        // println!("{} frames: {}", identity, frames.len());

        // remove the marked receivers from the map
        for peer_id in to_remove {
            println!("removing receiver for {}", peer_id);
            receiver_map.remove(&peer_id);
        }

        let mixed_audio = mix_frames(&frames);

        if let Err(error) = socket.send(Bytes::from(mixed_audio)).await {
            println!("audio sender error for {}: {:?}", identity, error);
            break;
        }
    }

    subscribers.write().await.remove(&identity);
}

/// receives messages from a single client
async fn message_receiver(
    identity: PeerId,
    message_sender: AsyncSender<(PeerId, Message)>,
    mut socket: SplitStream<Transport<TransportStream>>,
) {
    loop {
        match read_message::<Message, _>(&mut socket).await {
            Ok(message) => {
                match message.message {
                    Some(messages::message::Message::Goodbye(_)) => {
                        let message = Message::room_join(identity.to_bytes());
                        _ = message_sender.send((identity, message)).await;
                        break;
                    }
                    Some(
                        messages::message::Message::ConnectionRestored(_)
                        | messages::message::Message::ConnectionInterrupted(_),
                    ) => {
                        continue;
                    }
                    _ => (),
                }

                if message_sender.send((identity, message)).await.is_err() {
                    println!("message receiver ended for {}", identity);
                    break;
                }
            }
            Err(error) => {
                println!("message receiver error for {}: {:?}", identity, error);

                let message = Message::room_leave(identity.to_bytes());
                _ = message_sender.send((identity, message)).await;
                break;
            }
        }
    }
}

/// mix an array of frames into a single frame
fn mix_frames(frames: &[&[i16]]) -> &'static [u8] {
    // check if there are no frames
    if frames.is_empty() {
        return &[0];
    }

    let mixed_samples: Vec<i16> = (0..FRAME_SIZE)
        .into_par_iter()
        .map(|i| {
            let samples = frames.iter().map(|frame| frame[i]);
            mix_samples(samples)
        })
        .collect();

    unsafe { std::slice::from_raw_parts(mixed_samples.as_ptr() as *const u8, BUFFER_SIZE) }
}

/// mix an iterator of samples into a single sample
fn mix_samples(samples: impl Iterator<Item = i16>) -> i16 {
    // convert all samples to unsigned (0..65535)
    let mut unsigned_samples = samples.map(|sample| ((sample / 2) as i32) + 32768);

    // initialize mixed result with the first sample
    let mut m = unsigned_samples.next().unwrap_or(0);

    // iteratively mix each sample
    for b in unsigned_samples {
        let a = m;

        if a < 32768 || b < 32768 {
            // Viktor's first equation
            m = a * b / 32768;
        } else {
            // Viktor's second equation
            m = 2 * (a + b) - (a * b) / 32768 - 65536;
        }
    }

    // convert the result back to signed (-32768..32767)
    if m == 65536 {
        m = 65535;
    }

    (m - 32768) as i16
}

/// Writes a protobuf message to the stream
async fn write_message<M: prost::Message, W>(
    transport: &mut SplitSink<Transport<W>, Bytes>,
    message: M,
) -> Result<()>
where
    W: AsyncWrite + Unpin,
    SplitSink<Transport<W>, Bytes>: Sink<Bytes> + Unpin,
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
    transport: &mut SplitStream<Transport<R>>,
) -> Result<M> {
    if let Some(Ok(buffer)) = transport.next().await {
        let message = M::decode(&buffer[..])?; // decode the message
        Ok(message)
    } else {
        Err(ErrorKind::TransportRecv.into())
    }
}

fn generate_ed25519(secret_key_seed: u8) -> Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}

async fn simulated_client(i: usize) {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
        .with_quic()
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .unwrap()
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
        .unwrap()
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
        .build();

    // Listen on all interfaces
    let listen_addr_tcp = Multiaddr::empty()
        .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Tcp(0));

    swarm.listen_on(listen_addr_tcp).unwrap();

    let listen_addr_quic = Multiaddr::empty()
        .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Udp(0))
        .with(Protocol::QuicV1);

    swarm.listen_on(listen_addr_quic).unwrap();

    let mut control = swarm.behaviour().stream.new_control();

    let keypair = generate_ed25519(15);
    let peer_id = keypair.public().to_peer_id();

    let room_addr = Multiaddr::empty()
        .with(Protocol::from(Ipv4Addr::LOCALHOST))
        .with(Protocol::Udp(40142))
        .with(Protocol::QuicV1);

    println!("dialing room: {:?}", room_addr);

    swarm.dial(room_addr).unwrap();

    tokio::spawn(async move {
        loop {
            swarm.next().await;
        }
    });

    sleep(Duration::from_secs(1)).await;

    let control_stream = control.open_stream(peer_id, PROTOCOL).await.unwrap();

    tokio::spawn(async move {
        let transport = LengthDelimitedCodec::builder()
            .max_frame_length(usize::MAX)
            .length_field_type::<u64>()
            .new_framed(control_stream.compat());

        let (_write, mut read) = transport.split();

        loop {
            match read_message::<Message, _>(&mut read).await {
                Ok(message) => {
                    println!("{} message: {:?}", i, message);
                }
                Err(error) => {
                    println!("message receiver error: {:?}", error);
                    break;
                }
            }
        }
    });

    let audio_stream = control.open_stream(peer_id, PROTOCOL).await.unwrap();

    let transport = LengthDelimitedCodec::builder()
        .max_frame_length(BUFFER_SIZE)
        .length_field_type::<u16>()
        .new_framed(audio_stream.compat());

    let (mut write, mut read) = transport.split();

    let audio_header = AudioHeader {
        channels: 1,
        sample_rate: 48_000,
        sample_format: "f32".to_string(),
    };

    write_message(&mut write, audio_header).await.unwrap();
    let _header: AudioHeader = read_message(&mut read).await.unwrap();

    let send = tokio::spawn(async move {
        loop {
            let mut file = match i {
                1 => File::open("test-1.raw").await.unwrap(),
                2 => File::open("test-2.raw").await.unwrap(),
                3 => File::open("test-3.raw").await.unwrap(),
                _ => File::open("test-4.raw").await.unwrap(),
            };

            let mut interval = interval(FRAME_DURATION);

            let mut buffer = vec![0u8; BUFFER_SIZE];

            loop {
                interval.tick().await;

                let read = file.read(&mut buffer).await.unwrap();
                if read == 0 {
                    break;
                }

                write.send(Bytes::copy_from_slice(&buffer)).await.unwrap();
            }
        }
    });

    let receive = tokio::spawn(async move {
        let mut file = match i {
            1 => File::create("test-1-output.raw").await.unwrap(),
            2 => File::create("test-2-output.raw").await.unwrap(),
            3 => File::create("test-3-output.raw").await.unwrap(),
            _ => File::create("test-4-output.raw").await.unwrap(),
        };

        loop {
            match read.next().await {
                Some(Ok(buffer)) => {
                    if buffer.len() == 1 {
                        file.write_all(&[0; BUFFER_SIZE]).await.unwrap();
                        continue;
                    }

                    file.write_all(&buffer).await.unwrap();
                }
                result => {
                    println!("audio receiver error: {:?}", result);
                    break;
                }
            }
        }
    });

    _ = join![send, receive];
}
