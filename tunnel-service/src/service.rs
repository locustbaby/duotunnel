use crate::proto::{ConfigSnapshot, WatchEvent};
use crate::token::cache::TokenCacheProvider;
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
    token_cache: Arc<dyn TokenCacheProvider>,
    /// Monotonically increasing; incremented on every mutation.
    resource_version: std::sync::atomic::AtomicU64,
    watch_tx: watch::Sender<Arc<WatchEvent>>,
    /// Kept alive so the channel never closes while ControlService lives.
    _watch_rx: watch::Receiver<Arc<WatchEvent>>,
    /// Signal channel: any mutation sends a () here; the debounce task does the actual rebuild.
    publish_tx: mpsc::Sender<()>,
}

impl ControlService {
    /// Create and initialise the service. Loads initial snapshot from stores.
    pub async fn new(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        token_cache: Arc<dyn TokenCacheProvider>,
    ) -> Result<Arc<Self>> {
        let initial = Self::build_snapshot(&*rule_store, &*token_cache, 1).await?;
        let event = Arc::new(WatchEvent::Snapshot(initial));
        let (watch_tx, _watch_rx) = watch::channel(event);
        // Channel capacity 1: multiple senders collapse into a single pending signal.
        let (publish_tx, publish_rx) = mpsc::channel::<()>(1);
        let svc = Arc::new(Self {
            auth_store,
            rule_store,
            token_cache,
            resource_version: std::sync::atomic::AtomicU64::new(1),
            watch_tx,
            _watch_rx,
            publish_tx,
        });
        info!("ControlService initialised, resource_version=1");
        // Spawn the debounce task. Uses a Weak reference so the task exits cleanly
        // when the last strong Arc<ControlService> is dropped.
        let weak = Arc::downgrade(&svc);
        tokio::spawn(crate::reactor::debounce_publish_task(weak.clone(), publish_rx));
        Ok(svc)
    }

    /// Subscribe to the watch stream. New subscribers immediately get the
    /// current snapshot via `borrow()`, then block on `changed()` for updates.
    pub fn subscribe(&self) -> watch::Receiver<Arc<WatchEvent>> {
        self.watch_tx.subscribe()
    }

    pub fn current_version(&self) -> u64 {
        self.resource_version.load(Ordering::Acquire)
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
            crate::proto::routing_data_to_proto(&routing);
        Ok(ConfigSnapshot {
            resource_version: version,
            ingress_listeners,
            client_groups,
            egress_upstreams,
            egress_vhost_rules,
            token_cache,
        })
    }

    /// Rebuilds snapshot from DB and broadcasts Patch event to all watchers.
    /// Called only from the debounce task — not directly from mutation methods.
    pub(crate) async fn do_publish(&self) -> Result<()> {
        // Read the candidate version WITHOUT incrementing yet.
        // Only commit the increment after a successful snapshot build so that
        // a failed DB query does not leave a gap in the version sequence.
        let next = self.resource_version.load(Ordering::Acquire) + 1;
        let snapshot = Self::build_snapshot(&*self.rule_store, &*self.token_cache, next).await?;
        // Build succeeded — now commit the version increment.
        self.resource_version.fetch_add(1, Ordering::AcqRel);
        info!(
            resource_version = next,
            "broadcasting config patch to watchers"
        );
        let _ = self.watch_tx.send(Arc::new(WatchEvent::Patch(snapshot)));
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

