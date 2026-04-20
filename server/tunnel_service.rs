use anyhow::Result;
use async_trait::async_trait;

use tunnel_lib::plugin::{AdmissionReq, PhaseOutcome, PhaseResult, TunnelService};

/// Default server-side implementation of `TunnelService`.
///
/// Admission currently always allows — QUIC login performs token validation
/// upstream. Logging is a placeholder until PR ④ wires `MetricsSink` in.
///
/// Route resolution lives in `RouteResolver` (see `server/plugins/vhost/`),
/// injected into the registry at startup.
pub struct DefaultTunnelService;

#[async_trait]
impl TunnelService for DefaultTunnelService {
    async fn admission(&self, _req: &AdmissionReq) -> Result<PhaseResult> {
        Ok(PhaseResult::Continue(()))
    }

    fn logging(&self, _outcome: &PhaseOutcome) {}
}
