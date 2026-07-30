use crate::control::proto::ConfigSnapshot;
#[cfg(test)]
use crate::control::revision::EphemeralControlRevisionStore;
use crate::control::revision::{ControlRevisionStore, SqliteControlRevisionStore};
use crate::control::token::cache::{SqliteTokenCacheProvider, TokenCacheProvider};
use crate::storage::db::sqlite::{
    begin_immediate, commit_immediate, insert_admin_idempotency_on, load_admin_idempotency_on,
    rollback_immediate, AdminIdempotencyInsert, SqliteAuthStore,
    ADMIN_IDEMPOTENCY_PLAINTEXT_RESPONSE_ENCODING, ADMIN_IDEMPOTENCY_REDACTED_RESPONSE_ENCODING,
};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::storage::rules::RoutingData;
use crate::storage::{AuthStore, RuleStore, TokenListEntry};
use tokio::sync::{mpsc, watch};
use tracing::info;

const ADMIN_IDEMPOTENCY_SCOPE: &str = "admin";
const ADMIN_MUTATION_FINGERPRINT_VERSION: u8 = 1;
const ADMIN_RESPONSE_CACHE_MAX_ENTRIES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdminErrorKind {
    InvalidRequest,
    NotFound,
    Conflict,
}

#[derive(Debug)]
pub(crate) struct AdminMutationError {
    kind: AdminErrorKind,
    message: String,
}

impl AdminMutationError {
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self {
            kind: AdminErrorKind::InvalidRequest,
            message: message.into(),
        }
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: AdminErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self {
            kind: AdminErrorKind::Conflict,
            message: message.into(),
        }
    }

    pub(crate) fn kind(&self) -> AdminErrorKind {
        self.kind
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for AdminMutationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for AdminMutationError {}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) enum AdminMutation {
    ApplyConfigOverride(duotunnel_lib::ctld_proto::ConfigOperation),
    ClearConfigOverride { resource: String, key: String },
    CreateClient(String),
    RotateToken(String),
    RevokeToken(String),
}

impl AdminMutation {
    pub(crate) fn canonical_fingerprint(&self) -> Result<String> {
        let canonical = serde_json::to_vec(&(ADMIN_MUTATION_FINGERPRINT_VERSION, self))?;
        Ok(hex::encode(Sha256::digest(canonical)))
    }

