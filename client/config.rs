use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub log_level: String,
    pub trace_enabled: bool,
    pub server_addr: String,
    pub server_port: u16,
    pub client_group_id: String,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub http_entry_port: Option<u16>,
    #[serde(default)]
    pub grpc_entry_port: Option<u16>,
    #[serde(default)]
    pub wss_entry_port: Option<u16>,
    
    /// Tunnels where Client listens and forwards to Server -> Target
    #[serde(default)]
    pub forward_tunnels: Vec<ForwardTunnel>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForwardTunnel {
    pub local_port: u16,
    pub remote_target: String, // e.g., "google.com:80"
}

impl ClientConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ClientConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    /// Get full server address with port
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
    
    /// Get client_id (alias for client_group_id for compatibility)
    pub fn client_id(&self) -> String {
        self.client_group_id.clone()
    }
}