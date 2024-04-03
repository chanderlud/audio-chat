use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use aes::Aes256;
use ctr::Ctr128BE;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::Rng;
use sha2::Sha256;
use std::mem;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

use crate::error::ErrorKind;
use crate::items::Identity;
use crate::{AesCipher, Result, Time};

/// A pair of stream ciphers one for sending and one for receiving
pub struct PairedCipher<T> {
    pub send_cipher: T,
    pub receive_cipher: T,
}

impl<T: StreamCipher + StreamCipherSeek> PairedCipher<T> {
    pub fn new(send_cipher: T, receive_cipher: T) -> Self {
        Self {
            send_cipher,
            receive_cipher,
        }
    }

    pub fn swap(&mut self) {
        mem::swap(&mut self.send_cipher, &mut self.receive_cipher);
    }

    pub fn into_parts(self) -> (T, T) {
        (self.send_cipher, self.receive_cipher)
    }

    pub fn mut_parts(&mut self) -> (&mut T, &mut T) {
        (&mut self.send_cipher, &mut self.receive_cipher)
    }
}

/// Creates a cipher from the HKDF
pub fn cipher_factory(hk: &Hkdf<Sha256>, key_info: &[u8], iv_info: &[u8]) -> Result<AesCipher> {
    let mut key = [0_u8; 32];
    hk.expand(key_info, &mut key)?;

    let mut iv = [0_u8; 16];
    hk.expand(iv_info, &mut iv)?;

    Ok(Ctr128BE::<Aes256>::new(
        key.as_slice().into(),
        iv.as_slice().into(),
    ))
}

/// Performs the key exchange
pub async fn key_exchange<W: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> Result<SharedSecret> {
    let secret = EphemeralSecret::random();

    // send our public key
    let our_public = PublicKey::from(&secret);
    stream.write_all(our_public.as_bytes()).await?;

    // receive their public key
    let their_public = read_public(stream).await?;
    let shared_secret = secret.diffie_hellman(&their_public);

    Ok(shared_secret)
}

/// Reads a public key from the stream
async fn read_public<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<PublicKey> {
    let mut buffer = [0; 32];
    reader.read_exact(&mut buffer).await?;
    Ok(PublicKey::from(buffer))
}

pub async fn identity_factory(time: &Time, signing_key: &SigningKey) -> Result<Identity> {
    // create a random nonce
    let mut nonce = [0; 128];
    OsRng.fill(&mut nonce[16..]);

    // adds the current timestamp to the nonce
    {
        let time = time.lock().await;
        let timestamp = time.current_timestamp();
        nonce[0..16].copy_from_slice(&timestamp.to_be_bytes());
    }

    let signature = signing_key.sign(&nonce);

    // create the identity message
    Ok(Identity::new(
        &nonce,
        signature,
        signing_key.verifying_key().as_bytes(),
    ))
}

pub async fn verify_identity(time: &Time, identity: &Identity) -> Result<[u8; 32]> {
    let timestamp = u128::from_be_bytes(identity.nonce[0..16].try_into()?);

    let delta = {
        let time = time.lock().await;
        let current_timestamp = time.current_timestamp();

        if current_timestamp > timestamp {
            current_timestamp - timestamp
        } else {
            timestamp - current_timestamp
        }
    };

    // a max delta of 60 seconds should prevent replay attacks
    if delta > 60_000_000 {
        return Err(ErrorKind::InvalidIdentity.into());
    }

    match identity.public_key.clone().try_into() {
        Ok(verifying_key_bytes) => {
            // verify the signature
            let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes)?;
            let signature = Signature::from_slice(&identity.signature)?;
            verifying_key.verify_strict(&identity.nonce, &signature)?;

            Ok(verifying_key_bytes)
        }
        Err(_) => Err(ErrorKind::InvalidIdentity.into()),
    }
}
