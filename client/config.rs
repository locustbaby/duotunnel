use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct ClientConfigFile {
    pub server_addr: String,
    pub server_port: u16,
    #[serde(default = "default_client_id")]
    pub client_id: String,
    #[serde(default)]
    pub client_group_id: Option<String>,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub log_level: Option<String>,
    #[serde(default)]
    pub http_entry_port: Option<u16>,
    /// Skip TLS certificate verification (INSECURE - use only for development)
    #[serde(default)]
    pub tls_skip_verify: bool,
    /// Path to CA certificate file for server verification
    #[serde(default)]
    pub tls_ca_cert: Option<String>,
    /// Maximum concurrent bidirectional QUIC streams (default: 100)
    #[serde(default = "default_max_streams")]
    pub max_concurrent_streams: u32,
}

fn default_max_streams() -> u32 {
    100
}

fn default_client_id() -> String {
    format!("client-{}", uuid_simple())
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{:x}", now.as_nanos())
}

impl ClientConfigFile {
    pub fn load(path: &str) -> Result<Self> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                // Try looking in parent directory (useful when running from crate dir)
                std::fs::read_to_string(format!("../{}", path))
                    .map_err(|_| anyhow::anyhow!("Config file not found: {} (checked ./ and ../)", path))?
            }
        };
        let config: ClientConfigFile = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
}