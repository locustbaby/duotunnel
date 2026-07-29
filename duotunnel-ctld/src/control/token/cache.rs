use crate::control::proto::TokenCacheEntry;
use anyhow::Result;
use async_trait::async_trait;
use duotunnel_lib::{ClientStatus, TokenStatus};
use sqlx::Row;

#[async_trait]
pub trait TokenCacheProvider: Send + Sync {
    async fn load_token_cache(&self) -> Result<Vec<TokenCacheEntry>>;
    #[allow(dead_code)]
    async fn data_version(&self) -> Result<i64>;
}

pub struct SqliteTokenCacheProvider {
    pool: sqlx::sqlite::SqlitePool,
}

impl SqliteTokenCacheProvider {
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn load_token_cache_on(
        conn: &mut sqlx::SqliteConnection,
    ) -> Result<Vec<TokenCacheEntry>> {
        let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT c.name as client_group, c.status as client_status,
                    t.token_hash, t.status as token_status
             FROM client_tokens t
             JOIN clients c ON c.id = t.client_id
             ORDER BY t.id",
        )
        .fetch_all(&mut *conn)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r: sqlx::sqlite::SqliteRow| TokenCacheEntry {
                hash_hex: r.get("token_hash"),
                client_group: r.get::<String, _>("client_group").into(),
                client_status: ClientStatus::parse(&r.get::<String, _>("client_status"))
                    .unwrap_or(ClientStatus::Disabled),
                token_status: TokenStatus::parse(&r.get::<String, _>("token_status"))
                    .unwrap_or(TokenStatus::Revoked),
            })
            .collect())
    }
}

#[async_trait]
impl TokenCacheProvider for SqliteTokenCacheProvider {
    async fn load_token_cache(&self) -> Result<Vec<TokenCacheEntry>> {
        let mut conn = self.pool.acquire().await?;
        Self::load_token_cache_on(&mut conn).await
    }

    #[allow(dead_code)]
    async fn data_version(&self) -> Result<i64> {
        let version: i64 = sqlx::query_scalar("PRAGMA data_version")
            .fetch_one(&self.pool)
            .await?;
        Ok(version)
    }
}
