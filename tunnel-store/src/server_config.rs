use crate::rules::{
    ClientGroup, ClientUpstream, EgressUpstreamDef, EgressVhostRule, IngressListener,
    IngressListenerMode, IngressVhostRule, RoutingData, UpstreamServer,
};
use anyhow::Result;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;
/// Parses the routing sections of a server.yaml and converts them to
/// [`RoutingData`] that can be saved to any [`RuleStore`].
///
/// This module is gated behind the `server-config` feature so that consumers
/// that never need YAML parsing (e.g. pure-store binaries) pay no extra deps.
use std::collections::HashMap;
use tunnel_lib::config::{HttpPoolConfig, ProxyBufferConfig, QuicConfig, TcpConfig};
use tunnel_lib::PkiParams;

// ── Full server config file schema ───────────────────────────────────────────

/// Mirrors the on-disk `server.yaml` layout. Only the routing sections
/// (`tunnel_management`, `server_egress_upstream`) are used by this module;
/// `ServerBasicConfig` is included so the file parses without errors even when
/// called from ctld (which doesn't use the runtime params).
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
    #[serde(default)]
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

fn default_max_connections() -> usize {
    10_000
}
fn default_login_timeout_secs() -> u64 {
    10
}
fn default_h2_single_authority() -> bool {
    true
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
        self.server_ingress_routing.listeners.is_empty() && self.client_configs.groups.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IngressRouting {
    #[serde(default)]
    pub listeners: Vec<IngressListenerDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngressListenerDef {
    pub port: u16,
    #[serde(flatten)]
    pub mode: IngressModeDef,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum IngressModeDef {
    Http(HttpListenerDef),
    Tcp(TcpListenerDef),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct HttpListenerDef {
    #[serde(default)]
    pub vhost: Vec<VhostRuleDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TcpListenerDef {
    pub client_group: String,
    pub proxy_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VhostRuleDef {
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

// ── Parsing ───────────────────────────────────────────────────────────────────

impl ServerConfigFile {
    /// Load and parse a server YAML config file.
    pub fn load(path: &str) -> Result<Self> {
        let cfg: ServerConfigFile = Figment::new()
            .merge(Yaml::file(path))
            .merge(
                Env::prefixed("TUNNEL_SERVER__")
                    .only(&["server.log_level", "server.database_url"])
                    .split("__"),
            )
            .extract()?;
        Ok(cfg)
    }
}

// ── Conversion to RoutingData ─────────────────────────────────────────────────

/// Convert the routing sections of a parsed [`ServerConfigFile`] into a
/// [`RoutingData`] suitable for saving to any [`RuleStore`].
pub fn routing_data_from_server_config(cfg: &ServerConfigFile) -> RoutingData {
    let tm = &cfg.tunnel_management;
    let eg = &cfg.server_egress_upstream;

    let ingress_listeners = tm
        .server_ingress_routing
        .listeners
        .iter()
        .map(|l| IngressListener {
            id: 0, // DB assigns real IDs on save
            port: l.port,
            mode: match &l.mode {
                IngressModeDef::Http(h) => IngressListenerMode::Http {
                    vhost: h
                        .vhost
                        .iter()
                        .map(|r| IngressVhostRule {
                            match_host: r.match_host.clone(),
                            group_id: r.client_group.clone(),
                            proxy_name: r.proxy_name.clone(),
                        })
                        .collect(),
                },
                IngressModeDef::Tcp(t) => IngressListenerMode::Tcp {
                    group_id: t.client_group.clone(),
                    proxy_name: t.proxy_name.clone(),
                },
            },
        })
        .collect();

    let client_groups = tm
        .client_configs
        .groups
        .iter()
        .map(|(gid, g)| ClientGroup {
            group_id: gid.clone(),
            config_version: g.config_version.clone(),
            upstreams: g
                .upstreams
                .iter()
                .map(|(name, def)| ClientUpstream {
                    name: name.clone(),
                    lb_policy: def.lb_policy.clone(),
                    servers: def
                        .servers
                        .iter()
                        .map(|s| UpstreamServer {
                            address: s.address.clone(),
                            resolve: s.resolve,
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();

    let egress_upstreams = eg
        .upstreams
        .iter()
        .map(|(name, def)| EgressUpstreamDef {
            name: name.clone(),
            lb_policy: def.lb_policy.clone(),
            servers: def
                .servers
                .iter()
                .map(|s| UpstreamServer {
                    address: s.address.clone(),
                    resolve: s.resolve,
                })
                .collect(),
        })
        .collect();

    let egress_vhost_rules = eg
        .rules
        .vhost
        .iter()
        .map(|r| EgressVhostRule {
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        })
        .collect();

    RoutingData {
        ingress_listeners,
        client_groups,
        egress_upstreams,
        egress_vhost_rules,
    }
}
