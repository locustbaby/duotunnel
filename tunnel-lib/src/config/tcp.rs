use serde::{Deserialize, Serialize};
use crate::transport::tcp_params::TcpParams;

/// TCP socket options applied to accepted/connected streams.
/// Nests under `server.tcp` or top-level `tcp` in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TcpConfig {
    /// Disable Nagle's algorithm (TCP_NODELAY). Default: true.
    pub nodelay: bool,
    /// SO_RCVBUF in bytes. `null` = OS default.
    pub recv_buf_size: Option<u32>,
    /// SO_SNDBUF in bytes. `null` = OS default.
    pub send_buf_size: Option<u32>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self { nodelay: true, recv_buf_size: None, send_buf_size: None }
    }
}

impl From<&TcpConfig> for TcpParams {
    fn from(c: &TcpConfig) -> Self {
        TcpParams {
            nodelay: c.nodelay,
            recv_buf_size: c.recv_buf_size,
            send_buf_size: c.send_buf_size,
        }
    }
}
