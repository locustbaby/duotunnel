use anyhow::{anyhow, Result};
use async_trait::async_trait;
use duotunnel_lib::{ClientStatus, TokenStatus};
use sqlx::sqlite::{SqliteConnection, SqlitePool, SqlitePoolOptions};
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

pub const ADMIN_IDEMPOTENCY_PLAINTEXT_RESPONSE_ENCODING: &str = "plaintext-v1";
pub const ADMIN_IDEMPOTENCY_REDACTED_RESPONSE_ENCODING: &str = "redacted-v1";
const ADMIN_IDEMPOTENCY_RETENTION_DAYS: u32 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminIdempotencyRecord {
    pub fingerprint: String,
    pub operation: String,
    pub status_code: u16,
    pub response_body: String,
    pub response_encoding: String,
}

pub struct AdminIdempotencyInsert<'a> {
    pub scope: &'a str,
    pub request_key: &'a str,
    pub fingerprint: &'a str,
    pub operation: &'a str,
    pub status_code: u16,
    pub response_body: &'a str,
    pub response_encoding: &'a str,
}

pub async fn begin_immediate(
    pool: &SqlitePool,
) -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>> {
    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    Ok(conn)
}

pub async fn commit_immediate(conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>) -> Result<()> {
    sqlx::query("COMMIT").execute(&mut **conn).await?;
    Ok(())
}

pub async fn rollback_immediate(conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>) -> Result<()> {
    sqlx::query("ROLLBACK").execute(&mut **conn).await?;
    Ok(())
}

pub async fn load_admin_idempotency_on(
    conn: &mut SqliteConnection,
    scope: &str,
    request_key: &str,
) -> Result<Option<AdminIdempotencyRecord>> {
    let row = sqlx::query(
        "SELECT fingerprint, operation, status_code, response_body, response_encoding
         FROM admin_idempotency
         WHERE scope = ?1 AND request_key = ?2",
    )
    .bind(scope)
    .bind(request_key)
    .fetch_optional(&mut *conn)
    .await?;
    row.map(|row| {
        Ok(AdminIdempotencyRecord {
            fingerprint: row.try_get("fingerprint")?,
            operation: row.try_get("operation")?,
            status_code: u16::try_from(row.try_get::<i64, _>("status_code")?)?,
            response_body: row.try_get("response_body")?,
            response_encoding: row.try_get("response_encoding")?,
        })
    })
    .transpose()
}

pub async fn insert_admin_idempotency_on(
    conn: &mut SqliteConnection,
    record: &AdminIdempotencyInsert<'_>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO admin_idempotency
            (scope, request_key, fingerprint, operation, status_code,
             response_body, response_encoding)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(record.scope)
    .bind(record.request_key)
    .bind(record.fingerprint)
    .bind(record.operation)
    .bind(i64::from(record.status_code))
    .bind(record.response_body)
    .bind(record.response_encoding)
    .execute(&mut *conn)
    .await?;
    Ok(())
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
        ensure_admin_idempotency_schema(&self.pool).await?;
        info!("database migrations applied");
        Ok(())
    }
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_client_on(conn: &mut SqliteConnection, name: &str) -> Result<String> {
        let raw_token = generate_token();
        let token_hash = hash_token(&raw_token);
        let client_id: i64 = match sqlx::query("SELECT id FROM clients WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut *conn)
            .await?
        {
            Some(row) => {
                let id: i64 = row.get("id");
                let active_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM client_tokens WHERE client_id = ? AND status = 'active'",
                )
                .bind(id)
                .fetch_one(&mut *conn)
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
                .execute(&mut *conn)
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
        .execute(&mut *conn)
        .await?;
        Ok(raw_token)
    }

    pub async fn revoke_token_on(conn: &mut SqliteConnection, name: &str) -> Result<()> {
        let result = sqlx::query(
            "UPDATE client_tokens SET status = 'revoked', revoked_at = datetime('now')
             WHERE client_id = (SELECT id FROM clients WHERE name = ?)
             AND status = 'active'",
        )
        .bind(name)
        .execute(&mut *conn)
        .await?;
        if result.rows_affected() == 0 {
            return Err(anyhow!("no active token found for client '{}'", name));
        }
        info!(name = % name, revoked = result.rows_affected(), "tokens revoked");
        Ok(())
    }

    pub async fn rotate_token_on(conn: &mut SqliteConnection, name: &str) -> Result<String> {
        let revoked = sqlx::query(
            "UPDATE client_tokens SET status = 'revoked', revoked_at = datetime('now')
             WHERE client_id = (SELECT id FROM clients WHERE name = ?)
             AND status = 'active'",
        )
        .bind(name)
        .execute(&mut *conn)
        .await?;
        if revoked.rows_affected() == 0 {
            return Err(anyhow!("no active token found for client '{}'", name));
        }
        let raw_token = generate_token();
        let token_hash = hash_token(&raw_token);
        let client_id: i64 = sqlx::query_scalar("SELECT id FROM clients WHERE name = ?")
            .bind(name)
            .fetch_one(&mut *conn)
            .await?;
        sqlx::query(
            "INSERT INTO client_tokens (client_id, token_hash, status, created_at)
             VALUES (?, ?, 'active', datetime('now'))",
        )
        .bind(client_id)
        .bind(&token_hash)
        .execute(&mut *conn)
        .await?;
        Ok(raw_token)
    }
}

pub async fn ensure_admin_idempotency_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS admin_idempotency (
            scope TEXT NOT NULL,
            request_key TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            operation TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            response_body TEXT NOT NULL,
            response_encoding TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (scope, request_key)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_admin_idempotency_created_at
         ON admin_idempotency(created_at)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "DELETE FROM admin_idempotency
         WHERE created_at < datetime('now', ?1)",
    )
    .bind(format!("-{} days", ADMIN_IDEMPOTENCY_RETENTION_DAYS))
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE admin_idempotency
         SET response_body = '', response_encoding = 'redacted-v1'
         WHERE operation IN ('create_client', 'rotate_token')
           AND response_encoding = 'plaintext-v1'",
    )
    .execute(pool)
    .await?;
    Ok(())
}
#[async_trait]
impl AuthStore for SqliteAuthStore {
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
}
