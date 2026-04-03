/// Build a token cache by querying the SQLite pool directly.
/// This is separate from AuthStore::list_tokens() which doesn't expose hash values.
use anyhow::Result;
use sqlx::Row;
use crate::proto::TokenCacheEntry;

pub async fn load_token_cache(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<TokenCacheEntry>> {
    let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
        "SELECT c.name as client_group, c.status as client_status,
                t.token_hash, t.status as token_status
         FROM client_tokens t
         JOIN clients c ON c.id = t.client_id
         ORDER BY t.id",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r: sqlx::sqlite::SqliteRow| TokenCacheEntry {
            hash_hex: r.get("token_hash"),
            client_group: r.get("client_group"),
            client_status: r.get("client_status"),
            token_status: r.get("token_status"),
        })
        .collect())
}
