use crate::egress::http::HttpClientParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpPoolConfig {
    pub idle_timeout_secs: u64,

    pub max_idle_per_host: usize,
}

impl Default for HttpPoolConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 90,
            max_idle_per_host: 10,
        }
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
