use bincode::{Decode, Encode};

#[derive(Debug, Decode, Encode, Clone)]
pub enum Message {
    Hello {
        ringtone: Option<Vec<u8>>,
    },
    HelloAck,
    Reject,
    Busy,
    Goodbye {
        reason: Option<String>,
    },
    Chat {
        text: String,
        attachments: Vec<Attachment>,
    },
    ConnectionInterrupted,
    ConnectionRestored,
    KeepAlive,
    ScreenshareHeader {
        encoder_name: String,
    },
    RoomWelcome {
        peers: Vec<Vec<u8>>,
    },
    RoomJoin {
        peer: Vec<u8>,
    },
    RoomLeave {
        peer: Vec<u8>,
    },
}

#[derive(Debug, Decode, Encode)]
pub struct AudioHeader {
    pub channels: u32,
    pub sample_rate: u32,
    pub sample_format: String,
    pub codec_enabled: bool,
    pub vbr: bool,
    pub residual_bits: f64,
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct Attachment {
    pub name: String,
    pub data: Vec<u8>,
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
            codec_enabled: false,
            vbr: false,
            residual_bits: 0_f64,
        }
    }
}

// impl Message {
//     pub fn hello(ringtone: Option<Vec<u8>>) -> Self {
//         Self {
//             message: Some(message::Message::Hello(Hello {
//                 ringtone: ringtone.unwrap_or_default(),
//             })),
//         }
//     }
//
//     pub fn hello_ack() -> Self {
//         Self {
//             message: Some(message::Message::HelloAck(HelloAck {})),
//         }
//     }
//
//     pub fn reject() -> Self {
//         Self {
//             message: Some(message::Message::Reject(Reject {})),
//         }
//     }
//
//     pub fn busy() -> Self {
//         Self {
//             message: Some(message::Message::Busy(Busy {})),
//         }
//     }
//
//     pub fn goodbye_reason(reason: String) -> Self {
//         Self {
//             message: Some(message::Message::Goodbye(Goodbye { reason })),
//         }
//     }
//
//     pub fn goodbye() -> Self {
//         Self {
//             message: Some(message::Message::Goodbye(Goodbye {
//                 reason: String::new(),
//             })),
//         }
//     }
//
//     pub fn chat(text: String, attachments: Vec<Attachment>) -> Self {
//         Self {
//             message: Some(message::Message::Chat(Chat { text, attachments })),
//         }
//     }
//
//     pub fn connection_interrupted() -> Self {
//         Self {
//             message: Some(message::Message::ConnectionInterrupted(
//                 ConnectionInterrupted {},
//             )),
//         }
//     }
//
//     pub fn connection_restored() -> Self {
//         Self {
//             message: Some(message::Message::ConnectionRestored(ConnectionRestored {})),
//         }
//     }
//
//     pub fn keep_alive() -> Self {
//         Self {
//             message: Some(message::Message::KeepAlive(KeepAlive {})),
//         }
//     }
//
//     pub fn screenshare(encoder: &str) -> Self {
//         Self {
//             message: Some(message::Message::ScreenshareHeader(ScreenshareHeader {
//                 encoder: encoder.to_string(),
//             })),
//         }
//     }
//
//     pub fn room_welcome(peers: Vec<Vec<u8>>) -> Self {
//         Self {
//             message: Some(message::Message::RoomWelcome(RoomWelcome { peers })),
//         }
//     }
//
//     pub fn room_join(peer: Vec<u8>) -> Self {
//         Self {
//             message: Some(message::Message::RoomJoin(RoomJoin { peer })),
//         }
//     }
//
//     pub fn room_leave(peer: Vec<u8>) -> Self {
//         Self {
//             message: Some(message::Message::RoomLeave(RoomLeave { peer })),
//         }
//     }
// }
