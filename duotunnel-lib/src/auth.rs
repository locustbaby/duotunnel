use sha2::{Digest, Sha256};

use crate::GroupId;

#[derive(Debug, Clone)]
pub struct AuthResult {
    pub client_group: GroupId,
    pub token_hash: [u8; 32],
}

pub enum AuthError {
    InvalidToken,
    TokenRevoked,
    ClientDisabled,
    Internal(anyhow::Error),
}

fn mask_token_in_str(input: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if index + 3 <= chars.len()
            && chars[index] == 'd'
            && chars[index + 1] == 't'
            && chars[index + 2] == '_'
        {
            let start = index;
            index += 3;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '-'
                    || chars[index] == '_')
            {
                index += 1;
            }
            if index - start >= 10 {
                let token: String = chars[start..index].iter().collect();
                let digest = Sha256::digest(token.as_bytes());
                let masked = hex::encode(digest);
                result.push_str(&format!("dt_masked_{}", &masked[..8]));
            } else {
                result.extend(chars[start..index].iter());
            }
        } else {
            result.push(chars[index]);
            index += 1;
        }
    }
    result
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => write!(formatter, "InvalidToken"),
            Self::TokenRevoked => write!(formatter, "TokenRevoked"),
            Self::ClientDisabled => write!(formatter, "ClientDisabled"),
            Self::Internal(error) => formatter
                .debug_tuple("Internal")
                .field(&mask_token_in_str(&format!("{error:?}")))
                .finish(),
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => write!(f, "invalid token"),
            Self::TokenRevoked => write!(f, "token has been revoked"),
            Self::ClientDisabled => write!(f, "client is disabled"),
            Self::Internal(error) => write!(f, "internal error: {error}"),
        }
    }
}

impl std::error::Error for AuthError {}

pub fn hash_token_bytes(raw_token: &str) -> [u8; 32] {
    Sha256::digest(raw_token.as_bytes()).into()
}
