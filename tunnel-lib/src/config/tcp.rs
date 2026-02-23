use crate::transport::tcp_params::TcpParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TcpConfig {
    pub nodelay: bool,

    pub recv_buf_size: Option<u32>,

    pub send_buf_size: Option<u32>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            nodelay: true,
            recv_buf_size: None,
            send_buf_size: None,
        }
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
