use serde::Deserialize;
use std::collections::HashMap;
use anyhow::Result;
use figment::{Figment, providers::{Format, Yaml, Env}};
use tunnel_lib::config::{TcpConfig, QuicConfig, HttpPoolConfig, ProxyBufferConfig};
use tunnel_lib::PkiParams;

// ============================================================
// Top-level file structure
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfigFile {
    pub server: ServerBasicConfig,
    #[serde(default)]
    pub server_egress_upstream: ServerEgressUpstream,
    #[serde(default)]
    pub tunnel_management: TunnelManagement,
}

// ============================================================
// SECTION 1 — System config (static after startup)
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ServerBasicConfig {
    pub tunnel_port: u16,
    #[serde(default)]
    pub entry_port: Option<u16>,
    #[serde(default)]
    pub log_level: Option<String>,
    /// Enable tracing spans (verbose — development only).
    #[serde(default)]
    pub trace_enabled: bool,
    /// Authentication tokens: group_id → token
    #[serde(default)]
    pub auth_tokens: HashMap<String, String>,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_max_connections")]
    pub max_tcp_connections: usize,
    #[serde(default)]
    pub metrics_port: Option<u16>,

    // --- Sub-configs (all optional; fall back to Default if section absent) ---

    #[serde(default)]
    pub quic: QuicConfig,
    #[serde(default)]
    pub tcp: TcpConfig,
    #[serde(default)]
    pub http_pool: HttpPoolConfig,
    #[serde(default)]
    pub proxy_buffers: ProxyBufferConfig,
    /// PKI / cert-cache tuning. `PkiParams` is used directly — no wrapper type.
    #[serde(default)]
    pub pki: PkiParams,
}

fn default_max_connections() -> usize { 10_000 }

// ============================================================
// SECTION 2 — Server egress upstream (hot-reloadable)
// ============================================================

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
    pub http: Vec<EgressHttpRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EgressHttpRule {
    pub match_host: String,
    pub action_upstream: String,
}

// ============================================================
// SECTION 3 — Tunnel management (hot-reloadable)
// ============================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TunnelManagement {
    #[serde(default)]
    pub server_ingress_routing: IngressRouting,
    #[serde(default)]
    pub client_configs: ClientConfigs,
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

fn default_lb_policy() -> String { "round_robin".to_string() }

#[derive(Debug, Clone, Deserialize)]
pub struct ServerDef {
    pub address: String,
    #[serde(default)]
    pub resolve: bool,
}

// ============================================================
// Loading + validation
// ============================================================

impl ServerConfigFile {
    /// Load configuration with Figment:
    /// 1. YAML file as base
    /// 2. Env vars with prefix `TUNNEL_SERVER__` for selected sensitive fields
    ///    (log_level, auth_tokens). Only fields with a matching env var are overridden.
    pub fn load(path: &str) -> Result<Self> {
        let resolved = tunnel_lib::resolve_config_path(path)?;

        let config: ServerConfigFile = Figment::new()
            .merge(Yaml::file(&resolved))
            .merge(
                Env::prefixed("TUNNEL_SERVER__")
                    .only(&["server.log_level", "server.auth_tokens"])
                    .split("__"),
            )
            .extract()?;

        config.validate()?;
        Ok(config)
    }

    /// Validate semantic constraints that serde cannot enforce.
    /// Collects all errors and reports them in one message.
    fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        if self.server.tunnel_port == 0 {
            errors.push("server.tunnel_port must not be 0".into());
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
                    "server_egress_upstream.upstreams.{}: must have at least one server", name
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
            Err(anyhow::anyhow!("Config validation failed:\n  - {}", errors.join("\n  - ")))
        }
    }

    /// Validate authentication token for a group (constant-time comparison).
    pub fn validate_token(&self, group_id: &str, token: &str) -> bool {
        use subtle::ConstantTimeEq;
        if self.server.auth_tokens.is_empty() {
            return true;
        }
        self.server.auth_tokens.get(group_id)
            .map(|expected| bool::from(expected.as_bytes().ct_eq(token.as_bytes())))
            .unwrap_or(false)
    }
}

/// Build the `ClientConfig` message sent to a client on login.
/// Takes only the hot-reloadable `TunnelManagement` so it can be called
/// with a freshly-loaded snapshot during hot reload.
pub fn build_client_config_for_group(
    tm: &TunnelManagement,
    group_id: &str,
) -> Option<tunnel_lib::ClientConfig> {
    let group_config = tm.client_configs.client_egress_routings.get(group_id)?;

    let proxies = collect_proxies_for_group(tm, group_id);

    let upstreams = group_config.upstreams.iter().map(|(name, def)| {
        tunnel_lib::UpstreamConfig {
            name: name.clone(),
            servers: def.servers.iter().map(|s| tunnel_lib::UpstreamServer {
                address: s.address.clone(),
                resolve: s.resolve,
            }).collect(),
            lb_policy: def.lb_policy.clone(),
        }
    }).collect();

    let rules: Vec<tunnel_lib::RuleConfig> = group_config.rules.vhost.iter().map(|r| {
        tunnel_lib::RuleConfig {
            rule_type: "vhost".to_string(),
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        }
    }).collect();

    Some(tunnel_lib::ClientConfig {
        config_version: group_config.config_version.clone(),
        proxies,
        upstreams,
        rules,
    })
}

fn collect_proxies_for_group(tm: &TunnelManagement, group_id: &str) -> Vec<tunnel_lib::ProxyConfig> {
    let mut proxies = Vec::new();

    for rule in &tm.server_ingress_routing.rules.vhost {
        if rule.action_client_group == group_id {
            let proxy_name = rule.action_proxy_name.clone()
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
