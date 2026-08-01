use crate::config::{HttpPoolConfig, ProxyBufferConfig, QuicConfig, TcpConfig};
use crate::{
    canonicalize_egress_host, ClientGroupDef as ClientGroup, ClientUpstreamDef as ClientUpstream,
    EgressUpstreamDef, EgressVhostRuleDef as EgressVhostRule, GroupId,
    IngressListenerDef as IngressListener, IngressListenerModeDef as IngressListenerMode,
    IngressVhostRuleDef as IngressVhostRule, PkiParams, ProxyName,
    UpstreamServerDef as UpstreamServer,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutingData {
    pub ingress_listeners: Vec<IngressListener>,
    pub client_groups: Vec<ClientGroup>,
    pub egress_upstreams: Vec<EgressUpstreamDef>,
    pub egress_vhost_rules: Vec<EgressVhostRule>,
}

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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RoutingConfigFile {
    #[serde(default)]
    pub server_egress_upstream: ServerEgressUpstream,
    #[serde(default)]
    pub tunnel_management: TunnelManagement,
}

impl RoutingConfigFile {
    pub fn load(path: &str) -> Result<Self> {
        Ok(Figment::new().merge(Yaml::file(path)).extract()?)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OverloadConfig {
    pub emfile_backoff_ms: u64,
    pub max_pending_streams: Option<usize>,
}

impl Default for OverloadConfig {
    fn default() -> Self {
        Self {
            emfile_backoff_ms: 100,
            max_pending_streams: None,
        }
    }
}

impl OverloadConfig {
    pub fn resolve(&self, max_concurrent_streams: u32) -> crate::OverloadLimits {
        crate::OverloadLimits::resolve(max_concurrent_streams, self.max_pending_streams)
    }
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
    /// Cap on concurrent QUIC connections that have not yet completed
    /// authentication; incomings beyond it are refused before the handshake.
    /// Sized for the expected 2-3 clients per group plus reconnect bursts,
    /// not for the authenticated slot table.
    #[serde(default = "default_max_unauthenticated_connections")]
    pub max_unauthenticated_connections: usize,
    /// Maximum number of authenticated client connections held by the server registry.
    #[serde(default = "default_connection_registry_capacity")]
    pub connection_registry_capacity: usize,
    #[serde(default = "default_open_stream_timeout_ms")]
    pub open_stream_timeout_ms: u64,
    #[serde(default = "default_h2_single_authority")]
    pub h2_single_authority: bool,
    #[serde(default)]
    pub accept_workers: Option<usize>,
    #[serde(default)]
    pub overload: OverloadConfig,
}

fn default_login_timeout_secs() -> u64 {
    10
}
fn default_max_unauthenticated_connections() -> usize {
    64
}
fn default_connection_registry_capacity() -> usize {
    4096
}
fn default_open_stream_timeout_ms() -> u64 {
    5000
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
    pub client_group: GroupId,
    pub proxy_name: ProxyName,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VhostRuleDef {
    pub match_host: String,
    pub client_group: GroupId,
    pub proxy_name: ProxyName,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClientConfigs {
    #[serde(default)]
    pub groups: HashMap<GroupId, GroupConfig>,
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
                Env::prefixed("DUOTUNNEL_SERVER__")
                    .only(&[
                        "server.log_level",
                        "server.database_url",
                        "server.connection_registry_capacity",
                    ])
                    .split("__"),
            )
            .extract()?;
        cfg.validate()?;
        Ok(cfg)
    }
    pub fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();
        if self.server.tunnel_port == 0 {
            errors.push("server.tunnel_port must not be 0".into());
        }
        if self.server.login_timeout_secs == 0 {
            errors.push("server.login_timeout_secs must be >= 1".into());
        }
        if self.server.max_unauthenticated_connections == 0 {
            errors.push("server.max_unauthenticated_connections must be >= 1".into());
        }
        if self.server.connection_registry_capacity == 0 {
            errors.push("server.connection_registry_capacity must be >= 1".into());
        }
        if self.server.open_stream_timeout_ms == 0 {
            errors.push("server.open_stream_timeout_ms must be >= 1".into());
        }
        if let Err(error) = self.server.proxy_buffers.validate() {
            errors.push(format!("server.proxy_buffers: {error}"));
        }
        if matches!(self.server.quic.shards, Some(0)) {
            errors.push("server.quic.shards must be >= 1 when set".into());
        }
        if matches!(self.server.overload.max_pending_streams, Some(0)) {
            errors.push("server.overload.max_pending_streams must be >= 1 when set".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Server config validation failed:\n  - {}",
                errors.join("\n  - ")
            ))
        }
    }
}

// ── Conversion to RoutingData ─────────────────────────────────────────────────

/// Convert the routing sections of a parsed [`ServerConfigFile`] into a
/// [`RoutingData`] suitable for saving to any [`RuleStore`].
pub fn routing_data_from_server_config(cfg: &ServerConfigFile) -> RoutingData {
    routing_data_from_parts(&cfg.tunnel_management, &cfg.server_egress_upstream)
}

pub fn routing_data_from_routing_config(cfg: &RoutingConfigFile) -> RoutingData {
    routing_data_from_parts(&cfg.tunnel_management, &cfg.server_egress_upstream)
}

fn routing_data_from_parts(tm: &TunnelManagement, eg: &ServerEgressUpstream) -> RoutingData {
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
            match_host: canonicalize_egress_host(&r.match_host)
                .unwrap_or_else(|_| r.match_host.clone()),
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_routing_data_from_server_config_empty() {
        // Create an empty ServerConfigFile using default values for everything
        // Note: ServerConfigFile doesn't have a Default implementation, so we build it.
        let cfg = ServerConfigFile {
            server: ServerBasicConfig {
                tunnel_port: 8080,
                log_level: None,
                trace_enabled: false,
                database_url: "".to_string(),
                metrics_port: None,
                quic: Default::default(),
                tcp: Default::default(),
                http_pool: Default::default(),
                proxy_buffers: Default::default(),
                pki: Default::default(),
                login_timeout_secs: 10,
                open_stream_timeout_ms: 5000,
                h2_single_authority: true,
                accept_workers: None,
                overload: Default::default(),
                max_unauthenticated_connections: 64,
                connection_registry_capacity: 4096,
            },
            server_egress_upstream: Default::default(),
            tunnel_management: Default::default(),
        };

        let routing_data = routing_data_from_server_config(&cfg);

        assert!(routing_data.ingress_listeners.is_empty());
        assert!(routing_data.client_groups.is_empty());
        assert!(routing_data.egress_upstreams.is_empty());
        assert!(routing_data.egress_vhost_rules.is_empty());
        assert!(cfg.validate().is_ok());
        let mut invalid = cfg.clone();
        invalid.server.connection_registry_capacity = 0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_routing_data_from_server_config_populated() {
        let mut client_upstreams = HashMap::new();
        client_upstreams.insert(
            "client_up_1".to_string(),
            UpstreamDef {
                servers: vec![ServerDef {
                    address: "10.0.0.1:80".to_string(),
                    resolve: true,
                }],
                lb_policy: "round_robin".to_string(),
            },
        );

        let mut groups = HashMap::new();
        groups.insert(
            "group_a".to_string().into(),
            GroupConfig {
                config_version: "v1".to_string(),
                upstreams: client_upstreams,
            },
        );

        let mut egress_upstreams = HashMap::new();
        egress_upstreams.insert(
            "egress_up_1".to_string(),
            UpstreamDef {
                servers: vec![ServerDef {
                    address: "20.0.0.1:443".to_string(),
                    resolve: false,
                }],
                lb_policy: "least_conn".to_string(),
            },
        );

        let cfg = ServerConfigFile {
            server: ServerBasicConfig {
                tunnel_port: 8080,
                log_level: None,
                trace_enabled: false,
                database_url: "".to_string(),
                metrics_port: None,
                quic: Default::default(),
                tcp: Default::default(),
                http_pool: Default::default(),
                proxy_buffers: Default::default(),
                pki: Default::default(),
                login_timeout_secs: 10,
                open_stream_timeout_ms: 5000,
                h2_single_authority: true,
                accept_workers: None,
                overload: Default::default(),
                max_unauthenticated_connections: 64,
                connection_registry_capacity: 4096,
            },
            server_egress_upstream: ServerEgressUpstream {
                upstreams: egress_upstreams,
                rules: EgressRules {
                    vhost: vec![EgressHttpRule {
                        match_host: "Example.COM:443".to_string(),
                        action_upstream: "egress_up_1".to_string(),
                    }],
                },
            },
            tunnel_management: TunnelManagement {
                server_ingress_routing: IngressRouting {
                    listeners: vec![
                        IngressListenerDef {
                            port: 80,
                            mode: IngressModeDef::Http(HttpListenerDef {
                                vhost: vec![VhostRuleDef {
                                    match_host: "test.local".to_string(),
                                    client_group: "group_a".into(),
                                    proxy_name: "proxy_1".into(),
                                }],
                            }),
                        },
                        IngressListenerDef {
                            port: 443,
                            mode: IngressModeDef::Tcp(TcpListenerDef {
                                client_group: "group_b".into(),
                                proxy_name: "proxy_2".into(),
                            }),
                        },
                    ],
                },
                client_configs: ClientConfigs { groups },
            },
        };

        let routing_data = routing_data_from_server_config(&cfg);

        // Assert Ingress Listeners
        assert_eq!(routing_data.ingress_listeners.len(), 2);

        let http_listener = routing_data
            .ingress_listeners
            .iter()
            .find(|l| l.port == 80)
            .unwrap();
        assert_eq!(
            http_listener.id, 0,
            "IngressListener ID should be initialized to 0 for DB auto-assignment"
        );
        if let IngressListenerMode::Http { vhost } = &http_listener.mode {
            assert_eq!(vhost.len(), 1);
            assert_eq!(vhost[0].match_host, "test.local");
            assert_eq!(vhost[0].group_id, "group_a");
            assert_eq!(vhost[0].proxy_name, "proxy_1");
        } else {
            panic!("Expected Http mode");
        }

        let tcp_listener = routing_data
            .ingress_listeners
            .iter()
            .find(|l| l.port == 443)
            .unwrap();
        assert_eq!(
            tcp_listener.id, 0,
            "IngressListener ID should be initialized to 0 for DB auto-assignment"
        );
        if let IngressListenerMode::Tcp {
            group_id,
            proxy_name,
        } = &tcp_listener.mode
        {
            assert_eq!(group_id, "group_b");
            assert_eq!(proxy_name, "proxy_2");
        } else {
            panic!("Expected Tcp mode");
        }

        // Assert Client Groups
        assert_eq!(routing_data.client_groups.len(), 1);
        let group = &routing_data.client_groups[0];
        assert_eq!(group.group_id, "group_a");
        assert_eq!(group.config_version, "v1");
        assert_eq!(group.upstreams.len(), 1);
        let up = &group.upstreams[0];
        assert_eq!(up.name, "client_up_1");
        assert_eq!(up.lb_policy, "round_robin");
        assert_eq!(up.servers.len(), 1);
        assert_eq!(up.servers[0].address, "10.0.0.1:80");
        assert!(up.servers[0].resolve);

        // Assert Egress Upstreams
        assert_eq!(routing_data.egress_upstreams.len(), 1);
        let eup = &routing_data.egress_upstreams[0];
        assert_eq!(eup.name, "egress_up_1");
        assert_eq!(eup.lb_policy, "least_conn");
        assert_eq!(eup.servers.len(), 1);
        assert_eq!(eup.servers[0].address, "20.0.0.1:443");
        assert!(!eup.servers[0].resolve);

        // Assert Egress Vhost Rules
        assert_eq!(routing_data.egress_vhost_rules.len(), 1);
        let erule = &routing_data.egress_vhost_rules[0];
        assert_eq!(erule.match_host, "example.com");
        assert_eq!(erule.action_upstream, "egress_up_1");
    }

    #[test]
    fn test_tunnel_management_is_empty() {
        let mut tm = TunnelManagement::default();
        assert!(
            tm.is_empty(),
            "Expected empty TunnelManagement to return true for is_empty"
        );

        tm.server_ingress_routing
            .listeners
            .push(IngressListenerDef {
                port: 8080,
                mode: IngressModeDef::Http(HttpListenerDef::default()),
            });
        assert!(
            !tm.is_empty(),
            "Expected non-empty listeners to return false for is_empty"
        );

        let mut tm = TunnelManagement::default();
        tm.client_configs.groups.insert(
            "group1".to_string().into(),
            GroupConfig {
                config_version: "v1".to_string(),
                upstreams: HashMap::new(),
            },
        );
        assert!(
            !tm.is_empty(),
            "Expected non-empty groups to return false for is_empty"
        );

        tm.server_ingress_routing
            .listeners
            .push(IngressListenerDef {
                port: 8080,
                mode: IngressModeDef::Http(HttpListenerDef::default()),
            });
        assert!(
            !tm.is_empty(),
            "Expected both non-empty listeners and groups to return false for is_empty"
        );
    }
}
