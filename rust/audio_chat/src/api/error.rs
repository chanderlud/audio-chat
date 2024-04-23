use std::array::TryFromSliceError;
use std::fmt::{Display, Formatter};
use std::net::AddrParseError;

use cpal::{BuildStreamError, DefaultStreamConfigError, DevicesError, PlayStreamError};
use rubato::{ResampleError, ResamplerConstructionError};
use tokio::task::JoinError;
use tokio::time::error::Elapsed;

use common::InvalidLength;

/// generic error type for audio chat
#[derive(Debug)]
pub(crate) struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(std::io::Error),
    Decode(prost::DecodeError),
    StreamConfig(DefaultStreamConfigError),
    BuildStream(BuildStreamError),
    PlayStream(PlayStreamError),
    Devices(DevicesError),
    ResamplerConstruction(ResamplerConstructionError),
    Resample(ResampleError),
    KanalSend(kanal::SendError),
    KanalReceive(kanal::ReceiveError),
    HkdfLength(InvalidLength),
    Join(JoinError),
    AddrParse(AddrParseError),
    Ed25519(ed25519_dalek::ed25519::Error),
    Common(common::error::ErrorKind),
    Timeout(Elapsed),
    Ice(webrtc_ice::Error),
    Sctp(webrtc_sctp::Error),
    NoOutputDevice,
    NoInputDevice,
    InvalidContactFormat,
    InCall,
    UnknownSampleFormat,
    InvalidWav,
    TryFromSlice(TryFromSliceError),
    AcceptStream,
    MissingCredentials,
    ManagerRestarted,
    InvalidSigningKey,
    SessionTimeout,
    AgentNotFound,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
        }
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Self {
        Self {
            kind: ErrorKind::Decode(err),
        }
    }
}

impl From<DefaultStreamConfigError> for Error {
    fn from(err: DefaultStreamConfigError) -> Self {
        Self {
            kind: ErrorKind::StreamConfig(err),
        }
    }
}

impl From<BuildStreamError> for Error {
    fn from(err: BuildStreamError) -> Self {
        Self {
            kind: ErrorKind::BuildStream(err),
        }
    }
}

impl From<PlayStreamError> for Error {
    fn from(err: PlayStreamError) -> Self {
        Self {
            kind: ErrorKind::PlayStream(err),
        }
    }
}

impl From<ResamplerConstructionError> for Error {
    fn from(err: ResamplerConstructionError) -> Self {
        Self {
            kind: ErrorKind::ResamplerConstruction(err),
        }
    }
}

impl From<ResampleError> for Error {
    fn from(err: ResampleError) -> Self {
        Self {
            kind: ErrorKind::Resample(err),
        }
    }
}

impl From<kanal::SendError> for Error {
    fn from(err: kanal::SendError) -> Self {
        Self {
            kind: ErrorKind::KanalSend(err),
        }
    }
}

impl From<kanal::ReceiveError> for Error {
    fn from(err: kanal::ReceiveError) -> Self {
        Self {
            kind: ErrorKind::KanalReceive(err),
        }
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Self {
            kind: ErrorKind::Join(err),
        }
    }
}

impl From<InvalidLength> for Error {
    fn from(err: InvalidLength) -> Self {
        Self {
            kind: ErrorKind::HkdfLength(err),
        }
    }
}

impl From<DevicesError> for Error {
    fn from(err: DevicesError) -> Self {
        Self {
            kind: ErrorKind::Devices(err),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Self {
            kind: ErrorKind::AddrParse(err),
        }
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(err: ed25519_dalek::ed25519::Error) -> Self {
        Self {
            kind: ErrorKind::Ed25519(err),
        }
    }
}

impl From<Elapsed> for Error {
    fn from(err: Elapsed) -> Self {
        Self {
            kind: ErrorKind::Timeout(err),
        }
    }
}

impl From<TryFromSliceError> for Error {
    fn from(err: TryFromSliceError) -> Self {
        Self {
            kind: ErrorKind::TryFromSlice(err),
        }
    }
}

impl From<common::error::Error> for Error {
    fn from(err: common::error::Error) -> Self {
        Self {
            kind: ErrorKind::Common(err.kind),
        }
    }
}

impl From<webrtc_sctp::Error> for Error {
    fn from(err: webrtc_sctp::Error) -> Self {
        Self {
            kind: ErrorKind::Sctp(err),
        }
    }
}

impl From<webrtc_ice::Error> for Error {
    fn from(err: webrtc_ice::Error) -> Self {
        Self {
            kind: ErrorKind::Ice(err),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.kind {
                ErrorKind::Io(ref err) => format!("IO error: {}", err),
                ErrorKind::Decode(ref err) => format!("Decode error: {}", err),
                ErrorKind::StreamConfig(ref err) => format!("Stream config error: {}", err),
                ErrorKind::BuildStream(ref err) => format!("Build stream error: {}", err),
                ErrorKind::PlayStream(ref err) => format!("Play stream error: {}", err),
                ErrorKind::Devices(ref err) => format!("Devices error: {}", err),
                ErrorKind::ResamplerConstruction(ref err) =>
                    format!("Resampler construction error: {}", err),
                ErrorKind::Resample(ref err) => format!("Resample error: {}", err),
                ErrorKind::KanalSend(ref err) => format!("Kanal send error: {}", err),
                ErrorKind::KanalReceive(ref err) => format!("Kanal receive error: {}", err),
                ErrorKind::HkdfLength(ref err) => format!("HKDF length error: {}", err),
                ErrorKind::Join(ref err) => format!("Join error: {}", err),
                ErrorKind::Ed25519(ref err) => format!("Ed25519 error: {}", err),
                ErrorKind::Timeout(_) => "The connection timed out".to_string(),
                ErrorKind::TryFromSlice(ref err) => format!("Try from slice error: {}", err),
                ErrorKind::AddrParse(ref err) => err.to_string(),
                ErrorKind::Common(ref err) => format!("Common error: {:?}", err),
                ErrorKind::Ice(ref err) => format!("ICE error: {}", err),
                ErrorKind::Sctp(ref err) => format!("SCTP error: {}", err),
                ErrorKind::NoOutputDevice => "No output device found".to_string(),
                ErrorKind::NoInputDevice => "No input device found".to_string(),
                ErrorKind::InvalidContactFormat => "Invalid contact format".to_string(),
                ErrorKind::InCall => "Cannot change this option while a call is active".to_string(),
                ErrorKind::UnknownSampleFormat => "Unknown sample format".to_string(),
                ErrorKind::InvalidWav => "Invalid WAV file".to_string(),
                ErrorKind::AcceptStream => "Failed to accept stream".to_string(),
                ErrorKind::MissingCredentials => "Missing credentials".to_string(),
                ErrorKind::ManagerRestarted => "Session manager restarted".to_string(),
                ErrorKind::InvalidSigningKey => "Invalid signing key".to_string(),
                ErrorKind::SessionTimeout => "Session timed out".to_string(),
                ErrorKind::AgentNotFound => "Agent not found".to_string(),
            }
        )
    }
}

pub struct DartError {
    pub message: String,
}

impl From<Error> for DartError {
    fn from(err: Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<ErrorKind> for DartError {
    fn from(kind: ErrorKind) -> Self {
        Self {
            message: Error { kind }.to_string(),
        }
    }
}

impl From<String> for DartError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
