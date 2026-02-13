use lazy_static::lazy_static;
use prometheus::{
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    Opts, Registry, Encoder, TextEncoder,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Connection metrics
    pub static ref ACTIVE_QUIC_CONNECTIONS: IntGauge = IntGauge::new(
        "duotunnel_active_quic_connections",
        "Number of active QUIC client connections"
    ).unwrap();

    pub static ref ACTIVE_TCP_CONNECTIONS: IntGauge = IntGauge::new(
        "duotunnel_active_tcp_connections",
        "Number of active TCP connections"
    ).unwrap();

    pub static ref TOTAL_QUIC_CONNECTIONS: IntCounter = IntCounter::new(
        "duotunnel_total_quic_connections",
        "Total QUIC connections established"
    ).unwrap();

    pub static ref TOTAL_TCP_CONNECTIONS: IntCounter = IntCounter::new(
        "duotunnel_total_tcp_connections",
        "Total TCP connections established"
    ).unwrap();

    // Auth metrics
    pub static ref AUTH_SUCCESS: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_auth_success_total", "Successful authentications"),
        &["group_id"]
    ).unwrap();

    pub static ref AUTH_FAILURE: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_auth_failure_total", "Failed authentications"),
        &["group_id"]
    ).unwrap();

    // Client metrics
    pub static ref CLIENTS_PER_GROUP: IntGaugeVec = IntGaugeVec::new(
        Opts::new("duotunnel_clients_per_group", "Connected clients per group"),
        &["group_id"]
    ).unwrap();

    // Request metrics
    pub static ref REQUESTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_requests_total", "Total requests processed"),
        &["protocol", "status"]
    ).unwrap();

    // Backpressure metrics
    pub static ref CONNECTIONS_REJECTED: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_connections_rejected_total", "Connections rejected due to backpressure"),
        &["type"]
    ).unwrap();

    // Duplicate client metrics
    pub static ref DUPLICATE_CLIENTS: IntCounter = IntCounter::new(
        "duotunnel_duplicate_clients_total",
        "Duplicate client connections closed"
    ).unwrap();
}

pub fn init() {
    REGISTRY.register(Box::new(ACTIVE_QUIC_CONNECTIONS.clone())).ok();
    REGISTRY.register(Box::new(ACTIVE_TCP_CONNECTIONS.clone())).ok();
    REGISTRY.register(Box::new(TOTAL_QUIC_CONNECTIONS.clone())).ok();
    REGISTRY.register(Box::new(TOTAL_TCP_CONNECTIONS.clone())).ok();
    REGISTRY.register(Box::new(AUTH_SUCCESS.clone())).ok();
    REGISTRY.register(Box::new(AUTH_FAILURE.clone())).ok();
    REGISTRY.register(Box::new(CLIENTS_PER_GROUP.clone())).ok();
    REGISTRY.register(Box::new(REQUESTS_TOTAL.clone())).ok();
    REGISTRY.register(Box::new(CONNECTIONS_REJECTED.clone())).ok();
    REGISTRY.register(Box::new(DUPLICATE_CLIENTS.clone())).ok();
}

pub fn encode() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

// Helper functions
pub fn quic_connection_opened() {
    TOTAL_QUIC_CONNECTIONS.inc();
    ACTIVE_QUIC_CONNECTIONS.inc();
}

pub fn quic_connection_closed() {
    ACTIVE_QUIC_CONNECTIONS.dec();
}

pub fn tcp_connection_opened() {
    TOTAL_TCP_CONNECTIONS.inc();
    ACTIVE_TCP_CONNECTIONS.inc();
}

pub fn tcp_connection_closed() {
    ACTIVE_TCP_CONNECTIONS.dec();
}

pub fn auth_success(group_id: &str) {
    AUTH_SUCCESS.with_label_values(&[group_id]).inc();
}

pub fn auth_failure(group_id: &str) {
    AUTH_FAILURE.with_label_values(&[group_id]).inc();
}

pub fn client_registered(group_id: &str) {
    CLIENTS_PER_GROUP.with_label_values(&[group_id]).inc();
}

pub fn client_unregistered(group_id: &str) {
    CLIENTS_PER_GROUP.with_label_values(&[group_id]).dec();
}

pub fn request_completed(protocol: &str, status: &str) {
    REQUESTS_TOTAL.with_label_values(&[protocol, status]).inc();
}

pub fn connection_rejected(conn_type: &str) {
    CONNECTIONS_REJECTED.with_label_values(&[conn_type]).inc();
}

pub fn duplicate_client_closed() {
    DUPLICATE_CLIENTS.inc();
}
