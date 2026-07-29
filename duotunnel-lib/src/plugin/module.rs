use anyhow::Result;
use async_trait::async_trait;

use super::ctx::{AdmissionReq, PhaseOutcome, PhaseResult};

/// Cross-cutting concern that participates in the connection lifecycle.
///
/// Multiple `ConnectionModule`s can be registered; they execute in ascending
/// `order()` order.  Each module has its own context type — the framework
/// doesn't merge them.
///
/// Analogous to Pingora's `HttpModule` trait.
#[async_trait]
pub trait ConnectionModule: Send + Sync + 'static {
    /// Execution order within the module chain.  Lower values run first.
    /// Modules with the same order value run in registration order.
    fn order(&self) -> i32 {
        0
    }

    /// Called before the `TunnelService::admission` phase.
    ///
    /// Useful for IP-based allow/deny lists, rate limiting, or anything that
    /// should short-circuit before token validation.
    async fn pre_admission(&self, _req: &AdmissionReq) -> Result<PhaseResult> {
        Ok(PhaseResult::Continue(()))
    }

    /// Called after the tunnel is fully closed (success or error).
    ///
    /// Suitable for access logging, metrics flush, or cleanup.  Errors
    /// returned here are logged and swallowed — do not rely on them propagating.
    async fn on_complete(&self, _outcome: &PhaseOutcome) {}
}
