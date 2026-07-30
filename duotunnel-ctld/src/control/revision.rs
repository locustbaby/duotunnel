use anyhow::{anyhow, Result};
use async_trait::async_trait;
use duotunnel_lib::ctld_proto::ControlRevision;
use sqlx::{Row, SqlitePool};
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::Mutex;

#[async_trait]
pub trait ControlRevisionStore: Send + Sync {
    async fn current(&self) -> Result<ControlRevision>;
    async fn commit_snapshot_hash(&self, content_hash: &str) -> Result<ControlRevision>;
}

#[cfg(test)]
pub struct EphemeralControlRevisionStore {
    epoch: String,
    sequence: AtomicU64,
    content_hash: Mutex<Option<String>>,
}

#[cfg(test)]
impl EphemeralControlRevisionStore {
    pub fn new() -> Self {
        Self {
            epoch: "ephemeral".to_string(),
            sequence: AtomicU64::new(1),
            content_hash: Mutex::new(None),
        }
    }
}

#[async_trait]
#[cfg(test)]
impl ControlRevisionStore for EphemeralControlRevisionStore {
    async fn current(&self) -> Result<ControlRevision> {
        Ok(ControlRevision {
            epoch: self.epoch.clone(),
            sequence: self.sequence.load(Ordering::Acquire),
        })
    }

    async fn commit_snapshot_hash(&self, content_hash: &str) -> Result<ControlRevision> {
        let mut current_hash = self.content_hash.lock().unwrap();
        let sequence = if current_hash
            .as_deref()
            .is_none_or(|hash| hash == content_hash)
        {
            self.sequence.load(Ordering::Acquire)
        } else {
            self.sequence.fetch_add(1, Ordering::AcqRel) + 1
        };
        *current_hash = Some(content_hash.to_string());
        Ok(ControlRevision {
            epoch: self.epoch.clone(),
            sequence,
        })
    }
}

pub struct SqliteControlRevisionStore {
    pool: SqlitePool,
}

impl SqliteControlRevisionStore {
    pub async fn record_migration_on(
        conn: &mut sqlx::SqliteConnection,
        migration: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO schema_migrations(migration)
             VALUES (?1)",
        )
        .bind(migration)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn ensure_config_state_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                migration TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS config_state (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                yaml_source_revision TEXT NOT NULL DEFAULT '',
                sqlite_source_revision TEXT NOT NULL DEFAULT '',
                effective_revision INTEGER NOT NULL DEFAULT 0,
                effective_hash TEXT NOT NULL DEFAULT '',
                initialized INTEGER NOT NULL DEFAULT 0,
                degraded INTEGER NOT NULL DEFAULT 0,
                yaml_degraded INTEGER NOT NULL DEFAULT 0,
                sqlite_degraded INTEGER NOT NULL DEFAULT 0,
                coordinator_degraded INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await?;
        let columns = sqlx::query("PRAGMA table_info(config_state)")
            .fetch_all(pool)
            .await?;
        for (name, definition) in [
            ("singleton", "INTEGER NOT NULL DEFAULT 1"),
            ("yaml_source_revision", "TEXT NOT NULL DEFAULT ''"),
            ("sqlite_source_revision", "TEXT NOT NULL DEFAULT ''"),
            ("effective_revision", "INTEGER NOT NULL DEFAULT 0"),
            ("effective_hash", "TEXT NOT NULL DEFAULT ''"),
            ("initialized", "INTEGER NOT NULL DEFAULT 0"),
            ("degraded", "INTEGER NOT NULL DEFAULT 0"),
            ("yaml_degraded", "INTEGER NOT NULL DEFAULT 0"),
            ("sqlite_degraded", "INTEGER NOT NULL DEFAULT 0"),
            ("coordinator_degraded", "INTEGER NOT NULL DEFAULT 0"),
        ] {
            let exists = columns.iter().any(|row| {
                row.try_get::<String, _>("name")
                    .is_ok_and(|column| column == name)
            });
            if !exists {
                sqlx::query(&format!(
                    "ALTER TABLE config_state ADD COLUMN {name} {definition}"
                ))
                .execute(pool)
                .await?;
            }
        }
        sqlx::query("INSERT OR IGNORE INTO config_state(singleton) VALUES (1)")
            .execute(pool)
            .await?;
        let mut conn = pool.acquire().await?;
        Self::record_migration_on(&mut conn, "effective-config-state-v1").await?;
        Ok(())
    }

