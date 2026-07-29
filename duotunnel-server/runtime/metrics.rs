use crate::ingress::plugins::prometheus::PrometheusSink;
use duotunnel_lib::ProxyError;
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
        duotunnel_lib::METRICS.waiting_tasks()
    ));
    for (name, help, value) in [
        (
            "duotunnel_reverse_streams_active",
            "Active reverse tunnel streams.",
            duotunnel_lib::METRICS.reverse_streams(),
        ),
        (
            "duotunnel_http_requests_active",
            "Active HTTP requests.",
            duotunnel_lib::METRICS.http_requests(),
        ),
        (
            "duotunnel_udp_tasks_active",
            "Active UDP dispatch and reply tasks.",
            duotunnel_lib::METRICS.udp_tasks(),
        ),
    ] {
        body.push_str(&format!("# HELP {name} {help}\n"));
        body.push_str(&format!("# TYPE {name} gauge\n"));
        body.push_str(&format!("{name} {value}\n"));
    }
}

pub fn quic_connection_opened() {
    metrics::counter!("duotunnel_total_quic_connections").increment(1);
    metrics::gauge!("duotunnel_active_quic_connections").increment(1.0);
}

pub fn quic_connection_closed() {
    metrics::gauge!("duotunnel_active_quic_connections").decrement(1.0);
}

pub fn unauthenticated_connection_refused() {
    metrics::counter!("duotunnel_unauthenticated_connections_refused_total").increment(1);
}

pub fn connection_rejected_not_ready(protocol: &'static str) {
    metrics::counter!(
        "duotunnel_connection_rejected_not_ready_total",
        "protocol" => protocol
    )
    .increment(1);
}

pub fn reverse_stream_rejected(reason: &'static str) {
    metrics::counter!(
        "duotunnel_reverse_stream_rejected_total",
        "reason" => reason
    )
    .increment(1);
}

pub fn tcp_connection_opened() {
    metrics::counter!("duotunnel_total_tcp_connections").increment(1);
    metrics::gauge!("duotunnel_active_tcp_connections").increment(1.0);
}

pub fn tcp_connection_closed() {
    metrics::gauge!("duotunnel_active_tcp_connections").decrement(1.0);
}

pub fn auth_success(_group_id: &str) {
    metrics::counter!("duotunnel_auth_success_total").increment(1);
}

pub fn auth_failure(_group_id: &str) {
    metrics::counter!("duotunnel_auth_failure_total").increment(1);
}

pub fn client_registered(_group_id: &str) {
    metrics::gauge!("duotunnel_clients_active").increment(1.0);
}

pub fn client_unregistered(_group_id: &str) {
    metrics::gauge!("duotunnel_clients_active").decrement(1.0);
}

pub fn request_completed(protocol: &'static str, status: &'static str) {
    metrics::counter!("duotunnel_requests_total", "protocol" => protocol, "status" => status)
        .increment(1);
}

pub fn request_failed(protocol: &'static str, error: &anyhow::Error) {
    request_completed(protocol, "error");
    if let Some(proxy_error) = error.downcast_ref::<ProxyError>() {
        duotunnel_lib::plugin::observe_proxy_error(&PrometheusSink, protocol, proxy_error);
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

pub fn udp_datagram_dropped(reason: &'static str) {
    metrics::counter!("duotunnel_udp_datagram_dropped_total", "reason" => reason).increment(1);
}
