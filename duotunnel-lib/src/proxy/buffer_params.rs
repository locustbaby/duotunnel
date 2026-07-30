pub const DEFAULT_RELAY_BUF_SIZE: usize = 65536;
pub const MIN_RELAY_BUF_SIZE: usize = 4096;
pub const MIN_PEEK_BUF_SIZE: usize = 1024;
pub const MAX_PEEK_BUF_SIZE: usize = 1024 * 1024;
pub const MIN_HTTP_HEADER_BUF_SIZE: usize = 1024;
pub const MAX_HTTP_HEADER_BUF_SIZE: usize = 64 * 1024;
pub const MIN_HTTP_BODY_CHUNK_SIZE: usize = 1024;
pub const MAX_HTTP_BODY_CHUNK_SIZE: usize = 1024 * 1024;
pub const MAX_RELAY_BUF_SIZE: usize = 4 * 1024 * 1024;

pub fn normalize_relay_buf_size(value: usize) -> usize {
    value.clamp(MIN_RELAY_BUF_SIZE, MAX_RELAY_BUF_SIZE)
}

pub fn normalize_peek_buf_size(value: usize) -> usize {
    value.clamp(MIN_PEEK_BUF_SIZE, MAX_PEEK_BUF_SIZE)
}

pub fn normalize_http_header_buf_size(value: usize) -> usize {
    value.clamp(MIN_HTTP_HEADER_BUF_SIZE, MAX_HTTP_HEADER_BUF_SIZE)
}

pub fn normalize_http_body_chunk_size(value: usize) -> usize {
    value.clamp(MIN_HTTP_BODY_CHUNK_SIZE, MAX_HTTP_BODY_CHUNK_SIZE)
}

#[derive(Debug, Clone)]
pub struct ProxyBufferParams {
    pub peek_buf_size: usize,
    pub http_header_buf_size: usize,
    pub http_body_chunk_size: usize,
    pub relay_buf_size: usize,
    pub sniff_timeout_ms: u64,
}
impl Default for ProxyBufferParams {
    fn default() -> Self {
        Self {
            peek_buf_size: 16384,
            http_header_buf_size: 8192,
            http_body_chunk_size: 8192,
            relay_buf_size: 65536,
            sniff_timeout_ms: 2500,
        }
    }
}

impl ProxyBufferParams {
    pub fn validate(&self) -> Result<(), String> {
        let checks = [
            (
                "peek_buf_size",
                self.peek_buf_size,
                MIN_PEEK_BUF_SIZE,
                MAX_PEEK_BUF_SIZE,
            ),
            (
                "http_header_buf_size",
                self.http_header_buf_size,
                MIN_HTTP_HEADER_BUF_SIZE,
                MAX_HTTP_HEADER_BUF_SIZE,
            ),
            (
                "http_body_chunk_size",
                self.http_body_chunk_size,
                MIN_HTTP_BODY_CHUNK_SIZE,
                MAX_HTTP_BODY_CHUNK_SIZE,
            ),
            (
                "relay_buf_size",
                self.relay_buf_size,
                MIN_RELAY_BUF_SIZE,
                MAX_RELAY_BUF_SIZE,
            ),
        ];
        for (name, value, min, max) in checks {
            if !(min..=max).contains(&value) {
                return Err(format!(
                    "{name} must be between {min} and {max}, got {value}"
                ));
            }
        }
        Ok(())
    }
}
