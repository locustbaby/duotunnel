use lazy_static::lazy_static;
use prometheus::{
    Encoder, Gauge, Histogram, HistogramOpts, IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    Opts, Registry, TextEncoder,
};
lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref ACTIVE_QUIC_CONNECTIONS: IntGauge = IntGauge::new(
        "duotunnel_active_quic_connections",
        "Number of active QUIC client connections"
    )
    .unwrap();
    pub static ref ACTIVE_TCP_CONNECTIONS: IntGauge = IntGauge::new(
        "duotunnel_active_tcp_connections",
        "Number of active TCP connections"
    )
    .unwrap();
    pub static ref TOTAL_QUIC_CONNECTIONS: IntCounter = IntCounter::new(
        "duotunnel_total_quic_connections",
        "Total QUIC connections established"
    )
    .unwrap();
    pub static ref TOTAL_TCP_CONNECTIONS: IntCounter = IntCounter::new(
        "duotunnel_total_tcp_connections",
        "Total TCP connections established"
    )
    .unwrap();
    pub static ref AUTH_SUCCESS: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_auth_success_total", "Successful authentications"),
        &["group_id"]
    )
    .unwrap();
    pub static ref AUTH_FAILURE: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_auth_failure_total", "Failed authentications"),
        &["group_id"]
    )
    .unwrap();
    pub static ref CLIENTS_PER_GROUP: IntGaugeVec = IntGaugeVec::new(
        Opts::new("duotunnel_clients_per_group", "Connected clients per group"),
        &["group_id"]
    )
    .unwrap();
    pub static ref REQUESTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("duotunnel_requests_total", "Total requests processed"),
        &["protocol", "status"]
    )
    .unwrap();
    pub static ref CONNECTIONS_REJECTED: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "duotunnel_connections_rejected_total",
            "Connections rejected due to backpressure"
        ),
        &["type"]
    )
    .unwrap();
    pub static ref OPEN_BI_WAIT_MS: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "duotunnel_open_bi_wait_ms",
            "open_bi wait time in milliseconds"
        )
        .buckets(vec![
            0.1, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0,
            1024.0,
        ]),
    )
    .unwrap();
    pub static ref OPEN_BI_TOTAL: IntCounter = IntCounter::new(
        "duotunnel_open_bi_total",
        "Total open_bi attempts"
    )
    .unwrap();
    pub static ref OPEN_BI_TIMEOUT_TOTAL: IntCounter = IntCounter::new(
        "duotunnel_open_bi_timeout_total",
        "Total open_bi timeout events"
    )
    .unwrap();
    pub static ref OPEN_BI_TIMEOUT_RATIO: Gauge = Gauge::new(
        "duotunnel_open_bi_timeout_ratio",
        "open_bi timeout ratio = timeout_total / open_bi_total"
    )
    .unwrap();
    pub static ref OPEN_BI_INFLIGHT: IntGauge = IntGauge::new(
        "duotunnel_open_bi_inflight",
        "Currently in-flight open_bi calls"
    )
    .unwrap();

    // ── Quinn QUIC metrics (aggregated across all connections) ────────────────
    pub static ref QUIC_RTT_US: Gauge = Gauge::new(
        "duotunnel_quic_rtt_us",
        "Smoothed RTT of active QUIC connections (microseconds, mean across conns)"
    )
    .unwrap();
    pub static ref QUIC_CWND_BYTES: Gauge = Gauge::new(
        "duotunnel_quic_cwnd_bytes",
        "Congestion window of active QUIC connections (bytes, mean across conns)"
    )
    .unwrap();
    pub static ref QUIC_LOST_PACKETS: IntCounter = IntCounter::new(
        "duotunnel_quic_lost_packets_total",
        "Total QUIC lost packets (monotonically increasing, summed across conns)"
    )
    .unwrap();
    pub static ref QUIC_SENT_PACKETS: IntCounter = IntCounter::new(
        "duotunnel_quic_sent_packets_total",
        "Total QUIC sent packets"
    )
    .unwrap();
    pub static ref QUIC_CONGESTION_EVENTS: IntCounter = IntCounter::new(
        "duotunnel_quic_congestion_events_total",
        "Total QUIC congestion events"
    )
    .unwrap();
    pub static ref QUIC_TX_BYTES: IntCounter = IntCounter::new(
        "duotunnel_quic_tx_bytes_total",
        "Total QUIC UDP bytes transmitted"
    )
    .unwrap();
    pub static ref QUIC_RX_BYTES: IntCounter = IntCounter::new(
        "duotunnel_quic_rx_bytes_total",
        "Total QUIC UDP bytes received"
    )
    .unwrap();

    // ── Tokio runtime metrics ─────────────────────────────────────────────────
    pub static ref TOKIO_NUM_WORKERS: IntGauge = IntGauge::new(
        "duotunnel_tokio_num_workers",
        "Number of tokio worker threads"
    )
    .unwrap();
    pub static ref TOKIO_ALIVE_TASKS: IntGauge = IntGauge::new(
        "duotunnel_tokio_alive_tasks",
        "Number of alive tokio tasks"
    )
    .unwrap();
    pub static ref TOKIO_GLOBAL_QUEUE_DEPTH: IntGauge = IntGauge::new(
        "duotunnel_tokio_global_queue_depth",
        "Depth of the tokio global task queue"
    )
    .unwrap();
    pub static ref TOKIO_SPAWNED_TASKS: IntGauge = IntGauge::new(
        "duotunnel_tokio_spawned_tasks_total",
        "Cumulative count of spawned tokio tasks"
    )
    .unwrap();
    pub static ref TOKIO_PARK_COUNT: IntGauge = IntGauge::new(
        "duotunnel_tokio_park_count_total",
        "Cumulative park count across all workers"
    )
    .unwrap();
    pub static ref TOKIO_STEAL_COUNT: IntGauge = IntGauge::new(
        "duotunnel_tokio_steal_count_total",
        "Cumulative steal count across all workers"
    )
    .unwrap();
    pub static ref TOKIO_POLL_COUNT: IntGauge = IntGauge::new(
        "duotunnel_tokio_poll_count_total",
        "Cumulative poll count across all workers"
    )
    .unwrap();
    pub static ref TOKIO_BUSY_DURATION_US: IntGauge = IntGauge::new(
        "duotunnel_tokio_busy_duration_us_total",
        "Cumulative busy duration across all workers in microseconds"
    )
    .unwrap();
    pub static ref TOKIO_LOCAL_QUEUE_DEPTH: IntGauge = IntGauge::new(
        "duotunnel_tokio_local_queue_depth_total",
        "Sum of local queue depths across all workers"
    )
    .unwrap();
    pub static ref TOKIO_BLOCKING_THREADS: IntGauge = IntGauge::new(
        "duotunnel_tokio_blocking_threads",
        "Number of blocking pool threads"
    )
    .unwrap();
    pub static ref TOKIO_IO_FD_COUNT: IntGauge = IntGauge::new(
        "duotunnel_tokio_io_fd_registered",
        "Number of I/O driver registered file descriptors"
    )
    .unwrap();
    pub static ref TOKIO_IO_READY_COUNT: IntGauge = IntGauge::new(
        "duotunnel_tokio_io_ready_total",
        "Cumulative I/O driver ready event count"
    )
    .unwrap();
    pub static ref TOKIO_MEAN_POLL_TIME_US: Gauge = Gauge::new(
        "duotunnel_tokio_mean_poll_time_us",
        "Mean task poll time across all workers (microseconds)"
    )
    .unwrap();
    pub static ref TOKIO_OVERFLOW_COUNT: IntGauge = IntGauge::new(
        "duotunnel_tokio_overflow_count_total",
        "Cumulative local queue overflow count across all workers"
    )
    .unwrap();
}

