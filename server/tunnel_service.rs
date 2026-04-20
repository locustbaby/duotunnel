use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use tunnel_lib::plugin::{AdmissionReq, MetricsSink, PhaseOutcome, PhaseResult, TunnelService};

/// Default server-side implementation of `TunnelService`.
///
/// Admission currently always allows — QUIC login performs token validation
/// upstream. Logging forwards phase timings and error counts into the injected
/// `MetricsSink`.
///
/// Route resolution lives in `RouteResolver` (see `server/plugins/vhost/`),
/// injected into the registry at startup.
pub struct DefaultTunnelService {
    pub metrics: Arc<dyn MetricsSink>,
}

#[async_trait]
impl TunnelService for DefaultTunnelService {
    async fn admission(&self, _req: &AdmissionReq) -> Result<PhaseResult> {
        Ok(PhaseResult::Continue(()))
    }

    fn logging(&self, outcome: &PhaseOutcome) {
        let status = if outcome.error.is_some() { "error" } else { "success" };
        self.metrics
            .incr("duotunnel_ingress_requests_total", &[("status", status)]);

        if let Some(total) = outcome.timing.total() {
            self.metrics.observe(
                "duotunnel_ingress_total_ms",
                total.as_secs_f64() * 1000.0,
                &[("status", status)],
            );
        }
    }
}
