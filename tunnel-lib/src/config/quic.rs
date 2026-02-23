use crate::transport::quic::QuicTransportParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct QuicConfig {
    pub max_concurrent_streams: Option<u32>,

    pub stream_window_mb: Option<u64>,

    pub connection_window_mb: Option<u64>,

    pub keepalive_secs: Option<u64>,

    pub idle_timeout_secs: Option<u64>,
    pub congestion: Option<String>,
}

impl From<&QuicConfig> for QuicTransportParams {
    fn from(c: &QuicConfig) -> Self {
        let d = QuicTransportParams::default();
        QuicTransportParams {
            max_concurrent_streams: c.max_concurrent_streams.unwrap_or(d.max_concurrent_streams),
            stream_receive_window_bytes: c
                .stream_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.stream_receive_window_bytes),
            connection_receive_window_bytes: c
                .connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.connection_receive_window_bytes),
            send_window_bytes: c
                .connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.send_window_bytes),
            keepalive_secs: c.keepalive_secs.unwrap_or(d.keepalive_secs),
            idle_timeout_secs: c.idle_timeout_secs.unwrap_or(d.idle_timeout_secs),
            congestion: c.congestion.clone().or(d.congestion),
        }
    }
}
