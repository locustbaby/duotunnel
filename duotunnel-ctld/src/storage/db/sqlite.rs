use anyhow::{anyhow, Result};
use async_trait::async_trait;
use duotunnel_lib::{ClientStatus, TokenStatus};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use tracing::info;
pub async fn open_sqlite_pool(database_url: &str, max_connections: u32) -> Result<SqlitePool> {
    if let Some(path) = database_url
        .strip_prefix("sqlite://")
        .and_then(|s| s.split('?').next())
    {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys=ON").execute(conn).await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?;
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await?;
    Ok(pool)
}
use crate::storage::token::{generate_token, hash_token};
use crate::storage::{AuthStore, TokenListEntry};
pub struct SqliteAuthStore {
    pool: SqlitePool,
}
impl SqliteAuthStore {
    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS client_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id INTEGER NOT NULL REFERENCES clients(id),
                token_hash TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                revoked_at TEXT
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_client_tokens_hash ON client_tokens(token_hash)",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_client_tokens_client_id ON client_tokens(client_id)",
        )
        .execute(&self.pool)
        .await?;
        info!("database migrations applied");
        Ok(())
    }
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl AuthStore for SqliteAuthStore {
    async fn create_client(&self, name: &str) -> Result<String> {
        let raw_token = generate_token();
        let token_hash = hash_token(&raw_token);
        let mut tx = self.pool.begin().await?;
        let client_id: i64 = match sqlx::query("SELECT id FROM clients WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut *tx)
            .await?
        {
            Some(row) => {
                let id: i64 = row.get("id");
                let active_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM client_tokens WHERE client_id = ? AND status = 'active'",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
                if active_count > 0 {
                    return Err(anyhow!(
                        "client '{}' already has an active token; use 'rotate' to replace it",
                        name
                    ));
                }
                id
            }
            None => {
                let result = sqlx::query(
                    "INSERT INTO clients (name, status, created_at, updated_at)
                     VALUES (?, 'active', datetime('now'), datetime('now'))",
                )
                .bind(name)
                .execute(&mut *tx)
                .await?;
                result.last_insert_rowid()
            }
        };
        sqlx::query(
            "INSERT INTO client_tokens (client_id, token_hash, status, created_at)
             VALUES (?, ?, 'active', datetime('now'))",
        )
        .bind(client_id)
        .bind(&token_hash)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(raw_token)
    }
    async fn list_tokens(&self) -> Result<Vec<TokenListEntry>> {
        let rows = sqlx::query(
            "SELECT c.name, c.status as client_status,
                    t.id as token_id, t.status as token_status,
                    t.created_at, t.revoked_at
             FROM clients c
             LEFT JOIN client_tokens t ON t.client_id = c.id
             ORDER BY c.name, t.id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| {
                let token_id: Option<i64> = r.get("token_id");
                TokenListEntry {
                    client_name: r.get("name"),
                    client_status: ClientStatus::parse(r.get::<&str, _>("client_status"))
                        .unwrap_or(ClientStatus::Disabled),
                    token_id: token_id.unwrap_or(-1),
                    token_status: r
                        .try_get::<&str, _>("token_status")
                        .ok()
                        .and_then(TokenStatus::parse),
                    created_at: r
                        .try_get::<String, _>("created_at")
                        .unwrap_or_else(|_| "-".into()),
                    revoked_at: r.get("revoked_at"),
                }
            })
            .collect())
    }
    async fn revoke_token(&self, name: &str) -> Result<()> {
        let result = sqlx::query(
            "UPDATE client_tokens SET status = 'revoked', revoked_at = datetime('now')
             WHERE client_id = (SELECT id FROM clients WHERE name = ?)
             AND status = 'active'",
        )
        .bind(name)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(anyhow!("no active token found for client '{}'", name));
        }
        info!(name = % name, revoked = result.rows_affected(), "tokens revoked");
        Ok(())
    }
    async fn rotate_token(&self, name: &str) -> Result<String> {
        let mut tx = self.pool.begin().await?;
        let revoked = sqlx::query(
            "UPDATE client_tokens SET status = 'revoked', revoked_at = datetime('now')
             WHERE client_id = (SELECT id FROM clients WHERE name = ?)
             AND status = 'active'",
        )
        .bind(name)
        .execute(&mut *tx)
        .await?;
        if revoked.rows_affected() == 0 {
            return Err(anyhow!("no active token found for client '{}'", name));
        }
        let raw_token = generate_token();
        let token_hash = hash_token(&raw_token);
        let client_id: i64 = sqlx::query_scalar("SELECT id FROM clients WHERE name = ?")
            .bind(name)
            .fetch_one(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO client_tokens (client_id, token_hash, status, created_at)
             VALUES (?, ?, 'active', datetime('now'))",
        )
        .bind(client_id)
        .bind(&token_hash)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(raw_token)
    }
}