pub fn init() {
    REGISTRY
        .register(Box::new(ACTIVE_QUIC_CONNECTIONS.clone()))
        .ok();
    REGISTRY
        .register(Box::new(ACTIVE_TCP_CONNECTIONS.clone()))
        .ok();
    REGISTRY
        .register(Box::new(TOTAL_QUIC_CONNECTIONS.clone()))
        .ok();
    REGISTRY
        .register(Box::new(TOTAL_TCP_CONNECTIONS.clone()))
        .ok();
    REGISTRY.register(Box::new(AUTH_SUCCESS.clone())).ok();
    REGISTRY.register(Box::new(AUTH_FAILURE.clone())).ok();
    REGISTRY.register(Box::new(CLIENTS_PER_GROUP.clone())).ok();
    REGISTRY.register(Box::new(REQUESTS_TOTAL.clone())).ok();
    REGISTRY
        .register(Box::new(CONNECTIONS_REJECTED.clone()))
        .ok();
    REGISTRY.register(Box::new(OPEN_BI_WAIT_MS.clone())).ok();
    REGISTRY.register(Box::new(OPEN_BI_TOTAL.clone())).ok();
    REGISTRY.register(Box::new(OPEN_BI_TIMEOUT_TOTAL.clone())).ok();
    REGISTRY.register(Box::new(OPEN_BI_TIMEOUT_RATIO.clone())).ok();
    REGISTRY.register(Box::new(OPEN_BI_INFLIGHT.clone())).ok();
    REGISTRY.register(Box::new(QUIC_RTT_US.clone())).ok();
    REGISTRY.register(Box::new(QUIC_CWND_BYTES.clone())).ok();
    REGISTRY.register(Box::new(QUIC_LOST_PACKETS.clone())).ok();
    REGISTRY.register(Box::new(QUIC_SENT_PACKETS.clone())).ok();
    REGISTRY.register(Box::new(QUIC_CONGESTION_EVENTS.clone())).ok();
    REGISTRY.register(Box::new(QUIC_TX_BYTES.clone())).ok();
    REGISTRY.register(Box::new(QUIC_RX_BYTES.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_NUM_WORKERS.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_ALIVE_TASKS.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_GLOBAL_QUEUE_DEPTH.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_SPAWNED_TASKS.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_PARK_COUNT.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_STEAL_COUNT.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_POLL_COUNT.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_BUSY_DURATION_US.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_LOCAL_QUEUE_DEPTH.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_BLOCKING_THREADS.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_IO_FD_COUNT.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_IO_READY_COUNT.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_MEAN_POLL_TIME_US.clone())).ok();
    REGISTRY.register(Box::new(TOKIO_OVERFLOW_COUNT.clone())).ok();
}

