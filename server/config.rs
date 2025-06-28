use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub server: ServerSection,
    pub forward: ForwardSection,
    pub reverse_proxy: ReverseProxySection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSection {
    pub config_version: String,
    pub tunnel_port: u16,
    pub http_entry_port: u16,
    pub grpc_entry_port: u16,
    pub log_level: String,
    pub trace_enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Upstream {
    pub servers: Vec<ServerAddr>,
    pub lb_policy: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerAddr {
    pub address: String,
    pub resolve: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForwardSection {
    pub upstreams: HashMap<String, Upstream>,
    pub rules: ForwardRules,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ForwardRules {
    #[serde(default)]
    pub http: Vec<Rule>,
    #[serde(default)]
    pub grpc: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReverseProxySection {
    pub rules: ReverseProxyRules,
    pub client_groups: HashMap<String, ClientGroupConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ReverseProxyRules {
    #[serde(default)]
    pub http: Vec<Rule>,
    #[serde(default)]
    pub grpc: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub match_host: Option<String>,
    pub match_path_prefix: Option<String>,
    pub match_service: Option<String>,
    pub action_upstream: Option<String>,      // forward 用
    pub action_client_group: Option<String>,  // reverse_proxy 用
    pub action_set_host: Option<String>,      // 新增：支持 Host 头替换
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientGroupConfig {
    pub config_version: String,
    pub upstreams: HashMap<String, Upstream>,
    pub rules: ClientGroupRules,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ClientGroupRules {
    #[serde(default)]
    pub http: Vec<Rule>,
    #[serde(default)]
    pub grpc: Vec<Rule>,
}

impl ServerConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 校验所有 http 规则，保证同一 host 只能有唯一规则，不能路由到多个 backend/client group。
    pub fn validate_rules(&self) -> anyhow::Result<()> {
        use std::collections::HashSet;
        // forward.rules.http
        let mut forward_hosts = HashSet::new();
        for rule in &self.forward.rules.http {
            let host = rule.match_host.clone().unwrap_or_default();
            if !host.is_empty() && !forward_hosts.insert(host.clone()) {
                anyhow::bail!("Duplicate match_host in forward.rules.http: {}", host);
            }
        }
        // reverse_proxy.rules.http
        let mut reverse_hosts = HashSet::new();
        for rule in &self.reverse_proxy.rules.http {
            let host = rule.match_host.clone().unwrap_or_default();
            if !host.is_empty() && !reverse_hosts.insert(host.clone()) {
                anyhow::bail!("Duplicate match_host in reverse_proxy.rules.http: {}", host);
            }
        }
        // forward 和 reverse_proxy 不能有相同 host
        for host in &forward_hosts {
            if reverse_hosts.contains(host) {
                anyhow::bail!("match_host '{}' appears in both forward and reverse_proxy rules", host);
            }
        }
        // client_groups
        for (group, group_cfg) in &self.reverse_proxy.client_groups {
            let mut group_hosts = HashSet::new();
            for rule in &group_cfg.rules.http {
                let host = rule.match_host.clone().unwrap_or_default();
                if !host.is_empty() && !group_hosts.insert(host.clone()) {
                    anyhow::bail!("Duplicate match_host in client_group '{}': {}", group, host);
                }
            }
        }
        Ok(())
    }
}

 