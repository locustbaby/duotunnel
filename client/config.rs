use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub log_level: String,
    pub server_addr: String,
    pub server_port: u16,
    pub client_group_id: String,
    #[serde(default)]
    pub http_entry_port: Option<u16>,
    #[serde(default)]
    pub grpc_entry_port: Option<u16>,
    #[serde(default)]
    pub wss_entry_port: Option<u16>,
}

impl ClientConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ClientConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
    

    pub fn client_id(&self) -> String {
        self.client_group_id.clone()
    }
}