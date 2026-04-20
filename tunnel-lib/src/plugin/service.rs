use anyhow::Result;
use async_trait::async_trait;

use super::ctx::{AdmissionReq, PhaseOutcome, PhaseResult, ServerCtx};

/// Service-level strategy trait for the tunnel pipeline.
///
/// One implementation per deployed service (server-side or client-side).
/// The framework calls the methods in fixed phase order; implementations
/// override only the phases they care about.
///
/// Analogous to Pingora's `ProxyHttp` trait. Route resolution is owned by
/// `RouteResolver` in the plugin registry, not this trait.
#[async_trait]
pub trait TunnelService: Send + Sync + 'static {
    async fn admission(
        &self,
        req: &AdmissionReq,
    ) -> Result<PhaseResult>;

    /// Emit access logs / metrics after the tunnel closes.
    ///
    /// Implementations **must not panic** — the dispatcher calls this on
    /// every path (success, admission reject, route reject, handler error)
    /// and catches unwinds to protect the accept worker.
    fn logging(&self, ctx: &ServerCtx, outcome: &PhaseOutcome);
}
