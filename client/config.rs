use serde::Deserialize;
use anyhow::Result;
use figment::{Figment, providers::{Format, Yaml, Env}};
use tunnel_lib::config::{TcpConfig, HttpPoolConfig, ProxyBufferConfig};
use tunnel_lib::transport::quic::QuicTransportParams;

// ============================================================
// Client QUIC config (standalone — no serde flatten to keep
// figment env-var overrides working correctly)
//
// NOTE: Fields keepalive_secs, idle_timeout_secs, stream_window_mb,
// connection_window_mb, and congestion are intentionally duplicated
// from tunnel_lib::config::QuicConfig.  True struct composition via
// #[serde(flatten)] breaks figment's env-var layer, so both structs
// must be kept in sync manually.
// SYNC WITH: tunnel-lib/src/config/quic.rs → QuicConfig
// ============================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ClientQuicConfig {
    /// Number of parallel QUIC connections to the server. Default: 1.
    pub connections: u32,
    /// Max concurrent bidirectional streams accepted from the server. Default: 100.
    pub max_concurrent_streams: u32,
    /// Per-stream receive window in MB. Default: 1 MB.
    pub stream_window_mb: Option<u64>,
    /// Per-connection receive/send window in MB. Default: 8 MB.
    pub connection_window_mb: Option<u64>,
    /// Keep-alive interval in seconds. Default: 20.
    pub keepalive_secs: Option<u64>,
    /// Idle timeout in seconds. Default: 60.
    pub idle_timeout_secs: Option<u64>,
    /// Congestion controller: `"bbr"` or omit for NewReno.
    pub congestion: Option<String>,
}

impl Default for ClientQuicConfig {
    fn default() -> Self {
        Self {
            connections: 1,
            max_concurrent_streams: 100,
            stream_window_mb: None,
            connection_window_mb: None,
            keepalive_secs: None,
            idle_timeout_secs: None,
            congestion: None,
        }
    }
}

impl From<&ClientQuicConfig> for QuicTransportParams {
    fn from(c: &ClientQuicConfig) -> Self {
        let d = QuicTransportParams::default();
        QuicTransportParams {
            max_concurrent_streams: c.max_concurrent_streams,
            stream_receive_window_bytes: c.stream_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.stream_receive_window_bytes),
            connection_receive_window_bytes: c.connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.connection_receive_window_bytes),
            send_window_bytes: c.connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.send_window_bytes),
            keepalive_secs: c.keepalive_secs.unwrap_or(d.keepalive_secs),
            idle_timeout_secs: c.idle_timeout_secs.unwrap_or(d.idle_timeout_secs),
            congestion: c.congestion.clone().or(d.congestion),
        }
    }
}

// ============================================================
// Reconnect backoff config (client-only)
// ============================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ReconnectConfig {
    /// Initial retry delay after a connection failure (ms). Default: 1000.
    pub initial_delay_ms: u64,
    /// Maximum retry delay after repeated failures (ms). Default: 60000.
    pub max_delay_ms: u64,
    /// Grace period before reconnecting after a clean disconnect (ms). Default: 100.
    pub grace_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self { initial_delay_ms: 1000, max_delay_ms: 60_000, grace_ms: 100 }
    }
}

// ============================================================
// Top-level client config
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ClientConfigFile {
    pub server_addr: String,
    pub server_port: u16,
    #[serde(default = "default_client_id")]
    pub client_id: String,
    #[serde(default)]
    pub client_group_id: Option<String>,
    /// Auth token — supports env override: TUNNEL_CLIENT__AUTH_TOKEN
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub log_level: Option<String>,
    /// Enable tracing spans (verbose — development only). Not yet wired to tracing-subscriber.
    #[serde(default)]
    #[allow(dead_code)]
    pub trace_enabled: bool,
    #[serde(default)]
    pub http_entry_port: Option<u16>,
    /// Skip TLS certificate verification (INSECURE — dev only)
    #[serde(default)]
    pub tls_skip_verify: bool,
    #[serde(default)]
    pub tls_ca_cert: Option<String>,

    // --- Nested sub-configs ---

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
}

fn default_client_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("client-{:x}", now.as_nanos())
}

impl ClientConfigFile {
    /// Load configuration with Figment:
    /// - YAML file as base
    /// - Env vars TUNNEL_CLIENT__AUTH_TOKEN, TUNNEL_CLIENT__LOG_LEVEL,
    ///   TUNNEL_CLIENT__SERVER_ADDR, TUNNEL_CLIENT__SERVER_PORT for overrides
    pub fn load(path: &str) -> Result<Self> {
        let resolved = tunnel_lib::resolve_config_path(path)?;

        let config: ClientConfigFile = Figment::new()
            .merge(Yaml::file(&resolved))
            .merge(
                Env::prefixed("TUNNEL_CLIENT__")
                    .only(&["auth_token", "log_level", "server_addr", "server_port"])
                    .split("__"),
            )
            .extract()?;

        config.validate()?;
        Ok(config)
    }

    /// Validate semantic constraints that serde cannot enforce.
    /// Collects all errors and reports them in one message.
    fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        if self.server_addr.is_empty() {
            errors.push("server_addr is required".into());
        }
        if self.server_port == 0 {
            errors.push("server_port must not be 0".into());
        }
        if self.quic.connections == 0 {
            errors.push("quic.connections must be >= 1".into());
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Config validation failed:\n  - {}", errors.join("\n  - ")))
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
}
