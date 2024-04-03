use hkdf::InvalidLength;
use std::fmt::Display;

/// generic error type
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Decode(prost::DecodeError),
    HkdfLength(InvalidLength),
    TryFromSlice(std::array::TryFromSliceError),
    Ed25519(ed25519_dalek::ed25519::Error),
    InvalidIdentity,
    TransportSend,
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

impl From<InvalidLength> for Error {
    fn from(err: InvalidLength) -> Self {
        Self {
            kind: ErrorKind::HkdfLength(err),
        }
    }
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(err: std::array::TryFromSliceError) -> Self {
        Self {
            kind: ErrorKind::TryFromSlice(err),
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

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.kind {
                ErrorKind::Io(ref error) => error.to_string(),
                ErrorKind::Decode(ref error) => error.to_string(),
                ErrorKind::HkdfLength(ref error) => error.to_string(),
                ErrorKind::TryFromSlice(ref error) => error.to_string(),
                ErrorKind::Ed25519(ref error) => error.to_string(),
                ErrorKind::InvalidIdentity => "Invalid identity".to_string(),
                ErrorKind::TransportSend => "Failed to send message".to_string(),
            }
        )
    }
}
