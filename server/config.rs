use anyhow::Result;
use async_trait::async_trait;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use std::sync::Arc;
use serde::Deserialize;
use std::collections::HashMap;
use tunnel_lib::config::{HttpPoolConfig, ProxyBufferConfig, QuicConfig, TcpConfig};
use tunnel_lib::PkiParams;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfigFile {
    pub server: ServerBasicConfig,
    #[serde(default)]
    pub server_egress_upstream: ServerEgressUpstream,
    #[serde(default)]
    pub tunnel_management: TunnelManagement,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerBasicConfig {
    pub tunnel_port: u16,
    #[serde(default)]
    pub entry_port: Option<u16>,
    #[serde(default)]
    pub log_level: Option<String>,

    #[serde(default)]
    pub trace_enabled: bool,

    pub database_url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_max_connections")]
    pub max_tcp_connections: usize,
    #[serde(default)]
    pub metrics_port: Option<u16>,

    #[serde(default)]
    pub quic: QuicConfig,
    #[serde(default)]
    pub tcp: TcpConfig,
    #[serde(default)]
    pub http_pool: HttpPoolConfig,
    #[serde(default)]
    pub proxy_buffers: ProxyBufferConfig,

    #[serde(default)]
    pub pki: PkiParams,

    #[serde(default = "default_login_timeout_secs")]
    pub login_timeout_secs: u64,
}

fn default_login_timeout_secs() -> u64 {
    10
}

fn default_max_connections() -> usize {
    10_000
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerEgressUpstream {
    #[serde(default)]
    pub upstreams: HashMap<String, UpstreamDef>,
    #[serde(default)]
    pub rules: EgressRules,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EgressRules {
    #[serde(default)]
    pub vhost: Vec<EgressHttpRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EgressHttpRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TunnelManagement {
    #[serde(default)]
    pub server_ingress_routing: IngressRouting,
    #[serde(default)]
    pub client_configs: ClientConfigs,
}

impl TunnelManagement {
    pub fn is_empty(&self) -> bool {
        self.server_ingress_routing.rules.vhost.is_empty()
            && self.server_ingress_routing.rules.tcp.is_empty()
            && self.client_configs.client_egress_routings.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IngressRouting {
    #[serde(default)]
    pub rules: IngressRules,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IngressRules {
    #[serde(default)]
    pub vhost: Vec<VhostRule>,
    #[serde(default)]
    pub tcp: Vec<TcpRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VhostRule {
    pub match_host: String,
    pub action_client_group: String,
    #[serde(default)]
    pub action_proxy_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TcpRule {
    pub match_port: u16,
    pub action_client_group: String,
    pub action_proxy_name: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClientConfigs {
    #[serde(default)]
    pub client_egress_routings: HashMap<String, GroupConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupConfig {
    #[serde(default)]
    pub config_version: String,
    #[serde(default)]
    pub upstreams: HashMap<String, UpstreamDef>,
    #[serde(default)]
    pub rules: GroupRules,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GroupRules {
    #[serde(default)]
    pub vhost: Vec<RuleDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleDef {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpstreamDef {
    pub servers: Vec<ServerDef>,
    #[serde(default = "default_lb_policy")]
    pub lb_policy: String,
}

fn default_lb_policy() -> String {
    "round_robin".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerDef {
    pub address: String,
    #[serde(default)]
    pub resolve: bool,
}

impl ServerConfigFile {
    pub fn load(path: &str) -> Result<Self> {
        let resolved = tunnel_lib::resolve_config_path(path)?;

        let config: ServerConfigFile = Figment::new()
            .merge(Yaml::file(&resolved))
            .merge(
                Env::prefixed("TUNNEL_SERVER__")
                    .only(&["server.log_level", "server.database_url"])
                    .split("__"),
            )
            .extract()?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        if self.server.tunnel_port == 0 {
            errors.push("server.tunnel_port must not be 0".into());
        }
        if self.server.database_url.trim().is_empty() {
            errors.push("server.database_url is required".into());
        }
        if self.server.max_connections == 0 {
            errors.push("server.max_connections must be >= 1".into());
        }
        if self.server.max_tcp_connections == 0 {
            errors.push("server.max_tcp_connections must be >= 1".into());
        }

        for (name, upstream) in &self.server_egress_upstream.upstreams {
            if upstream.servers.is_empty() {
                errors.push(format!(
                    "server_egress_upstream.upstreams.{}: must have at least one server",
                    name
                ));
            }
        }

        for rule in &self.tunnel_management.server_ingress_routing.rules.vhost {
            if rule.match_host.is_empty() {
                errors.push("tunnel_management: vhost rule has empty match_host".into());
            }
            if rule.action_client_group.is_empty() {
                errors.push("tunnel_management: vhost rule has empty action_client_group".into());
            }
        }

        for (group_id, group) in &self.tunnel_management.client_configs.client_egress_routings {
            for (name, upstream) in &group.upstreams {
                if upstream.servers.is_empty() {
                    errors.push(format!(
                        "tunnel_management.client_configs.{}.upstreams.{}: must have at least one server",
                        group_id, name
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Config validation failed:\n  - {}",
                errors.join("\n  - ")
            ))
        }
    }
}

pub fn build_client_config_for_group(
    tm: &TunnelManagement,
    group_id: &str,
) -> Option<tunnel_lib::ClientConfig> {
    let group_config = tm.client_configs.client_egress_routings.get(group_id)?;

    let proxies = collect_proxies_for_group(tm, group_id);

    let upstreams = group_config
        .upstreams
        .iter()
        .map(|(name, def)| tunnel_lib::UpstreamConfig {
            name: name.clone(),
            servers: def
                .servers
                .iter()
                .map(|s| tunnel_lib::UpstreamServer {
                    address: s.address.clone(),
                    resolve: s.resolve,
                })
                .collect(),
            lb_policy: def.lb_policy.clone(),
        })
        .collect();

    let rules: Vec<tunnel_lib::RuleConfig> = group_config
        .rules
        .vhost
        .iter()
        .map(|r| tunnel_lib::RuleConfig {
            rule_type: "vhost".to_string(),
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        })
        .collect();

    Some(tunnel_lib::ClientConfig {
        config_version: group_config.config_version.clone(),
        proxies,
        upstreams,
        rules,
    })
}

fn collect_proxies_for_group(
    tm: &TunnelManagement,
    group_id: &str,
) -> Vec<tunnel_lib::ProxyConfig> {
    let mut proxies = Vec::new();

    for rule in &tm.server_ingress_routing.rules.vhost {
        if rule.action_client_group == group_id {
            let proxy_name = rule
                .action_proxy_name
                .clone()
                .unwrap_or_else(|| rule.match_host.clone());
            proxies.push(tunnel_lib::ProxyConfig {
                name: proxy_name,
                proxy_type: "http".to_string(),
                domains: vec![rule.match_host.clone()],
                remote_port: None,
            });
        }
    }

    for rule in &tm.server_ingress_routing.rules.tcp {
        if rule.action_client_group == group_id {
            proxies.push(tunnel_lib::ProxyConfig {
                name: rule.action_proxy_name.clone(),
                proxy_type: "tcp".to_string(),
                domains: vec![],
                remote_port: Some(rule.match_port),
            });
        }
    }

    proxies
}

// ---------------------------------------------------------------------------
// ConfigSource abstraction (TODO-52)
// ---------------------------------------------------------------------------

/// A source that can provide routing configuration (ingress rules + client configs).
/// Both `TunnelManagement` and `ServerEgressUpstream` are returned together.
#[async_trait]
pub trait ConfigSource: Send + Sync {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)>;
}

/// Loads routing config from the YAML file (original behaviour).
pub struct FileSource {
    config_path: String,
}

impl FileSource {
    pub fn new(config_path: impl Into<String>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }
}

#[async_trait]
impl ConfigSource for FileSource {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)> {
        let cfg = ServerConfigFile::load(&self.config_path)?;
        Ok((cfg.tunnel_management, cfg.server_egress_upstream))
    }
}

/// Loads routing config from the database via `RuleStore`.
pub struct DbSource {
    rule_store: Arc<dyn tunnel_store::RuleStore>,
}

impl DbSource {
    pub fn new(rule_store: Arc<dyn tunnel_store::RuleStore>) -> Self {
        Self { rule_store }
    }
}

#[async_trait]
impl ConfigSource for DbSource {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)> {

        let data = self.rule_store.load_routing().await?;

        // Convert store types → server config types
        let ingress_vhost: Vec<VhostRule> = data
            .ingress_vhost
            .into_iter()
            .map(|r| VhostRule {
                match_host: r.match_host,
                action_client_group: r.action_client_group,
                action_proxy_name: r.action_proxy_name,
            })
            .collect();

        let ingress_tcp: Vec<TcpRule> = data
            .ingress_tcp
            .into_iter()
            .map(|r| TcpRule {
                match_port: r.match_port,
                action_client_group: r.action_client_group,
                action_proxy_name: r.action_proxy_name,
            })
            .collect();

        let tm = TunnelManagement {
            server_ingress_routing: IngressRouting {
                rules: IngressRules {
                    vhost: ingress_vhost,
                    tcp: ingress_tcp,
                },
            },
            client_configs: ClientConfigs {
                client_egress_routings: data
                    .client_groups
                    .into_iter()
                    .map(|(group_id, g)| {
                        (
                            group_id,
                            GroupConfig {
                                config_version: g.config_version,
                                upstreams: convert_upstreams(g.upstreams),
                                rules: GroupRules {
                                    vhost: g
                                        .vhost_rules
                                        .into_iter()
                                        .map(|r| RuleDef {
                                            match_host: r.match_host,
                                            action_upstream: r.action_upstream,
                                        })
                                        .collect(),
                                },
                            },
                        )
                    })
                    .collect(),
            },
        };

        let egress = ServerEgressUpstream {
            upstreams: convert_upstreams(data.server_egress_upstreams),
            rules: EgressRules {
                vhost: data
                    .server_egress_vhost_rules
                    .into_iter()
                    .map(|r| EgressHttpRule {
                        match_host: r.match_host,
                        action_upstream: r.action_upstream,
                    })
                    .collect(),
            },
        };

        Ok((tm, egress))
    }
}

fn convert_upstreams(
    src: HashMap<String, tunnel_store::UpstreamDef>,
) -> HashMap<String, UpstreamDef> {
    src.into_iter()
        .map(|(name, def)| {
            (
                name,
                UpstreamDef {
                    servers: def
                        .servers
                        .into_iter()
                        .map(|s| ServerDef {
                            address: s.address,
                            resolve: s.resolve,
                        })
                        .collect(),
                    lb_policy: def.lb_policy,
                },
            )
        })
        .collect()
}

/// Merges two sources: `primary` is tried first; on failure (or if empty) falls back to
/// `fallback`.  If primary succeeds, its data wins entirely.
pub struct MergedSource {
    primary: Box<dyn ConfigSource>,
    fallback: Box<dyn ConfigSource>,
}

impl MergedSource {
    pub fn new(primary: Box<dyn ConfigSource>, fallback: Box<dyn ConfigSource>) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl ConfigSource for MergedSource {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)> {
        match self.primary.load().await {
            Ok(result) => {
                let (tm, _) = &result;
                if tm.is_empty() {
                    tracing::debug!("primary ConfigSource returned empty routing, using fallback");
                    return self.fallback.load().await;
                }
                Ok(result)
            }
            Err(e) => {
                tracing::warn!(error = %e, "primary ConfigSource failed, using fallback");
                self.fallback.load().await
            }
        }
    }
}

/// Converts a `ServerConfigFile` (from YAML) into store types and persists it into the
/// DB so the DB stays in sync with the config file (e.g. on first boot).
pub async fn sync_file_to_db(
    cfg: &ServerConfigFile,
    rule_store: &dyn tunnel_store::RuleStore,
) -> Result<()> {
    use tunnel_store::{
        EgressVhostRule as StoreEgress, GroupConfig as StoreGroup, IngressTcpRule as StoreTcp,
        IngressVhostRule as StoreVhost, RoutingData, UpstreamDef as StoreUpstream,
        UpstreamServer as StoreServer,
    };

    fn to_store_upstream(src: &HashMap<String, UpstreamDef>) -> HashMap<String, StoreUpstream> {
        src.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    StoreUpstream {
                        servers: v
                            .servers
                            .iter()
                            .map(|s| StoreServer {
                                address: s.address.clone(),
                                resolve: s.resolve,
                            })
                            .collect(),
                        lb_policy: v.lb_policy.clone(),
                    },
                )
            })
            .collect()
    }

    let tm = &cfg.tunnel_management;
    let eg = &cfg.server_egress_upstream;

    let data = RoutingData {
        ingress_vhost: tm
            .server_ingress_routing
            .rules
            .vhost
            .iter()
            .map(|r| StoreVhost {
                match_host: r.match_host.clone(),
                action_client_group: r.action_client_group.clone(),
                action_proxy_name: r.action_proxy_name.clone(),
            })
            .collect(),
        ingress_tcp: tm
            .server_ingress_routing
            .rules
            .tcp
            .iter()
            .map(|r| StoreTcp {
                match_port: r.match_port,
                action_client_group: r.action_client_group.clone(),
                action_proxy_name: r.action_proxy_name.clone(),
            })
            .collect(),
        server_egress_upstreams: to_store_upstream(&eg.upstreams),
        server_egress_vhost_rules: eg
            .rules
            .vhost
            .iter()
            .map(|r| StoreEgress {
                match_host: r.match_host.clone(),
                action_upstream: r.action_upstream.clone(),
            })
            .collect(),
        client_groups: tm
            .client_configs
            .client_egress_routings
            .iter()
            .map(|(gid, g)| {
                (
                    gid.clone(),
                    StoreGroup {
                        config_version: g.config_version.clone(),
                        upstreams: to_store_upstream(&g.upstreams),
                        vhost_rules: g
                            .rules
                            .vhost
                            .iter()
                            .map(|r| StoreEgress {
                                match_host: r.match_host.clone(),
                                action_upstream: r.action_upstream.clone(),
                            })
                            .collect(),
                    },
                )
            })
            .collect(),
    };

    rule_store.save_routing(&data).await
}
