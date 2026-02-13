use serde::Deserialize;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfigFile {
    pub server: ServerBasicConfig,
    #[serde(default)]
    pub server_egress_upstream: ServerEgressUpstream,
    #[serde(default)]
    pub tunnel_management: TunnelManagement,
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
    pub http: Vec<EgressHttpRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EgressHttpRule {
    pub match_host: String,
    pub action_upstream: String,
}



#[derive(Debug, Clone, Deserialize)]
pub struct ServerBasicConfig {
    pub tunnel_port: u16,
    #[serde(default)]
    pub entry_port: Option<u16>,
    #[serde(default)]
    pub log_level: Option<String>,
    /// Authentication tokens: group_id -> token
    #[serde(default)]
    pub auth_tokens: HashMap<String, String>,
    /// Maximum concurrent QUIC connections (default: 10000)
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    /// Maximum concurrent TCP connections per listener (default: 10000)
    #[serde(default = "default_max_connections")]
    pub max_tcp_connections: usize,
    /// Prometheus metrics port (optional)
    #[serde(default)]
    pub metrics_port: Option<u16>,
}

fn default_max_connections() -> usize {
    10000
}

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
    /// Validate authentication token for a group
    pub fn validate_token(&self, group_id: &str, token: &str) -> bool {
        // If no tokens configured, allow all (backward compatible)
        if self.server.auth_tokens.is_empty() {
            return true;
        }
        // Check if token matches for the group
        self.server.auth_tokens.get(group_id)
            .map(|expected| expected == token)
            .unwrap_or(false)
    }

    pub fn load(path: &str) -> Result<Self> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                // Try looking in parent directory (useful when running from crate dir)
                std::fs::read_to_string(format!("../{}", path))
                    .map_err(|_| anyhow::anyhow!("Config file not found: {} (checked ./ and ../)", path))?
            }
        };
        let config: ServerConfigFile = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn get_group_config(&self, group_id: &str) -> Option<&GroupConfig> {
        self.tunnel_management.client_configs.client_egress_routings.get(group_id)
    }

    pub fn to_client_config(&self, group_id: &str) -> Option<tunnel_lib::ClientConfig> {
        let group_config = self.get_group_config(group_id)?;
        
        let proxies = self.get_proxies_for_group(group_id);
        
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

    fn get_proxies_for_group(&self, group_id: &str) -> Vec<tunnel_lib::ProxyConfig> {
        let mut proxies = Vec::new();

        for rule in &self.tunnel_management.server_ingress_routing.rules.vhost {
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

        for rule in &self.tunnel_management.server_ingress_routing.rules.tcp {
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
}