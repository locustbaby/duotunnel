/// Default relay read-buffer size. Matches `ProxyBufferConfig::relay_buf_size` default.
pub const DEFAULT_RELAY_BUF_SIZE: usize = 65536;

#[derive(Debug, Clone)]
pub struct ProxyBufferParams {
    pub peek_buf_size: usize,
    pub http_header_buf_size: usize,
    pub http_body_chunk_size: usize,
    pub relay_buf_size: usize,
}
impl Default for ProxyBufferParams {
    fn default() -> Self {
        Self {
            peek_buf_size: 16384,
            http_header_buf_size: 8192,
            http_body_chunk_size: 8192,
            relay_buf_size: 65536,
        }
    }
}
