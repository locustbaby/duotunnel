use crate::overload::OverloadLimits;
use crate::transport::tcp_params::TcpParams;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::ingress::ProtocolHint;
use super::metrics::MetricsSink;

// ── PhaseResult ───────────────────────────────────────────────────────────────

/// Uniform return type for every phase callback.
///
/// `Continue(T)` advances to the next phase.
/// `Reject` short-circuits the pipeline: the connection is closed (or an error
/// response sent) with the given status code and optional message body.
#[derive(Debug)]
pub enum PhaseResult<T = ()> {
    Continue(T),
    Reject { status: u16, message: Bytes },
}

impl<T> PhaseResult<T> {
    pub fn is_continue(&self) -> bool {
        matches!(self, PhaseResult::Continue(_))
    }
}

// ── Timeouts ──────────────────────────────────────────────────────────────────

/// Per-connection timeout budgets shared across server and egress contexts.
#[derive(Debug, Clone)]
pub struct Timeouts {
    pub open_stream_ms: u64,
    pub login_ms: u64,
    pub connect_ms: u64,
    pub resolve_ms: u64,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            open_stream_ms: 5_000,
            login_ms: 10_000,
            connect_ms: 10_000,
            resolve_ms: 5_000,
        }
    }
}

// ── PhaseTiming ───────────────────────────────────────────────────────────────

/// Timestamps recorded at phase boundaries, used by the logging phase to
/// emit latency metrics without requiring timers inside each handler.
#[derive(Debug, Clone)]
pub struct PhaseTiming {
    pub accepted_at: Instant,
    pub sniff_done_at: Option<Instant>,
    pub admission_done_at: Option<Instant>,
    pub route_done_at: Option<Instant>,
    pub tunnel_open_at: Option<Instant>,
    pub completed_at: Option<Instant>,
}

impl PhaseTiming {
    pub fn new() -> Self {
        Self {
            accepted_at: Instant::now(),
            sniff_done_at: None,
            admission_done_at: None,
            route_done_at: None,
            tunnel_open_at: None,
            completed_at: None,
        }
    }

    pub fn total(&self) -> Option<Duration> {
        self.completed_at.map(|t| t - self.accepted_at)
    }
}

impl Default for PhaseTiming {
    fn default() -> Self {
        Self::new()
    }
}

// ── PhaseOutcome ──────────────────────────────────────────────────────────────

/// Summary passed to the logging phase at the end of a connection lifecycle.
#[derive(Debug)]
pub struct PhaseOutcome {
    pub timing: PhaseTiming,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub error: Option<String>,
}

// ── Route ─────────────────────────────────────────────────────────────────────

/// Result of a successful route lookup (Phase 3).
///
/// Mirrors `RouteTarget` from `transport::listener` but lives in the plugin
/// layer so ingress handlers can depend on it without touching the transport
/// crate directly.
#[derive(Debug, Clone)]
pub struct Route {
    pub group_id: Arc<str>,
    pub proxy_name: Arc<str>,
}

impl Route {
    pub fn new(group_id: impl Into<Arc<str>>, proxy_name: impl Into<Arc<str>>) -> Self {
        Self {
            group_id: group_id.into(),
            proxy_name: proxy_name.into(),
        }
    }
}

// ── AdmissionReq ─────────────────────────────────────────────────────────────

/// Input to the admission phase (Phase 2).
#[derive(Debug, Clone)]
pub struct AdmissionReq {
    pub peer_addr: SocketAddr,
    pub hint: Option<ProtocolHint>,
    pub token: Option<String>,
}

// ── RouteCtx ─────────────────────────────────────────────────────────────────

/// Input to the route-resolve phase (Phase 3).
#[derive(Debug, Clone)]
pub struct RouteCtx {
    /// Port the server accepted this connection on (the listener's local port,
    /// not the ephemeral remote port).
    pub listener_port: u16,
    /// Remote peer that opened the connection.
    pub client_addr: SocketAddr,
    pub hint: ProtocolHint,
}

// ── ServerCtx ─────────────────────────────────────────────────────────────────

/// Per-connection context for the server (ingress) side.
///
/// Only contains fields relevant to server-side phases.  Egress-specific
/// capabilities (`lb`, `dialer`) are absent by design.
pub struct ServerCtx {
    // Injected shared capabilities
    pub metrics: Arc<dyn MetricsSink>,

    // Phase outputs — populated as the pipeline advances
    pub hint: Option<ProtocolHint>,  // filled at Phase 1
    pub route: Option<Route>,         // filled at Phase 3
    pub admitted: bool,               // set true after Phase 2 passes

    // Shared read-only config (no need to abstract — single implementation)
    pub tcp_params: Arc<TcpParams>,
    pub overload: OverloadLimits,
    pub timeouts: Timeouts,
    pub peer_addr: SocketAddr,
    pub listener_port: u16,
    pub relay_buf_size: usize,

    // Timing for the logging phase
    pub timing: PhaseTiming,
}

impl ServerCtx {
    pub fn new(
        peer_addr: SocketAddr,
        metrics: Arc<dyn MetricsSink>,
        tcp_params: Arc<TcpParams>,
        overload: OverloadLimits,
        timeouts: Timeouts,
        listener_port: u16,
        relay_buf_size: usize,
    ) -> Self {
        Self {
            metrics,
            hint: None,
            route: None,
            admitted: false,
            tcp_params,
            overload,
            timeouts,
            peer_addr,
            listener_port,
            relay_buf_size,
            timing: PhaseTiming::new(),
        }
    }
}

// ── ConnectInfo ───────────────────────────────────────────────────────────────

/// Metadata about a successfully opened upstream connection.
#[derive(Debug, Clone)]
pub struct ConnectInfo {
    pub remote_addr: SocketAddr,
    pub protocol: String,
}

// ── Target ────────────────────────────────────────────────────────────────────

/// A single upstream endpoint considered for load balancing.
#[derive(Debug, Clone)]
pub struct Target {
    pub host: String,
    pub port: u16,
    pub scheme: crate::proxy::tcp::UpstreamScheme,
}

// ── PickCtx / DialCtx ────────────────────────────────────────────────────────

/// Context passed to `LoadBalancer::pick`.
#[derive(Debug)]
pub struct PickCtx {
    pub client_addr: SocketAddr,
}

/// Context passed to `UpstreamDialer::dial`.
#[derive(Debug)]
pub struct DialCtx {
    pub tcp_params: Arc<TcpParams>,
    pub timeout_ms: u64,
}

// ── EgressCtx ─────────────────────────────────────────────────────────────────

/// Per-stream context for the egress (client) side.
///
/// Only contains fields relevant to egress-side phases.  Server-specific
/// capabilities (`client_registry`) are absent by design.
pub struct EgressCtx {
    // Injected shared capabilities
    pub metrics: Arc<dyn MetricsSink>,

    // Phase outputs
    pub resolved: Vec<std::net::SocketAddr>,  // filled at Phase 1
    pub selected: Option<Target>,              // filled at Phase 2
    pub connected: Option<ConnectInfo>,        // filled at Phase 3

    // Shared read-only config
    pub tcp_params: Arc<TcpParams>,
    pub timeouts: Timeouts,

    // Timing
    pub timing: PhaseTiming,
}

impl EgressCtx {
    pub fn new(
        metrics: Arc<dyn MetricsSink>,
        tcp_params: Arc<TcpParams>,
        timeouts: Timeouts,
    ) -> Self {
        Self {
            metrics,
            resolved: Vec::new(),
            selected: None,
            connected: None,
            tcp_params,
            timeouts,
            timing: PhaseTiming::new(),
        }
    }
}
