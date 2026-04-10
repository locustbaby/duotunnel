use lazy_static::lazy_static;
use prometheus::{
    Encoder, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Registry, TextEncoder,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // ── QUIC connection pool ──────────────────────────────────────────────────
    pub static ref QUIC_POOL_SIZE: IntGauge = IntGauge::new(
        "duotunnel_client_quic_pool_size",
        "Number of QUIC connections in the entry pool"
    ).unwrap();

    // ── Work streams ──────────────────────────────────────────────────────────
    pub static ref STREAMS_ACCEPTED: IntCounter = IntCounter::new(
        "duotunnel_client_streams_accepted_total",
        "Total work streams accepted from server"
    ).unwrap();
    pub static ref STREAMS_REJECTED: IntCounter = IntCounter::new(
        "duotunnel_client_streams_rejected_total",
        "Work streams rejected (semaphore full)"
    ).unwrap();
    pub static ref STREAMS_ERROR: IntCounter = IntCounter::new(
        "duotunnel_client_streams_error_total",
        "Work streams that completed with an error"
    ).unwrap();
    pub static ref STREAMS_INFLIGHT: IntGauge = IntGauge::new(
        "duotunnel_client_streams_inflight",
        "Currently in-flight work streams"
    ).unwrap();

    // ── Entry egress (client-side HTTP entry port) ────────────────────────────
    pub static ref ENTRY_CONNS_ACTIVE: IntGauge = IntGauge::new(
        "duotunnel_client_entry_conns_active",
        "Active entry TCP connections (egress path)"
    ).unwrap();
    pub static ref ENTRY_CONNS_TOTAL: IntCounter = IntCounter::new(
        "duotunnel_client_entry_conns_total",
        "Total entry TCP connections accepted"
    ).unwrap();

    // ── QUIC stats (aggregated across pool connections) ───────────────────────
    pub static ref QUIC_RTT_US: Gauge = Gauge::new(
        "duotunnel_client_quic_rtt_us",
        "Mean smoothed RTT across pool connections (microseconds)"
    ).unwrap();
    pub static ref QUIC_CWND_BYTES: Gauge = Gauge::new(
        "duotunnel_client_quic_cwnd_bytes",
        "Mean congestion window across pool connections (bytes)"
    ).unwrap();
    pub static ref QUIC_LOST_PACKETS: IntCounter = IntCounter::new(
        "duotunnel_client_quic_lost_packets_total",
        "Cumulative lost packets across pool connections"
    ).unwrap();
    pub static ref QUIC_SENT_PACKETS: IntCounter = IntCounter::new(
        "duotunnel_client_quic_sent_packets_total",
        "Cumulative sent packets across pool connections"
    ).unwrap();
    pub static ref QUIC_CONGESTION_EVENTS: IntCounter = IntCounter::new(
        "duotunnel_client_quic_congestion_events_total",
        "Cumulative congestion events across pool connections"
    ).unwrap();
    pub static ref QUIC_TX_BYTES: IntCounter = IntCounter::new(
        "duotunnel_client_quic_tx_bytes_total",
        "Cumulative UDP bytes transmitted"
    ).unwrap();
    pub static ref QUIC_RX_BYTES: IntCounter = IntCounter::new(
        "duotunnel_client_quic_rx_bytes_total",
        "Cumulative UDP bytes received"
    ).unwrap();

    // ── Reconnect / login ─────────────────────────────────────────────────────
    pub static ref RECONNECTS_TOTAL: IntCounter = IntCounter::new(
        "duotunnel_client_reconnects_total",
        "Total reconnection attempts"
    ).unwrap();
    pub static ref LOGIN_SUCCESS: IntCounter = IntCounter::new(
        "duotunnel_client_login_success_total",
        "Successful logins"
    ).unwrap();
    pub static ref LOGIN_FAILURE: IntCounter = IntCounter::new(
        "duotunnel_client_login_failure_total",
        "Failed logins"
    ).unwrap();

    // ── open_bi wait (egress path: client opens stream to server) ────────────
    pub static ref OPEN_BI_WAIT_MS: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "duotunnel_client_open_bi_wait_ms",
            "open_bi wait time on egress path (milliseconds)"
        )
        .buckets(vec![
            0.1, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0, 1024.0,
        ]),
    ).unwrap();
    pub static ref OPEN_BI_TOTAL: IntCounter = IntCounter::new(
        "duotunnel_client_open_bi_total",
        "Total open_bi calls on egress path"
    ).unwrap();
    pub static ref OPEN_BI_TIMEOUT: IntCounter = IntCounter::new(
        "duotunnel_client_open_bi_timeout_total",
        "open_bi timeouts on egress path"
    ).unwrap();

    // ── Tokio runtime ─────────────────────────────────────────────────────────
    pub static ref TOKIO_NUM_WORKERS: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_num_workers",
        "Number of tokio worker threads"
    ).unwrap();
    pub static ref TOKIO_ALIVE_TASKS: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_alive_tasks",
        "Number of alive tokio tasks"
    ).unwrap();
    pub static ref TOKIO_GLOBAL_QUEUE_DEPTH: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_global_queue_depth",
        "Depth of the tokio global task queue"
    ).unwrap();
    pub static ref TOKIO_SPAWNED_TASKS: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_spawned_tasks_total",
        "Cumulative spawned task count"
    ).unwrap();
    pub static ref TOKIO_PARK_COUNT: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_park_count_total",
        "Cumulative park count across all workers"
    ).unwrap();
    pub static ref TOKIO_STEAL_COUNT: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_steal_count_total",
        "Cumulative steal count across all workers"
    ).unwrap();
    pub static ref TOKIO_POLL_COUNT: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_poll_count_total",
        "Cumulative poll count across all workers"
    ).unwrap();
    pub static ref TOKIO_BUSY_DURATION_US: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_busy_duration_us_total",
        "Cumulative busy duration across all workers (microseconds)"
    ).unwrap();
    pub static ref TOKIO_LOCAL_QUEUE_DEPTH: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_local_queue_depth_total",
        "Sum of local queue depths across all workers"
    ).unwrap();
    pub static ref TOKIO_IO_READY_COUNT: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_io_ready_total",
        "Cumulative I/O driver ready event count"
    ).unwrap();
    pub static ref TOKIO_MEAN_POLL_TIME_US: Gauge = Gauge::new(
        "duotunnel_client_tokio_mean_poll_time_us",
        "Mean task poll time across all workers (microseconds)"
    ).unwrap();
    pub static ref TOKIO_OVERFLOW_COUNT: IntGauge = IntGauge::new(
        "duotunnel_client_tokio_overflow_count_total",
        "Cumulative local queue overflow count across all workers"
    ).unwrap();
}

