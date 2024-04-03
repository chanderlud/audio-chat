use ed25519_dalek::Signature;

include!(concat!(env!("OUT_DIR"), "/common.items.rs"));

impl Message {
    pub fn new(message: message::Message, to: &[u8], from: &[u8]) -> Self {
        Self {
            to: to.to_vec(),
            from: from.to_vec(),
            message: Some(message),
        }
    }
}

impl From<RequestSession> for message::Message {
    fn from(request: RequestSession) -> Self {
        Self::RequestSession(request)
    }
}

impl From<RequestOutcome> for message::Message {
    fn from(outcome: RequestOutcome) -> Self {
        Self::RequestOutcome(outcome)
    }
}

impl From<Candidate> for message::Message {
    fn from(candidate: Candidate) -> Self {
        Self::Candidate(candidate)
    }
}

impl From<ServerError> for message::Message {
    fn from(error: ServerError) -> Self {
        Self::ServerError(error)
    }
}

impl From<EndSession> for message::Message {
    fn from(end: EndSession) -> Self {
        Self::EndSession(end)
    }
}

impl Identity {
    pub fn new(nonce: &[u8], signature: Signature, public_key: &[u8]) -> Self {
        Self {
            nonce: nonce.to_vec(),
            signature: signature.to_bytes().to_vec(),
            public_key: public_key.to_vec(),
        }
    }
}

impl RequestSession {
    pub fn new(ufrag: &str, pwd: &str) -> Self {
        Self {
            ufrag: ufrag.to_string(),
            pwd: pwd.to_string(),
        }
    }
}

impl RequestOutcome {
    pub fn success(ufrag: &str, pwd: &str) -> Self {
        Self {
            success: true,
            reason: None,
            ufrag: Some(ufrag.to_string()),
            pwd: Some(pwd.to_string()),
        }
    }

    pub fn failure(reason: &str) -> Self {
        Self {
            success: false,
            reason: Some(reason.to_string()),
            ufrag: None,
            pwd: None,
        }
    }
}

impl From<String> for Candidate {
    fn from(candidate: String) -> Self {
        Self { candidate }
    }
}

impl ServerError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl EndSession {
    pub fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}
