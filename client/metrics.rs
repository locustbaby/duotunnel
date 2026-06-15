use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn set_handle(handle: PrometheusHandle) {
    HANDLE.set(handle).ok();
}

pub fn encode() -> String {
    HANDLE.get().map(|h| h.render()).unwrap_or_default()
}

pub fn egress_rejection(reason: &str, host: &str) {
    metrics::counter!(
        "egress_rejections_total",
        "reason" => reason.to_string(),
        "host" => host.to_string()
    )
    .increment(1);
}
