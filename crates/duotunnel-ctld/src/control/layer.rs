use anyhow::Result;
use async_trait::async_trait;
use duotunnel_core::ctld_proto::ConfigOperation;
use duotunnel_store::rules::{RoutingData, RuleStore};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigLayer {
    pub source_revision: String,
    pub routing: RoutingData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SqliteOverrideLayer {
    pub routing: RoutingData,
    #[serde(default)]
    pub tombstones: OverrideTombstones,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OverrideTombstones {
    pub ingress_ports: Vec<u16>,
    pub client_groups: Vec<String>,
    pub egress_upstreams: Vec<String>,
    pub egress_vhost_rules: Vec<String>,
}

pub async fn initialize_sqlite_layer(
    pool: &sqlx::SqlitePool,
    rule_store: &dyn RuleStore,
) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config_layers (
            source TEXT PRIMARY KEY,
            payload TEXT NOT NULL,
            source_revision TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config_state (
            singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
            yaml_source_revision TEXT NOT NULL DEFAULT '',
            sqlite_source_revision TEXT NOT NULL DEFAULT '',
            effective_revision INTEGER NOT NULL DEFAULT 0,
            effective_hash TEXT NOT NULL DEFAULT '',
            initialized INTEGER NOT NULL DEFAULT 0,
            degraded INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query("INSERT OR IGNORE INTO config_state(singleton) VALUES (1)")
        .execute(pool)
        .await?;

    let exists: Option<String> =
        sqlx::query_scalar("SELECT payload FROM config_layers WHERE source = 'sqlite_override'")
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        let legacy = rule_store.load_routing().await?;
        let layer = SqliteOverrideLayer {
            routing: legacy,
            tombstones: OverrideTombstones::default(),
        };
        save_sqlite_layer(pool, &layer, "migration").await?;
    }
    Ok(())
}

pub async fn load_sqlite_layer(pool: &sqlx::SqlitePool) -> Result<SqliteOverrideLayer> {
    let payload: String =
        sqlx::query_scalar("SELECT payload FROM config_layers WHERE source = 'sqlite_override'")
            .fetch_one(pool)
            .await?;
    Ok(serde_json::from_str(&payload)?)
}

pub async fn save_sqlite_layer(
    pool: &sqlx::SqlitePool,
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
    .execute(pool)
    .await?;
    Ok(())
}

pub fn apply_override_operation(
    overrides: &mut SqliteOverrideLayer,
    operation: &ConfigOperation,
) -> Result<()> {
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
            let key = item.group_id.as_str().to_owned();
            remove_tombstone(&mut overrides.tombstones.client_groups, &key);
            replace_by(
                &mut overrides.routing.client_groups,
                item.clone(),
                |value| value.group_id.as_str().to_owned(),
            );
        }
        ConfigOperation::DeleteClientGroup { group_id } => {
            overrides
                .routing
                .client_groups
                .retain(|item| item.group_id.as_str() != group_id);
            insert_unique(&mut overrides.tombstones.client_groups, group_id.clone());
        }
        ConfigOperation::UpsertEgressUpstream(item) => {
            remove_tombstone(&mut overrides.tombstones.egress_upstreams, &item.name);
            replace_by(
                &mut overrides.routing.egress_upstreams,
                item.clone(),
                |value| value.name.clone(),
            );
        }
        ConfigOperation::DeleteEgressUpstream { name } => {
            overrides
                .routing
                .egress_upstreams
                .retain(|item| item.name != *name);
            insert_unique(&mut overrides.tombstones.egress_upstreams, name.clone());
        }
        ConfigOperation::UpsertEgressVhostRule(item) => {
            remove_tombstone(
                &mut overrides.tombstones.egress_vhost_rules,
                &item.match_host,
            );
            replace_by(
                &mut overrides.routing.egress_vhost_rules,
                item.clone(),
                |value| value.match_host.clone(),
            );
        }
        ConfigOperation::DeleteEgressVhostRule { match_host } => {
            overrides
                .routing
                .egress_vhost_rules
                .retain(|item| item.match_host != *match_host);
            insert_unique(
                &mut overrides.tombstones.egress_vhost_rules,
                match_host.clone(),
            );
        }
        ConfigOperation::UpsertToken(_) | ConfigOperation::DeleteToken { .. } => {
            anyhow::bail!("token operations must use the token admin API")
        }
    }
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
            overrides
                .routing
                .client_groups
                .retain(|item| item.group_id.as_str() != key);
            remove_tombstone(&mut overrides.tombstones.client_groups, &key.to_owned());
        }
        "egress_upstream" => {
            overrides
                .routing
                .egress_upstreams
                .retain(|item| item.name != key);
            remove_tombstone(&mut overrides.tombstones.egress_upstreams, &key.to_owned());
        }
        "egress_vhost_rule" => {
            overrides
                .routing
                .egress_vhost_rules
                .retain(|item| item.match_host != key);
            remove_tombstone(
                &mut overrides.tombstones.egress_vhost_rules,
                &key.to_owned(),
            );
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

pub fn merge_layers(base: &RoutingData, overrides: &SqliteOverrideLayer) -> RoutingData {
    let mut result = base.clone();
    result
        .ingress_listeners
        .retain(|item| !overrides.tombstones.ingress_ports.contains(&item.port));
    result.client_groups.retain(|item| {
        !overrides
            .tombstones
            .client_groups
            .iter()
            .any(|key| key == item.group_id.as_str())
    });
    result
        .egress_upstreams
        .retain(|item| !overrides.tombstones.egress_upstreams.contains(&item.name));
    result.egress_vhost_rules.retain(|item| {
        !overrides
            .tombstones
            .egress_vhost_rules
            .iter()
            .any(|key| key == &item.match_host)
    });

    for item in &overrides.routing.ingress_listeners {
        replace_by(&mut result.ingress_listeners, item.clone(), |value| {
            value.port
        });
    }
    for item in &overrides.routing.client_groups {
        replace_by(&mut result.client_groups, item.clone(), |value| {
            value.group_id.as_str().to_owned()
        });
    }
    for item in &overrides.routing.egress_upstreams {
        replace_by(&mut result.egress_upstreams, item.clone(), |value| {
            value.name.clone()
        });
    }
    for item in &overrides.routing.egress_vhost_rules {
        replace_by(&mut result.egress_vhost_rules, item.clone(), |value| {
            value.match_host.clone()
        });
    }
    result
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

pub async fn apply_yaml_layer(
    pool: &sqlx::SqlitePool,
    rule_store: &dyn RuleStore,
    layer: &ConfigLayer,
) -> Result<bool> {
    let overrides = load_sqlite_layer(pool).await?;
    let effective = merge_layers(&layer.routing, &overrides);
    let current = rule_store.load_routing().await?;
    let changed = current != effective;
    if changed {
        rule_store.save_routing(&effective).await?;
    }
    let effective_hash = hex::encode(Sha256::digest(serde_json::to_vec(&effective)?));
    sqlx::query(
        "UPDATE config_state SET yaml_source_revision = ?1,
            effective_hash = ?2, initialized = 1, degraded = 0 WHERE singleton = 1",
    )
    .bind(&layer.source_revision)
    .bind(effective_hash)
    .execute(pool)
    .await?;
    Ok(changed)
}

pub async fn apply_sqlite_override(
    pool: &sqlx::SqlitePool,
    rule_store: &dyn RuleStore,
    yaml_layer: &ConfigLayer,
    operation: &ConfigOperation,
) -> Result<bool> {
    let mut overrides = load_sqlite_layer(pool).await?;
    apply_override_operation(&mut overrides, operation)?;
    let source_revision = hex::encode(Sha256::digest(serde_json::to_vec(&overrides)?));
    save_sqlite_layer(pool, &overrides, &source_revision).await?;
    let effective = merge_layers(&yaml_layer.routing, &overrides);
    let current = rule_store.load_routing().await?;
    let changed = current != effective;
    if changed {
        rule_store.save_routing(&effective).await?;
    }
    let effective_hash = hex::encode(Sha256::digest(serde_json::to_vec(&effective)?));
    sqlx::query(
        "UPDATE config_state SET sqlite_source_revision = ?1,
            effective_hash = ?2, initialized = 1, degraded = 0 WHERE singleton = 1",
    )
    .bind(source_revision)
    .bind(effective_hash)
    .execute(pool)
    .await?;
    Ok(changed)
}

pub async fn clear_sqlite_override(
    pool: &sqlx::SqlitePool,
    rule_store: &dyn RuleStore,
    yaml_layer: &ConfigLayer,
    resource: &str,
    key: &str,
) -> Result<bool> {
    let mut overrides = load_sqlite_layer(pool).await?;
    clear_override(&mut overrides, resource, key)?;
    let source_revision = hex::encode(Sha256::digest(serde_json::to_vec(&overrides)?));
    save_sqlite_layer(pool, &overrides, &source_revision).await?;
    let effective = merge_layers(&yaml_layer.routing, &overrides);
    let current = rule_store.load_routing().await?;
    let changed = current != effective;
    if changed {
        rule_store.save_routing(&effective).await?;
    }
    let effective_hash = hex::encode(Sha256::digest(serde_json::to_vec(&effective)?));
    sqlx::query(
        "UPDATE config_state SET sqlite_source_revision = ?1,
            effective_hash = ?2, initialized = 1, degraded = 0 WHERE singleton = 1",
    )
    .bind(source_revision)
    .bind(effective_hash)
    .execute(pool)
    .await?;
    Ok(changed)
}

#[async_trait]
pub trait ConfigSource: Send + Sync {
    async fn load(&self) -> Result<ConfigLayer>;
    fn subscribe(&self) -> watch::Receiver<ConfigLayer>;
}

pub struct YamlConfigSource {
    path: PathBuf,
    tx: watch::Sender<ConfigLayer>,
}

impl YamlConfigSource {
    pub async fn new(path: impl Into<PathBuf>) -> Result<Arc<Self>> {
        let path = path.into();
        let layer = load_yaml(&path).await?;
        let mut last_revision = layer.source_revision.clone();
        let (tx, _rx) = watch::channel(layer);
        let source = Arc::new(Self {
            path: path.clone(),
            tx,
        });
        let weak = Arc::downgrade(&source);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let Some(source) = weak.upgrade() else { break };
                match load_yaml(&source.path).await {
                    Ok(layer) if layer.source_revision != last_revision => {
                        last_revision = layer.source_revision.clone();
                        let _ = source.tx.send(layer);
                    }
                    Ok(_) => {}
                    Err(error) => {
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
    async fn load(&self) -> Result<ConfigLayer> {
        load_yaml(&self.path).await
    }

    fn subscribe(&self) -> watch::Receiver<ConfigLayer> {
        self.tx.subscribe()
    }
}

async fn load_yaml(path: &Path) -> Result<ConfigLayer> {
    let bytes = tokio::fs::read(path).await?;
    let source_revision = {
        let digest = Sha256::digest(&bytes);
        hex::encode(digest)
    };
    let config = duotunnel_store::server_config::RoutingConfigFile::load(
        path.to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid YAML path"))?,
    )?;
    Ok(ConfigLayer {
        source_revision,
        routing: duotunnel_store::server_config::routing_data_from_routing_config(&config),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use duotunnel_store::rules::{IngressListener, IngressListenerMode};

    fn base() -> RoutingData {
        RoutingData {
            ingress_listeners: vec![IngressListener {
                id: 1,
                port: 8080,
                mode: IngressListenerMode::Http { vhost: vec![] },
            }],
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
            .ingress_listeners
            .is_empty());

        clear_override(&mut overrides, "ingress_listener", "8080").unwrap();
        assert_eq!(merge_layers(&base(), &overrides), base());

        apply_override_operation(
            &mut overrides,
            &ConfigOperation::UpsertIngressListener(IngressListener {
                id: 2,
                port: 8080,
                mode: IngressListenerMode::Tcp {
                    group_id: "override".into(),
                    proxy_name: "tcp".into(),
                },
            }),
        )
        .unwrap();
        assert!(matches!(
            &merge_layers(&base(), &overrides).ingress_listeners[0].mode,
            IngressListenerMode::Tcp { .. }
        ));
    }
}
