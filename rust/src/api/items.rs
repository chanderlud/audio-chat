use cpal::SupportedStreamConfig;
use ed25519_dalek::Signature;

include!(concat!(env!("OUT_DIR"), "/audio_chat.items.rs"));

impl Hello {
    pub(crate) fn new(port: u16) -> Self {
        Self { port: port as u32 }
    }
}

impl From<&SupportedStreamConfig> for InputConfig {
    fn from(value: &SupportedStreamConfig) -> Self {
        Self {
            channels: value.channels() as u32,
            sample_rate: value.sample_rate().0,
            sample_format: value.sample_format().to_string(),
        }
    }
}

impl Identity {
    pub(crate) fn new(nonce: [u8; 1024], signature: Signature) -> Self {
        Self { nonce: nonce.to_vec(), signature: signature.to_bytes().to_vec() }
    }
}