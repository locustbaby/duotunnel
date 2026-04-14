use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;

// Re-export the canonical config types from tunnel-store so the rest of the
// server crate can keep importing from `crate::config`.
pub use tunnel_store::server_config::routing_data_from_server_config;
pub use tunnel_store::server_config::{
    ClientConfigs, EgressHttpRule, EgressRules, GroupConfig, HttpListenerDef as HttpListenerConfig,
    IngressListenerDef as IngressListener, IngressModeDef as IngressMode, IngressRouting,
    ServerConfigFile, ServerDef, ServerEgressUpstream, TcpListenerDef as TcpListenerConfig,
    TunnelManagement, UpstreamDef, VhostRuleDef as VhostRule,
};

// ── ConfigSource trait and implementations ────────────────────────────────────

#[async_trait]
pub trait ConfigSource: Send + Sync {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)>;
}

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
                    tunnel_store::IngressListenerMode::Tcp {
                        group_id,
                        proxy_name,
                    } => IngressMode::Tcp(TcpListenerConfig {
                        client_group: group_id,
                        proxy_name,
                    }),
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
                                    .map(|s| ServerDef {
                                        address: s.address,
                                        resolve: s.resolve,
                                    })
                                    .collect(),
                                lb_policy: u.lb_policy,
                            },
                        )
                    })
                    .collect();
                (
                    g.group_id,
                    GroupConfig {
                        config_version: g.config_version,
                        upstreams,
                    },
                )
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
                            .map(|s| ServerDef {
                                address: s.address,
                                resolve: s.resolve,
                            })
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
                    tracing::debug!("primary ConfigSource returned empty routing, using fallback");
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

// ── Seed routing from YAML into the DB ───────────────────────────────────────

pub async fn sync_file_to_db(
    cfg: &ServerConfigFile,
    rule_store: &dyn tunnel_store::RuleStore,
) -> Result<()> {
    let data = routing_data_from_server_config(cfg);
    rule_store.save_routing(&data).await
}

// ── Validation ───────────────────────────────────────────────────────────────

pub fn validate_server_config(cfg: &ServerConfigFile) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();

    if cfg.server.tunnel_port == 0 {
        errors.push("server.tunnel_port must not be 0".into());
    }
    if cfg.server.open_stream_timeout_ms == 0 {
        errors.push("server.open_stream_timeout_ms must be >= 1".into());
    }
    if cfg.server.proxy_buffers.relay_buf_size < tunnel_lib::proxy::buffer_params::MIN_RELAY_BUF_SIZE {
        errors.push(format!(
            "server.proxy_buffers.relay_buf_size ({}) must be >= {}",
            cfg.server.proxy_buffers.relay_buf_size,
            tunnel_lib::proxy::buffer_params::MIN_RELAY_BUF_SIZE
        ));
    }
    for (name, upstream) in &cfg.server_egress_upstream.upstreams {
        if upstream.servers.is_empty() {
            errors.push(format!(
                "server_egress_upstream.upstreams.{name}: must have at least one server"
            ));
        }
    }
    let groups = &cfg.tunnel_management.client_configs.groups;
    let mut seen_ports: HashSet<u16> = HashSet::new();
    for listener in &cfg.tunnel_management.server_ingress_routing.listeners {
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
            IngressMode::Http(h) => {
                let mut seen_hosts: HashSet<&str> = HashSet::new();
                for rule in &h.vhost {
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
                                "port {}: match_host \"{}\": proxy_name \"{}\" not found in group \"{}\"",
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
            IngressMode::Tcp(t) => {
                if let Some(group) = groups.get(&t.client_group) {
                    if !group.upstreams.contains_key(&t.proxy_name) {
                        errors.push(format!(
                            "port {}: tcp proxy_name \"{}\" not found in group \"{}\"",
                            listener.port, t.proxy_name, t.client_group
                        ));
                    }
                } else {
                    errors.push(format!(
                        "port {}: tcp group \"{}\" not found in client_configs",
                        listener.port, t.client_group
                    ));
                }
            }
        }
    }
    for (group_id, group) in groups {
        for (name, upstream) in &group.upstreams {
            if upstream.servers.is_empty() {
                errors.push(format!(
                    "client_configs.groups.{group_id}.upstreams.{name}: must have at least one server"
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

// ── Client config helper ──────────────────────────────────────────────────────

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
