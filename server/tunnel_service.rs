use anyhow::Result;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;

use tunnel_lib::plugin::{
    AdmissionReq, PhaseOutcome, PhaseResult, Route, RouteCtx, TunnelService,
};

use crate::RoutingSnapshot;

// Re-export so handlers can use it without a full path.
pub use pported::PortedTunnelService;

mod pported {
    use super::*;

    /// Wraps `DefaultTunnelService` to inject a known listener `port` into
    /// `RouteCtx` so vhost lookup works correctly even when `peer_addr.port()`
    /// is the ephemeral client port, not the listener port.
    pub struct PortedTunnelService {
        pub inner: DefaultTunnelService,
        pub port: u16,
    }

    #[async_trait]
    impl TunnelService for PortedTunnelService {
        async fn admission(&self, req: &AdmissionReq) -> Result<PhaseResult> {
            self.inner.admission(req).await
        }

        async fn resolve_route(&self, ctx: &RouteCtx) -> Result<PhaseResult<Route>> {
            // Rebuild ctx with the correct listener port.
            let patched = RouteCtx {
                listener_port: self.port,
                ..ctx.clone()
            };
            self.inner.resolve_route(&patched).await
        }

        fn logging(&self, outcome: &PhaseOutcome) {
            self.inner.logging(outcome);
        }
    }
}

/// Default server-side implementation of `TunnelService`.
///
/// Phase 2 (admission): always allows for now — token validation is done
/// upstream in the QUIC login flow.  Will be replaced by `admission-token`
/// plugin in PR ③.
///
/// Phase 3 (route resolve): looks up the vhost router from the current
/// `RoutingSnapshot`.
pub struct DefaultTunnelService {
    pub routing: Arc<ArcSwap<RoutingSnapshot>>,
}

#[async_trait]
impl TunnelService for DefaultTunnelService {
    async fn admission(&self, _req: &AdmissionReq) -> Result<PhaseResult> {
        Ok(PhaseResult::Continue(()))
    }

    async fn resolve_route(&self, ctx: &RouteCtx) -> Result<PhaseResult<Route>> {
        let host = ctx
            .hint
            .sni
            .clone()
            .or_else(|| ctx.hint.authority.clone())
            .unwrap_or_default();

        let snapshot = self.routing.load();
        let target = snapshot
            .http_routers
            .get(&ctx.listener_port)
            .and_then(|router| router.get(&host));

        match target {
            Some(t) => Ok(PhaseResult::Continue(Route::new(
                t.group_id.as_ref(),
                t.proxy_name.as_ref(),
            ))),
            None => Ok(PhaseResult::Reject {
                status: 404,
                message: format!("no route for host: {}", host).into(),
            }),
        }
    }

    fn logging(&self, _outcome: &PhaseOutcome) {
        // Metrics will be hooked here in PR ④.
    }
}
