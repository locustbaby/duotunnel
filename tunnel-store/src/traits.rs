use anyhow::Result;
use async_trait::async_trait;
use tunnel_lib::{ClientStatus, TokenStatus};
pub struct AuthResult {
    pub client_group: String,
}
pub enum AuthError {
    InvalidToken,
    TokenRevoked,
    ClientDisabled,
    Internal(anyhow::Error),
}

fn mask_token_in_str(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 3 <= chars.len() && chars[i] == 'd' && chars[i + 1] == 't' && chars[i + 2] == '_' {
            let start = i;
            i += 3;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '-' || chars[i] == '_')
            {
                i += 1;
            }
            let token_len = i - start;
            if token_len >= 10 {
                let token: String = chars[start..i].iter().collect();
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(token.as_bytes());
                let hashed_hex = hex::encode(hash);
                result.push_str(&format!("dt_masked_{}", &hashed_hex[..8]));
            } else {
                result.push_str(&chars[start..i].iter().collect::<String>());
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => write!(f, "InvalidToken"),
            AuthError::TokenRevoked => write!(f, "TokenRevoked"),
            AuthError::ClientDisabled => write!(f, "ClientDisabled"),
            AuthError::Internal(err) => {
                let err_str = format!("{:?}", err);
                let masked = mask_token_in_str(&err_str);
                f.debug_tuple("Internal").field(&masked).finish()
            }
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => write!(f, "invalid token"),
            AuthError::TokenRevoked => write!(f, "token has been revoked"),
            AuthError::ClientDisabled => write!(f, "client is disabled"),
            AuthError::Internal(err) => {
                let err_str = err.to_string();
                let masked = mask_token_in_str(&err_str);
                write!(f, "internal error: {}", masked)
            }
        }
    }
}
impl std::error::Error for AuthError {}
#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn authenticate(&self, raw_token: &str) -> std::result::Result<AuthResult, AuthError>;
    async fn create_client(&self, name: &str) -> Result<String>;
    async fn list_tokens(&self) -> Result<Vec<TokenListEntry>>;
    async fn revoke_token(&self, name: &str) -> Result<()>;
    async fn rotate_token(&self, name: &str) -> Result<String>;
}
pub struct TokenListEntry {
    pub client_name: String,
    pub client_status: ClientStatus,
    pub token_id: i64,
    pub token_status: Option<TokenStatus>,
    pub created_at: String,
    pub revoked_at: Option<String>,
}
