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
            let status = if outcome.error.is_some() { "error" } else { "success" };
            ctx.metrics.observe(
                "duotunnel_ingress_total_ms",
                total.as_secs_f64() * 1000.0,
                &[("status", status)],
            );
        }
    }
}
