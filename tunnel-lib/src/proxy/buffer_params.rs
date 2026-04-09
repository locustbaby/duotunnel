pub const DEFAULT_RELAY_BUF_SIZE: usize = 65536;
pub const MIN_RELAY_BUF_SIZE: usize = 4096;

pub fn normalize_relay_buf_size(value: usize) -> usize {
    value.max(MIN_RELAY_BUF_SIZE)
}

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