pub fn init() {
    macro_rules! reg {
        ($($m:expr),* $(,)?) => { $( REGISTRY.register(Box::new($m.clone())).ok(); )* };
    }
    reg!(
        QUIC_POOL_SIZE,
        STREAMS_ACCEPTED, STREAMS_REJECTED, STREAMS_ERROR, STREAMS_INFLIGHT,
        ENTRY_CONNS_ACTIVE, ENTRY_CONNS_TOTAL,
        QUIC_RTT_US, QUIC_CWND_BYTES,
        QUIC_LOST_PACKETS, QUIC_SENT_PACKETS, QUIC_CONGESTION_EVENTS,
        QUIC_TX_BYTES, QUIC_RX_BYTES,
        RECONNECTS_TOTAL, LOGIN_SUCCESS, LOGIN_FAILURE,
        OPEN_BI_WAIT_MS, OPEN_BI_TOTAL, OPEN_BI_TIMEOUT,
        TOKIO_NUM_WORKERS, TOKIO_ALIVE_TASKS, TOKIO_GLOBAL_QUEUE_DEPTH,
        TOKIO_SPAWNED_TASKS, TOKIO_PARK_COUNT, TOKIO_STEAL_COUNT,
        TOKIO_POLL_COUNT, TOKIO_BUSY_DURATION_US, TOKIO_LOCAL_QUEUE_DEPTH,
        TOKIO_IO_READY_COUNT,
        TOKIO_MEAN_POLL_TIME_US,
        TOKIO_OVERFLOW_COUNT,
    );
}