pub fn encode() -> String {
    let total = OPEN_BI_TOTAL.get() as f64;
    let timeout = OPEN_BI_TIMEOUT_TOTAL.get() as f64;
    OPEN_BI_TIMEOUT_RATIO.set(if total > 0.0 { timeout / total } else { 0.0 });
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

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

pub struct OpenBiInflightGuard;

impl Drop for OpenBiInflightGuard {
    fn drop(&mut self) {
        OPEN_BI_INFLIGHT.dec();
    }
}

pub fn open_bi_begin(_conn_id: &std::sync::Arc<str>) -> OpenBiInflightGuard {
    OPEN_BI_TOTAL.inc();
    OPEN_BI_INFLIGHT.inc();
    OpenBiInflightGuard
}

pub fn open_bi_observe_wait_ms(wait_ms: f64) {
    OPEN_BI_WAIT_MS.observe(wait_ms);
}

pub fn open_bi_timeout() {
    OPEN_BI_TIMEOUT_TOTAL.inc();
}

pub fn update_quic_stats(conns: &[quinn::Connection]) {
    if conns.is_empty() {
        return;
    }
    let n = conns.len() as f64;
    let mut rtt_sum = 0u64;
    let mut cwnd_sum = 0u64;
    let mut lost_delta = 0u64;
    let mut sent_delta = 0u64;
    let mut congestion_delta = 0u64;
    let mut tx_bytes_delta = 0u64;
    let mut rx_bytes_delta = 0u64;
    for conn in conns {
        let s = conn.stats();
        rtt_sum += s.path.rtt.as_micros() as u64;
        cwnd_sum += s.path.cwnd;
        lost_delta += s.path.lost_packets;
        sent_delta += s.path.sent_packets;
        congestion_delta += s.path.congestion_events;
        tx_bytes_delta += s.udp_tx.bytes;
        rx_bytes_delta += s.udp_rx.bytes;
    }
    QUIC_RTT_US.set((rtt_sum as f64) / n);
    QUIC_CWND_BYTES.set((cwnd_sum as f64) / n);

    let prev_lost = QUIC_LOST_PACKETS.get();
    if lost_delta > prev_lost { QUIC_LOST_PACKETS.inc_by(lost_delta - prev_lost); }
    let prev_sent = QUIC_SENT_PACKETS.get();
    if sent_delta > prev_sent { QUIC_SENT_PACKETS.inc_by(sent_delta - prev_sent); }
    let prev_cong = QUIC_CONGESTION_EVENTS.get();
    if congestion_delta > prev_cong { QUIC_CONGESTION_EVENTS.inc_by(congestion_delta - prev_cong); }
    let prev_tx = QUIC_TX_BYTES.get();
    if tx_bytes_delta > prev_tx { QUIC_TX_BYTES.inc_by(tx_bytes_delta - prev_tx); }
    let prev_rx = QUIC_RX_BYTES.get();
    if rx_bytes_delta > prev_rx { QUIC_RX_BYTES.inc_by(rx_bytes_delta - prev_rx); }
}

#[cfg(tokio_unstable)]
pub fn update_tokio_metrics() {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        let m = handle.metrics();
        let nw = m.num_workers();
        TOKIO_NUM_WORKERS.set(nw as i64);
        TOKIO_ALIVE_TASKS.set(m.num_alive_tasks() as i64);
        TOKIO_GLOBAL_QUEUE_DEPTH.set(m.global_queue_depth() as i64);
        TOKIO_SPAWNED_TASKS.set(m.spawned_tasks_count() as i64);
        TOKIO_BLOCKING_THREADS.set(m.num_blocking_threads() as i64);

        let mut park_total = 0i64;
        let mut steal_total = 0i64;
        let mut poll_total = 0i64;
        let mut busy_us_total = 0i64;
        let mut local_q_total = 0i64;
        let mut overflow_total = 0i64;
        for w in 0..nw {
            park_total += m.worker_park_count(w) as i64;
            steal_total += m.worker_steal_count(w) as i64;
            poll_total += m.worker_poll_count(w) as i64;
            busy_us_total += m.worker_total_busy_duration(w).as_micros() as i64;
            local_q_total += m.worker_local_queue_depth(w) as i64;
            overflow_total += m.worker_overflow_count(w) as i64;
        }
        TOKIO_PARK_COUNT.set(park_total);
        TOKIO_STEAL_COUNT.set(steal_total);
        TOKIO_POLL_COUNT.set(poll_total);
        TOKIO_BUSY_DURATION_US.set(busy_us_total);
        TOKIO_LOCAL_QUEUE_DEPTH.set(local_q_total);
        TOKIO_OVERFLOW_COUNT.set(overflow_total);
        TOKIO_IO_FD_COUNT.set(m.io_driver_fd_registered_count() as i64);
        TOKIO_IO_READY_COUNT.set(m.io_driver_ready_count() as i64);

        if poll_total > 0 {
            TOKIO_MEAN_POLL_TIME_US.set(busy_us_total as f64 / poll_total as f64);
        }
    }
}

#[cfg(not(tokio_unstable))]
pub fn update_tokio_metrics() {}
