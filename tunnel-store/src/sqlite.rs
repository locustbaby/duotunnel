use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use tracing::{info, warn};
pub async fn open_sqlite_pool(database_url: &str) -> Result<SqlitePool> {
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
        .max_connections(5)
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
use crate::token::{generate_token, hash_token, hash_token_bytes};
use crate::{AuthError, AuthResult, AuthStore, TokenListEntry};
use subtle::ConstantTimeEq;
pub struct SqliteAuthStore {
    pool: SqlitePool,
}
impl SqliteAuthStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = open_sqlite_pool(database_url).await?;
        Ok(Self { pool })
    }
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
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl AuthStore for SqliteAuthStore {
    async fn authenticate(&self, raw_token: &str) -> std::result::Result<AuthResult, AuthError> {
        let candidate: [u8; 32] = hash_token_bytes(raw_token);
        let candidate_hex = hex::encode(candidate);
        const HASH_PREFIX_HEX_LEN: usize = 16;
        const MAX_CANDIDATES: usize = 8;
        let prefix = &candidate_hex[..HASH_PREFIX_HEX_LEN];
        let rows = sqlx::query(
            "SELECT c.name as client_name, c.status as client_status,
                        t.token_hash, t.status as token_status
             FROM client_tokens t
             JOIN clients c ON c.id = t.client_id
             WHERE substr(t.token_hash, 1, 16) = ?
             LIMIT 9",
        )
        .bind(prefix)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::Internal(e.into()))?;
        if rows.len() > MAX_CANDIDATES {
            return Err(AuthError::Internal(anyhow!(
                "token prefix collision limit exceeded"
            )));
        }
        let mut matched: Option<(String, String, String)> = None;
        for row in rows {
            let stored_hex: String = row.get("token_hash");
            let stored_bytes =
                hex::decode(&stored_hex).map_err(|e| AuthError::Internal(e.into()))?;
            let stored: [u8; 32] = stored_bytes
                .try_into()
                .map_err(|_| AuthError::Internal(anyhow!("invalid stored hash length")))?;
            if candidate.as_ref().ct_eq(stored.as_ref()).unwrap_u8() == 1 {
                matched = Some((
                    row.get("client_name"),
                    row.get("client_status"),
                    row.get("token_status"),
                ));
            }
        }
        let (client_group, client_status, token_status) = match matched {
            Some(values) => values,
            None => return Err(AuthError::InvalidToken),
        };
        if token_status == "revoked" {
            warn!(client_group = %client_group, "login attempt with revoked token");
            return Err(AuthError::TokenRevoked);
        }
        if client_status != "active" {
            warn!(client_group = %client_group, "login attempt for disabled client");
            return Err(AuthError::ClientDisabled);
        }
        Ok(AuthResult { client_group })
    }
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
                    client_status: r.get("client_status"),
                    token_id: token_id.unwrap_or(-1),
                    token_status: r
                        .try_get::<String, _>("token_status")
                        .unwrap_or_else(|_| "-".into()),
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
