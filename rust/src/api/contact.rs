use base64::Engine;
use std::net::SocketAddr;
use std::str::FromStr;

use base64::prelude::BASE64_STANDARD;
use flutter_rust_bridge::frb;
use itertools::Itertools;
use uuid::Uuid;

use crate::api::error::{DartError, Error};

#[derive(Clone, Debug)]
#[frb(opaque)]
pub struct Contact {
    /// A random ID to identify the contact
    pub(crate) id: String,

    /// The nickname of the contact
    pub(crate) nickname: String,

    /// The public/verifying key for the contact
    pub(crate) verifying_key: [u8; 32],

    /// The address of the contact
    pub(crate) address: SocketAddr,
}

impl FromStr for Contact {
    type Err = DartError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: (&str, &str, &str, &str) = s
            .splitn(4, ',')
            .collect_tuple()
            .ok_or(Error::invalid_contact_format())?;

        let id = parts.0.to_string();
        let nickname = parts.1.to_string();
        let verifying_key = BASE64_STANDARD
            .decode(parts.2.as_bytes())
            .map_err(Error::from)?;
        let address = parts.3.parse().map_err(Error::from)?;

        Ok(Self {
            id,
            nickname,
            verifying_key: verifying_key
                .try_into()
                .map_err(|_| Error::invalid_contact_format())?,
            address,
        })
    }
}

impl Contact {
    #[frb(sync)]
    pub fn new(
        nickname: String,
        verifying_key: String,
        address: String,
    ) -> std::result::Result<Contact, DartError> {
        let key = BASE64_STANDARD
            .decode(verifying_key.as_bytes())
            .map_err(Error::from)?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            nickname,
            verifying_key: key
                .try_into()
                .map_err(|_| Error::invalid_contact_format())?,
            address: address.parse().map_err(Error::from)?,
        })
    }

    #[frb(sync)]
    pub fn parse(s: String) -> std::result::Result<Contact, DartError> {
        Self::from_str(&s)
    }

    #[frb(sync)]
    pub fn verifying_key(&self) -> Vec<u8> {
        self.verifying_key.to_vec()
    }

    #[frb(sync)]
    pub fn verifying_key_str(&self) -> String {
        BASE64_STANDARD.encode(&self.verifying_key)
    }

    #[frb(sync)]
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }

    #[frb(sync)]
    pub fn address_str(&self) -> String {
        self.address.to_string()
    }

    #[frb(sync)]
    pub fn ip_str(&self) -> String {
        self.address.ip().to_string()
    }

    #[frb(sync)]
    pub fn store(&self) -> String {
        format!(
            "{},{},{},{}",
            self.id,
            self.nickname,
            self.verifying_key_str(),
            self.address
        )
    }

    #[frb(sync)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[frb(sync)]
    pub fn set_address(&mut self, address: String) -> std::result::Result<(), DartError> {
        self.address = address.parse().map_err(Error::from)?;
        Ok(())
    }

    #[frb(sync)]
    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = nickname;
    }

    #[frb(sync)]
    pub fn pub_clone(&self) -> Contact {
        self.clone()
    }
}
