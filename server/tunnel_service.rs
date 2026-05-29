use anyhow::Result;
use async_trait::async_trait;

use tunnel_lib::plugin::{AdmissionReq, PhaseOutcome, PhaseResult, ServerCtx, TunnelService};

/// Default server-side implementation of `TunnelService`.
///
/// Admission currently always allows — QUIC login performs token validation
/// upstream. Logging forwards phase timing and success/error counts through
/// the `MetricsSink` held by `ServerCtx`, so the service itself is stateless.
///
/// Route resolution lives in `RouteResolver` (see `server/plugins/vhost/`),
/// injected into the registry at startup.
pub struct DefaultTunnelService;

#[async_trait]
impl TunnelService for DefaultTunnelService {
    async fn admission(&self, _req: &AdmissionReq) -> Result<PhaseResult> {
        Ok(PhaseResult::Continue(()))
    }

    fn logging(&self, ctx: &ServerCtx, outcome: &PhaseOutcome) {
        // Per-connection timing. The request-completed counter lives on
        // `server/metrics.rs::request_completed` (named
        // `duotunnel_requests_total`) to preserve existing dashboards;
        // don't duplicate it here.
        if let Some(total) = outcome.timing.total() {
            let (status, error_kind) = if let Some(err_str) = &outcome.error {
                let mut kind = "unknown";
                let err_lower = err_str.to_lowercase();
                if err_lower.contains("quic open timed out") || err_lower.contains("open_bi timed out") {
                    kind = "quic_open_timed_out";
                } else if err_lower.contains("quic connection lost") || err_lower.contains("connection lost") {
                    kind = "quic_connection_lost";
                } else if err_lower.contains("quic connection fatal") || err_lower.contains("fatal connection error") {
                    kind = "quic_connection_fatal";
                } else if err_lower.contains("route not found") {
                    kind = "route_not_found";
                } else if err_lower.contains("no client available") {
                    kind = "no_client_available";
                } else if err_lower.contains("resolve") || err_lower.contains("dns") {
                    kind = "resolve_upstream";
                } else if err_lower.contains("upstream connect") || err_lower.contains("connect failed") {
                    kind = "upstream_connect";
                } else if err_lower.contains("forward") {
                    kind = "upstream_forward";
                } else if err_lower.contains("tls handshake") {
                    kind = "tls_handshake";
                } else if err_lower.contains("http upstream") {
                    kind = "http_upstream_request";
                }
                ("error", kind)
            } else {
                ("success", "none")
            };
            ctx.metrics.observe(
                "duotunnel_ingress_total_ms",
                total.as_secs_f64() * 1000.0,
                &[("status", status), ("error_kind", error_kind)],
            );
        }
    }
}
