use serde::{Deserialize, Serialize};
use crate::proxy::buffer_params::ProxyBufferParams;

/// Protocol detection and HTTP processing buffer sizes.
/// Nests under `server.proxy_buffers` or top-level `proxy_buffers` in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyBufferConfig {
    /// Peek buffer for protocol detection at entry listeners (bytes). Default: 16384.
    pub peek_buf_size: usize,
    /// HTTP header read buffer — requests with larger headers are rejected (bytes). Default: 8192.
    pub http_header_buf_size: usize,
    /// HTTP body streaming chunk size (bytes). Default: 8192.
    pub http_body_chunk_size: usize,
}

impl Default for ProxyBufferConfig {
    fn default() -> Self {
        Self { peek_buf_size: 16384, http_header_buf_size: 8192, http_body_chunk_size: 8192 }
    }
}

impl From<&ProxyBufferConfig> for ProxyBufferParams {
    fn from(c: &ProxyBufferConfig) -> Self {
        ProxyBufferParams {
            peek_buf_size: c.peek_buf_size,
            http_header_buf_size: c.http_header_buf_size,
            http_body_chunk_size: c.http_body_chunk_size,
        }
    }
}
