use cpal::SupportedStreamConfig;
use ed25519_dalek::Signature;

include!(concat!(env!("OUT_DIR"), "/audio_chat.items.rs"));

impl Ports {
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
        let bits_per_sample = u16::from_le_bytes([value[34], value[35]]);
        let audio_format = u16::from_le_bytes([value[20], value[21]]);

        let sample_format = match (audio_format, bits_per_sample) {
            (1, 8) => "u8",
            (1, 16) => "i16",
            (1, 32) => "i32",
            (3, 32) => "f32",
            (3, 64) => "f64",
            _ => "unknown",
        };

        Self {
            channels: u16::from_le_bytes([value[22], value[23]]) as u32,
            sample_rate: u32::from_le_bytes([value[24], value[25], value[26], value[27]]),
            sample_format: String::from(sample_format),
        }
    }
}

impl Identity {
    pub(crate) fn new(nonce: [u8; 128], signature: Signature, public_key: &[u8; 32]) -> Self {
        Self {
            nonce: nonce.to_vec(),
            signature: signature.to_bytes().to_vec(),
            public_key: public_key.to_vec(),
        }
    }
}

impl Message {
    pub(crate) fn hello(ringtone: Option<Vec<u8>>) -> Self {
        Self {
            message: Some(message::Message::Hello(Hello {
                ringtone: ringtone.unwrap_or_default(),
            })),
        }
    }

    pub(crate) fn reject() -> Self {
        Self {
            message: Some(message::Message::Reject(Reject {})),
        }
    }

    pub(crate) fn busy() -> Self {
        Self {
            message: Some(message::Message::Busy(Busy {})),
        }
    }

    pub(crate) fn goodbye_reason(reason: String) -> Self {
        Self {
            message: Some(message::Message::Goodbye(Goodbye { reason })),
        }
    }

    pub(crate) fn goodbye() -> Self {
        Self {
            message: Some(message::Message::Goodbye(Goodbye {
                reason: String::new(),
            })),
        }
    }

    pub(crate) fn latency_test(timestamp: u128) -> Self {
        Self {
            message: Some(message::Message::LatencyTest(LatencyTest {
                timestamp: timestamp.to_be_bytes().to_vec(),
            })),
        }
    }

    pub(crate) fn chat(message: String) -> Self {
        Self {
            message: Some(message::Message::Chat(Chat {
                message,
                attachment: vec![],
            })),
        }
    }
}
