use flutter_rust_bridge::frb;
use libp2p::identity::Keypair;

use crate::api::error::DartError;

#[frb(sync)]
pub fn generate_keys() -> Result<(String, Vec<u8>), DartError> {
    let pair = Keypair::generate_ed25519();

    let peer_id = pair.public().to_peer_id();

    Ok((
        peer_id.to_string(),
        pair.to_protobuf_encoding()
            .map_err(|e| DartError::from(e.to_string()))?,
    ))
}