    pub async fn initialize(pool: SqlitePool) -> Result<Self> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS control_revision (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                epoch TEXT NOT NULL,
                sequence INTEGER NOT NULL CHECK (sequence >= 1),
                content_hash TEXT
            )",
        )
        .execute(&pool)
        .await?;
        let columns = sqlx::query("PRAGMA table_info(control_revision)")
            .fetch_all(&pool)
            .await?;
        let has_content_hash = columns.iter().any(|row| {
            row.try_get::<String, _>("name")
                .is_ok_and(|name| name == "content_hash")
        });
        if !has_content_hash {
            sqlx::query("ALTER TABLE control_revision ADD COLUMN content_hash TEXT")
                .execute(&pool)
                .await?;
        }
        sqlx::query(
            "INSERT OR IGNORE INTO control_revision(singleton, epoch, sequence)
             VALUES (
                 1,
                 strftime('%s', 'now') || substr(strftime('%f', 'now'), 4, 3)
                     || '-' || lower(hex(randomblob(8))),
                 1
             )",
        )
        .execute(&pool)
        .await?;
        Self::ensure_config_state_schema(&pool).await?;
        Ok(Self { pool })
    }

    async fn read_revision(&self) -> Result<ControlRevision> {
        let row = sqlx::query("SELECT epoch, sequence FROM control_revision WHERE singleton = 1")
            .fetch_one(&self.pool)
            .await?;
        let sequence: i64 = row.try_get("sequence")?;
        Ok(ControlRevision {
            epoch: row.try_get("epoch")?,
            sequence: u64::try_from(sequence)
                .map_err(|_| anyhow!("control revision sequence is negative"))?,
        })
    }

    pub async fn commit_snapshot_hash_on(
        conn: &mut sqlx::SqliteConnection,
        content_hash: &str,
    ) -> Result<ControlRevision> {
        Self::commit_snapshot_hash_on_impl(conn, content_hash, None).await
    }

    pub async fn commit_snapshot_hash_on_for_epoch(
        conn: &mut sqlx::SqliteConnection,
        content_hash: &str,
        expected_epoch: &str,
    ) -> Result<ControlRevision> {
        Self::commit_snapshot_hash_on_impl(conn, content_hash, Some(expected_epoch)).await
    }

    async fn commit_snapshot_hash_on_impl(
        conn: &mut sqlx::SqliteConnection,
        content_hash: &str,
        expected_epoch: Option<&str>,
    ) -> Result<ControlRevision> {
        let row = if let Some(expected_epoch) = expected_epoch {
            sqlx::query(
                "UPDATE control_revision
                 SET sequence = CASE
                         WHEN content_hash IS NULL OR content_hash = ?1 THEN sequence
                         ELSE sequence + 1
                     END,
                     content_hash = ?1
                 WHERE singleton = 1 AND epoch = ?2
                 RETURNING epoch, sequence",
            )
            .bind(content_hash)
            .bind(expected_epoch)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| anyhow!("control revision epoch changed while committing snapshot"))?
        } else {
            sqlx::query(
                "UPDATE control_revision
                 SET sequence = CASE
                         WHEN content_hash IS NULL OR content_hash = ?1 THEN sequence
                         ELSE sequence + 1
                     END,
                     content_hash = ?1
                 WHERE singleton = 1
                 RETURNING epoch, sequence",
            )
            .bind(content_hash)
            .fetch_one(&mut *conn)
            .await?
        };
        let sequence: i64 = row.try_get("sequence")?;
        sqlx::query(
            "UPDATE config_state SET effective_revision = ?1, effective_hash = ?2
             WHERE singleton = 1",
        )
        .bind(sequence)
        .bind(content_hash)
        .execute(&mut *conn)
        .await?;
        Ok(ControlRevision {
            epoch: row.try_get("epoch")?,
            sequence: u64::try_from(sequence)
                .map_err(|_| anyhow!("control revision sequence is negative"))?,
        })
    }
}

#[async_trait]
impl ControlRevisionStore for SqliteControlRevisionStore {
    async fn current(&self) -> Result<ControlRevision> {
        self.read_revision().await
    }

    async fn commit_snapshot_hash(&self, content_hash: &str) -> Result<ControlRevision> {
        let mut tx = self.pool.begin().await?;
        let revision = Self::commit_snapshot_hash_on(&mut tx, content_hash).await?;
        tx.commit().await?;
        Ok(revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::open_sqlite_pool;

    #[tokio::test]
    async fn sqlite_revision_survives_store_recreation() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let first = SqliteControlRevisionStore::initialize(pool.clone())
            .await
            .unwrap();
        let initial = first.current().await.unwrap();
        let initialized = first.commit_snapshot_hash("hash-a").await.unwrap();
        let advanced = first.commit_snapshot_hash("hash-b").await.unwrap();
        drop(first);

        let reopened = SqliteControlRevisionStore::initialize(pool).await.unwrap();
        let recovered = reopened.current().await.unwrap();

        assert_eq!(initial, initialized);
        assert_eq!(initial.epoch, advanced.epoch);
        assert_eq!(advanced, recovered);
        assert_eq!(advanced.sequence, initial.sequence + 1);
    }
}
