use anyhow::Result;
use async_trait::async_trait;

use super::ctx::{AdmissionReq, PhaseOutcome, PhaseResult, Route, RouteCtx};

/// Service-level strategy trait for the tunnel pipeline.
///
/// One implementation per deployed service (server-side or client-side).
/// The framework calls the methods in fixed phase order; implementations
/// override only the phases they care about.
///
/// Analogous to Pingora's `ProxyHttp` trait.
#[async_trait]
pub trait TunnelService: Send + Sync + 'static {
    // ── Phase 2: Admission ────────────────────────────────────────────────────

    /// Decide whether to accept the incoming connection.
    ///
    /// Return `PhaseResult::Reject` to close the connection immediately
    /// (optionally with a status code and human-readable message in logs).
    /// Return `PhaseResult::Continue(())` to advance to route resolution.
    async fn admission(
        &self,
        req: &AdmissionReq,
    ) -> Result<PhaseResult>;

    // ── Phase 3: Route resolve ────────────────────────────────────────────────

    /// Map the incoming connection to a `Route` (group_id + proxy_name).
    ///
    /// Return `PhaseResult::Continue(route)` on success, or
    /// `PhaseResult::Reject` if no matching route exists.
    async fn resolve_route(
        &self,
        ctx: &RouteCtx,
    ) -> Result<PhaseResult<Route>>;

    // ── Phase 5: Logging ──────────────────────────────────────────────────────

    /// Emit access logs and metrics after the tunnel closes.
    ///
    /// This method must not fail — errors are swallowed by the framework.
    fn logging(&self, outcome: &PhaseOutcome);
}
