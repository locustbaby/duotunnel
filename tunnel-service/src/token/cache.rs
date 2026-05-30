use crate::proto::TokenCacheEntry;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::Row;
use tunnel_store::{ClientStatus, TokenStatus};

#[async_trait]
pub trait TokenCacheProvider: Send + Sync {
    async fn load_token_cache(&self) -> Result<Vec<TokenCacheEntry>>;
}

pub struct SqliteTokenCacheProvider {
    pool: sqlx::sqlite::SqlitePool,
}

impl SqliteTokenCacheProvider {
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TokenCacheProvider for SqliteTokenCacheProvider {
    async fn load_token_cache(&self) -> Result<Vec<TokenCacheEntry>> {
        let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT c.name as client_group, c.status as client_status,
                    t.token_hash, t.status as token_status
             FROM client_tokens t
             JOIN clients c ON c.id = t.client_id
             ORDER BY t.id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r: sqlx::sqlite::SqliteRow| TokenCacheEntry {
                hash_hex: r.get("token_hash"),
                client_group: r.get("client_group"),
                client_status: ClientStatus::parse(&r.get::<String, _>("client_status"))
                    .unwrap_or(ClientStatus::Disabled),
                token_status: TokenStatus::parse(&r.get::<String, _>("token_status"))
                    .unwrap_or(TokenStatus::Revoked),
            })
            .collect())
    }
}
