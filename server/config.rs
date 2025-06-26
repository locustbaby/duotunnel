use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub server: ServerSection,
    pub upstreams: HashMap<String, Upstream>,
    pub forward: ForwardSection,
    pub reverse_proxy: ReverseProxySection,
    pub client_groups: HashMap<String, ClientGroupConfig>,
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
}

 