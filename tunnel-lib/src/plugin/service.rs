use anyhow::Result;
use async_trait::async_trait;

use super::ctx::{AdmissionReq, PhaseOutcome, PhaseResult};

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

    fn logging(&self, outcome: &PhaseOutcome);
}
