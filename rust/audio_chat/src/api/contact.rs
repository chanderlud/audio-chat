use flutter_rust_bridge::frb;
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
    pub(crate) verifying_key: [u8; 32],
}

impl Contact {
    #[frb(sync)]
    pub fn new(nickname: String, key_bytes: Vec<u8>) -> Result<Contact, DartError> {
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            nickname,
            verifying_key: key_bytes
                .try_into()
                .map_err(|_| ErrorKind::InvalidContactFormat)?,
        })
    }

    #[frb(sync)]
    pub fn from_parts(
        id: String,
        nickname: String,
        verifying_key: Vec<u8>,
    ) -> Result<Contact, DartError> {
        Ok(Self {
            id,
            nickname,
            verifying_key: verifying_key
                .try_into()
                .map_err(|_| ErrorKind::InvalidContactFormat)?,
        })
    }

    #[frb(sync)]
    pub fn verifying_key(&self) -> [u8; 32] {
        self.verifying_key
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
    pub fn key_eq(&self, key: Vec<u8>) -> bool {
        key.eq(&self.verifying_key)
    }
}
