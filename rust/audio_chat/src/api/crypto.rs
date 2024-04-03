use ed25519_dalek::SigningKey;
use flutter_rust_bridge::frb;
use rand::rngs::OsRng;

#[frb(sync)]
pub fn generate_keys() -> [u8; 64] {
    let signing_key: SigningKey = SigningKey::generate(&mut OsRng);
    signing_key.to_keypair_bytes()
}
