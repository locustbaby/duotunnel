use crate::proxy::buffer_params::ProxyBufferParams;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyBufferConfig {
    pub peek_buf_size: usize,
    pub http_header_buf_size: usize,
    pub http_body_chunk_size: usize,
    /// Read buffer size for each direction of a relay (BufReader capacity).
    /// Larger values reduce syscall density at the cost of memory per stream.
    /// Default 65536 (64 KiB) matches nginx/envoy and halves syscalls vs 8 KiB.
    pub relay_buf_size: usize,
}
impl Default for ProxyBufferConfig {
    fn default() -> Self {
        Self {
            peek_buf_size: 16384,
            http_header_buf_size: 8192,
            http_body_chunk_size: 8192,
            relay_buf_size: 65536,
        }
    }
}
impl From<&ProxyBufferConfig> for ProxyBufferParams {
    fn from(c: &ProxyBufferConfig) -> Self {
        ProxyBufferParams {
            peek_buf_size: c.peek_buf_size,
            http_header_buf_size: c.http_header_buf_size,
            http_body_chunk_size: c.http_body_chunk_size,
            relay_buf_size: c.relay_buf_size,
        }
    }
}
