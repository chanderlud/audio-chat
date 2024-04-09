use std::array::TryFromSliceError;
use std::fmt::{Display, Formatter};
use std::{fmt, io};

use kanal::ReceiveError;

#[derive(Debug)]
pub(crate) struct Error {
    pub(crate) kind: ErrorKind,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(io::Error),
    TryFromSlice(TryFromSliceError),
    Ed25519(ed25519_dalek::ed25519::Error),
    CommonError(common::error::Error),
    Receive(ReceiveError),
    InvalidPublicKey(Vec<u8>),
    MissingLocalState,
    Timeout,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(error),
        }
    }
}

impl From<TryFromSliceError> for Error {
    fn from(error: TryFromSliceError) -> Self {
        Self {
            kind: ErrorKind::TryFromSlice(error),
        }
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(error: ed25519_dalek::ed25519::Error) -> Self {
        Self {
            kind: ErrorKind::Ed25519(error),
        }
    }
}

impl From<common::error::Error> for Error {
    fn from(error: common::error::Error) -> Self {
        Self {
            kind: ErrorKind::CommonError(error),
        }
    }
}

impl From<ReceiveError> for Error {
    fn from(error: ReceiveError) -> Self {
        Self {
            kind: ErrorKind::Receive(error),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.kind {
                ErrorKind::Io(ref error) => error.to_string(),
                ErrorKind::TryFromSlice(ref error) => error.to_string(),
                ErrorKind::Ed25519(ref error) => error.to_string(),
                ErrorKind::CommonError(ref error) => error.to_string(),
                ErrorKind::Receive(ref error) => error.to_string(),
                ErrorKind::InvalidPublicKey(_) => "Invalid public key".to_string(),
                ErrorKind::MissingLocalState => "Missing local state".to_string(),
                ErrorKind::Timeout => "Session timeout".to_string(),
            }
        )
    }
}
