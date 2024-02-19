use cpal::SupportedStreamConfig;
use ed25519_dalek::Signature;

include!(concat!(env!("OUT_DIR"), "/audio_chat.items.rs"));

impl Hello {
    pub(crate) fn new(port: u16) -> Self {
        Self { port: port as u32 }
    }
}

impl From<&SupportedStreamConfig> for AudioHeader {
    fn from(value: &SupportedStreamConfig) -> Self {
        Self {
            channels: value.channels() as u32,
            sample_rate: value.sample_rate().0,
            sample_format: value.sample_format().to_string(),
        }
    }
}

impl From<&[u8]> for AudioHeader {
    fn from(value: &[u8]) -> Self {
        // let bits_per_sample = u16::from_le_bytes([value[36], value[37]]);

        Self {
            channels: u16::from_le_bytes([value[22], value[23]]) as u32,
            sample_rate: u32::from_le_bytes([value[24], value[25], value[26], value[27]]),
            sample_format: String::from("I16"),
        }
    }
}

impl Identity {
    pub(crate) fn new(nonce: [u8; 128], signature: Signature) -> Self {
        Self {
            nonce: nonce.to_vec(),
            signature: signature.to_bytes().to_vec(),
        }
    }
}
