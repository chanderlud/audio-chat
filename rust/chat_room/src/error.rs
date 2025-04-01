use std::fmt::{Display, Formatter};

use crate::behaviour::BehaviourEvent;
use libp2p::identity::{DecodingError, ParseError};
use libp2p::swarm::{DialError, SwarmEvent};
use libp2p::TransportError;
use libp2p_stream::{AlreadyRegistered, OpenStreamError};
use tokio::time::error::Elapsed;

/// generic error type for audio chat
#[derive(Debug)]
pub(crate) struct Error {
    pub(crate) kind: ErrorKind,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(std::io::Error),
    Decode(bincode::error::DecodeError),
    KanalSend(kanal::SendError),
    KanalReceive(kanal::ReceiveError),
    Timeout(Elapsed),
    IdentityDecode(DecodingError),
    OpenStream(OpenStreamError),
    Dial(DialError),
    IdentityParse(ParseError),
    Transport(TransportError<std::io::Error>),
    AlreadyRegistered(AlreadyRegistered),
    TransportSend,
    TransportRecv,
    UnexpectedSwarmEvent,
    SwarmBuild,
    SwarmEnded,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
        }
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(err: bincode::error::DecodeError) -> Self {
        Self {
            kind: ErrorKind::Decode(err),
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

impl From<Elapsed> for Error {
    fn from(err: Elapsed) -> Self {
        Self {
            kind: ErrorKind::Timeout(err),
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
                ErrorKind::KanalSend(ref err) => format!("Kanal send error: {}", err),
                ErrorKind::KanalReceive(ref err) => format!("Kanal receive error: {}", err),
                ErrorKind::Timeout(_) => "The connection timed out".to_string(),
                ErrorKind::IdentityDecode(ref err) => format!("Identity decode error: {}", err),
                ErrorKind::OpenStream(ref err) => format!("Open stream error: {}", err),
                ErrorKind::Dial(ref err) => format!("Dial error: {}", err),
                ErrorKind::IdentityParse(ref err) => format!("Identity parse error: {}", err),
                ErrorKind::Transport(ref err) => format!("Transport error: {}", err),
                ErrorKind::AlreadyRegistered(ref err) => format!("Already registered: {}", err),
                ErrorKind::TransportSend => "Transport failed on send".to_string(),
                ErrorKind::TransportRecv => "Transport failed on receive".to_string(),
                ErrorKind::UnexpectedSwarmEvent => "Unexpected swarm event".to_string(),
                ErrorKind::SwarmBuild => "Swarm build error".to_string(),
                ErrorKind::SwarmEnded => "Swarm ended".to_string(),
            }
        )
    }
}
