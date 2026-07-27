use crate::control::proto::{ConfigSnapshot, TokenCacheEntry};
use crate::control::revision::ControlRevisionStore;
#[cfg(test)]
use crate::control::revision::EphemeralControlRevisionStore;
use crate::control::token::cache::TokenCacheProvider;
use anyhow::Result;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::sync::{mpsc, watch};
use tracing::info;
use tunnel_store::rules::RoutingData;
use tunnel_store::{AuthStore, RuleStore, TokenListEntry};

/// Central coordinator: owns the stores, maintains the watch channel.
pub struct ControlService {
    auth_store: Arc<dyn AuthStore>,
    rule_store: Arc<dyn RuleStore>,
    pub(crate) token_cache: Arc<dyn TokenCacheProvider>,
    current_snapshot: arc_swap::ArcSwap<ConfigSnapshot>,
    /// Monotonically increasing; incremented on every mutation.
    resource_version: std::sync::atomic::AtomicU64,
    revision_epoch: String,
    revision_store: Arc<dyn ControlRevisionStore>,
    watch_tx: watch::Sender<u64>,
    /// Kept alive so the channel never closes while ControlService lives.
    _watch_rx: watch::Receiver<u64>,
    /// Signal channel: any mutation sends a () here; the debounce task does the actual rebuild.
    publish_tx: mpsc::Sender<()>,
}

impl ControlService {
    /// Create and initialise the service. Loads initial snapshot from stores.
    #[cfg(test)]
    pub async fn new(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        token_cache: Arc<dyn TokenCacheProvider>,
    ) -> Result<Arc<Self>> {
        Self::new_with_revision_store(
            auth_store,
            rule_store,
            token_cache,
            Arc::new(EphemeralControlRevisionStore::new()),
        )
        .await
    }

    pub async fn new_with_revision_store(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        token_cache: Arc<dyn TokenCacheProvider>,
        revision_store: Arc<dyn ControlRevisionStore>,
    ) -> Result<Arc<Self>> {
        let persisted_revision = revision_store.current().await?;
        let mut initial =
            Self::build_snapshot(&*rule_store, &*token_cache, persisted_revision.sequence).await?;
        let content_hash = tunnel_lib::ctld_proto::snapshot_content_hash(&initial)?;
        let revision = revision_store.commit_snapshot_hash(&content_hash).await?;
        initial.resource_version = revision.sequence;
        let (watch_tx, _watch_rx) = watch::channel(initial.resource_version);
        // Channel capacity 1: multiple senders collapse into a single pending signal.
        let (publish_tx, publish_rx) = mpsc::channel::<()>(1);
        let svc = Arc::new(Self {
            auth_store,
            rule_store,
            token_cache,
            current_snapshot: arc_swap::ArcSwap::from_pointee(initial.clone()),
            resource_version: std::sync::atomic::AtomicU64::new(revision.sequence),
            revision_epoch: revision.epoch,
            revision_store,
            watch_tx,
            _watch_rx,
            publish_tx,
        });
        info!(
            epoch = %svc.revision_epoch,
            resource_version = initial.resource_version,
            "ControlService initialised"
        );
        // Spawn the debounce task. Uses a Weak reference so the task exits cleanly
        // when the last strong Arc<ControlService> is dropped.
        let weak = Arc::downgrade(&svc);
        tokio::spawn(crate::control::reactor::debounce_publish_task(
            weak.clone(),
            publish_rx,
        ));
        tokio::spawn(crate::control::reactor::db_poll_task(weak));
        Ok(svc)
    }

    pub async fn load_token_cache(&self) -> Result<Vec<TokenCacheEntry>> {
        self.token_cache.load_token_cache().await
    }

