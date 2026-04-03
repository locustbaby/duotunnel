use anyhow::Result;
use async_trait::async_trait;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serde::Deserialize;
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
    #[serde(default = "default_h2_single_authority")]
    pub h2_single_authority: bool,
}
fn default_login_timeout_secs() -> u64 {
    10
}
fn default_h2_single_authority() -> bool {
    true
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
        self.server_ingress_routing.listeners.is_empty()
            && self.client_configs.groups.is_empty()
    }
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct IngressRouting {
    #[serde(default)]
    pub listeners: Vec<IngressListener>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct IngressListener {
    pub port: u16,
    #[serde(flatten)]
    pub mode: IngressMode,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum IngressMode {
    Http(HttpListenerConfig),
    Tcp(TcpListenerConfig),
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HttpListenerConfig {
    #[serde(default)]
    pub vhost: Vec<VhostRule>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TcpListenerConfig {
    pub client_group: String,
    pub proxy_name: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct VhostRule {
    pub match_host: String,
    pub client_group: String,
    pub proxy_name: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClientConfigs {
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GroupConfig {
    #[serde(default)]
    pub config_version: String,
    #[serde(default)]
    pub upstreams: HashMap<String, UpstreamDef>,
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
        let groups = &self.tunnel_management.client_configs.groups;
        let mut seen_ports: HashSet<u16> = HashSet::new();
        for listener in &self.tunnel_management.server_ingress_routing.listeners {
            if listener.port == 0 {
                errors.push("tunnel_management: listener port must not be 0".into());
            }
            if !seen_ports.insert(listener.port) {
                errors.push(format!(
                    "tunnel_management: duplicate listener port {}",
                    listener.port
                ));
            }
            match &listener.mode {
                IngressMode::Http(cfg) => {
                    let mut seen_hosts: HashSet<&str> = HashSet::new();
                    for rule in &cfg.vhost {
                        if rule.match_host.is_empty() {
                            errors.push(format!(
                                "port {}: vhost rule has empty match_host",
                                listener.port
                            ));
                        }
                        if !seen_hosts.insert(rule.match_host.as_str()) {
                            errors.push(format!(
                                "port {}: duplicate match_host \"{}\"",
                                listener.port, rule.match_host
                            ));
                        }
                        if let Some(group) = groups.get(&rule.client_group) {
                            if !group.upstreams.contains_key(&rule.proxy_name) {
                                errors.push(format!(
                                    "port {}: match_host \"{}\": proxy_name \"{}\" not found in group \"{}\" upstreams",
                                    listener.port, rule.match_host, rule.proxy_name, rule.client_group
                                ));
                            }
                        } else {
                            errors.push(format!(
                                "port {}: match_host \"{}\": group \"{}\" not found in client_configs",
                                listener.port, rule.match_host, rule.client_group
                            ));
                        }
                    }
                }
                IngressMode::Tcp(cfg) => {
                    if let Some(group) = groups.get(&cfg.client_group) {
                        if !group.upstreams.contains_key(&cfg.proxy_name) {
                            errors.push(format!(
                                "port {}: tcp proxy_name \"{}\" not found in group \"{}\" upstreams",
                                listener.port, cfg.proxy_name, cfg.client_group
                            ));
                        }
                    } else {
                        errors.push(format!(
                            "port {}: tcp group \"{}\" not found in client_configs",
                            listener.port, cfg.client_group
                        ));
                    }
                }
            }
        }
        for (group_id, group) in groups {
            for (name, upstream) in &group.upstreams {
                if upstream.servers.is_empty() {
                    errors.push(format!(
                        "client_configs.groups.{}.upstreams.{}: must have at least one server",
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
    let group = tm.client_configs.groups.get(group_id)?;
    let upstreams = group
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
    Some(tunnel_lib::ClientConfig {
        config_version: group.config_version.clone(),
        upstreams,
    })
}
#[async_trait]
pub trait ConfigSource: Send + Sync {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)>;
}
pub struct FileSource {
    config_path: String,
}
impl FileSource {
    pub fn new(config_path: impl Into<String>) -> Self {
        Self { config_path: config_path.into() }
    }
}
#[async_trait]
impl ConfigSource for FileSource {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)> {
        let cfg = ServerConfigFile::load(&self.config_path)?;
        Ok((cfg.tunnel_management, cfg.server_egress_upstream))
    }
}
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
        let listeners = data
            .ingress_listeners
            .into_iter()
            .map(|l| IngressListener {
                port: l.port,
                mode: match l.mode {
                    tunnel_store::IngressListenerMode::Http { vhost } => {
                        IngressMode::Http(HttpListenerConfig {
                            vhost: vhost
                                .into_iter()
                                .map(|r| VhostRule {
                                    match_host: r.match_host,
                                    client_group: r.group_id,
                                    proxy_name: r.proxy_name,
                                })
                                .collect(),
                        })
                    }
                    tunnel_store::IngressListenerMode::Tcp { group_id, proxy_name } => {
                        IngressMode::Tcp(TcpListenerConfig {
                            client_group: group_id,
                            proxy_name,
                        })
                    }
                },
            })
            .collect();
        let groups = data
            .client_groups
            .into_iter()
            .map(|g| {
                let upstreams = g
                    .upstreams
                    .into_iter()
                    .map(|u| {
                        (
                            u.name,
                            UpstreamDef {
                                servers: u
                                    .servers
                                    .into_iter()
                                    .map(|s| ServerDef { address: s.address, resolve: s.resolve })
                                    .collect(),
                                lb_policy: u.lb_policy,
                            },
                        )
                    })
                    .collect();
                (g.group_id, GroupConfig { config_version: g.config_version, upstreams })
            })
            .collect();
        let tm = TunnelManagement {
            server_ingress_routing: IngressRouting { listeners },
            client_configs: ClientConfigs { groups },
        };
        let egress_upstreams = data
            .egress_upstreams
            .into_iter()
            .map(|u| {
                (
                    u.name,
                    UpstreamDef {
                        servers: u
                            .servers
                            .into_iter()
                            .map(|s| ServerDef { address: s.address, resolve: s.resolve })
                            .collect(),
                        lb_policy: u.lb_policy,
                    },
                )
            })
            .collect();
        let egress = ServerEgressUpstream {
            upstreams: egress_upstreams,
            rules: EgressRules {
                vhost: data
                    .egress_vhost_rules
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
            Ok((tm, egress)) => {
                if tm.is_empty() {
                    tracing::debug!(
                        "primary ConfigSource returned empty routing, using fallback"
                    );
                    let (fallback_tm, fallback_egress) = self.fallback.load().await?;
                    let merged_egress =
                        if egress.upstreams.is_empty() && egress.rules.vhost.is_empty() {
                            fallback_egress
                        } else {
                            egress
                        };
                    return Ok((fallback_tm, merged_egress));
                }
                Ok((tm, egress))
            }
            Err(e) => {
                tracing::warn!(error = %e, "primary ConfigSource failed, using fallback");
                self.fallback.load().await
            }
        }
    }
}
pub async fn sync_file_to_db(
    cfg: &ServerConfigFile,
    rule_store: &dyn tunnel_store::RuleStore,
) -> Result<()> {
    use tunnel_store::{
        ClientGroup as StoreGroup, ClientUpstream as StoreUpstream,
        EgressUpstreamDef as StoreEgressUpstream, EgressVhostRule as StoreEgressVhost,
        IngressListener as StoreListener, IngressListenerMode as StoreMode,
        IngressVhostRule as StoreVhost, RoutingData, UpstreamServer as StoreServer,
    };
    let tm = &cfg.tunnel_management;
    let eg = &cfg.server_egress_upstream;
    let ingress_listeners = tm
        .server_ingress_routing
        .listeners
        .iter()
        .map(|l| StoreListener {
            id: 0,
            port: l.port,
            mode: match &l.mode {
                IngressMode::Http(cfg) => StoreMode::Http {
                    vhost: cfg
                        .vhost
                        .iter()
                        .map(|r| StoreVhost {
                            match_host: r.match_host.clone(),
                            group_id: r.client_group.clone(),
                            proxy_name: r.proxy_name.clone(),
                        })
                        .collect(),
                },
                IngressMode::Tcp(cfg) => StoreMode::Tcp {
                    group_id: cfg.client_group.clone(),
                    proxy_name: cfg.proxy_name.clone(),
                },
            },
        })
        .collect();
    let client_groups = tm
        .client_configs
        .groups
        .iter()
        .map(|(gid, g)| StoreGroup {
            group_id: gid.clone(),
            config_version: g.config_version.clone(),
            upstreams: g
                .upstreams
                .iter()
                .map(|(name, def)| StoreUpstream {
                    name: name.clone(),
                    lb_policy: def.lb_policy.clone(),
                    servers: def
                        .servers
                        .iter()
                        .map(|s| StoreServer { address: s.address.clone(), resolve: s.resolve })
                        .collect(),
                })
                .collect(),
        })
        .collect();
    let egress_upstreams = eg
        .upstreams
        .iter()
        .map(|(name, def)| StoreEgressUpstream {
            name: name.clone(),
            lb_policy: def.lb_policy.clone(),
            servers: def
                .servers
                .iter()
                .map(|s| StoreServer { address: s.address.clone(), resolve: s.resolve })
                .collect(),
        })
        .collect();
    let egress_vhost_rules = eg
        .rules
        .vhost
        .iter()
        .map(|r| StoreEgressVhost {
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        })
        .collect();
    rule_store
        .save_routing(&RoutingData {
            ingress_listeners,
            client_groups,
            egress_upstreams,
            egress_vhost_rules,
        })
        .await
}
