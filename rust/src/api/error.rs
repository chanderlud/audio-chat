use std::fmt::{Display, Formatter};
use std::net::AddrParseError;
use cpal::{BuildStreamError, DefaultStreamConfigError, DevicesError, PlayStreamError};
use hkdf::InvalidLength;
use rubato::{ResampleError, ResamplerConstructionError};
use tokio::task::JoinError;

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
    HkdfLength(InvalidLength),
    Join(JoinError),
    AddrParse(AddrParseError),
    NoOutputDevice,
    NoInputDevice,
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.kind {
            ErrorKind::Io(ref err) => format!("IO error: {}", err),
            ErrorKind::Decode(ref err) => format!("Decode error: {}", err),
            ErrorKind::StreamConfig(ref err) => format!("Stream config error: {}", err),
            ErrorKind::BuildStream(ref err) => format!("Build stream error: {}", err),
            ErrorKind::PlayStream(ref err) => format!("Play stream error: {}", err),
            ErrorKind::Devices(ref err) => format!("Devices error: {}", err),
            ErrorKind::ResamplerConstruction(ref err) => format!("Resampler construction error: {}", err),
            ErrorKind::Resample(ref err) => format!("Resample error: {}", err),
            ErrorKind::KanalSend(ref err) => format!("Kanal send error: {}", err),
            ErrorKind::HkdfLength(ref err) => format!("HKDF length error: {}", err),
            ErrorKind::Join(ref err) => format!("Join error: {}", err),
            ErrorKind::AddrParse(ref err) => format!("Addr parse error: {}", err),
            ErrorKind::NoOutputDevice => "No output device".to_string(),
            ErrorKind::NoInputDevice => "No input device".to_string(),
        })
    }
}

impl Error {
    pub(crate) fn no_output_device() -> Self {
        Self {
            kind: ErrorKind::NoOutputDevice,
        }
    }

    pub(crate) fn no_input_device() -> Self {
        Self {
            kind: ErrorKind::NoInputDevice,
        }
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
