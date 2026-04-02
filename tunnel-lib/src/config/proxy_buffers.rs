use crate::proxy::buffer_params::ProxyBufferParams;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyBufferConfig {
    pub peek_buf_size: usize,
    pub http_header_buf_size: usize,
    pub http_body_chunk_size: usize,
}
impl Default for ProxyBufferConfig {
    fn default() -> Self {
        Self {
            peek_buf_size: 16384,
            http_header_buf_size: 8192,
            http_body_chunk_size: 8192,
        }
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
