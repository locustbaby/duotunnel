use anyhow::Result;
use async_trait::async_trait;
use duotunnel_lib::{ClientStatus, TokenStatus};
#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn list_tokens(&self) -> Result<Vec<TokenListEntry>>;
}
pub struct TokenListEntry {
    pub client_name: String,
    pub client_status: ClientStatus,
    pub token_id: i64,
    pub token_status: Option<TokenStatus>,
    pub created_at: String,
    pub revoked_at: Option<String>,
}
