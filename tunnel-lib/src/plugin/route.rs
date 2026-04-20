use anyhow::Result;
use async_trait::async_trait;

use super::ctx::{PhaseResult, Route, RouteCtx};

/// Abstraction over route-resolution strategies.
///
/// Implementations live in `server/plugins/vhost/` (vhost lookup) and
/// `server/plugins/static_port/` (direct group:proxy mapping).
/// The CORE fallback is `StaticPortResolver`.
#[async_trait]
pub trait RouteResolver: Send + Sync + 'static {
    /// Map an incoming connection to a `Route`.
    ///
    /// Returns `PhaseResult::Continue(route)` on success or
    /// `PhaseResult::Reject` when no matching route is found.
    async fn resolve(&self, ctx: &RouteCtx) -> Result<PhaseResult<Route>>;
}

/// CORE fallback resolver: always rejects (no route configured).
///
/// Used when no RouteResolver plugin is installed so that startup fails
/// explicitly rather than silently routing nowhere.
pub struct NoRouteResolver;

#[async_trait]
impl RouteResolver for NoRouteResolver {
    async fn resolve(&self, ctx: &RouteCtx) -> Result<PhaseResult<Route>> {
        Ok(PhaseResult::Reject {
            status: 404,
            message: format!(
                "no RouteResolver configured for port {}",
                ctx.listener_port
            )
            .into(),
        })
    }
}
