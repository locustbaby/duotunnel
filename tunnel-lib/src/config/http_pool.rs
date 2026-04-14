use crate::egress::http::HttpClientParams;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpPoolConfig {
    pub idle_timeout_secs: Option<u64>,
    pub max_idle_per_host: usize,
    pub tcp_keepalive_secs: Option<u64>,
}
impl Default for HttpPoolConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: None,
            max_idle_per_host: 128,
            tcp_keepalive_secs: Some(15),
        }
    }
}
impl From<&HttpPoolConfig> for HttpClientParams {
    fn from(c: &HttpPoolConfig) -> Self {
        HttpClientParams {
            pool_idle_timeout_secs: c.idle_timeout_secs,
            pool_max_idle_per_host: c.max_idle_per_host,
            tcp_keepalive_secs: c.tcp_keepalive_secs,
        }
    }
}
