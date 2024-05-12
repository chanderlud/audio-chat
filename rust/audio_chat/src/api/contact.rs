use flutter_rust_bridge::frb;
use libp2p::PeerId;
use std::str::FromStr;
use uuid::Uuid;

use crate::api::error::{DartError, ErrorKind};

#[derive(Clone, Debug)]
#[frb(opaque)]
pub struct Contact {
    /// A random ID to identify the contact
    pub(crate) id: String,

    /// The nickname of the contact
    pub(crate) nickname: String,

    /// The public/verifying key for the contact
    pub(crate) peer_id: PeerId,
}

impl Contact {
    #[frb(sync)]
    pub fn new(nickname: String, peer_id: String) -> Result<Contact, DartError> {
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            nickname,
            peer_id: PeerId::from_str(&peer_id).map_err(|_| ErrorKind::InvalidContactFormat)?,
        })
    }

    #[frb(sync)]
    pub fn from_parts(id: String, nickname: String, peer_id: String) -> Result<Contact, DartError> {
        Ok(Self {
            id,
            nickname,
            peer_id: PeerId::from_str(&peer_id).map_err(|_| ErrorKind::InvalidContactFormat)?,
        })
    }

    #[frb(sync)]
    pub fn peer_id(&self) -> String {
        self.peer_id.to_string()
    }

    #[frb(sync)]
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }

    #[frb(sync)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[frb(sync)]
    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = nickname;
    }

    #[frb(sync)]
    pub fn pub_clone(&self) -> Contact {
        self.clone()
    }

    #[frb(sync)]
    pub fn id_eq(&self, id: Vec<u8>) -> bool {
        self.peer_id.to_bytes() == id
    }
}