    /// Subscribe to snapshot-version changes.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.watch_tx.subscribe()
    }

    pub fn snapshot(&self) -> Arc<ConfigSnapshot> {
        self.current_snapshot.load_full()
    }

    pub fn revision_for_snapshot(
        &self,
        snapshot: &ConfigSnapshot,
    ) -> tunnel_lib::ctld_proto::ControlRevision {
        tunnel_lib::ctld_proto::ControlRevision {
            epoch: self.revision_epoch.clone(),
            sequence: snapshot.resource_version,
        }
    }

    // ── Snapshot builder ────────────────────────────────────────────────────

    async fn build_snapshot(
        rule_store: &dyn RuleStore,
        token_cache_provider: &dyn TokenCacheProvider,
        version: u64,
    ) -> Result<ConfigSnapshot> {
        let routing = rule_store.load_routing().await?;
        let token_cache = token_cache_provider.load_token_cache().await?;
        let (ingress_listeners, client_groups, egress_upstreams, egress_vhost_rules) =
            crate::control::proto::routing_data_to_proto(&routing);
        Ok(ConfigSnapshot {
            resource_version: version,
            ingress_listeners,
            client_groups,
            egress_upstreams,
            egress_vhost_rules,
            token_cache,
        })
    }

    /// Rebuilds the snapshot from DB and signals all watchers.
    /// Called only from the debounce task — not directly from mutation methods.
    pub(crate) async fn do_publish(&self) -> Result<()> {
        let candidate_version = self.resource_version.load(Ordering::Acquire) + 1;
        let mut snapshot =
            Self::build_snapshot(&*self.rule_store, &*self.token_cache, candidate_version).await?;
        let content_hash = tunnel_lib::ctld_proto::snapshot_content_hash(&snapshot)?;
        let revision = self
            .revision_store
            .commit_snapshot_hash(&content_hash)
            .await?;
        if revision.epoch != self.revision_epoch {
            anyhow::bail!("control revision epoch changed while service was running");
        }
        let next = revision.sequence;
        if next == self.resource_version.load(Ordering::Acquire) {
            return Ok(());
        }
        snapshot.resource_version = next;
        self.current_snapshot.store(Arc::new(snapshot));
        self.resource_version.store(next, Ordering::Release);
        info!(
            resource_version = next,
            "signalling config snapshot change to watchers"
        );
        let _ = self.watch_tx.send(next);
        Ok(())
    }

    /// Signal that a mutation has occurred. The debounce task will coalesce
    /// rapid signals and perform one rebuild per debounce window.
    pub(crate) fn publish(&self) {
        // try_send: if the channel already has a pending signal, this is a no-op —
        // which is exactly what we want (debounce collapses multiple signals).
        let _ = self.publish_tx.try_send(());
    }

    // ── Token lifecycle ──────────────────────────────────────────────────────

    pub async fn create_client(&self, name: &str) -> Result<String> {
        let token = self.auth_store.create_client(name).await?;
        info!(name = %name, "client created");
        self.publish();
        Ok(token)
    }

    pub async fn revoke_token(&self, name: &str) -> Result<()> {
        self.auth_store.revoke_token(name).await?;
        info!(name = %name, "token revoked");
        self.publish();
        Ok(())
    }

    pub async fn rotate_token(&self, name: &str) -> Result<String> {
        let token = self.auth_store.rotate_token(name).await?;
        info!(name = %name, "token rotated");
        self.publish();
        Ok(token)
    }

    pub async fn list_tokens(&self) -> Result<Vec<TokenListEntry>> {
        self.auth_store.list_tokens().await
    }

    // ── Routing CRUD ─────────────────────────────────────────────────────────

    #[allow(dead_code)]
    pub async fn save_routing(&self, data: &RoutingData) -> Result<()> {
        self.rule_store.save_routing(data).await?;
        info!("routing saved");
        self.publish();
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn load_routing(&self) -> Result<RoutingData> {
        self.rule_store.load_routing().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::revision::SqliteControlRevisionStore;
    use crate::control::token::cache::SqliteTokenCacheProvider;
    use tunnel_store::sqlite::{open_sqlite_pool, SqliteAuthStore};
    use tunnel_store::sqlite_rules::SqliteRuleStore;
    use tunnel_store::TokenStatus;

    #[tokio::test]
    async fn coalesced_change_signal_exposes_latest_full_snapshot() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let auth_store = Arc::new(SqliteAuthStore::from_pool(pool.clone()));
        auth_store.migrate().await.unwrap();
        let rule_store = Arc::new(SqliteRuleStore::new(pool.clone()));
        rule_store.migrate().await.unwrap();
        let token_cache = Arc::new(SqliteTokenCacheProvider::new(pool));

        let svc = ControlService::new(auth_store.clone(), rule_store, token_cache)
            .await
            .unwrap();
        let mut changes = svc.subscribe();

        auth_store.create_client("client-a").await.unwrap();
        svc.do_publish().await.unwrap();
        auth_store.rotate_token("client-a").await.unwrap();
        svc.do_publish().await.unwrap();

        changes.changed().await.unwrap();
        assert_eq!(*changes.borrow_and_update(), 3);

        let snapshot = svc.snapshot();
        assert_eq!(snapshot.resource_version, 3);
        assert_eq!(snapshot.token_cache.len(), 2);
        assert_eq!(
            snapshot
                .token_cache
                .iter()
                .filter(|entry| entry.token_status == TokenStatus::Active)
                .count(),
            1
        );
        assert_eq!(
            snapshot
                .token_cache
                .iter()
                .filter(|entry| entry.token_status == TokenStatus::Revoked)
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn restart_advances_revision_when_db_changed_before_publish() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let auth_store = Arc::new(SqliteAuthStore::from_pool(pool.clone()));
        auth_store.migrate().await.unwrap();
        let rule_store = Arc::new(SqliteRuleStore::new(pool.clone()));
        rule_store.migrate().await.unwrap();

        let first = ControlService::new_with_revision_store(
            auth_store.clone(),
            rule_store.clone(),
            Arc::new(SqliteTokenCacheProvider::new(pool.clone())),
            Arc::new(
                SqliteControlRevisionStore::initialize(pool.clone())
                    .await
                    .unwrap(),
            ),
        )
        .await
        .unwrap();
        let first_revision = first.snapshot().resource_version;
        drop(first);

        auth_store
            .create_client("crash-window-client")
            .await
            .unwrap();

        let restarted = ControlService::new_with_revision_store(
            auth_store,
            rule_store,
            Arc::new(SqliteTokenCacheProvider::new(pool.clone())),
            Arc::new(SqliteControlRevisionStore::initialize(pool).await.unwrap()),
        )
        .await
        .unwrap();

        assert_eq!(restarted.snapshot().resource_version, first_revision + 1);
    }
}
