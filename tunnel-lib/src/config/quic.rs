use serde::{Deserialize, Serialize};
use crate::transport::quic::QuicTransportParams;

/// QUIC transport tuning. Nests under `server.quic` in YAML.
/// Window sizes are expressed in MB for readability; converted to bytes via `From`.
///
/// NOTE: The transport fields (keepalive_secs, idle_timeout_secs, stream_window_mb,
/// connection_window_mb, congestion, max_concurrent_streams) are intentionally mirrored
/// in `client::config::ClientQuicConfig`.  `#[serde(flatten)]` cannot be used to share
/// them because it breaks figment's env-var layer.  When adding or renaming a transport
/// field, update both structs.
/// SYNC WITH: client/config.rs → ClientQuicConfig
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct QuicConfig {
    /// Max concurrent bidirectional streams. Default: 1000.
    pub max_concurrent_streams: Option<u32>,
    /// Per-stream receive window in MB. Default: 1 MB.
    pub stream_window_mb: Option<u64>,
    /// Per-connection receive/send window in MB. Default: 8 MB.
    pub connection_window_mb: Option<u64>,
    /// Keep-alive interval in seconds. Default: 20.
    pub keepalive_secs: Option<u64>,
    /// Idle timeout in seconds. Default: 60.
    pub idle_timeout_secs: Option<u64>,
    /// Congestion controller: `"bbr"` or omit for NewReno.
    pub congestion: Option<String>,
}


impl From<&QuicConfig> for QuicTransportParams {
    fn from(c: &QuicConfig) -> Self {
        let d = QuicTransportParams::default();
        QuicTransportParams {
            max_concurrent_streams: c.max_concurrent_streams.unwrap_or(d.max_concurrent_streams),
            stream_receive_window_bytes: c.stream_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.stream_receive_window_bytes),
            connection_receive_window_bytes: c.connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.connection_receive_window_bytes),
            send_window_bytes: c.connection_window_mb
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(d.send_window_bytes),
            keepalive_secs: c.keepalive_secs.unwrap_or(d.keepalive_secs),
            idle_timeout_secs: c.idle_timeout_secs.unwrap_or(d.idle_timeout_secs),
            congestion: c.congestion.clone().or(d.congestion),
        }
    }
}
