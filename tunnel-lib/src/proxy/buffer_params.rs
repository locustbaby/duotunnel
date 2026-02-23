#[derive(Debug, Clone)]
pub struct ProxyBufferParams {
    pub peek_buf_size: usize,

    pub http_header_buf_size: usize,

    pub http_body_chunk_size: usize,
}

impl Default for ProxyBufferParams {
    fn default() -> Self {
        Self {
            peek_buf_size: 16384,
            http_header_buf_size: 8192,
            http_body_chunk_size: 8192,
        }
    }
}
