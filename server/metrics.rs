use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn set_handle(handle: PrometheusHandle) {
    HANDLE.set(handle).ok();
}


pub fn encode() -> String {
    HANDLE.get().map(|h| h.render()).unwrap_or_default()
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
    metrics::counter!("duotunnel_auth_success_total", "group_id" => group_id.to_string()).increment(1);
}

pub fn auth_failure(group_id: &str) {
    metrics::counter!("duotunnel_auth_failure_total", "group_id" => group_id.to_string()).increment(1);
}

pub fn client_registered(group_id: &str) {
    metrics::gauge!("duotunnel_clients_per_group", "group_id" => group_id.to_string()).increment(1.0);
}

pub fn client_unregistered(group_id: &str) {
    metrics::gauge!("duotunnel_clients_per_group", "group_id" => group_id.to_string()).decrement(1.0);
}

pub fn request_completed(protocol: &'static str, status: &'static str) {
    metrics::counter!("duotunnel_requests_total", "protocol" => protocol, "status" => status).increment(1);
}


pub struct OpenBiInflightGuard;

impl Drop for OpenBiInflightGuard {
    fn drop(&mut self) {
        metrics::gauge!("duotunnel_open_bi_inflight").decrement(1.0);
    }
}

pub fn open_bi_begin(_conn_id: &std::sync::Arc<str>) -> OpenBiInflightGuard {
    metrics::counter!("duotunnel_open_bi_total").increment(1);
    metrics::gauge!("duotunnel_open_bi_inflight").increment(1.0);
    OpenBiInflightGuard
}

pub fn open_bi_observe_wait_ms(wait_ms: f64) {
    metrics::histogram!("duotunnel_open_bi_wait_ms").record(wait_ms);
}

pub fn open_bi_timeout() {
    metrics::counter!("duotunnel_open_bi_timeout_total").increment(1);
}
