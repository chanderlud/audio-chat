use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use async_throttle::RateLimiter;
use kanal::{unbounded_async, AsyncSender};
use log::{error, info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tokio_util::codec::LengthDelimitedCodec;

use common::crypto::{cipher_factory, key_exchange, verify_identity};
use common::items::{message, EndSession, Identity, Message, RequestOutcome, ServerError};
use common::time::synchronize;
use common::{read_message, write_message, Hkdf, Sha256, Time, SALT};

use crate::error::{Error, ErrorKind};

type Database = Arc<RwLock<HashMap<[u8; 32], State>>>;
type Result<T> = std::result::Result<T, Error>;

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
                database.insert(public_key, State::new(sender));
                (receiver, public_key)
            }
            Err(error) => {
                error!("Failed to receive identity: {:?}", error);
                return Ok(());
            }
        };

    // rate limits all connections to prevent abuse
    let period = Duration::from_millis(1);
    let rate_limiter = RateLimiter::new(period);

    loop {
        let future =
            async {
                let message = rate_limiter.throttle(|| async { select! {
                result = message_receiver.recv() => {
                    Ok::<Message, Error>(result?)
                },
                result = read_message::<Message, _, _>(&mut transport, &mut sr_cipher) => {
                    Ok::<Message, Error>(result?)
                },
                // timeout for the other branches
                _ = sleep(Duration::from_secs(30)) => {
                    Err(ErrorKind::Timeout.into())
                }
            }}).await?;

                let from_clone = message.from.clone();
                let from: [u8; 32] = from_clone
                    .clone()
                    .try_into()
                    .map_err(|_| ErrorKind::InvalidPublicKey(from_clone))?;

                let to_clone = message.to.clone();
                let to: [u8; 32] = to_clone
                    .clone()
                    .try_into()
                    .map_err(|_| ErrorKind::InvalidPublicKey(to_clone))?;

                if message.to == message.from {
                    return Ok(());
                } else if message.from == public_key {
                    match &message.message {
                        // RequestSession always responds with RequestOutcome
                        Some(message::Message::RequestSession(_)) => {
                            info!("read RequestSession aimed at {:?}", &to[0..5]);

                            if let Some(state) = database.read().await.get(&to) {
                                if state.sender.send(message).await.is_err() {
                                    let outcome = RequestOutcome::failure("Session closed");
                                    let message = Message::new(outcome.into(), &public_key, &to);
                                    write_message(&mut transport, message, &mut ss_cipher).await?;
                                }
                            } else {
                                let outcome = RequestOutcome::failure("No session found");
                                let message = Message::new(outcome.into(), &public_key, &to);
                                write_message(&mut transport, message, &mut ss_cipher).await?;
                            }
                        }
                        // Candidate only responds if an error occurs
                        Some(message::Message::Candidate(_)) => {
                            info!("read Candidate aimed at {:?}", &to[0..5]);

                            if let Some(state) = database.read().await.get(&to) {
                                if state.sender.send(message).await.is_err() {
                                    let error = ServerError::new("Session closed");
                                    let message = Message::new(error.into(), &public_key, &to);
                                    write_message(&mut transport, message, &mut ss_cipher).await?;
                                }
                            } else {
                                let error = ServerError::new("No session found");
                                let message = Message::new(error.into(), &public_key, &to);
                                write_message(&mut transport, message, &mut ss_cipher).await?;
                            }
                        }
                        // this is a response to RequestSession
                        Some(message::Message::RequestOutcome(_)) => {
                            info!("read RequestOutcome aimed at {:?}", &to[0..5]);

                            let option = database
                                .read()
                                .await
                                .get(&to)
                                .map(|state| state.sender.clone());

                            if let Some(sender) = option {
                                let mut database = database.write().await;
                                let state = database
                                    .get_mut(&public_key)
                                    .ok_or(ErrorKind::MissingLocalState)?;

                                if state.connected.contains(&to) {
                                    warn!(
                                        "{:?} already connected to {:?}",
                                        &message.from[0..5],
                                        &to[0..5]
                                    );
                                    return Ok(());
                                }

                                if sender.send(message).await.is_ok() {
                                    state.connected.insert(to);
                                } else {
                                    warn!("Failed to deliver RequestOutcome to {:?}", &to[0..5]);
                                }
                            } else {
                                warn!("No session found for {:?}", &to[0..5]);
                            }
                        }
                        // this message is sent when the session closes locally
                        Some(message::Message::EndSession(_)) => {
                            info!("read EndSession aimed at {:?}", &to[0..5]);

                            // no message is returned if the state does not exist or is closed
                            if let Some(state) = database.read().await.get(&to) {
                                _ = state.sender.send(message).await;
                            }
                        }
                        _ => warn!("read unexpected message: {:?}", message),
                    }
                } else {
                    match &message.message {
                        // this message is proxied from another session
                        Some(message::Message::RequestSession(_)) => {
                            info!("received RequestSession from {:?}", &from[0..5]);
                            write_message(&mut transport, message, &mut ss_cipher).await?;
                        }
                        // this is always a response to RequestSession
                        Some(message::Message::RequestOutcome(outcome)) => {
                            info!("received RequestOutcome from {:?}", &from[0..5]);

                            if outcome.success {
                                let mut datebase = database.write().await;
                                let state = datebase
                                    .get_mut(&public_key)
                                    .ok_or(ErrorKind::MissingLocalState)?;

                                if state.connected.contains(&from) {
                                    warn!(
                                        "{:?} already connected to {:?}",
                                        &message.to[0..5],
                                        &from[0..5]
                                    );
                                    return Ok(());
                                }

                                state.connected.insert(from);
                            }

                            write_message(&mut transport, message, &mut ss_cipher).await?;
                        }
                        Some(message::Message::EndSession(_)) => {
                            info!("received EndSession from {:?}", &from[0..5]);

                            let mut database = database.write().await;
                            let state = database
                                .get_mut(&public_key)
                                .ok_or(ErrorKind::MissingLocalState)?;
                            state.connected.remove(&from);
                            write_message(&mut transport, message, &mut ss_cipher).await?;
                        }
                        Some(message::Message::Ping(_)) => {
                            info!("{:?} pinged", &public_key[0..5]);
                            write_message(&mut transport, message, &mut ss_cipher).await?
                        }
                        _ => {
                            info!(
                                "forwarding message from {:?} to {:?}",
                                &from[0..5],
                                &to[0..5]
                            );
                            write_message(&mut transport, message, &mut ss_cipher).await?
                        }
                    }
                }

                Ok::<(), Error>(())
            };

        if let Err(error) = future.await {
            error!("Error in {:?} handler: {:?}", &public_key[0..5], error);

            match error.kind {
                ErrorKind::InvalidPublicKey(to) => {
                    let error = ServerError::new("Invalid public key");
                    let message = Message::new(error.into(), &to, &public_key);
                    write_message(&mut transport, message, &mut ss_cipher).await?;
                }
                _ => break,
            }
        }
    }

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