    fn operation_name(&self) -> &'static str {
        match self {
            Self::ApplyConfigOverride(_) => "apply_config_override",
            Self::ClearConfigOverride { .. } => "clear_config_override",
            Self::CreateClient(_) => "create_client",
            Self::RotateToken(_) => "rotate_token",
            Self::RevokeToken(_) => "revoke_token",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdminMutationResponse {
    pub(crate) status: u16,
    pub(crate) body: String,
}

struct AdminMutationCommit {
    response: AdminMutationResponse,
    snapshot: Option<(ConfigSnapshot, duotunnel_lib::ctld_proto::ControlRevision)>,
    publish: bool,
    cache_sensitive_response: bool,
}

#[derive(Clone)]
struct CachedAdminResponse {
    fingerprint: String,
    response: AdminMutationResponse,
}

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
    snapshot_pool: Option<sqlx::SqlitePool>,
    yaml_layer: arc_swap::ArcSwap<crate::control::layer::ConfigLayer>,
    config_mutation_lock: tokio::sync::Mutex<()>,
    watch_tx: watch::Sender<u64>,
    /// Kept alive so the channel never closes while ControlService lives.
    _watch_rx: watch::Receiver<u64>,
    /// Signal channel: any mutation sends a () here; the debounce task does the actual rebuild.
    publish_tx: mpsc::Sender<()>,
    admin_response_cache: tokio::sync::Mutex<HashMap<String, CachedAdminResponse>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DegradedSource {
    Yaml,
    Sqlite,
    Coordinator,
}

impl ControlService {
    pub(crate) async fn apply_yaml_layer(
        &self,
        source: &crate::control::layer::ConfigLayer,
    ) -> Result<bool> {
        let _guard = self.config_mutation_lock.lock().await;
        let Some(pool) = self.snapshot_pool.as_ref() else {
            anyhow::bail!("layered configuration requires SQLite");
        };
        let mut tx = pool.begin().await?;
        let overrides = crate::control::layer::load_sqlite_layer_on(&mut tx).await?;
        let effective = crate::control::layer::merge_layers(&source.routing, &overrides)?;
        let (snapshot, revision, changed) = Self::commit_effective_on(
            &mut tx,
            &effective,
            Some(("yaml_source_revision", source.source_revision.as_str())),
            Some(&self.revision_epoch),
        )
        .await?;
        if revision.epoch != self.revision_epoch {
            anyhow::bail!("control revision epoch changed while service was running");
        }
        tx.commit().await?;
        self.yaml_layer.store(Arc::new(source.clone()));
        self.install_snapshot(snapshot, revision);
        Ok(changed)
    }

    pub(crate) async fn apply_sqlite_layer(
        &self,
        source: &crate::control::layer::SqliteConfigLayer,
    ) -> Result<bool> {
        let _guard = self.config_mutation_lock.lock().await;
        let Some(pool) = self.snapshot_pool.as_ref() else {
            anyhow::bail!("layered configuration requires SQLite");
        };
        let mut tx = pool.begin().await?;
        let yaml_layer = self.yaml_layer.load_full();
        let effective =
            crate::control::layer::merge_layers(&yaml_layer.routing, &source.overrides)?;
        let (snapshot, revision, changed) = Self::commit_effective_on(
            &mut tx,
            &effective,
            Some(("sqlite_source_revision", source.source_revision.as_str())),
            Some(&self.revision_epoch),
        )
        .await?;
        if revision.epoch != self.revision_epoch {
            anyhow::bail!("control revision epoch changed while service was running");
        }
        tx.commit().await?;
        self.install_snapshot(snapshot, revision);
        Ok(changed)
    }

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

    #[cfg(test)]
    pub async fn new_with_revision_store(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        token_cache: Arc<dyn TokenCacheProvider>,
        revision_store: Arc<dyn ControlRevisionStore>,
    ) -> Result<Arc<Self>> {
        Self::new_with_snapshot_pool(auth_store, rule_store, token_cache, revision_store, None)
            .await
    }

    pub async fn new_with_sqlite_revision_store(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        token_cache: Arc<dyn TokenCacheProvider>,
        revision_store: Arc<dyn ControlRevisionStore>,
        snapshot_pool: sqlx::SqlitePool,
    ) -> Result<Arc<Self>> {
        Self::new_with_snapshot_pool(
            auth_store,
            rule_store,
            token_cache,
            revision_store,
            Some(snapshot_pool),
        )
        .await
    }

    async fn new_with_snapshot_pool(
        auth_store: Arc<dyn AuthStore>,
        rule_store: Arc<dyn RuleStore>,
        token_cache: Arc<dyn TokenCacheProvider>,
        revision_store: Arc<dyn ControlRevisionStore>,
        snapshot_pool: Option<sqlx::SqlitePool>,
    ) -> Result<Arc<Self>> {
        let persisted_revision = revision_store.current().await?;
        let (initial, revision) = Self::build_and_commit_snapshot(
            &*rule_store,
            &*token_cache,
            &*revision_store,
            snapshot_pool.as_ref(),
            persisted_revision.sequence,
        )
        .await?;
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
            snapshot_pool,
            yaml_layer: arc_swap::ArcSwap::from_pointee(crate::control::layer::ConfigLayer {
                source_revision: String::new(),
                routing: RoutingData::default(),
            }),
            config_mutation_lock: tokio::sync::Mutex::new(()),
            watch_tx,
            _watch_rx,
            publish_tx,
            admin_response_cache: tokio::sync::Mutex::new(HashMap::new()),
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
        Ok(svc)
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
    ) -> duotunnel_lib::ctld_proto::ControlRevision {
        duotunnel_lib::ctld_proto::ControlRevision {
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

    async fn build_and_commit_snapshot(
        rule_store: &dyn RuleStore,
        token_cache_provider: &dyn TokenCacheProvider,
        revision_store: &dyn ControlRevisionStore,
        snapshot_pool: Option<&sqlx::SqlitePool>,
        version: u64,
    ) -> Result<(ConfigSnapshot, duotunnel_lib::ctld_proto::ControlRevision)> {
        if let Some(pool) = snapshot_pool {
            let mut tx = pool.begin().await?;
            let routing =
                crate::storage::sqlite_rules::SqliteRuleStore::load_routing_on(&mut tx).await?;
            let token_cache = SqliteTokenCacheProvider::load_token_cache_on(&mut tx).await?;
            let (ingress_listeners, client_groups, egress_upstreams, egress_vhost_rules) =
                crate::control::proto::routing_data_to_proto(&routing);
            let mut snapshot = ConfigSnapshot {
                resource_version: version,
                ingress_listeners,
                client_groups,
                egress_upstreams,
                egress_vhost_rules,
                token_cache,
            };
            duotunnel_lib::ctld_proto::validate_config_snapshot(&snapshot)?;
            let content_hash = duotunnel_lib::ctld_proto::snapshot_content_hash(&snapshot)?;
            let revision =
                SqliteControlRevisionStore::commit_snapshot_hash_on(&mut tx, &content_hash).await?;
            snapshot.resource_version = revision.sequence;
            sqlx::query(
                "UPDATE config_state SET initialized = 1
                 WHERE singleton = 1",
            )
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok((snapshot, revision));
        }

        let mut snapshot = Self::build_snapshot(rule_store, token_cache_provider, version).await?;
        duotunnel_lib::ctld_proto::validate_config_snapshot(&snapshot)?;
        let content_hash = duotunnel_lib::ctld_proto::snapshot_content_hash(&snapshot)?;
        let revision = revision_store.commit_snapshot_hash(&content_hash).await?;
        snapshot.resource_version = revision.sequence;
        Ok((snapshot, revision))
    }

    async fn commit_effective_on(
        conn: &mut sqlx::SqliteConnection,
        effective: &RoutingData,
        source_revision: Option<(&str, &str)>,
        expected_epoch: Option<&str>,
    ) -> Result<(
        ConfigSnapshot,
        duotunnel_lib::ctld_proto::ControlRevision,
        bool,
    )> {
        let current = crate::storage::sqlite_rules::SqliteRuleStore::load_routing_on(conn).await?;
        let changed = current != *effective;
        if changed {
            crate::storage::sqlite_rules::SqliteRuleStore::save_routing_on(conn, effective).await?;
        }
        if let Some((column, revision)) = source_revision {
            let query = match column {
                "yaml_source_revision" => {
                    "UPDATE config_state SET yaml_source_revision = ?1, initialized = 1,
                     yaml_degraded = 0,
                     degraded = CASE WHEN sqlite_degraded != 0 OR coordinator_degraded != 0
                                     THEN 1 ELSE 0 END
                     WHERE singleton = 1"
                }
                "sqlite_source_revision" => {
                    "UPDATE config_state SET sqlite_source_revision = ?1, initialized = 1,
                     sqlite_degraded = 0,
                     degraded = CASE WHEN yaml_degraded != 0 OR coordinator_degraded != 0
                                     THEN 1 ELSE 0 END
                     WHERE singleton = 1"
                }
                _ => unreachable!("source revision column is fixed by the coordinator"),
            };
            sqlx::query(query)
                .bind(revision)
                .execute(&mut *conn)
                .await?;
        } else {
            sqlx::query(
                "UPDATE config_state SET initialized = 1
                 WHERE singleton = 1",
            )
            .execute(&mut *conn)
            .await?;
        }
        let token_cache = SqliteTokenCacheProvider::load_token_cache_on(&mut *conn).await?;
        let (ingress_listeners, client_groups, egress_upstreams, egress_vhost_rules) =
            crate::control::proto::routing_data_to_proto(effective);
        let mut snapshot = ConfigSnapshot {
            resource_version: 0,
            ingress_listeners,
            client_groups,
            egress_upstreams,
            egress_vhost_rules,
            token_cache,
        };
        duotunnel_lib::ctld_proto::validate_config_snapshot(&snapshot)?;
        let content_hash = duotunnel_lib::ctld_proto::snapshot_content_hash(&snapshot)?;
        let revision = match expected_epoch {
            Some(epoch) => {
                SqliteControlRevisionStore::commit_snapshot_hash_on_for_epoch(
                    conn,
                    &content_hash,
                    epoch,
                )
                .await?
            }
            None => {
                SqliteControlRevisionStore::commit_snapshot_hash_on(conn, &content_hash).await?
            }
        };
        snapshot.resource_version = revision.sequence;
        Ok((snapshot, revision, changed))
    }

    fn install_snapshot(
        &self,
        snapshot: ConfigSnapshot,
        revision: duotunnel_lib::ctld_proto::ControlRevision,
    ) {
        let next = revision.sequence;
        if next == self.resource_version.load(Ordering::Acquire) {
            return;
        }
        self.current_snapshot.store(Arc::new(snapshot));
        self.resource_version.store(next, Ordering::Release);
        info!(
            resource_version = next,
            "signalling config snapshot change to watchers"
        );
        let _ = self.watch_tx.send(next);
    }

    /// Rebuilds the snapshot from DB and signals all watchers.
    /// Called only from the debounce task — not directly from mutation methods.
    pub(crate) async fn do_publish(&self) -> Result<()> {
        let _guard = self.config_mutation_lock.lock().await;
        let candidate_version = self.resource_version.load(Ordering::Acquire) + 1;
        let (snapshot, revision) = Self::build_and_commit_snapshot(
            &*self.rule_store,
            &*self.token_cache,
            &*self.revision_store,
            self.snapshot_pool.as_ref(),
            candidate_version,
        )
        .await?;
        if revision.epoch != self.revision_epoch {
            anyhow::bail!("control revision epoch changed while service was running");
        }
        let next = revision.sequence;
        if next == self.resource_version.load(Ordering::Acquire) {
            return Ok(());
        }
        self.install_snapshot(snapshot, revision);
        Ok(())
    }

    /// Signal that a mutation has occurred. The debounce task will coalesce
    /// rapid signals and perform one rebuild per debounce window.
    pub(crate) fn publish(&self) {
        // try_send: if the channel already has a pending signal, this is a no-op —
        // which is exactly what we want (debounce collapses multiple signals).
        let _ = self.publish_tx.try_send(());
    }

    pub(crate) async fn set_source_degraded(
        &self,
        source: DegradedSource,
        degraded: bool,
    ) -> Result<()> {
        let _guard = self.config_mutation_lock.lock().await;
        let Some(pool) = self.snapshot_pool.as_ref() else {
            return Ok(());
        };
        let column = match source {
            DegradedSource::Yaml => "yaml_degraded",
            DegradedSource::Sqlite => "sqlite_degraded",
            DegradedSource::Coordinator => "coordinator_degraded",
        };
        let mut tx = pool.begin().await?;
        sqlx::query(&format!(
            "UPDATE config_state SET {column} = ?1 WHERE singleton = 1"
        ))
        .bind(degraded)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE config_state SET degraded = CASE
                WHEN yaml_degraded != 0 OR sqlite_degraded != 0 OR coordinator_degraded != 0
                THEN 1 ELSE 0 END
             WHERE singleton = 1",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn active_token_count_on(conn: &mut sqlx::SqliteConnection, name: &str) -> Result<i64> {
        Ok(sqlx::query_scalar(
            "SELECT COUNT(*) FROM client_tokens
             WHERE client_id = (SELECT id FROM clients WHERE name = ?)
             AND status = 'active'",
        )
        .bind(name)
        .fetch_one(&mut *conn)
        .await?)
    }

    pub(crate) async fn execute_admin_mutation(
        &self,
        request_id: &str,
        fingerprint: &str,
        mutation: AdminMutation,
    ) -> Result<AdminMutationResponse> {
        let _guard = self.config_mutation_lock.lock().await;
        let Some(pool) = self.snapshot_pool.as_ref() else {
            anyhow::bail!("durable admin idempotency requires SQLite");
        };
        let operation = mutation.operation_name();
        let mut conn = begin_immediate(pool).await?;
        let outcome = async {
            if let Some(record) =
                load_admin_idempotency_on(&mut conn, ADMIN_IDEMPOTENCY_SCOPE, request_id).await?
            {
                if record.fingerprint != fingerprint {
                    return Ok(AdminMutationCommit {
                        response: AdminMutationResponse {
                            status: 409,
                            body: "Idempotency-Key was already used for a different request".into(),
                        },
                        snapshot: None,
                        publish: false,
                        cache_sensitive_response: false,
                    });
                }
                match record.response_encoding.as_str() {
                    ADMIN_IDEMPOTENCY_PLAINTEXT_RESPONSE_ENCODING => {
                        if matches!(record.operation.as_str(), "create_client" | "rotate_token") {
                            return Ok(AdminMutationCommit {
                                response: AdminMutationResponse {
                                    status: 410,
                                    body: "request was committed but its bearer-token response is no longer available; use a new Idempotency-Key".into(),
                                },
                                snapshot: None,
                                publish: false,
                                cache_sensitive_response: false,
                            });
                        }
                        return Ok(AdminMutationCommit {
                            response: AdminMutationResponse {
                                status: record.status_code,
                                body: record.response_body,
                            },
                            snapshot: None,
                            publish: false,
                            cache_sensitive_response: false,
                        });
                    }
                    ADMIN_IDEMPOTENCY_REDACTED_RESPONSE_ENCODING => {
                        if let Some(cached) = self
                            .admin_response_cache
                            .lock()
                            .await
                            .get(request_id)
                            .filter(|cached| cached.fingerprint == fingerprint)
                            .cloned()
                        {
                            return Ok(AdminMutationCommit {
                                response: cached.response,
                                snapshot: None,
                                publish: false,
                                cache_sensitive_response: false,
                            });
                        }
                        return Ok(AdminMutationCommit {
                            response: AdminMutationResponse {
                                status: 410,
                                body: "request was committed but its bearer-token response is no longer available; use a new Idempotency-Key".into(),
                            },
                            snapshot: None,
                            publish: false,
                            cache_sensitive_response: false,
                        });
                    }
                    encoding => anyhow::bail!(
                        "unsupported persisted admin response encoding: {encoding}"
                    ),
                }
            }

            let cache_sensitive_response = matches!(
                mutation,
                AdminMutation::CreateClient(_) | AdminMutation::RotateToken(_)
            );
            let (body, snapshot, publish) = match mutation {
                AdminMutation::ApplyConfigOverride(operation) => {
                    let (snapshot, revision) =
                        self.apply_config_override_on(&mut conn, &operation).await?;
                    ("ok".to_string(), Some((snapshot, revision)), false)
                }
                AdminMutation::ClearConfigOverride { resource, key } => {
                    let (snapshot, revision) = self
                        .clear_config_override_on(&mut conn, &resource, &key)
                        .await?;
                    ("ok".to_string(), Some((snapshot, revision)), false)
                }
                AdminMutation::CreateClient(name) => {
                    if Self::active_token_count_on(&mut conn, &name).await? > 0 {
                        return Err(AdminMutationError::conflict(format!(
                            "client '{}' already has an active token; use 'rotate' to replace it",
                            name
                        ))
                        .into());
                    }
                    (
                        SqliteAuthStore::create_client_on(&mut conn, &name).await?,
                        None,
                        true,
                    )
                }
                AdminMutation::RotateToken(name) => {
                    if Self::active_token_count_on(&mut conn, &name).await? == 0 {
                        return Err(AdminMutationError::not_found(format!(
                            "no active token found for client '{}'",
                            name
                        ))
                        .into());
                    }
                    (
                        SqliteAuthStore::rotate_token_on(&mut conn, &name).await?,
                        None,
                        true,
                    )
                }
                AdminMutation::RevokeToken(name) => {
                    if Self::active_token_count_on(&mut conn, &name).await? == 0 {
                        return Err(AdminMutationError::not_found(format!(
                            "no active token found for client '{}'",
                            name
                        ))
                        .into());
                    }
                    SqliteAuthStore::revoke_token_on(&mut conn, &name).await?;
                    ("ok".to_string(), None, true)
                }
            };
            insert_admin_idempotency_on(
                &mut conn,
                &AdminIdempotencyInsert {
                    scope: ADMIN_IDEMPOTENCY_SCOPE,
                    request_key: request_id,
                    fingerprint,
                    operation,
                    status_code: 200,
                    response_body: if cache_sensitive_response { "" } else { &body },
                    response_encoding: if cache_sensitive_response {
                        ADMIN_IDEMPOTENCY_REDACTED_RESPONSE_ENCODING
                    } else {
                        ADMIN_IDEMPOTENCY_PLAINTEXT_RESPONSE_ENCODING
                    },
                },
            )
            .await?;
            Ok(AdminMutationCommit {
                response: AdminMutationResponse { status: 200, body },
                snapshot,
                publish,
                cache_sensitive_response,
            })
        }
        .await;

        match outcome {
            Ok(commit) => {
                if let Err(error) = commit_immediate(&mut conn).await {
                    let _ = rollback_immediate(&mut conn).await;
                    return Err(error);
                }
                if let Some((snapshot, revision)) = commit.snapshot {
                    self.install_snapshot(snapshot, revision);
                }
                if commit.publish {
                    self.publish();
                }
                if commit.cache_sensitive_response {
                    let mut cache = self.admin_response_cache.lock().await;
                    cache.insert(
                        request_id.to_owned(),
                        CachedAdminResponse {
                            fingerprint: fingerprint.to_owned(),
                            response: commit.response.clone(),
                        },
                    );
                    if cache.len() > ADMIN_RESPONSE_CACHE_MAX_ENTRIES {
                        if let Some(oldest_key) = cache.keys().next().cloned() {
                            cache.remove(&oldest_key);
                        }
                    }
                }
                Ok(commit.response)
            }
            Err(error) => {
                let _ = rollback_immediate(&mut conn).await;
                Err(error)
            }
        }
    }

    pub async fn list_tokens(&self) -> Result<Vec<TokenListEntry>> {
        self.auth_store.list_tokens().await
    }

    async fn apply_config_override_on(
        &self,
        conn: &mut sqlx::SqliteConnection,
        operation: &duotunnel_lib::ctld_proto::ConfigOperation,
    ) -> Result<(ConfigSnapshot, duotunnel_lib::ctld_proto::ControlRevision)> {
        let yaml_layer = self.yaml_layer.load_full();
        let mut overrides = crate::control::layer::load_sqlite_layer_on(conn).await?;
        crate::control::layer::apply_override_operation(&mut overrides, operation)
            .map_err(|error| AdminMutationError::invalid(error.to_string()))?;
        let source_revision = hex::encode(Sha256::digest(serde_json::to_vec(&overrides)?));
        crate::control::layer::save_sqlite_layer_on(conn, &overrides, &source_revision).await?;
        let effective = crate::control::layer::merge_layers(&yaml_layer.routing, &overrides)
            .map_err(|error| AdminMutationError::invalid(error.to_string()))?;
        let (snapshot, revision, _) = Self::commit_effective_on(
            conn,
            &effective,
            Some(("sqlite_source_revision", source_revision.as_str())),
            Some(&self.revision_epoch),
        )
        .await?;
        if revision.epoch != self.revision_epoch {
            anyhow::bail!("control revision epoch changed while service was running");
        }
        Ok((snapshot, revision))
    }

    async fn clear_config_override_on(
        &self,
        conn: &mut sqlx::SqliteConnection,
        resource: &str,
        key: &str,
    ) -> Result<(ConfigSnapshot, duotunnel_lib::ctld_proto::ControlRevision)> {
        let yaml_layer = self.yaml_layer.load_full();
        let mut overrides = crate::control::layer::load_sqlite_layer_on(conn).await?;
        crate::control::layer::clear_override(&mut overrides, resource, key)
            .map_err(|error| AdminMutationError::invalid(error.to_string()))?;
        let source_revision = hex::encode(Sha256::digest(serde_json::to_vec(&overrides)?));
        crate::control::layer::save_sqlite_layer_on(conn, &overrides, &source_revision).await?;
        let effective = crate::control::layer::merge_layers(&yaml_layer.routing, &overrides)
            .map_err(|error| AdminMutationError::invalid(error.to_string()))?;
        let (snapshot, revision, _) = Self::commit_effective_on(
            conn,
            &effective,
            Some(("sqlite_source_revision", source_revision.as_str())),
            Some(&self.revision_epoch),
        )
        .await?;
        if revision.epoch != self.revision_epoch {
            anyhow::bail!("control revision epoch changed while service was running");
        }
        Ok((snapshot, revision))
    }

    // ── Routing CRUD ─────────────────────────────────────────────────────────

    #[cfg(test)]
    pub(crate) async fn save_routing(&self, data: &RoutingData) -> Result<()> {
        let _guard = self.config_mutation_lock.lock().await;
        if let Some(pool) = self.snapshot_pool.as_ref() {
            let mut tx = pool.begin().await?;
            let (snapshot, revision, _) =
                Self::commit_effective_on(&mut tx, data, None, Some(&self.revision_epoch)).await?;
            if revision.epoch != self.revision_epoch {
                anyhow::bail!("control revision epoch changed while service was running");
            }
            tx.commit().await?;
            self.install_snapshot(snapshot, revision);
            info!("routing saved");
            return Ok(());
        }
        self.rule_store.save_routing(data).await?;
        info!("routing saved");
        self.publish();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::revision::SqliteControlRevisionStore;
    use crate::control::token::cache::SqliteTokenCacheProvider;
    use crate::storage::sqlite::{open_sqlite_pool, SqliteAuthStore};
    use crate::storage::sqlite_rules::SqliteRuleStore;
    use duotunnel_lib::TokenStatus;

    async fn setup_sqlite_service(pool: sqlx::SqlitePool) -> Arc<ControlService> {
        let auth_store = Arc::new(SqliteAuthStore::from_pool(pool.clone()));
        auth_store.migrate().await.unwrap();
        let rule_store = Arc::new(SqliteRuleStore::new(pool.clone()));
        rule_store.migrate().await.unwrap();
        crate::control::layer::initialize_sqlite_layer(&pool, rule_store.as_ref(), false)
            .await
            .unwrap();
        let revision_store = Arc::new(
            SqliteControlRevisionStore::initialize(pool.clone())
                .await
                .unwrap(),
        );
        ControlService::new_with_sqlite_revision_store(
            auth_store,
            rule_store,
            Arc::new(SqliteTokenCacheProvider::new(pool.clone())),
            revision_store,
            pool,
        )
        .await
        .unwrap()
    }

    async fn create_client_for_test(pool: &sqlx::SqlitePool, name: &str) -> String {
        let mut tx = pool.begin().await.unwrap();
        let token = SqliteAuthStore::create_client_on(&mut tx, name)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        token
    }

    async fn rotate_token_for_test(pool: &sqlx::SqlitePool, name: &str) -> String {
        let mut tx = pool.begin().await.unwrap();
        let token = SqliteAuthStore::rotate_token_on(&mut tx, name)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        token
    }

    #[tokio::test]
    async fn coalesced_change_signal_exposes_latest_full_snapshot() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let auth_store = Arc::new(SqliteAuthStore::from_pool(pool.clone()));
        auth_store.migrate().await.unwrap();
        let rule_store = Arc::new(SqliteRuleStore::new(pool.clone()));
        rule_store.migrate().await.unwrap();
        let token_cache = Arc::new(SqliteTokenCacheProvider::new(pool.clone()));

        let svc = ControlService::new(auth_store.clone(), rule_store, token_cache)
            .await
            .unwrap();
        let mut changes = svc.subscribe();

        create_client_for_test(&pool, "client-a").await;
        svc.do_publish().await.unwrap();
        rotate_token_for_test(&pool, "client-a").await;
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

        create_client_for_test(&pool, "crash-window-client").await;

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

    #[tokio::test]
    async fn durable_admin_mutations_replay_and_rotate_once() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let auth_store = SqliteAuthStore::from_pool(pool.clone());
        auth_store.migrate().await.unwrap();
        create_client_for_test(&pool, "client-a").await;

        let first_service = setup_sqlite_service(pool.clone()).await;
        let rotate = AdminMutation::RotateToken("client-a".into());
        let rotate_fingerprint = rotate.canonical_fingerprint().unwrap();
        let first = first_service
            .execute_admin_mutation("rotate-1", &rotate_fingerprint, rotate)
            .await
            .unwrap();
        assert_eq!(first.status, 200);
        assert_eq!(
            first_service
                .execute_admin_mutation(
                    "rotate-1",
                    &rotate_fingerprint,
                    AdminMutation::RotateToken("client-a".into()),
                )
                .await
                .unwrap(),
            first
        );
        drop(first_service);

        let restarted_service = setup_sqlite_service(pool.clone()).await;
        let replay = restarted_service
            .execute_admin_mutation(
                "rotate-1",
                &rotate_fingerprint,
                AdminMutation::RotateToken("client-a".into()),
            )
            .await
            .unwrap();
        assert_eq!(replay.status, 410);
        assert!(replay.body.contains("response is no longer available"));
        let (stored_body, stored_encoding): (String, String) = sqlx::query_as(
            "SELECT response_body, response_encoding FROM admin_idempotency
             WHERE scope = 'admin' AND request_key = 'rotate-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(stored_body.is_empty());
        assert_eq!(stored_encoding, "redacted-v1");

        let active_tokens: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM client_tokens WHERE client_id =
             (SELECT id FROM clients WHERE name = 'client-a') AND status = 'active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let total_tokens: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM client_tokens WHERE client_id =
             (SELECT id FROM clients WHERE name = 'client-a')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_tokens, 1);
        assert_eq!(total_tokens, 2);

        let revoke = AdminMutation::RevokeToken("client-a".into());
        let revoke_fingerprint = revoke.canonical_fingerprint().unwrap();
        let revoked = restarted_service
            .execute_admin_mutation("revoke-1", &revoke_fingerprint, revoke)
            .await
            .unwrap();
        assert_eq!(revoked.status, 200);
        assert_eq!(
            restarted_service
                .execute_admin_mutation(
                    "revoke-1",
                    &revoke_fingerprint,
                    AdminMutation::RevokeToken("client-a".into()),
                )
                .await
                .unwrap(),
            revoked
        );

        let failed = restarted_service
            .execute_admin_mutation(
                "failed-1",
                &AdminMutation::RotateToken("missing".into())
                    .canonical_fingerprint()
                    .unwrap(),
                AdminMutation::RotateToken("missing".into()),
            )
            .await;
        let error = failed.unwrap_err();
        let typed = error
            .downcast_ref::<AdminMutationError>()
            .expect("missing typed admin mutation error");
        assert_eq!(typed.kind(), AdminErrorKind::NotFound);
        let create = AdminMutation::CreateClient("client-b".into());
        let create_fingerprint = create.canonical_fingerprint().unwrap();
        let created = restarted_service
            .execute_admin_mutation("failed-1", &create_fingerprint, create)
            .await
            .unwrap();
        assert_eq!(created.status, 200);

        let duplicate = AdminMutation::CreateClient("client-b".into());
        let duplicate_error = restarted_service
            .execute_admin_mutation(
                "duplicate-1",
                &duplicate.canonical_fingerprint().unwrap(),
                duplicate,
            )
            .await
            .unwrap_err();
        let typed = duplicate_error
            .downcast_ref::<AdminMutationError>()
            .expect("missing typed conflict error");
        assert_eq!(typed.kind(), AdminErrorKind::Conflict);
    }

    #[tokio::test]
    async fn degraded_sources_are_tracked_independently() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let svc = setup_sqlite_service(pool.clone()).await;

        svc.set_source_degraded(DegradedSource::Yaml, true)
            .await
            .unwrap();
        svc.set_source_degraded(DegradedSource::Sqlite, false)
            .await
            .unwrap();
        let degraded: i64 =
            sqlx::query_scalar("SELECT degraded FROM config_state WHERE singleton = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(degraded, 1);

        svc.do_publish().await.unwrap();
        let degraded: i64 =
            sqlx::query_scalar("SELECT degraded FROM config_state WHERE singleton = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(degraded, 1);

        svc.set_source_degraded(DegradedSource::Yaml, false)
            .await
            .unwrap();
        let degraded: i64 =
            sqlx::query_scalar("SELECT degraded FROM config_state WHERE singleton = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(degraded, 0);
    }

    #[tokio::test]
    async fn durable_config_override_and_clear_replay() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let service = setup_sqlite_service(pool.clone()).await;
        let group_operation = duotunnel_lib::ctld_proto::ConfigOperation::UpsertClientGroup(
            duotunnel_lib::ctld_proto::ProtoClientGroup {
                group_id: "ci".into(),
                config_version: "1".into(),
                upstreams: vec![duotunnel_lib::ctld_proto::ProtoClientUpstream {
                    name: "default".into(),
                    lb_policy: "round_robin".into(),
                    servers: vec![duotunnel_lib::ctld_proto::ProtoUpstreamServer {
                        address: "127.0.0.1:8080".into(),
                        resolve: false,
                    }],
                }],
            },
        );
        let group_mutation = AdminMutation::ApplyConfigOverride(group_operation);
        let group_fingerprint = group_mutation.canonical_fingerprint().unwrap();
        service
            .execute_admin_mutation("group-1", &group_fingerprint, group_mutation)
            .await
            .unwrap();
        let operation = duotunnel_lib::ctld_proto::ConfigOperation::UpsertIngressListener(
            duotunnel_lib::ctld_proto::ProtoIngressListener {
                id: 1,
                port: 8080,
                mode: duotunnel_lib::ctld_proto::ProtoIngressListenerMode::Tcp {
                    group_id: "ci".into(),
                    proxy_name: "default".into(),
                },
            },
        );
        let mutation = AdminMutation::ApplyConfigOverride(operation.clone());
        let fingerprint = mutation.canonical_fingerprint().unwrap();
        let first = service
            .execute_admin_mutation("override-1", &fingerprint, mutation)
            .await
            .unwrap();
        let replay = service
            .execute_admin_mutation(
                "override-1",
                &fingerprint,
                AdminMutation::ApplyConfigOverride(operation),
            )
            .await
            .unwrap();
        assert_eq!(replay, first);
        assert_eq!(service.snapshot().ingress_listeners.len(), 1);

        let clear = AdminMutation::ClearConfigOverride {
            resource: "ingress_listener".into(),
            key: "8080".into(),
        };
        let clear_fingerprint = clear.canonical_fingerprint().unwrap();
        let cleared = service
            .execute_admin_mutation("clear-1", &clear_fingerprint, clear)
            .await
            .unwrap();
        assert_eq!(cleared.status, 200);
        assert_eq!(
            service
                .execute_admin_mutation(
                    "clear-1",
                    &clear_fingerprint,
                    AdminMutation::ClearConfigOverride {
                        resource: "ingress_listener".into(),
                        key: "8080".into(),
                    },
                )
                .await
                .unwrap(),
            cleared
        );
        assert!(service.snapshot().ingress_listeners.is_empty());

        let conflict = service
            .execute_admin_mutation(
                "override-1",
                &AdminMutation::CreateClient("different".into())
                    .canonical_fingerprint()
                    .unwrap(),
                AdminMutation::CreateClient("different".into()),
            )
            .await
            .unwrap();
        assert_eq!(conflict.status, 409);
    }
}
