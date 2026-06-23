use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn set_handle(handle: PrometheusHandle) {
    HANDLE.set(handle).ok();
}

pub fn encode() -> String {
    let mut body = HANDLE.get().map(|h| h.render()).unwrap_or_default();
    append_resource_metrics(&mut body);
    body
}

fn append_resource_metrics(body: &mut String) {
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str("# HELP duotunnel_slowpath_waiting_tasks Active tasks waiting in slowpath overload backoff.\n");
    body.push_str("# TYPE duotunnel_slowpath_waiting_tasks gauge\n");
    body.push_str(&format!(
        "duotunnel_slowpath_waiting_tasks {}\n",
        tunnel_lib::METRICS.waiting_tasks()
    ));
}

pub fn egress_rejection(reason: &str, host: &str) {
    metrics::counter!(
        "egress_rejections_total",
        "reason" => reason.to_string(),
        "host" => host.to_string()
    )
    .increment(1);
}
