use crate::ingress::plugins::prometheus::PrometheusSink;
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;
use tunnel_lib::ProxyError;

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

pub fn quic_connection_opened() {
    metrics::counter!("duotunnel_total_quic_connections").increment(1);
    metrics::gauge!("duotunnel_active_quic_connections").increment(1.0);
}

pub fn quic_connection_closed() {
    metrics::gauge!("duotunnel_active_quic_connections").decrement(1.0);
}

pub fn tcp_connection_opened() {
    metrics::counter!("duotunnel_total_tcp_connections").increment(1);
    metrics::gauge!("duotunnel_active_tcp_connections").increment(1.0);
}

pub fn tcp_connection_closed() {
    metrics::gauge!("duotunnel_active_tcp_connections").decrement(1.0);
}

pub fn auth_success(group_id: &str) {
    metrics::counter!("duotunnel_auth_success_total", "group_id" => group_id.to_string())
        .increment(1);
}

pub fn auth_failure(group_id: &str) {
    metrics::counter!("duotunnel_auth_failure_total", "group_id" => group_id.to_string())
        .increment(1);
}

pub fn client_registered(group_id: &str) {
    metrics::gauge!("duotunnel_clients_per_group", "group_id" => group_id.to_string())
        .increment(1.0);
}

pub fn client_unregistered(group_id: &str) {
    metrics::gauge!("duotunnel_clients_per_group", "group_id" => group_id.to_string())
        .decrement(1.0);
}

pub fn request_completed(protocol: &'static str, status: &'static str) {
    metrics::counter!("duotunnel_requests_total", "protocol" => protocol, "status" => status)
        .increment(1);
}

pub fn request_failed(protocol: &'static str, error: &anyhow::Error) {
    request_completed(protocol, "error");
    if let Some(proxy_error) = error.downcast_ref::<ProxyError>() {
        tunnel_lib::plugin::observe_proxy_error(&PrometheusSink, protocol, proxy_error);
    }
}

pub struct OpenBiInflightGuard;

impl Drop for OpenBiInflightGuard {
    fn drop(&mut self) {
        metrics::gauge!("duotunnel_open_bi_inflight").decrement(1.0);
    }
}

pub fn open_bi_begin(_conn_id: &str) -> OpenBiInflightGuard {
    metrics::counter!("duotunnel_open_bi_total").increment(1);
    metrics::gauge!("duotunnel_open_bi_inflight").increment(1.0);
    OpenBiInflightGuard
}

pub fn open_bi_observe_wait_ms(wait_ms: f64) {
    metrics::histogram!("duotunnel_open_bi_wait_ms").record(wait_ms);
}

pub fn open_bi_timed_out() {
    metrics::counter!("duotunnel_open_bi_timed_out_total").increment(1);
}

pub fn open_bi_rejected_overloaded() {
    metrics::counter!("duotunnel_open_bi_rejected_overloaded_total").increment(1);
}
