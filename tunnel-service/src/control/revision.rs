use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::Mutex;
use tunnel_lib::ctld_proto::ControlRevision;

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
}

#[async_trait]
impl ControlRevisionStore for SqliteControlRevisionStore {
    async fn current(&self) -> Result<ControlRevision> {
        self.read_revision().await
    }

    async fn commit_snapshot_hash(&self, content_hash: &str) -> Result<ControlRevision> {
        let row = sqlx::query(
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
        .fetch_one(&self.pool)
        .await?;
        let sequence: i64 = row.try_get("sequence")?;
        Ok(ControlRevision {
            epoch: row.try_get("epoch")?,
            sequence: u64::try_from(sequence)
                .map_err(|_| anyhow!("control revision sequence is negative"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tunnel_store::sqlite::open_sqlite_pool;

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
