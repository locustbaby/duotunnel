use anyhow::Result;
use async_trait::async_trait;
pub struct AuthResult {
    pub client_group: String,
}
#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    TokenRevoked,
    ClientDisabled,
    Internal(anyhow::Error),
}
impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => write!(f, "invalid token"),
            AuthError::TokenRevoked => write!(f, "token has been revoked"),
            AuthError::ClientDisabled => write!(f, "client is disabled"),
            AuthError::Internal(e) => write!(f, "internal error: {}", e),
        }
    }
}
impl std::error::Error for AuthError {}
#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn authenticate(
        &self,
        raw_token: &str,
    ) -> std::result::Result<AuthResult, AuthError>;
    async fn create_client(&self, name: &str) -> Result<String>;
    async fn list_tokens(&self) -> Result<Vec<TokenListEntry>>;
    async fn revoke_token(&self, name: &str) -> Result<()>;
    async fn rotate_token(&self, name: &str) -> Result<String>;
}
pub struct TokenListEntry {
    pub client_name: String,
    pub client_status: String,
    pub token_id: i64,
    pub token_status: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
}
