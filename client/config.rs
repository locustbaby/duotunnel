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
    /// Number of parallel QUIC connections to maintain (default: 1).
    /// Values > 1 allow bypassing single-connection congestion control limits.
    #[serde(default = "default_quic_connections")]
    pub quic_connections: u32,

    // --- QUIC transport tuning (all optional; defaults match previous hard-coded values) ---

    /// Per-stream receive window in MB (default: 1)
    #[serde(default)]
    pub quic_stream_window_mb: Option<u64>,
    /// Per-connection receive/send window in MB (default: 8)
    #[serde(default)]
    pub quic_connection_window_mb: Option<u64>,
    /// QUIC keep-alive interval in seconds (default: 20)
    #[serde(default)]
    pub quic_keepalive_secs: Option<u64>,
    /// QUIC idle timeout in seconds (default: 60)
    #[serde(default)]
    pub quic_idle_timeout_secs: Option<u64>,
    /// Congestion controller: "bbr" or omit for default NewReno
    #[serde(default)]
    pub quic_congestion: Option<String>,
}

fn default_max_streams() -> u32 {
    100
}

fn default_quic_connections() -> u32 {
    1
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
    /// Convert YAML QUIC fields into a `QuicTransportParams`, applying defaults for any
    /// field not explicitly set in the config file.
    pub fn quic_transport_params(&self) -> tunnel_lib::QuicTransportParams {
        let defaults = tunnel_lib::QuicTransportParams::default();
        tunnel_lib::QuicTransportParams {
            max_concurrent_streams: self.max_concurrent_streams,
            stream_receive_window_bytes: self.quic_stream_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(defaults.stream_receive_window_bytes),
            connection_receive_window_bytes: self.quic_connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(defaults.connection_receive_window_bytes),
            send_window_bytes: self.quic_connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(defaults.send_window_bytes),
            keepalive_secs: self.quic_keepalive_secs.unwrap_or(defaults.keepalive_secs),
            idle_timeout_secs: self.quic_idle_timeout_secs.unwrap_or(defaults.idle_timeout_secs),
            congestion: self.quic_congestion.clone().or(defaults.congestion),
        }
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
        let config: ClientConfigFile = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
}