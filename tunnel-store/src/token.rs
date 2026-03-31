use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    format!("dt_{}", URL_SAFE_NO_PAD.encode(bytes))
}

pub fn hash_token(raw_token: &str) -> String {
    let hash = Sha256::digest(raw_token.as_bytes());
    hex::encode(hash)
}
