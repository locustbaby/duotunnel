use anyhow::Result;
use duotunnel_lib::config::{HttpPoolConfig, ProxyBufferConfig, TcpConfig};
use duotunnel_lib::transport::quic::QuicTransportParams;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct EntryConfig {
    pub port: Option<u16>,
    pub accept_workers: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UdpEntryConfig {
    pub port: u16,
    pub proxy_name: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ClientQuicConfig {
    pub connections: u32,
    pub min_ready_tunnels: u32,
    pub shards: Option<usize>,
    pub max_concurrent_streams: u32,
    pub stream_window_mb: Option<u64>,
    pub connection_window_mb: Option<u64>,
    pub send_window_mb: Option<u64>,
    pub keepalive_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
    pub congestion: Option<String>,
    pub udp_recv_buf_mb: Option<u32>,
    pub udp_send_buf_mb: Option<u32>,
}
impl Default for ClientQuicConfig {
    fn default() -> Self {
        Self {
            connections: 0,
            min_ready_tunnels: 1,
            shards: None,
            max_concurrent_streams: 1000,
            stream_window_mb: None,
            connection_window_mb: None,
            send_window_mb: None,
            keepalive_secs: None,
            idle_timeout_secs: None,
            congestion: None,
            udp_recv_buf_mb: None,
            udp_send_buf_mb: None,
        }
    }
}
impl From<&ClientQuicConfig> for QuicTransportParams {
    fn from(c: &ClientQuicConfig) -> Self {
        let d = QuicTransportParams::default();
        QuicTransportParams {
            max_concurrent_streams: c.max_concurrent_streams,
            stream_receive_window_bytes: c
                .stream_window_mb
                .map(|mb| mb.saturating_mul(1024 * 1024))
                .unwrap_or(d.stream_receive_window_bytes),
            connection_receive_window_bytes: c
                .connection_window_mb
                .map(|mb| mb.saturating_mul(1024 * 1024))
                .unwrap_or(d.connection_receive_window_bytes),
            send_window_bytes: c
                .send_window_mb
                .map(|mb| mb.saturating_mul(1024 * 1024))
                .unwrap_or(d.send_window_bytes),
            keepalive_secs: c.keepalive_secs.unwrap_or(d.keepalive_secs),
            idle_timeout_secs: c.idle_timeout_secs.unwrap_or(d.idle_timeout_secs),
            congestion: c.congestion.clone().or(d.congestion),
            udp_recv_buf_bytes: c
                .udp_recv_buf_mb
                .map(|mb| (mb as usize).saturating_mul(1024 * 1024))
                .unwrap_or(d.udp_recv_buf_bytes),
            udp_send_buf_bytes: c
                .udp_send_buf_mb
                .map(|mb| (mb as usize).saturating_mul(1024 * 1024))
                .unwrap_or(d.udp_send_buf_bytes),
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OverloadConfig {
    pub max_pending_streams: Option<usize>,
}

impl Default for OverloadConfig {
    fn default() -> Self {
        Self {
            max_pending_streams: None,
        }
    }
}

impl OverloadConfig {
    pub fn resolve(&self, max_concurrent_streams: u32) -> duotunnel_lib::OverloadLimits {
        duotunnel_lib::OverloadLimits::resolve(max_concurrent_streams, self.max_pending_streams)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ReconnectConfig {
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub grace_ms: u64,
    pub connect_timeout_ms: u64,
    pub resolve_timeout_ms: u64,
    pub login_timeout_ms: u64,
    pub startup_jitter_ms: u64,
    /// Timeout for `open_bi()` in the entry listener (waiting for a QUIC stream slot).
    /// Separate from login_timeout_ms — stream acquisition can legitimately take longer
    /// under backpressure. Defaults to 5000ms.
    pub open_stream_timeout_ms: u64,
}
impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 60_000,
            grace_ms: 100,
            connect_timeout_ms: 10_000,
            resolve_timeout_ms: 5_000,
            login_timeout_ms: 10_000,
            startup_jitter_ms: 300,
            open_stream_timeout_ms: 5_000,
        }
    }
}
#[derive(Clone, Deserialize)]
pub struct ClientConfigFile {
    pub server_addr: String,
    pub server_port: u16,
    pub auth_token: String,
    #[serde(default)]
    pub log_level: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub trace_enabled: bool,
    #[serde(default)]
    pub entry: EntryConfig,
    #[serde(default)]
    pub udp_entries: Vec<UdpEntryConfig>,
    #[serde(default)]
    pub metrics_port: Option<u16>,
    #[serde(default)]
    pub tls_skip_verify: bool,
    #[serde(default)]
    pub tls_ca_cert: Option<String>,
    #[serde(default)]
    pub tls_server_name: Option<String>,
    #[serde(default)]
    pub allow_insecure_fallback: bool,
    #[serde(default)]
    pub quic: ClientQuicConfig,
    #[serde(default)]
    pub tcp: TcpConfig,
    #[serde(default)]
    pub http_pool: HttpPoolConfig,
    #[serde(default)]
    pub proxy_buffers: ProxyBufferConfig,
    #[serde(default)]
    pub reconnect: ReconnectConfig,
    #[serde(default)]
    pub overload: OverloadConfig,
}

impl std::fmt::Debug for ClientConfigFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let masked_token = if self.auth_token.starts_with("dt_") && self.auth_token.len() >= 10 {
            "dt_masked_...".to_string()
        } else {
            "***".to_string()
        };
        f.debug_struct("ClientConfigFile")
            .field("server_addr", &self.server_addr)
            .field("server_port", &self.server_port)
            .field("auth_token", &masked_token)
            .field("log_level", &self.log_level)
            .field("trace_enabled", &self.trace_enabled)
            .field("entry", &self.entry)
            .field("udp_entries", &self.udp_entries)
            .field("metrics_port", &self.metrics_port)
            .field("tls_skip_verify", &self.tls_skip_verify)
            .field("tls_ca_cert", &self.tls_ca_cert)
            .field("tls_server_name", &self.tls_server_name)
            .field("allow_insecure_fallback", &self.allow_insecure_fallback)
            .field("quic", &self.quic)
            .field("tcp", &self.tcp)
            .field("http_pool", &self.http_pool)
            .field("proxy_buffers", &self.proxy_buffers)
            .field("reconnect", &self.reconnect)
            .field("overload", &self.overload)
            .finish()
    }
}
impl ClientConfigFile {
    pub fn load(path: &str) -> Result<Self> {
        let resolved = duotunnel_lib::resolve_config_path(path)?;
        let config: ClientConfigFile = Figment::new()
            .merge(Yaml::file(&resolved))
            .merge(
                Env::prefixed("DUOTUNNEL_CLIENT__")
                    .only(&[
                        "auth_token",
                        "log_level",
                        "server_addr",
                        "server_port",
                        "quic.connections",
                        "quic.min_ready_tunnels",
                        "quic.shards",
                    ])
                    .split("__"),
            )
            .extract()?;
        config.validate()?;
        Ok(config)
    }
    fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();
        if self.server_addr.trim().is_empty() {
            errors.push("server_addr is required".into());
        }
        if self.server_port == 0 {
            errors.push("server_port must not be 0".into());
        }
        if self.auth_token.trim().is_empty() {
            errors.push("auth_token is required".into());
        }
        let mut udp_ports = HashSet::new();
        let mut udp_proxy_names = HashSet::new();
        for entry in &self.udp_entries {
            if entry.port == 0 {
                errors.push("udp_entries[].port must be >= 1".into());
            }
            if entry.proxy_name.trim().is_empty() {
                errors.push("udp_entries[].proxy_name is required".into());
            }
            if !udp_ports.insert(entry.port) {
                errors.push(format!("udp_entries has duplicate port {}", entry.port));
            }
            if !udp_proxy_names.insert(entry.proxy_name.trim().to_string()) {
                errors.push(format!(
                    "udp_entries has duplicate proxy_name {}",
                    entry.proxy_name.trim()
                ));
            }
        }
        if matches!(self.quic.shards, Some(0)) {
            errors.push("quic.shards must be >= 1 when set".into());
        }
        if self.quic.min_ready_tunnels == 0 {
            errors.push("quic.min_ready_tunnels must be >= 1".into());
        }
        if self.quic.connections > 0 && self.quic.min_ready_tunnels > self.quic.connections {
            errors.push(format!(
                "quic.min_ready_tunnels ({}) must be <= quic.connections ({})",
                self.quic.min_ready_tunnels, self.quic.connections
            ));
        }
        if self.quic.max_concurrent_streams == 0 {
            errors.push("quic.max_concurrent_streams must be >= 1".into());
        }
        if self.reconnect.initial_delay_ms > self.reconnect.max_delay_ms {
            errors.push(format!(
                "reconnect.initial_delay_ms ({}) must be <= max_delay_ms ({})",
                self.reconnect.initial_delay_ms, self.reconnect.max_delay_ms
            ));
        }
        if self.reconnect.connect_timeout_ms == 0 {
            errors.push("reconnect.connect_timeout_ms must be >= 1".into());
        }
        if self.reconnect.resolve_timeout_ms == 0 {
            errors.push("reconnect.resolve_timeout_ms must be >= 1".into());
        }
        if self.reconnect.login_timeout_ms == 0 {
            errors.push("reconnect.login_timeout_ms must be >= 1".into());
        }
        if self.reconnect.open_stream_timeout_ms == 0 {
            errors.push("reconnect.open_stream_timeout_ms must be >= 1".into());
        }
        if self.proxy_buffers.relay_buf_size
            < duotunnel_lib::proxy::buffer_params::MIN_RELAY_BUF_SIZE
        {
            errors.push(format!(
                "proxy_buffers.relay_buf_size ({}) must be >= {}",
                self.proxy_buffers.relay_buf_size,
                duotunnel_lib::proxy::buffer_params::MIN_RELAY_BUF_SIZE
            ));
        }
        if let Err(error) = self.proxy_buffers.validate() {
            errors.push(format!("proxy_buffers: {error}"));
        }
        if let Some(name) = self.tls_server_name.as_ref() {
            if name.trim().is_empty() {
                errors.push("tls_server_name must not be empty when set".into());
            }
        }
        if matches!(self.overload.max_pending_streams, Some(0)) {
            errors.push("overload.max_pending_streams must be >= 1 when set".into());
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
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
    pub fn tls_server_name(&self) -> &str {
        self.tls_server_name
            .as_deref()
            .map(str::trim)
            .unwrap_or_else(|| self.server_addr.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_config(
        server_addr: &str,
        server_port: u16,
        tls_server_name: Option<String>,
    ) -> ClientConfigFile {
        ClientConfigFile {
            server_addr: server_addr.to_string(),
            server_port,
            auth_token: "test_token".to_string(),
            log_level: None,
            trace_enabled: false,
            entry: Default::default(),
            udp_entries: Vec::new(),
            metrics_port: None,
            tls_skip_verify: false,
            tls_ca_cert: None,
            tls_server_name,
            allow_insecure_fallback: false,
            quic: Default::default(),
            tcp: Default::default(),
            http_pool: Default::default(),
            proxy_buffers: Default::default(),
            reconnect: Default::default(),
            overload: Default::default(),
        }
    }

    #[test]
    fn test_server_address() {
        let config = create_mock_config("example.com", 443, None);
        assert_eq!(config.server_address(), "example.com:443");

        let config_ip = create_mock_config("127.0.0.1", 8080, None);
        assert_eq!(config_ip.server_address(), "127.0.0.1:8080");

        let config_ipv6 = create_mock_config("[::1]", 8443, None);
        assert_eq!(config_ipv6.server_address(), "[::1]:8443");
    }

    #[test]
    fn test_tls_server_name() {
        // Fallback to server_addr when tls_server_name is None
        let config = create_mock_config("example.com", 443, None);
        assert_eq!(config.tls_server_name(), "example.com");

        // Should trim server_addr when falling back
        let config_space = create_mock_config(" example.com ", 443, None);
        assert_eq!(config_space.tls_server_name(), "example.com");

        // Use tls_server_name when provided
        let config_override =
            create_mock_config("example.com", 443, Some("tls.example.com".to_string()));
        assert_eq!(config_override.tls_server_name(), "tls.example.com");

        // Should trim tls_server_name when provided
        let config_override_space =
            create_mock_config("example.com", 443, Some(" tls.example.com ".to_string()));
        assert_eq!(config_override_space.tls_server_name(), "tls.example.com");
    }

    #[test]
    fn test_tls_server_name_returns_explicit_name() {
        let config = create_mock_config("example.com", 443, Some("custom.example.com".to_string()));
        assert_eq!(config.tls_server_name(), "custom.example.com");
    }

    #[test]
    fn test_tls_server_name_trims_explicit_name() {
        let config = create_mock_config(
            "example.com",
            443,
            Some("  custom.example.com  ".to_string()),
        );
        assert_eq!(config.tls_server_name(), "custom.example.com");
    }

    #[test]
    fn test_tls_server_name_falls_back_to_server_addr() {
        let config = create_mock_config("example.com", 443, None);
        assert_eq!(config.tls_server_name(), "example.com");
    }

    #[test]
    fn test_tls_server_name_trims_fallback_server_addr() {
        let config = create_mock_config("  example.com  ", 443, None);
        assert_eq!(config.tls_server_name(), "example.com");
    }

    #[test]
    fn test_udp_entries_validate_unique_proxy_name() {
        let mut config = create_mock_config("example.com", 443, None);
        config.udp_entries = vec![
            UdpEntryConfig {
                port: 5353,
                proxy_name: "dns".to_string(),
            },
            UdpEntryConfig {
                port: 5354,
                proxy_name: "dns".to_string(),
            },
        ];
        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("duplicate proxy_name"));
    }
}
