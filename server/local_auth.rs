/// In-memory AuthStore backed by a token cache received from tunnel-ctld.
///
/// Implements AuthStore so it can be a drop-in replacement for SqliteAuthStore
/// in the server process when running in managed (ctld-connected) mode.
///
/// Authentication is O(n) over the cache with constant-time hash comparison.
/// The cache is typically small (< 10k entries) so O(n) is fine.
use anyhow::Result;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tunnel_store::{AuthError, AuthResult, AuthStore, TokenListEntry};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub hash_bytes: [u8; 32],
    pub client_group: String,
    pub client_status: String,
    pub token_status: String,
}

/// Shared, atomically-swappable token cache.
#[derive(Clone)]
pub struct LocalTokenCache {
    inner: Arc<ArcSwap<Vec<CacheEntry>>>,
}

impl LocalTokenCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(Vec::new())),
        }
    }

    /// Replace the entire cache. Called when a Snapshot or Patch arrives from ctld.
    pub fn update(&self, entries: Vec<CacheEntry>) {
        self.inner.store(Arc::new(entries));
    }
}

#[async_trait]
impl AuthStore for LocalTokenCache {
    async fn authenticate(&self, raw_token: &str) -> std::result::Result<AuthResult, AuthError> {
        let candidate_bytes: [u8; 32] = tunnel_store::hash_token_bytes(raw_token);

        let cache = self.inner.load();
        for entry in cache.iter() {
            if entry.hash_bytes.ct_eq(&candidate_bytes).into() {
                if entry.client_status != "active" {
                    return Err(AuthError::ClientDisabled);
                }
                if entry.token_status != "active" {
                    return Err(AuthError::TokenRevoked);
                }
                return Ok(AuthResult { client_group: entry.client_group.clone() });
            }
        }
        Err(AuthError::InvalidToken)
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
