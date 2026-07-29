use duotunnel_core::GroupId;
use duotunnel_store::{AuthError, AuthResult, ClientStatus, TokenStatus};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub client_group: GroupId,
    pub client_status: ClientStatus,
    pub token_status: TokenStatus,
}

pub type TokenMap = HashMap<[u8; 32], CacheEntry>;

pub fn authenticate(map: &TokenMap, raw_token: &str) -> std::result::Result<AuthResult, AuthError> {
    let key = duotunnel_store::hash_token_bytes(raw_token);
    match map.get(&key) {
        None => Err(AuthError::InvalidToken),
        Some(entry) => {
            if entry.client_status != ClientStatus::Active {
                return Err(AuthError::ClientDisabled);
            }
            if entry.token_status != TokenStatus::Active {
                return Err(AuthError::TokenRevoked);
            }
            Ok(AuthResult {
                client_group: entry.client_group.clone(),
                token_hash: key,
            })
        }
    }
}
