/// In-memory AuthStore backed by a token cache received from tunnel-ctld.
///
/// Implements AuthStore so it can be a drop-in replacement for SqliteAuthStore
/// in the server process when running in managed (ctld-connected) mode.
///
/// Authentication is O(1) via HashMap keyed on the raw SHA-256 hash bytes.
/// The map is atomically swapped on every Snapshot/Patch from ctld.
use anyhow::Result;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tunnel_store::{AuthError, AuthResult, AuthStore, TokenListEntry};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub hash_bytes: [u8; 32],
    pub client_group: String,
    pub client_status: String,
    pub token_status: String,
}

/// Atomically-swappable O(1) token lookup map.
/// Key: raw SHA-256 bytes of token. Value: CacheEntry.
pub type TokenMap = HashMap<[u8; 32], CacheEntry>;

/// Shared, atomically-swappable token cache.
#[derive(Clone)]
pub struct LocalTokenCache {
    inner: Arc<ArcSwap<TokenMap>>,
}

impl LocalTokenCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(HashMap::new())),
        }
    }

    /// Replace the entire cache atomically. Called on every Snapshot or Patch from ctld.
    pub fn update(&self, entries: Vec<CacheEntry>) {
        let map: TokenMap = entries.into_iter().map(|e| (e.hash_bytes, e)).collect();
        self.inner.store(Arc::new(map));
    }
}

#[async_trait]
impl AuthStore for LocalTokenCache {
    async fn authenticate(&self, raw_token: &str) -> std::result::Result<AuthResult, AuthError> {
        let key: [u8; 32] = tunnel_store::hash_token_bytes(raw_token);
        let map = self.inner.load();
        match map.get(&key) {
            None => Err(AuthError::InvalidToken),
            Some(entry) => {
                if entry.client_status != "active" {
                    return Err(AuthError::ClientDisabled);
                }
                if entry.token_status != "active" {
                    return Err(AuthError::TokenRevoked);
                }
                Ok(AuthResult {
                    client_group: entry.client_group.clone(),
                })
            }
        }
    }

    async fn create_client(&self, _name: &str) -> Result<String> {
        anyhow::bail!("LocalTokenCache is read-only; use tunnel-ctld to manage clients")
    }

    async fn list_tokens(&self) -> Result<Vec<TokenListEntry>> {
        anyhow::bail!("LocalTokenCache does not support list_tokens; use tunnel-ctld")
    }

    async fn revoke_token(&self, _name: &str) -> Result<()> {
        anyhow::bail!("LocalTokenCache is read-only; use tunnel-ctld to manage clients")
    }

    async fn rotate_token(&self, _name: &str) -> Result<String> {
        anyhow::bail!("LocalTokenCache is read-only; use tunnel-ctld to manage clients")
    }
}
