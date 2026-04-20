use anyhow::Result;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;

use tunnel_lib::plugin::{PhaseResult, Route, RouteCtx, RouteResolver};

use crate::RoutingSnapshot;

/// Route resolver that looks up the vhost router from the current
/// `RoutingSnapshot` (exact + wildcard match).
///
/// Internally uses `VhostRouter<RouteTarget>` which is already populated by
/// `build_routing_snapshot` — no duplicate data structures.
pub struct VhostPlugin {
    pub routing: Arc<ArcSwap<RoutingSnapshot>>,
}

#[async_trait]
impl RouteResolver for VhostPlugin {
    async fn resolve(&self, ctx: &RouteCtx) -> Result<PhaseResult<Route>> {
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
                message: format!("no vhost route for host '{}' on port {}", host, ctx.listener_port).into(),
            }),
        }
    }
}
