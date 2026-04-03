use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::{mpsc, watch};
use tracing::{info, warn};
use tunnel_store::rules::RoutingData;
use tunnel_store::{AuthStore, RuleStore, TokenListEntry};
use crate::proto::{ConfigSnapshot, WatchEvent};
use crate::token::cache::load_token_cache;

/// Debounce window: multiple mutations within this window are collapsed into
/// a single DB rebuild + broadcast.
const PUBLISH_DEBOUNCE_MS: u64 = 50;

/// How often the daemon polls SQLite for out-of-band changes (e.g. made by the
/// CLI tool running as a separate process against the same DB).
const POLL_INTERVAL_MS: u64 = 1500;

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
    /// Signal channel: any mutation sends a () here; the debounce task does the actual rebuild.
    publish_tx: mpsc::Sender<()>,
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
        // Channel capacity 1: multiple senders collapse into a single pending signal.
        let (publish_tx, publish_rx) = mpsc::channel::<()>(1);
        let svc = Arc::new(Self {
            auth_store,
            rule_store,
            pool,
            resource_version: std::sync::atomic::AtomicU64::new(1),
            watch_tx,
            _watch_rx,
            publish_tx,
        });
        info!("ControlService initialised, resource_version=1");
        // Spawn the debounce task. Uses a Weak reference so the task exits cleanly
        // when the last strong Arc<ControlService> is dropped.
        let weak = Arc::downgrade(&svc);
        tokio::spawn(debounce_publish_task(weak.clone(), publish_rx));
        // Spawn the DB polling task to detect out-of-band writes (CLI tool running
        // as a separate process against the same SQLite DB).
        tokio::spawn(db_poll_task(weak));
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
    /// Called only from the debounce task — not directly from mutation methods.
    async fn do_publish(&self) -> Result<()> {
        // Read the candidate version WITHOUT incrementing yet.
        // Only commit the increment after a successful snapshot build so that
        // a failed DB query does not leave a gap in the version sequence.
        let next = self.resource_version.load(Ordering::Acquire) + 1;
        let snapshot = Self::build_snapshot(&*self.rule_store, &self.pool, next).await?;
        // Build succeeded — now commit the version increment.
        self.resource_version.fetch_add(1, Ordering::AcqRel);
        info!(resource_version = next, "broadcasting config patch to watchers");
        let _ = self.watch_tx.send(Arc::new(WatchEvent::Patch(snapshot)));
        Ok(())
    }

    /// Signal that a mutation has occurred. The debounce task will coalesce
    /// rapid signals and perform one rebuild per debounce window.
    fn publish(&self) {
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

/// Background task: waits for publish signals, debounces within PUBLISH_DEBOUNCE_MS,
/// then performs one full snapshot rebuild and broadcast.
/// On persistent failures, applies exponential backoff (1s → 32s) before retrying.
async fn debounce_publish_task(
    svc: std::sync::Weak<ControlService>,
    mut rx: mpsc::Receiver<()>,
) {
    let mut consecutive_failures: u32 = 0;
    loop {
        // Block until at least one signal arrives.
        if rx.recv().await.is_none() {
            // Channel closed — ControlService dropped.
            break;
        }
        // Debounce: wait for the burst to settle, then drain any queued signals.
        tokio::time::sleep(Duration::from_millis(PUBLISH_DEBOUNCE_MS)).await;
        while rx.try_recv().is_ok() {}

        match svc.upgrade() {
            None => break, // ControlService dropped while we were sleeping.
            Some(svc) => {
                if let Err(e) = svc.do_publish().await {
                    consecutive_failures += 1;
                    // Backoff: 1s, 2s, 4s, 8s, 16s, 32s (capped).
                    let backoff = Duration::from_secs(1 << consecutive_failures.min(5));
                    warn!(
                        error = %e,
                        consecutive_failures,
                        backoff_secs = backoff.as_secs(),
                        "debounced publish failed, backing off"
                    );
                    tokio::time::sleep(backoff).await;
                } else {
                    consecutive_failures = 0;
                }
            }
        }
    }
}

/// Background task: polls SQLite every POLL_INTERVAL_MS to detect token or
/// routing changes written by an out-of-process CLI tool.  When a change is
/// detected the normal publish path is triggered so all watchers (servers) get
/// an updated Patch.
async fn db_poll_task(svc: std::sync::Weak<ControlService>) {
    // Fingerprint = sorted list of (hash_hex, token_status, client_status) tuples.
    // We compare this cheaply without needing a DB version column.
    let mut last_fingerprint: Option<String> = None;

    loop {
        tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;

        let Some(svc) = svc.upgrade() else { break };

        match crate::token::cache::load_token_cache(&svc.pool).await {
            Err(e) => {
                warn!(error = %e, "db_poll: failed to load token cache");
            }
            Ok(entries) => {
                // Build a cheap fingerprint: sorted concatenation of all token rows.
                let mut parts: Vec<String> = entries
                    .iter()
                    .map(|e| format!("{}:{}:{}", e.hash_hex, e.token_status, e.client_status))
                    .collect();
                parts.sort_unstable();
                let fingerprint = parts.join("|");

                let changed = last_fingerprint.as_deref() != Some(&fingerprint);
                last_fingerprint = Some(fingerprint);

                if changed {
                    tracing::debug!("db_poll: token change detected, triggering publish");
                    svc.publish();
                }
            }
        }
    }
}
