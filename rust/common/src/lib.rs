use std::sync::Arc;

pub use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherCoreWrapper, StreamCipherSeek};
pub use aes::Aes256;
pub use ctr::{Ctr128BE, CtrCore};
use futures::StreamExt;
use futures::{Sink, SinkExt};
use hex_literal::hex;
pub use hkdf::{Hkdf, InvalidLength};
pub use sha2::Sha256;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;
use tokio_util::bytes::Bytes;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::error::ErrorKind;
use crate::time::SyncedTime;

pub mod crypto;
pub mod error;
pub mod items;
pub mod time;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type AesCipher = StreamCipherCoreWrapper<CtrCore<Aes256, ctr::flavors::Ctr128BE>>;
pub type Transport<T> = Framed<T, LengthDelimitedCodec>;
pub type Time = Arc<Mutex<SyncedTime>>;

/// A salt used by the HKDF
pub const SALT: [u8; 32] = hex!("04acee810b938239a6d2a09c109af6e3eaedc961fc66b9b6935a441c2690e336");

/// Writes a protobuf message to the stream
pub async fn write_message<M: prost::Message, C: StreamCipher + StreamCipherSeek, W>(
    transport: &mut Transport<W>,
    message: M,
    cipher: &mut C,
) -> Result<()>
where
    W: AsyncWrite + Unpin,
    Transport<W>: Sink<Bytes> + Unpin,
{
    let len = message.encoded_len(); // get the length of the message
    let mut buffer = Vec::with_capacity(len);

    message.encode(&mut buffer).unwrap(); // encode the message into the buffer (infallible)
    cipher.apply_keystream(&mut buffer);

    transport
        .send(Bytes::from(buffer))
        .await
        .map_err(|_| ErrorKind::TransportSend)
        .map_err(Into::into)
}

/// Reads a protobuf message from the stream
pub async fn read_message<
    M: prost::Message + Default,
    C: StreamCipher + StreamCipherSeek,
    R: AsyncRead + Unpin,
>(
    transport: &mut Transport<R>,
    cipher: &mut C,
) -> Result<M> {
    if let Some(Ok(mut buffer)) = transport.next().await {
        cipher.apply_keystream(&mut buffer); // apply the keystream to the buffer

        let message = M::decode(&buffer[..])?; // decode the message

        Ok(message)
    } else {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read failed").into())
    }
}
