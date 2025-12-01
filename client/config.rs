use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub client_group_id: String,
    pub auth_token: Option<String>,
    pub http_entry_port: Option<u16>,
    pub grpc_entry_port: Option<u16>,
    pub upstreams: Option<HashMap<String, Upstream>>,
    pub log_level: String,
    pub trace_enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Upstream {
    pub servers: Vec<ServerAddr>,
    pub lb_policy: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ServerAddr {
    pub address: String,
    pub resolve: bool,
}

impl ClientConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ClientConfig = toml::from_str(&content)?;
        Ok(config)
    }
} 