use crate::proxy::buffer_params::{
    normalize_http_body_chunk_size, normalize_http_header_buf_size, normalize_peek_buf_size,
    normalize_relay_buf_size, ProxyBufferParams,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyBufferConfig {
    pub peek_buf_size: usize,
    /// Initial and maximum header sniff/parser capacity for HTTP/1.1 paths.
    /// HTTP/2 uses its negotiated frame/header settings instead.
    pub http_header_buf_size: usize,
    /// Read chunk size for HTTP/1.1 request/response bodies. HTTP/2 does not
    /// consume this setting because its framing layer controls body delivery.
    pub http_body_chunk_size: usize,
    /// Read buffer size for each direction of a relay (BufReader capacity).
    /// Larger values reduce syscall density at the cost of memory per stream.
    /// Default 65536 (64 KiB) matches nginx/envoy and halves syscalls vs 8 KiB.
    pub relay_buf_size: usize,
    pub sniff_timeout_ms: u64,
}
impl Default for ProxyBufferConfig {
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
impl From<&ProxyBufferConfig> for ProxyBufferParams {
    fn from(c: &ProxyBufferConfig) -> Self {
        ProxyBufferParams {
            peek_buf_size: normalize_peek_buf_size(c.peek_buf_size),
            http_header_buf_size: normalize_http_header_buf_size(c.http_header_buf_size),
            http_body_chunk_size: normalize_http_body_chunk_size(c.http_body_chunk_size),
            relay_buf_size: normalize_relay_buf_size(c.relay_buf_size),
            sniff_timeout_ms: c.sniff_timeout_ms,
        }
    }
}

impl ProxyBufferConfig {
    pub fn validate(&self) -> Result<(), String> {
        ProxyBufferParams::from(self).validate().and_then(|_| {
            let values = [
                ("peek_buf_size", self.peek_buf_size),
                ("http_header_buf_size", self.http_header_buf_size),
                ("http_body_chunk_size", self.http_body_chunk_size),
                ("relay_buf_size", self.relay_buf_size),
            ];
            let normalized = ProxyBufferParams::from(self);
            for (name, value) in values {
                let normalized_value = match name {
                    "peek_buf_size" => normalized.peek_buf_size,
                    "http_header_buf_size" => normalized.http_header_buf_size,
                    "http_body_chunk_size" => normalized.http_body_chunk_size,
                    _ => normalized.relay_buf_size,
                };
                if value != normalized_value {
                    return Err(format!(
                        "{name} is outside the supported buffer range: {value}"
                    ));
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ProxyBufferConfig;

    #[test]
    fn default_buffer_config_is_valid() {
        assert!(ProxyBufferConfig::default().validate().is_ok());
    }

    #[test]
    fn buffer_config_rejects_values_outside_supported_range() {
        let config = ProxyBufferConfig {
            peek_buf_size: 0,
            ..ProxyBufferConfig::default()
        };
        assert!(config.validate().is_err());

        let config = ProxyBufferConfig {
            peek_buf_size: usize::MAX,
            ..ProxyBufferConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
