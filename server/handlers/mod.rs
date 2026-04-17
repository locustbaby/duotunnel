pub mod http;
pub mod metrics;
pub mod quic;
pub mod tcp;

use crate::registry::SelectedConnection;
use std::sync::atomic::Ordering;
use tunnel_lib::OverloadLimits;

pub async fn maybe_slow_path(selected: &SelectedConnection, limits: &OverloadLimits) {
    let inflight = selected.inflight.load(Ordering::Relaxed);
    tunnel_lib::maybe_slow_path(inflight, limits).await;
}
