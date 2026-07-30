use crate::control::revision::SqliteControlRevisionStore;
use crate::storage::rules::{RoutingData, RuleStore};
use anyhow::Result;
use async_trait::async_trait;
use duotunnel_lib::ctld_proto::ConfigOperation;
use figment::{providers::Format, providers::Yaml, Figment};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigLayer {
    pub source_revision: String,
    pub routing: RoutingData,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SqliteOverrideLayer {
    pub routing: RoutingData,
    #[serde(default)]
    pub tombstones: OverrideTombstones,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqliteConfigLayer {
    pub source_revision: String,
    pub overrides: SqliteOverrideLayer,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OverrideTombstones {
    pub ingress_ports: Vec<u16>,
    pub client_groups: Vec<String>,
    pub egress_upstreams: Vec<String>,
    pub egress_vhost_rules: Vec<String>,
}

pub async fn initialize_sqlite_layer(
    pool: &sqlx::SqlitePool,
    rule_store: &dyn RuleStore,
    legacy_server_config_source: bool,
) -> Result<()> {
    SqliteControlRevisionStore::ensure_config_state_schema(pool).await?;
    ensure_config_layers_schema(pool).await?;
    let legacy_routing = if sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM config_layers WHERE source = 'sqlite_override'",
    )
    .fetch_one(pool)
    .await?
        == 0
    {
        Some(rule_store.load_routing().await?)
    } else {
        None
    };

    let mut tx = pool.begin().await?;
    if let Some(legacy) = legacy_routing {
        let mut layer = SqliteOverrideLayer {
            routing: legacy,
            tombstones: OverrideTombstones::default(),
        };
        normalize_and_validate_overrides(&mut layer)?;
        let source_revision = sqlite_layer_revision(&layer)?;
        save_sqlite_layer_on(&mut tx, &layer, &source_revision).await?;
        SqliteControlRevisionStore::record_migration_on(
            &mut tx,
            "legacy-routing-to-sqlite-override-v1",
        )
        .await?;
    } else {
        let (source_revision, mut layer) = load_sqlite_layer_record_on(&mut tx).await?;
        normalize_and_validate_overrides(&mut layer)?;
        if source_revision == "migration" {
            let source_revision = sqlite_layer_revision(&layer)?;
            save_sqlite_layer_on(&mut tx, &layer, &source_revision).await?;
        }
    }
    if legacy_server_config_source {
        SqliteControlRevisionStore::record_migration_on(
            &mut tx,
            "legacy-server-config-yaml-base-v1",
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn ensure_config_layers_schema(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config_layers (
            source TEXT PRIMARY KEY,
            payload TEXT NOT NULL,
            source_revision TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(pool)
    .await?;
    let columns = sqlx::query("PRAGMA table_info(config_layers)")
        .fetch_all(pool)
        .await?;
    for (name, definition) in [
        ("source", "TEXT NOT NULL DEFAULT ''"),
        ("payload", "TEXT NOT NULL DEFAULT '{}'"),
        ("source_revision", "TEXT NOT NULL DEFAULT ''"),
    ] {
        let exists = columns.iter().any(|row| {
            row.try_get::<String, _>("name")
                .is_ok_and(|column| column == name)
        });
        if !exists {
            sqlx::query(&format!(
                "ALTER TABLE config_layers ADD COLUMN {name} {definition}"
            ))
            .execute(pool)
            .await?;
        }
    }
    sqlx::query(
        "INSERT OR IGNORE INTO schema_migrations(migration)
         VALUES ('effective-config-layers-v1')",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_sqlite_layer_on(
    conn: &mut sqlx::SqliteConnection,
) -> Result<SqliteOverrideLayer> {
    Ok(load_sqlite_layer_record_on(conn).await?.1)
}

pub async fn load_sqlite_layer_record_on(
    conn: &mut sqlx::SqliteConnection,
) -> Result<(String, SqliteOverrideLayer)> {
    let row = sqlx::query(
        "SELECT source_revision, payload FROM config_layers
         WHERE source = 'sqlite_override'",
    )
    .fetch_one(&mut *conn)
    .await?;
    let source_revision: String = row.try_get("source_revision")?;
    let mut layer: SqliteOverrideLayer = serde_json::from_str(row.try_get("payload")?)?;
    normalize_and_validate_overrides(&mut layer)?;
    Ok((source_revision, layer))
}

fn sqlite_layer_revision(layer: &SqliteOverrideLayer) -> Result<String> {
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(layer)?)))
}

pub async fn save_sqlite_layer_on(
    conn: &mut sqlx::SqliteConnection,
    layer: &SqliteOverrideLayer,
    source_revision: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO config_layers(source, payload, source_revision)
         VALUES ('sqlite_override', ?1, ?2)
         ON CONFLICT(source) DO UPDATE SET payload = excluded.payload,
             source_revision = excluded.source_revision",
    )
    .bind(serde_json::to_string(layer)?)
    .bind(source_revision)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

pub fn apply_override_operation(
    overrides: &mut SqliteOverrideLayer,
    operation: &ConfigOperation,
) -> Result<()> {
    normalize_and_validate_overrides(overrides)?;
    match operation {
        ConfigOperation::UpsertIngressListener(item) => {
            remove_tombstone(&mut overrides.tombstones.ingress_ports, &item.port);
            replace_by(
                &mut overrides.routing.ingress_listeners,
                item.clone(),
                |value| value.port,
            );
        }
        ConfigOperation::DeleteIngressListener { port } => {
            overrides
                .routing
                .ingress_listeners
                .retain(|item| item.port != *port);
            insert_unique(&mut overrides.tombstones.ingress_ports, *port);
        }
        ConfigOperation::UpsertClientGroup(item) => {
            let mut item = item.clone();
            item.group_id =
                normalized_identifier(item.group_id.as_str(), "client group id")?.into();
            let key = item.group_id.as_str().to_owned();
            remove_tombstone(&mut overrides.tombstones.client_groups, &key);
            replace_by(&mut overrides.routing.client_groups, item, |value| {
                value.group_id.as_str().to_owned()
            });
        }
        ConfigOperation::DeleteClientGroup { group_id } => {
            let key = normalized_identifier(group_id, "client group id")?;
            overrides
                .routing
                .client_groups
                .retain(|item| item.group_id.as_str() != key);
            insert_unique(&mut overrides.tombstones.client_groups, key);
        }
        ConfigOperation::UpsertEgressUpstream(item) => {
            let mut item = item.clone();
            normalize_identifier(&mut item.name, "egress upstream name")?;
            let key = item.name.clone();
            remove_tombstone(&mut overrides.tombstones.egress_upstreams, &key);
            replace_by(&mut overrides.routing.egress_upstreams, item, |value| {
                value.name.clone()
            });
        }
        ConfigOperation::DeleteEgressUpstream { name } => {
            let key = normalized_identifier(name, "egress upstream name")?;
            overrides
                .routing
                .egress_upstreams
                .retain(|item| item.name != key);
            insert_unique(&mut overrides.tombstones.egress_upstreams, key);
        }
        ConfigOperation::UpsertEgressVhostRule(item) => {
            let mut item = item.clone();
            item.match_host = normalize_host(&item.match_host)?;
            remove_tombstone(
                &mut overrides.tombstones.egress_vhost_rules,
                &item.match_host,
            );
            replace_by(&mut overrides.routing.egress_vhost_rules, item, |value| {
                value.match_host.clone()
            });
        }
        ConfigOperation::DeleteEgressVhostRule { match_host } => {
            let key = normalize_host(match_host)?;
            overrides
                .routing
                .egress_vhost_rules
                .retain(|item| item.match_host != key);
            insert_unique(&mut overrides.tombstones.egress_vhost_rules, key);
        }
        ConfigOperation::UpsertToken(_) | ConfigOperation::DeleteToken { .. } => {
            anyhow::bail!("token operations must use the token admin API")
        }
    }
    normalize_and_validate_overrides(overrides)?;
    Ok(())
}

pub fn clear_override(
    overrides: &mut SqliteOverrideLayer,
    resource: &str,
    key: &str,
) -> Result<()> {
    match resource {
        "ingress_listener" => {
            let port: u16 = key
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid ingress port"))?;
            overrides
                .routing
                .ingress_listeners
                .retain(|item| item.port != port);
            remove_tombstone(&mut overrides.tombstones.ingress_ports, &port);
        }
        "client_group" => {
            let key = normalized_identifier(key, "client group id")?;
            overrides
                .routing
                .client_groups
                .retain(|item| item.group_id.as_str() != key);
            remove_tombstone(&mut overrides.tombstones.client_groups, &key);
        }
        "egress_upstream" => {
            let key = normalized_identifier(key, "egress upstream name")?;
            overrides
                .routing
                .egress_upstreams
                .retain(|item| item.name != key);
            remove_tombstone(&mut overrides.tombstones.egress_upstreams, &key);
        }
        "egress_vhost_rule" => {
            let key = normalize_host(key)?;
            overrides
                .routing
                .egress_vhost_rules
                .retain(|item| item.match_host != key);
            remove_tombstone(&mut overrides.tombstones.egress_vhost_rules, &key);
        }
        _ => anyhow::bail!("unknown override resource: {resource}"),
    }
    Ok(())
}

fn insert_unique<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn remove_tombstone<T: PartialEq>(items: &mut Vec<T>, item: &T) {
    items.retain(|existing| existing != item);
}

fn normalized_identifier(value: &str, field: &str) -> Result<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(normalized.to_owned())
}

fn normalize_identifier(value: &mut String, field: &str) -> Result<()> {
    *value = normalized_identifier(value, field)?;
    Ok(())
}

fn normalize_host(value: &str) -> Result<String> {
    duotunnel_lib::canonicalize_egress_host(value)
}

fn ensure_unique<T, K, F>(items: &[T], resource: &str, key_of: F) -> Result<()>
where
    K: Eq + std::hash::Hash,
    F: Fn(&T) -> K,
{
    let mut keys = HashSet::with_capacity(items.len());
    for item in items {
        if !keys.insert(key_of(item)) {
            anyhow::bail!("duplicate {resource} key");
        }
    }
    Ok(())
}

pub fn normalize_and_validate_routing(routing: &mut RoutingData) -> Result<()> {
    ensure_unique(&routing.ingress_listeners, "ingress listener", |item| {
        item.port
    })?;

    for item in &mut routing.client_groups {
        item.group_id = normalized_identifier(item.group_id.as_str(), "client group id")?.into();
        for upstream in &mut item.upstreams {
            normalize_identifier(&mut upstream.name, "client upstream name")?;
        }
        ensure_unique(&item.upstreams, "client upstream", |upstream| {
            upstream.name.clone()
        })?;
    }
    ensure_unique(&routing.client_groups, "client group", |item| {
        item.group_id.as_str().to_owned()
    })?;

    for listener in &mut routing.ingress_listeners {
        match &mut listener.mode {
            duotunnel_lib::IngressListenerModeDef::Http { vhost } => {
                for rule in vhost.iter_mut() {
                    rule.match_host = normalize_host(&rule.match_host)?;
                    rule.group_id =
                        normalized_identifier(rule.group_id.as_str(), "ingress vhost group id")?
                            .into();
                }
                ensure_unique(vhost, "ingress vhost", |rule| rule.match_host.clone())?;
            }
            duotunnel_lib::IngressListenerModeDef::Tcp { group_id, .. } => {
                *group_id =
                    normalized_identifier(group_id.as_str(), "TCP listener group id")?.into();
            }
        }
    }

    for item in &mut routing.egress_upstreams {
        normalize_identifier(&mut item.name, "egress upstream name")?;
    }
    ensure_unique(&routing.egress_upstreams, "egress upstream", |item| {
        item.name.clone()
    })?;

    for item in &mut routing.egress_vhost_rules {
        item.match_host = normalize_host(&item.match_host)?;
        normalize_identifier(&mut item.action_upstream, "egress vhost upstream")?;
    }
    ensure_unique(&routing.egress_vhost_rules, "egress vhost rule", |item| {
        item.match_host.clone()
    })?;
    Ok(())
}

fn validate_routing_references(routing: &RoutingData) -> Result<()> {
    let group_ids: HashSet<_> = routing
        .client_groups
        .iter()
        .map(|group| group.group_id.as_str())
        .collect();
    for listener in &routing.ingress_listeners {
        match &listener.mode {
            duotunnel_lib::IngressListenerModeDef::Http { vhost } => {
                for rule in vhost {
                    if !group_ids.contains(rule.group_id.as_str()) {
                        anyhow::bail!(
                            "ingress vhost references unknown client group: {}",
                            rule.group_id
                        );
                    }
                }
            }
            duotunnel_lib::IngressListenerModeDef::Tcp { group_id, .. } => {
                if !group_ids.contains(group_id.as_str()) {
                    anyhow::bail!("TCP listener references unknown client group: {group_id}");
                }
            }
        }
    }
    let upstream_names: HashSet<_> = routing
        .egress_upstreams
        .iter()
        .map(|upstream| upstream.name.as_str())
        .collect();
    for rule in &routing.egress_vhost_rules {
        if !upstream_names.contains(rule.action_upstream.as_str()) {
            anyhow::bail!(
                "egress vhost references unknown upstream: {}",
                rule.action_upstream
            );
        }
    }
    Ok(())
}

pub fn normalize_and_validate_overrides(overrides: &mut SqliteOverrideLayer) -> Result<()> {
    normalize_and_validate_routing(&mut overrides.routing)?;

    ensure_unique(
        &overrides.tombstones.ingress_ports,
        "ingress listener tombstone",
        |key| *key,
    )?;

    for key in &mut overrides.tombstones.client_groups {
        normalize_identifier(key, "client group tombstone")?;
    }
    ensure_unique(
        &overrides.tombstones.client_groups,
        "client group tombstone",
        |key| key.clone(),
    )?;

    for key in &mut overrides.tombstones.egress_upstreams {
        normalize_identifier(key, "egress upstream tombstone")?;
    }
    ensure_unique(
        &overrides.tombstones.egress_upstreams,
        "egress upstream tombstone",
        |key| key.clone(),
    )?;

    for key in &mut overrides.tombstones.egress_vhost_rules {
        *key = normalize_host(key)?;
    }
    ensure_unique(
        &overrides.tombstones.egress_vhost_rules,
        "egress vhost rule tombstone",
        |key| key.clone(),
    )?;

    let ingress_keys: HashSet<_> = overrides
        .routing
        .ingress_listeners
        .iter()
        .map(|item| item.port)
        .collect();
    if ingress_keys
        .iter()
        .any(|key| overrides.tombstones.ingress_ports.contains(key))
    {
        anyhow::bail!("ingress listener override and tombstone share a key");
    }

    let group_keys: HashSet<_> = overrides
        .routing
        .client_groups
        .iter()
        .map(|item| item.group_id.as_str().to_owned())
        .collect();
    if group_keys
        .iter()
        .any(|key| overrides.tombstones.client_groups.contains(key))
    {
        anyhow::bail!("client group override and tombstone share a key");
    }

    let upstream_keys: HashSet<_> = overrides
        .routing
        .egress_upstreams
        .iter()
        .map(|item| item.name.clone())
        .collect();
    if upstream_keys
        .iter()
        .any(|key| overrides.tombstones.egress_upstreams.contains(key))
    {
        anyhow::bail!("egress upstream override and tombstone share a key");
    }

    let vhost_keys: HashSet<_> = overrides
        .routing
        .egress_vhost_rules
        .iter()
        .map(|item| item.match_host.clone())
        .collect();
    if vhost_keys
        .iter()
        .any(|key| overrides.tombstones.egress_vhost_rules.contains(key))
    {
        anyhow::bail!("egress vhost override and tombstone share a key");
    }
    Ok(())
}

pub fn normalize_and_validate_config_layer(layer: &mut ConfigLayer) -> Result<()> {
    normalize_and_validate_routing(&mut layer.routing)
}

pub fn merge_layers(base: &RoutingData, overrides: &SqliteOverrideLayer) -> Result<RoutingData> {
    let mut normalized_base = base.clone();
    normalize_and_validate_routing(&mut normalized_base)?;
    let mut normalized_overrides = overrides.clone();
    normalize_and_validate_overrides(&mut normalized_overrides)?;

    let mut result = normalized_base;
    result.ingress_listeners.retain(|item| {
        !normalized_overrides
            .tombstones
            .ingress_ports
            .contains(&item.port)
    });
    result.client_groups.retain(|item| {
        !normalized_overrides
            .tombstones
            .client_groups
            .iter()
            .any(|key| key == item.group_id.as_str())
    });
    result.egress_upstreams.retain(|item| {
        !normalized_overrides
            .tombstones
            .egress_upstreams
            .contains(&item.name)
    });
    result.egress_vhost_rules.retain(|item| {
        !normalized_overrides
            .tombstones
            .egress_vhost_rules
            .iter()
            .any(|key| key == &item.match_host)
    });

    for item in &normalized_overrides.routing.ingress_listeners {
        replace_by(&mut result.ingress_listeners, item.clone(), |value| {
            value.port
        });
    }
    for item in &normalized_overrides.routing.client_groups {
        replace_by(&mut result.client_groups, item.clone(), |value| {
            value.group_id.as_str().to_owned()
        });
    }
    for item in &normalized_overrides.routing.egress_upstreams {
        replace_by(&mut result.egress_upstreams, item.clone(), |value| {
            value.name.clone()
        });
    }
    for item in &normalized_overrides.routing.egress_vhost_rules {
        replace_by(&mut result.egress_vhost_rules, item.clone(), |value| {
            value.match_host.clone()
        });
    }
    normalize_and_validate_routing(&mut result)?;
    validate_routing_references(&result)?;
    Ok(result)
}

fn replace_by<T, K, F>(items: &mut Vec<T>, item: T, key_of: F)
where
    K: PartialEq,
    F: Fn(&T) -> K,
{
    let key = key_of(&item);
    if let Some(existing) = items.iter_mut().find(|existing| key_of(existing) == key) {
        *existing = item;
    } else {
        items.push(item);
    }
}

#[async_trait]
pub trait ConfigSource: Send + Sync {
    type Layer: Clone + Send + Sync + 'static;

    async fn load(&self) -> Result<Self::Layer>;
    fn subscribe(&self) -> watch::Receiver<Self::Layer>;
}

pub struct YamlConfigSource {
    path: PathBuf,
    tx: watch::Sender<ConfigLayer>,
    degraded_tx: watch::Sender<bool>,
}

impl YamlConfigSource {
    pub async fn new(path: impl Into<PathBuf>) -> Result<Arc<Self>> {
        let path = path.into();
        let layer = load_yaml(&path).await?;
        let mut last_revision = layer.source_revision.clone();
        let (tx, _rx) = watch::channel(layer);
        let (degraded_tx, _degraded_rx) = watch::channel(false);
        let source = Arc::new(Self {
            path: path.clone(),
            tx,
            degraded_tx,
        });
        let weak = Arc::downgrade(&source);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let Some(source) = weak.upgrade() else { break };
                match load_yaml(&source.path).await {
                    Ok(layer) => {
                        if layer.source_revision != last_revision {
                            last_revision = layer.source_revision.clone();
                            let _ = source.tx.send(layer);
                        }
                        if *source.degraded_tx.borrow() {
                            let _ = source.degraded_tx.send(false);
                        }
                    }
                    Err(error) => {
                        if !*source.degraded_tx.borrow() {
                            let _ = source.degraded_tx.send(true);
                        }
                        warn!(path = %source.path.display(), error = %error, "YAML source degraded; retaining last valid layer")
                    }
                }
            }
        });
        Ok(source)
    }
}

