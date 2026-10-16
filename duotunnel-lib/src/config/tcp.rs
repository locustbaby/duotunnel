use crate::transport::tcp_params::TcpParams;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TcpConfig {
    pub nodelay: bool,
    pub recv_buf_size: Option<u32>,
    pub send_buf_size: Option<u32>,
    pub keepalive: bool,
    pub user_timeout_ms: u32,
    pub connect_timeout_ms: u64,
    pub tls_handshake_timeout_ms: u64,
}
impl Default for TcpConfig {
    fn default() -> Self {
        let defaults = TcpParams::default();
        Self {
            nodelay: defaults.nodelay,
            recv_buf_size: defaults.recv_buf_size,
            send_buf_size: defaults.send_buf_size,
            keepalive: defaults.keepalive,
            user_timeout_ms: defaults.user_timeout_ms,
            connect_timeout_ms: defaults.connect_timeout_ms,
            tls_handshake_timeout_ms: defaults.tls_handshake_timeout_ms,
        }
    }
}
impl From<&TcpConfig> for TcpParams {
    fn from(c: &TcpConfig) -> Self {
        TcpParams {
            nodelay: c.nodelay,
            recv_buf_size: c.recv_buf_size,
            send_buf_size: c.send_buf_size,
            keepalive: c.keepalive,
            user_timeout_ms: c.user_timeout_ms,
            connect_timeout_ms: c.connect_timeout_ms,
            tls_handshake_timeout_ms: c.tls_handshake_timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_leaves_socket_buffers_to_os_autotuning() {
        let params = TcpParams::default();
        assert_eq!(params.recv_buf_size, None);
        assert_eq!(params.send_buf_size, None);

        let cfg = TcpConfig::default();
        assert_eq!(cfg.recv_buf_size, None);
        assert_eq!(cfg.send_buf_size, None);
    }

    #[test]
    fn explicit_buffer_sizes_propagate_from_config() {
        let cfg = TcpConfig {
            recv_buf_size: Some(256 * 1024),
            send_buf_size: Some(512 * 1024),
            ..TcpConfig::default()
        };
        let params = TcpParams::from(&cfg);
        assert_eq!(params.recv_buf_size, Some(256 * 1024));
        assert_eq!(params.send_buf_size, Some(512 * 1024));
    }
}
