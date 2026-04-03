use std::sync::Arc;
use std::sync::atomic::Ordering;
use anyhow::Result;
use tokio::sync::watch;
use tracing::{info, warn};
use tunnel_store::rules::RoutingData;
use tunnel_store::{AuthStore, RuleStore, TokenListEntry};
use crate::proto::{ConfigSnapshot, WatchEvent};
use crate::token::cache::load_token_cache;

/// Central coordinator: owns the stores, maintains the watch channel.
pub struct ControlService {
    auth_store: Arc<dyn AuthStore>,
    rule_store: Arc<dyn RuleStore>,
    /// SQLite pool for token cache queries (hash values not exposed by AuthStore trait).
    pool: sqlx::sqlite::SqlitePool,
    /// Monotonically increasing; incremented on every mutation.
    resource_version: std::sync::atomic::AtomicU64,
    watch_tx: watch::Sender<Arc<WatchEvent>>,
    /// Kept alive so the channel never closes while ControlService lives.
    _watch_rx: watch::Receiver<Arc<WatchEvent>>,
}

impl ControlService {
    /// Create and initialise the service. Loads initial snapshot from stores.
    pub async fn new(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        pool: sqlx::sqlite::SqlitePool,
    ) -> Result<Arc<Self>> {
        let initial = Self::build_snapshot(&*rule_store, &pool, 1).await?;
        let event = Arc::new(WatchEvent::Snapshot(initial));
        let (watch_tx, _watch_rx) = watch::channel(event);
        let svc = Arc::new(Self {
            auth_store,
            rule_store,
            pool,
            resource_version: std::sync::atomic::AtomicU64::new(1),
            watch_tx,
            _watch_rx,
        });
        info!("ControlService initialised, resource_version=1");
        Ok(svc)
    }

    /// Subscribe to the watch stream. New subscribers immediately get the
    /// current snapshot via `borrow()`, then block on `changed()` for updates.
    pub fn subscribe(&self) -> watch::Receiver<Arc<WatchEvent>> {
        self.watch_tx.subscribe()
    }

    pub fn current_version(&self) -> u64 {
        self.resource_version.load(Ordering::SeqCst)
    }

    // ── Snapshot builder ────────────────────────────────────────────────────

    async fn build_snapshot(
        rule_store: &dyn RuleStore,
        pool: &sqlx::sqlite::SqlitePool,
        version: u64,
    ) -> Result<ConfigSnapshot> {
        let routing = rule_store.load_routing().await?;
        let token_cache = load_token_cache(pool).await?;
        Ok(ConfigSnapshot {
            resource_version: version,
            ingress_listeners: routing.ingress_listeners,
            client_groups: routing.client_groups,
            egress_upstreams: routing.egress_upstreams,
            egress_vhost_rules: routing.egress_vhost_rules,
            token_cache,
        })
    }

    /// Rebuilds snapshot from DB and broadcasts Patch event to all watchers.
    async fn publish(&self) -> Result<()> {
        let version = self.resource_version.fetch_add(1, Ordering::SeqCst) + 1;
        let snapshot = Self::build_snapshot(&*self.rule_store, &self.pool, version).await?;
        info!(resource_version = version, "broadcasting config patch to watchers");
        let _ = self.watch_tx.send(Arc::new(WatchEvent::Patch(snapshot)));
        Ok(())
    }

    // ── Token lifecycle ──────────────────────────────────────────────────────

    pub async fn create_client(&self, name: &str) -> Result<String> {
        let token = self.auth_store.create_client(name).await?;
        info!(name = %name, "client created");
        if let Err(e) = self.publish().await {
            warn!(error = %e, "publish after create_client failed");
        }
        Ok(token)
    }

    pub async fn revoke_token(&self, name: &str) -> Result<()> {
        self.auth_store.revoke_token(name).await?;
        info!(name = %name, "token revoked");
        if let Err(e) = self.publish().await {
            warn!(error = %e, "publish after revoke_token failed");
        }
        Ok(())
    }

    pub async fn rotate_token(&self, name: &str) -> Result<String> {
        let token = self.auth_store.rotate_token(name).await?;
        info!(name = %name, "token rotated");
        if let Err(e) = self.publish().await {
            warn!(error = %e, "publish after rotate_token failed");
        }
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
        if let Err(e) = self.publish().await {
            warn!(error = %e, "publish after save_routing failed");
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn load_routing(&self) -> Result<RoutingData> {
        self.rule_store.load_routing().await
    }
}