#[async_trait]
impl ConfigSource for YamlConfigSource {
    type Layer = ConfigLayer;

    async fn load(&self) -> Result<Self::Layer> {
        load_yaml(&self.path).await
    }

    fn subscribe(&self) -> watch::Receiver<Self::Layer> {
        self.tx.subscribe()
    }
}

impl YamlConfigSource {
    pub fn subscribe_degraded(&self) -> watch::Receiver<bool> {
        self.degraded_tx.subscribe()
    }
}

pub struct SqliteConfigSource {
    pool: sqlx::SqlitePool,
    tx: watch::Sender<SqliteConfigLayer>,
    degraded_tx: watch::Sender<bool>,
}

impl SqliteConfigSource {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Arc<Self>> {
        let layer = load_sqlite_config_layer(&pool).await?;
        let (tx, _rx) = watch::channel(layer);
        let (degraded_tx, _degraded_rx) = watch::channel(false);
        let source = Arc::new(Self {
            pool,
            tx,
            degraded_tx,
        });
        let weak = Arc::downgrade(&source);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let Some(source) = weak.upgrade() else { break };
                match load_sqlite_config_layer(&source.pool).await {
                    Ok(layer) => {
                        let changed = *source.tx.borrow() != layer;
                        if changed {
                            let _ = source.tx.send(layer);
                        }
                        if *source.degraded_tx.borrow() {
                            let _ = source.degraded_tx.send(false);
                        }
                    }
                    Err(error) => {
                        if !*source.degraded_tx.borrow() {
                            let _ = source.degraded_tx.send(true);
                        }
                        warn!(error = %error, "SQLite config source degraded; retaining last valid layer")
                    }
                }
            }
        });
        Ok(source)
    }

    pub fn subscribe_degraded(&self) -> watch::Receiver<bool> {
        self.degraded_tx.subscribe()
    }
}

