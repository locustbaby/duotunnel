use crate::transport::quic::QuicTransportParams;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct QuicConfig {
    pub shards: Option<usize>,
    pub max_concurrent_streams: Option<u32>,
    pub stream_window_mb: Option<u64>,
    pub connection_window_mb: Option<u64>,
    pub send_window_mb: Option<u64>,
    pub keepalive_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
    pub congestion: Option<String>,
    pub udp_recv_buf_mb: Option<u32>,
    pub udp_send_buf_mb: Option<u32>,
}
impl From<&QuicConfig> for QuicTransportParams {
    fn from(c: &QuicConfig) -> Self {
        let d = QuicTransportParams::default();
        QuicTransportParams {
            max_concurrent_streams: c.max_concurrent_streams.unwrap_or(d.max_concurrent_streams),
            stream_receive_window_bytes: c
                .stream_window_mb
                .map(|mb| mb.saturating_mul(1024 * 1024))
                .unwrap_or(d.stream_receive_window_bytes),
            connection_receive_window_bytes: c
                .connection_window_mb
                .map(|mb| mb.saturating_mul(1024 * 1024))
                .unwrap_or(d.connection_receive_window_bytes),
            send_window_bytes: c
                .send_window_mb
                .map(|mb| mb.saturating_mul(1024 * 1024))
                .unwrap_or(d.send_window_bytes),
            keepalive_secs: c.keepalive_secs.unwrap_or(d.keepalive_secs),
            idle_timeout_secs: c.idle_timeout_secs.unwrap_or(d.idle_timeout_secs),
            congestion: c.congestion.clone().or(d.congestion),
            udp_recv_buf_bytes: c
                .udp_recv_buf_mb
                .map(|mb| (mb as usize).saturating_mul(1024 * 1024))
                .unwrap_or(d.udp_recv_buf_bytes),
            udp_send_buf_bytes: c
                .udp_send_buf_mb
                .map(|mb| (mb as usize).saturating_mul(1024 * 1024))
                .unwrap_or(d.udp_send_buf_bytes),
        }
    }
}
