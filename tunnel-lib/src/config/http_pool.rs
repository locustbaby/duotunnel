use serde::{Deserialize, Serialize};
use crate::egress::http::HttpClientParams;

/// Outbound HTTP/HTTPS connection pool tuning.
/// Nests under `server.http_pool` or top-level `http_pool` in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpPoolConfig {
    /// Idle connection TTL before eviction (seconds). Default: 90.
    pub idle_timeout_secs: u64,
    /// Max idle connections kept per host. Default: 10.
    pub max_idle_per_host: usize,
}

impl Default for HttpPoolConfig {
    fn default() -> Self {
        Self { idle_timeout_secs: 90, max_idle_per_host: 10 }
    }
}

impl From<&HttpPoolConfig> for HttpClientParams {
    fn from(c: &HttpPoolConfig) -> Self {
        HttpClientParams {
            pool_idle_timeout_secs: c.idle_timeout_secs,
            pool_max_idle_per_host: c.max_idle_per_host,
        }
    }
}