pub fn encode() -> String {
    let encoder = TextEncoder::new();
    let mut buf = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

// ── event helpers ─────────────────────────────────────────────────────────────

pub fn stream_accepted() {
    STREAMS_ACCEPTED.inc();
    STREAMS_INFLIGHT.inc();
}

pub fn stream_done(error: bool) {
    STREAMS_INFLIGHT.dec();
    if error {
        STREAMS_ERROR.inc();
    }
}

pub fn stream_rejected() {
    STREAMS_REJECTED.inc();
}

#[allow(dead_code)]
pub fn entry_conn_opened() {
    ENTRY_CONNS_TOTAL.inc();
    ENTRY_CONNS_ACTIVE.inc();
}

#[allow(dead_code)]
pub fn entry_conn_closed() {
    ENTRY_CONNS_ACTIVE.dec();
}

pub fn reconnect() {
    RECONNECTS_TOTAL.inc();
}

pub fn login_success() {
    LOGIN_SUCCESS.inc();
}

pub fn login_failure() {
    LOGIN_FAILURE.inc();
}

#[allow(dead_code)]
pub fn open_bi_observe(wait_ms: f64, timed_out: bool) {
    OPEN_BI_TOTAL.inc();
    OPEN_BI_WAIT_MS.observe(wait_ms);
    if timed_out {
        OPEN_BI_TIMEOUT.inc();
    }
}

pub fn update_quic_stats(conns: &[quinn::Connection]) {
    if conns.is_empty() {
        QUIC_POOL_SIZE.set(0);
        return;
    }
    let n = conns.len() as f64;
    QUIC_POOL_SIZE.set(conns.len() as i64);
    let mut rtt_sum = 0u64;
    let mut cwnd_sum = 0u64;
    let mut lost = 0u64;
    let mut sent = 0u64;
    let mut cong = 0u64;
    let mut tx = 0u64;
    let mut rx = 0u64;
    for c in conns {
        let s = c.stats();
        rtt_sum  += s.path.rtt.as_micros() as u64;
        cwnd_sum += s.path.cwnd;
        lost     += s.path.lost_packets;
        sent     += s.path.sent_packets;
        cong     += s.path.congestion_events;
        tx       += s.udp_tx.bytes;
        rx       += s.udp_rx.bytes;
    }
    QUIC_RTT_US.set(rtt_sum as f64 / n);
    QUIC_CWND_BYTES.set(cwnd_sum as f64 / n);

    macro_rules! delta_inc {
        ($counter:expr, $new:expr) => {{
            let prev = $counter.get() as u64;
            if $new > prev { $counter.inc_by($new - prev); }
        }};
    }
    delta_inc!(QUIC_LOST_PACKETS, lost);
    delta_inc!(QUIC_SENT_PACKETS, sent);
    delta_inc!(QUIC_CONGESTION_EVENTS, cong);
    delta_inc!(QUIC_TX_BYTES, tx);
    delta_inc!(QUIC_RX_BYTES, rx);
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
        TOKIO_IO_READY_COUNT.set(m.io_driver_ready_count() as i64);
        let (mut park, mut steal, mut poll, mut busy, mut lq, mut overflow) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
        for w in 0..nw {
            park     += m.worker_park_count(w) as i64;
            steal    += m.worker_steal_count(w) as i64;
            poll     += m.worker_poll_count(w) as i64;
            busy     += m.worker_total_busy_duration(w).as_micros() as i64;
            lq       += m.worker_local_queue_depth(w) as i64;
            overflow += m.worker_overflow_count(w) as i64;
        }
        TOKIO_PARK_COUNT.set(park);
        TOKIO_STEAL_COUNT.set(steal);
        TOKIO_POLL_COUNT.set(poll);
        TOKIO_BUSY_DURATION_US.set(busy);
        TOKIO_LOCAL_QUEUE_DEPTH.set(lq);
        TOKIO_OVERFLOW_COUNT.set(overflow);
        if poll > 0 {
            TOKIO_MEAN_POLL_TIME_US.set(busy as f64 / poll as f64);
        }
    }
}

#[cfg(not(tokio_unstable))]
pub fn update_tokio_metrics() {}
