use anyhow::Result;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;
use tunnel_lib::config::{HttpPoolConfig, ProxyBufferConfig, TcpConfig};
use tunnel_lib::transport::quic::QuicTransportParams;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EntryConfig {
    pub port: Option<u16>,
    pub accept_workers: usize,
}

impl Default for EntryConfig {
    fn default() -> Self {
        Self {
            port: None,
            accept_workers: tunnel_lib::DEFAULT_ACCEPT_WORKERS,
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ClientQuicConfig {
    pub connections: u32,
    pub max_concurrent_streams: u32,
    pub stream_window_mb: Option<u64>,
    pub connection_window_mb: Option<u64>,
    /// Independent send-window override. Falls back to connection_window_mb, then default.
    /// Useful for asymmetric links where upload/download windows differ.
    pub send_window_mb: Option<u64>,
    pub keepalive_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
    pub congestion: Option<String>,
}
impl Default for ClientQuicConfig {
    fn default() -> Self {
        Self {
            connections: 1,
            max_concurrent_streams: 100,
            stream_window_mb: None,
            connection_window_mb: None,
            send_window_mb: None,
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
            stream_receive_window_bytes: c
                .stream_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.stream_receive_window_bytes),
            connection_receive_window_bytes: c
                .connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.connection_receive_window_bytes),
            // send_window_mb takes priority; falls back to connection_window_mb then default.
            send_window_bytes: c
                .send_window_mb
                .or(c.connection_window_mb)
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.send_window_bytes),
            keepalive_secs: c.keepalive_secs.unwrap_or(d.keepalive_secs),
            idle_timeout_secs: c.idle_timeout_secs.unwrap_or(d.idle_timeout_secs),
            congestion: c.congestion.clone().or(d.congestion),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OverloadMode {
    #[default]
    InflightSlowpath,
    Burst,
}

impl From<OverloadMode> for tunnel_lib::SharedOverloadMode {
    fn from(m: OverloadMode) -> Self {
        match m {
            OverloadMode::InflightSlowpath => tunnel_lib::SharedOverloadMode::InflightSlowpath,
            OverloadMode::Burst => tunnel_lib::SharedOverloadMode::Burst,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OverloadConfig {
    pub mode: OverloadMode,
    pub inflight_yield_threshold: usize,
    pub inflight_sleep_threshold: usize,
    pub inflight_sleep_ms: u64,
    /// When set, overrides `inflight_yield_threshold` as a fraction of
    /// `quic.max_concurrent_streams` (0.0 – 1.0). Preferred over absolute values.
    pub inflight_yield_pct: Option<f32>,
    /// When set, overrides `inflight_sleep_threshold` as a fraction of
    /// `quic.max_concurrent_streams` (0.0 – 1.0). Preferred over absolute values.
    pub inflight_sleep_pct: Option<f32>,
}

impl Default for OverloadConfig {
    fn default() -> Self {
        Self {
            mode: OverloadMode::InflightSlowpath,
            inflight_yield_threshold: 800,
            inflight_sleep_threshold: 950,
            inflight_sleep_ms: 2,
            inflight_yield_pct: Some(0.80),
            inflight_sleep_pct: Some(0.95),
        }
    }
}

impl OverloadConfig {
    pub fn resolve(&self, max_concurrent_streams: u32) -> tunnel_lib::OverloadLimits {
        tunnel_lib::OverloadLimits::resolve(
            self.mode.clone().into(),
            max_concurrent_streams,
            self.inflight_yield_threshold,
            self.inflight_sleep_threshold,
            self.inflight_yield_pct,
            self.inflight_sleep_pct,
            self.inflight_sleep_ms,
        )
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
    /// under backpressure. Defaults to 3000ms.
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
            login_timeout_ms: 5_000,
            startup_jitter_ms: 300,
            open_stream_timeout_ms: 5_000,
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
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
impl ClientConfigFile {
    pub fn load(path: &str) -> Result<Self> {
        let resolved = tunnel_lib::resolve_config_path(path)?;
        let config: ClientConfigFile = Figment::new()
            .merge(Yaml::file(&resolved))
            .merge(
                Env::prefixed("TUNNEL_CLIENT__")
                    .only(&[
                        "auth_token",
                        "log_level",
                        "server_addr",
                        "server_port",
                        "quic.connections",
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
        if self.proxy_buffers.relay_buf_size < tunnel_lib::proxy::buffer_params::MIN_RELAY_BUF_SIZE {
            errors.push(format!(
                "proxy_buffers.relay_buf_size ({}) must be >= {}",
                self.proxy_buffers.relay_buf_size,
                tunnel_lib::proxy::buffer_params::MIN_RELAY_BUF_SIZE
            ));
        }
        if let Some(name) = self.tls_server_name.as_ref() {
            if name.trim().is_empty() {
                errors.push("tls_server_name must not be empty when set".into());
            }
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
