use async_throttle::RateLimiter;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use common::crypto::{cipher_factory, key_exchange, verify_identity};
use common::items::{message, EndSession, Identity, Message, RequestOutcome, ServerError};
use common::time::synchronize;
use common::{read_message, write_message, Hkdf, Sha256, Time, SALT};
use kanal::{unbounded_async, AsyncSender};
use log::{error, info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::codec::LengthDelimitedCodec;

type Database = Arc<RwLock<HashMap<[u8; 32], State>>>;
type Result<T> = std::result::Result<T, error::Error>;

mod error;

#[derive(Clone)]
struct State {
    /// Sends a message to this session
    sender: AsyncSender<Message>,

    /// The public keys of the sessions this session is connected to
    connected: HashSet<[u8; 32]>,
}

impl State {
    fn new(sender: AsyncSender<Message>) -> Self {
        Self {
            sender,
            connected: HashSet::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logging::log_to_stderr(log::LevelFilter::Info);

    let listener = TcpListener::bind(("0.0.0.0", 8957)).await.unwrap();

    let time: Time = Default::default();
    tokio::spawn(synchronize(time.clone()));

    let database: Database = Default::default();

    while let Ok((stream, address)) = listener.accept().await {
        info!("New connection from {}", address);
        tokio::spawn(handler(stream, time.clone(), database.clone()));
    }
}

/// Handles a single connection
async fn handler(mut stream: TcpStream, time: Time, database: Database) -> Result<()> {
    let shared_secret = key_exchange(&mut stream).await?;

    let mut transport = LengthDelimitedCodec::builder().new_framed(stream);

    // HKDF for the key derivation
    let hk = Hkdf::<Sha256>::new(Some(&SALT), shared_secret.as_bytes());

    // stream send cipher
    let mut ss_cipher = cipher_factory(&hk, b"ss-key", b"ss-iv")?;
    // stream read cipher
    let mut sr_cipher = cipher_factory(&hk, b"sr-key", b"sr-iv")?;

    let (message_receiver, public_key) =
        match read_message::<Identity, _, _>(&mut transport, &mut sr_cipher).await {
            Ok(identity) => {
                let public_key = verify_identity(&time, &identity).await?;

                let mut database = database.write().await;

                if database.contains_key(&public_key) {
                    warn!("{:?} already has a session", &public_key[0..5]);

                    write_message(
                        &mut transport,
                        Message::new(
                            ServerError::new("Session already exists").into(),
                            &public_key,
                            &[0; 32],
                        ),
                        &mut ss_cipher,
                    )
                    .await?;

                    return Ok(());
                } else {
                    info!("new session with: {:?}", &public_key[0..5]);
                }

                let (sender, receiver) = unbounded_async();
                let state = State::new(sender);
                database.insert(public_key, state);

                (receiver, public_key)
            }
            Err(error) => {
                error!("Failed to receive identity: {:?}", error);
                return Ok(());
            }
        };

    // rate limits all connections to prevent abuse
    let period = Duration::from_millis(10);
    let rate_limiter = RateLimiter::new(period);

    loop {
        select! {
            result = rate_limiter.throttle(|| async { message_receiver.recv().await }) => {
                match result {
                    Ok(message) => {
                        // in theory the key should already be verified
                        let from: [u8; 32] = message.from.clone().try_into().unwrap();
                        info!("MessageReceiver {:?} -> {:?}", &from[0..5], &message.to[0..5]);

                        match &message.message {
                            // this message is proxied from another session
                            Some(message::Message::RequestSession(_)) => {
                                info!("received proxied RequestSession from {:?}", &from[0..5]);
                                write_message(&mut transport, message, &mut ss_cipher).await?;
                            }
                            // this is always a response to RequestSession
                            Some(message::Message::RequestOutcome(outcome)) => {
                                if outcome.success {
                                    if let Some(state) = database.write().await.get_mut(&public_key) {
                                        if state.connected.contains(&from) {
                                            warn!("Session already connected to {:?}", &from[0..5]);
                                            continue;
                                        }

                                        state.connected.insert(from);
                                    }
                                }

                                write_message(&mut transport, message, &mut ss_cipher).await?;
                            }
                            Some(message::Message::Candidate(_)) => {
                                write_message(&mut transport, message, &mut ss_cipher).await?;
                            }
                            Some(message::Message::EndSession(_)) => {
                                if let Some(state) = database.write().await.get_mut(&public_key) {
                                    state.connected.remove(&from);
                                }

                                write_message(&mut transport, message, &mut ss_cipher).await?;
                            }
                            _ => warn!("Received unexpected message (handler): {:?}", message),
                        }
                    }
                    Err(_) => break,
                }
            },
            result = rate_limiter.throttle(|| async { read_message::<Message, _, _>(&mut transport, &mut sr_cipher).await }) => {
                match result {
                    Ok(message) => {
                        if message.to == message.from {
                            continue;
                        }

                        info!("MessageReader {:?} -> {:?}", &message.from[0..5], &message.to[0..5]);

                        let to: [u8; 32] = match message.to.clone().try_into() {
                            Ok(to) => to,
                            Err(to) => {
                                let error = ServerError::new("Invalid public key");
                                let message = Message::new(error.into(), &to, &public_key);
                                write_message(&mut transport, message, &mut ss_cipher).await?;
                                continue;
                            }
                        };

                        match &message.message {
                            // RequestSession always responds with RequestOutcome
                            Some(message::Message::RequestSession(_)) => {
                                info!("received RequestSession aimed at {:?}", &to[0..5]);

                                if let Some(state) = database.read().await.get(&to) {
                                    if state.sender.send(message).await.is_err() {
                                        let outcome = RequestOutcome::failure("Session closed");
                                        let message = Message::new(outcome.into(), &to, &public_key);
                                        write_message(&mut transport, message, &mut ss_cipher).await?;
                                    }
                                } else {
                                    let outcome = RequestOutcome::failure("No session found");
                                    let message = Message::new(outcome.into(), &to, &public_key);
                                    write_message(&mut transport, message, &mut ss_cipher).await?;
                                }
                            }
                            // Candidate only responds if an error occurs
                            Some(message::Message::Candidate(_)) => {
                                info!("received Candidate aimed at {:?}", &to[0..5]);

                                if let Some(state) = database.read().await.get(&to) {
                                    if state.sender.send(message).await.is_err() {
                                        let error = ServerError::new("Session closed");
                                        let message = Message::new(error.into(), &to, &public_key);
                                        write_message(&mut transport, message, &mut ss_cipher).await?;
                                    }
                                } else {
                                    let error = ServerError::new("No session found");
                                    let message = Message::new(error.into(), &to, &public_key);
                                    write_message(&mut transport, message, &mut ss_cipher).await?;
                                }
                            }
                            // this is a response to RequestSession
                            Some(message::Message::RequestOutcome(_)) => {
                                if let Some(state) = database.write().await.get_mut(&public_key) {
                                    if state.connected.contains(&to) {
                                        warn!("Session already connected to {:?}", &to[0..5]);
                                        continue;
                                    }

                                    state.connected.insert(to);
                                }

                                if let Some(state) = database.read().await.get(&to) {
                                    // if the other session is closed this message will not be delivered
                                    _ = state.sender.send(message).await;
                                } else {
                                    warn!("No session found for {:?}", &to[0..5]);

                                    if let Some(state) = database.write().await.get_mut(&public_key) {
                                        state.connected.remove(&to);
                                    }
                                }
                            }
                            _ => warn!("Received unexpected message: {:?}", message),
                        }
                    },
                    Err(error) => {
                        error!("Failed to receive message: {:?}", error);
                        break;
                    }
                }
            },
        }
    }

    // TODO if an error occurs this region of code will not be reached

    let state = database.write().await.remove(&public_key);

    if let Some(state) = state {
        for connected in &state.connected {
            if let Some(state) = database.read().await.get(connected) {
                let end_session = EndSession::new("Session closed");
                let message = Message::new(end_session.into(), connected, &public_key);
                _ = state.sender.send(message).await;
            }
        }
    }

    Ok(())
}