#[async_trait]
impl ConfigSource for SqliteConfigSource {
    type Layer = SqliteConfigLayer;

    async fn load(&self) -> Result<Self::Layer> {
        load_sqlite_config_layer(&self.pool).await
    }

    fn subscribe(&self) -> watch::Receiver<Self::Layer> {
        self.tx.subscribe()
    }
}

async fn load_sqlite_config_layer(pool: &sqlx::SqlitePool) -> Result<SqliteConfigLayer> {
    let mut conn = pool.acquire().await?;
    let (source_revision, overrides) = load_sqlite_layer_record_on(&mut conn).await?;
    Ok(SqliteConfigLayer {
        source_revision,
        overrides,
    })
}

async fn load_yaml(path: &Path) -> Result<ConfigLayer> {
    let bytes = tokio::fs::read(path).await?;
    parse_yaml_bytes(&bytes)
}

fn parse_yaml_bytes(bytes: &[u8]) -> Result<ConfigLayer> {
    let source_revision = hex::encode(Sha256::digest(bytes));
    let yaml = std::str::from_utf8(bytes)?;
    let config: duotunnel_lib::config::file::RoutingConfigFile =
        Figment::new().merge(Yaml::string(yaml)).extract()?;
    let mut layer = ConfigLayer {
        source_revision,
        routing: duotunnel_lib::config::file::routing_data_from_routing_config(&config),
    };
    normalize_and_validate_config_layer(&mut layer)?;
    Ok(layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::rules::{
        ClientGroup, ClientUpstream, EgressUpstreamDef, EgressVhostRule, IngressListener,
        IngressListenerMode, UpstreamServer,
    };
    use crate::storage::sqlite::open_sqlite_pool;
    use crate::storage::sqlite_rules::SqliteRuleStore;

    fn base() -> RoutingData {
        RoutingData {
            ingress_listeners: vec![IngressListener {
                id: 1,
                port: 8080,
                mode: IngressListenerMode::Http { vhost: vec![] },
            }],
            client_groups: vec![group("base")],
            ..RoutingData::default()
        }
    }

    #[test]
    fn sqlite_override_wins_and_clear_restores_yaml() {
        let mut overrides = SqliteOverrideLayer {
            routing: RoutingData::default(),
            tombstones: OverrideTombstones::default(),
        };
        apply_override_operation(
            &mut overrides,
            &ConfigOperation::DeleteIngressListener { port: 8080 },
        )
        .unwrap();
        assert!(merge_layers(&base(), &overrides)
            .unwrap()
            .ingress_listeners
            .is_empty());

        clear_override(&mut overrides, "ingress_listener", "8080").unwrap();
        assert_eq!(merge_layers(&base(), &overrides).unwrap(), base());

        apply_override_operation(
            &mut overrides,
            &ConfigOperation::UpsertIngressListener(IngressListener {
                id: 2,
                port: 8080,
                mode: IngressListenerMode::Tcp {
                    group_id: "base".into(),
                    proxy_name: "tcp".into(),
                },
            }),
        )
        .unwrap();
        assert!(matches!(
            &merge_layers(&base(), &overrides).unwrap().ingress_listeners[0].mode,
            IngressListenerMode::Tcp { .. }
        ));
    }

    fn group(id: &str) -> ClientGroup {
        ClientGroup {
            group_id: id.into(),
            config_version: "1".into(),
            upstreams: vec![ClientUpstream {
                name: "default".into(),
                lb_policy: "round_robin".into(),
                servers: vec![UpstreamServer {
                    address: "127.0.0.1:8080".into(),
                    resolve: false,
                }],
            }],
        }
    }

    fn upstream(name: &str) -> EgressUpstreamDef {
        EgressUpstreamDef {
            name: name.into(),
            lb_policy: "round_robin".into(),
            servers: vec![UpstreamServer {
                address: "127.0.0.1:8080".into(),
                resolve: false,
            }],
        }
    }

    #[test]
    fn normalize_routing_canonicalizes_hosts_and_identifiers() {
        let mut routing = RoutingData {
            client_groups: vec![group(" group-a ")],
            egress_upstreams: vec![upstream(" upstream-a ")],
            egress_vhost_rules: vec![EgressVhostRule {
                match_host: "Example.COM:443".into(),
                action_upstream: "upstream-a".into(),
            }],
            ..RoutingData::default()
        };

        normalize_and_validate_routing(&mut routing).unwrap();

        assert_eq!(routing.client_groups[0].group_id, "group-a");
        assert_eq!(routing.egress_upstreams[0].name, "upstream-a");
        assert_eq!(routing.egress_vhost_rules[0].match_host, "example.com");
    }

    #[test]
    fn normalize_routing_rejects_duplicate_keys() {
        let cases = [
            RoutingData {
                ingress_listeners: vec![
                    IngressListener {
                        id: 1,
                        port: 8080,
                        mode: IngressListenerMode::Http { vhost: vec![] },
                    },
                    IngressListener {
                        id: 2,
                        port: 8080,
                        mode: IngressListenerMode::Http { vhost: vec![] },
                    },
                ],
                ..RoutingData::default()
            },
            RoutingData {
                client_groups: vec![group("group-a"), group(" group-a ")],
                ..RoutingData::default()
            },
            RoutingData {
                egress_upstreams: vec![upstream("upstream-a"), upstream(" upstream-a ")],
                ..RoutingData::default()
            },
            RoutingData {
                egress_vhost_rules: vec![
                    EgressVhostRule {
                        match_host: "Example.COM".into(),
                        action_upstream: "a".into(),
                    },
                    EgressVhostRule {
                        match_host: "example.com".into(),
                        action_upstream: "b".into(),
                    },
                ],
                ..RoutingData::default()
            },
        ];

        for mut routing in cases {
            assert!(normalize_and_validate_routing(&mut routing).is_err());
        }
    }

    #[test]
    fn normalize_overrides_rejects_override_and_tombstone_conflicts() {
        let mut overrides = SqliteOverrideLayer {
            routing: RoutingData {
                egress_vhost_rules: vec![EgressVhostRule {
                    match_host: "Example.COM".into(),
                    action_upstream: "upstream-a".into(),
                }],
                ..RoutingData::default()
            },
            tombstones: OverrideTombstones {
                egress_vhost_rules: vec!["example.com".into()],
                ..OverrideTombstones::default()
            },
        };

        assert!(normalize_and_validate_overrides(&mut overrides).is_err());
    }

    #[test]
    fn merge_layers_normalizes_host_keys_before_tombstone_matching() {
        let base = RoutingData {
            egress_upstreams: vec![upstream("upstream-a")],
            egress_vhost_rules: vec![EgressVhostRule {
                match_host: "Example.COM:443".into(),
                action_upstream: "upstream-a".into(),
            }],
            ..RoutingData::default()
        };
        let overrides = SqliteOverrideLayer {
            routing: RoutingData::default(),
            tombstones: OverrideTombstones {
                egress_vhost_rules: vec!["example.com".into()],
                ..OverrideTombstones::default()
            },
        };

        assert!(merge_layers(&base, &overrides)
            .unwrap()
            .egress_vhost_rules
            .is_empty());
    }

    #[tokio::test]
    async fn parse_yaml_bytes_hashes_and_parses_the_same_bytes() {
        let bytes = br#"
server_egress_upstream:
  upstreams: {}
tunnel_management:
  groups: {}
  listeners: []
"#;

        let layer = parse_yaml_bytes(bytes).unwrap();
        assert_eq!(layer.source_revision, hex::encode(Sha256::digest(bytes)));
    }

    #[tokio::test]
    async fn legacy_routing_migration_is_repeatable_and_marked() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let rules = SqliteRuleStore::new(pool.clone());
        rules.migrate().await.unwrap();
        {
            let mut tx = pool.begin().await.unwrap();
            SqliteRuleStore::save_routing_on(&mut tx, &base())
                .await
                .unwrap();
            tx.commit().await.unwrap();
        }

        initialize_sqlite_layer(&pool, &rules, true).await.unwrap();
        let first = {
            let mut conn = pool.acquire().await.unwrap();
            load_sqlite_layer_record_on(&mut conn).await.unwrap()
        };
        assert_eq!(first.1.routing, base());
        assert_ne!(first.0, "migration");

        initialize_sqlite_layer(&pool, &rules, true).await.unwrap();
        let second = {
            let mut conn = pool.acquire().await.unwrap();
            load_sqlite_layer_record_on(&mut conn).await.unwrap()
        };
        assert_eq!(first, second);

        for migration in [
            "legacy-routing-to-sqlite-override-v1",
            "legacy-server-config-yaml-base-v1",
        ] {
            let applied: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE migration = ?1")
                    .bind(migration)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(applied, 1, "migration marker {migration} missing");
        }
    }

    #[tokio::test]
    async fn sqlite_source_observes_committed_override_without_writing() {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let rules = SqliteRuleStore::new(pool.clone());
        rules.migrate().await.unwrap();
        initialize_sqlite_layer(&pool, &rules, false).await.unwrap();
        let source = SqliteConfigSource::new(pool.clone()).await.unwrap();
        let mut changes = source.subscribe();

        let mut layer = {
            let mut conn = pool.acquire().await.unwrap();
            load_sqlite_layer_record_on(&mut conn).await.unwrap().1
        };
        apply_override_operation(
            &mut layer,
            &ConfigOperation::UpsertIngressListener(IngressListener {
                id: 2,
                port: 9090,
                mode: IngressListenerMode::Tcp {
                    group_id: "base".into(),
                    proxy_name: "tcp".into(),
                },
            }),
        )
        .unwrap();
        let revision = sqlite_layer_revision(&layer).unwrap();
        let mut tx = pool.begin().await.unwrap();
        save_sqlite_layer_on(&mut tx, &layer, &revision)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        tokio::time::timeout(std::time::Duration::from_secs(2), changes.changed())
            .await
            .unwrap()
            .unwrap();
        let observed = changes.borrow_and_update().clone();
        assert_eq!(observed.source_revision, revision);
        assert_eq!(observed.overrides, layer);
    }
}
