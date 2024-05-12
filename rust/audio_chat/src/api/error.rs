use std::array::TryFromSliceError;
use std::fmt::{Display, Formatter};
use std::net::AddrParseError;

use crate::BehaviourEvent;
use cpal::{BuildStreamError, DefaultStreamConfigError, DevicesError, PlayStreamError};
use libp2p::identity::{DecodingError, ParseError};
use libp2p::swarm::{DialError, SwarmEvent};
use libp2p::TransportError;
use libp2p_stream::{AlreadyRegistered, OpenStreamError};
use rubato::{ResampleError, ResamplerConstructionError};
use tokio::task::JoinError;
use tokio::time::error::Elapsed;

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
    Join(JoinError),
    AddrParse(AddrParseError),
    Timeout(Elapsed),
    IdentityDecode(DecodingError),
    OpenStream(OpenStreamError),
    Dial(DialError),
    IdentityParse(ParseError),
    Transport(TransportError<std::io::Error>),
    AlreadyRegistered(AlreadyRegistered),
    NoOutputDevice,
    NoInputDevice,
    InvalidContactFormat,
    InCall,
    UnknownSampleFormat,
    InvalidWav,
    TryFromSlice(TryFromSliceError),
    ManagerRestarted,
    TransportSend,
    TransportRecv,
    UnexpectedSwarmEvent,
    SwarmBuild,
    SwarmEnded,
    SessionStoped,
    CallEnded,
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

impl From<DecodingError> for Error {
    fn from(err: DecodingError) -> Self {
        Self {
            kind: ErrorKind::IdentityDecode(err),
        }
    }
}

impl From<OpenStreamError> for Error {
    fn from(err: OpenStreamError) -> Self {
        Self {
            kind: ErrorKind::OpenStream(err),
        }
    }
}

impl From<DialError> for Error {
    fn from(err: DialError) -> Self {
        Self {
            kind: ErrorKind::Dial(err),
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self {
            kind: ErrorKind::IdentityParse(err),
        }
    }
}

impl From<SwarmEvent<BehaviourEvent>> for Error {
    fn from(_: SwarmEvent<BehaviourEvent>) -> Self {
        Self {
            kind: ErrorKind::UnexpectedSwarmEvent,
        }
    }
}

impl From<TransportError<std::io::Error>> for Error {
    fn from(err: TransportError<std::io::Error>) -> Self {
        Self {
            kind: ErrorKind::Transport(err),
        }
    }
}

impl From<AlreadyRegistered> for Error {
    fn from(err: AlreadyRegistered) -> Self {
        Self {
            kind: ErrorKind::AlreadyRegistered(err),
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
                ErrorKind::Join(ref err) => format!("Join error: {}", err),
                ErrorKind::Timeout(_) => "The connection timed out".to_string(),
                ErrorKind::TryFromSlice(ref err) => format!("Try from slice error: {}", err),
                ErrorKind::AddrParse(ref err) => err.to_string(),
                ErrorKind::IdentityDecode(ref err) => format!("Identity decode error: {}", err),
                ErrorKind::OpenStream(ref err) => format!("Open stream error: {}", err),
                ErrorKind::Dial(ref err) => format!("Dial error: {}", err),
                ErrorKind::IdentityParse(ref err) => format!("Identity parse error: {}", err),
                ErrorKind::Transport(ref err) => format!("Transport error: {}", err),
                ErrorKind::AlreadyRegistered(ref err) => format!("Already registered: {}", err),
                ErrorKind::NoOutputDevice => "No output device found".to_string(),
                ErrorKind::NoInputDevice => "No input device found".to_string(),
                ErrorKind::InvalidContactFormat => "Invalid contact format".to_string(),
                ErrorKind::InCall => "Cannot change this option while a call is active".to_string(),
                ErrorKind::UnknownSampleFormat => "Unknown sample format".to_string(),
                ErrorKind::InvalidWav => "Invalid WAV file".to_string(),
                ErrorKind::ManagerRestarted => "Session manager restarted".to_string(),
                ErrorKind::TransportSend => "Transport failed on send".to_string(),
                ErrorKind::TransportRecv => "Transport failed on receive".to_string(),
                ErrorKind::UnexpectedSwarmEvent => "Unexpected swarm event".to_string(),
                ErrorKind::SwarmBuild => "Swarm build error".to_string(),
                ErrorKind::SwarmEnded => "Swarm ended".to_string(),
                ErrorKind::SessionStoped => "Session stopped".to_string(),
                ErrorKind::CallEnded => "Call ended".to_string(),
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
