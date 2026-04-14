pub mod http;
pub mod metrics;
pub mod quic;
pub mod tcp;

use crate::registry::SelectedConnection;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tunnel_store::server_config::{OverloadConfig, OverloadMode};

pub async fn maybe_slow_path(selected: &SelectedConnection, cfg: &OverloadConfig) {
    if cfg.mode == OverloadMode::Burst {
        return;
    }
    let inflight = selected.inflight.load(Ordering::Relaxed);
    if inflight >= cfg.inflight_sleep_threshold {
        tokio::time::sleep(Duration::from_millis(cfg.inflight_sleep_ms)).await;
    } else if inflight >= cfg.inflight_yield_threshold {
        tokio::task::yield_now().await;
    }
}
