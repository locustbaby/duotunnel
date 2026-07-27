use anyhow::Result;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;

use tunnel_lib::plugin::{PhaseResult, Route, RouteCtx, RouteResolver};

use crate::RuntimeGeneration;

/// Route resolver that looks up the vhost router from the current
/// `RoutingSnapshot` (exact + wildcard match).
///
/// Internally uses `VhostRouter<RouteTarget>` which is already populated by
/// `build_routing_snapshot` — no duplicate data structures.
pub struct VhostPlugin {
    pub generation: Arc<ArcSwap<RuntimeGeneration>>,
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

        let generation = self.generation.load();
        if generation.sequence() != ctx.runtime_generation {
            return Ok(PhaseResult::Reject {
                status: 503,
                message: b"runtime generation changed during admission"
                    .as_slice()
                    .into(),
            });
        }
        let target = generation.routing().route_target(ctx.listener_port, &host);

        match target {
            Some(t) => Ok(PhaseResult::Continue(Route::new(
                t.group_id.clone(),
                t.proxy_name.clone(),
            ))),
            None => Ok(PhaseResult::Reject {
                status: 404,
                message: format!(
                    "no vhost route for host '{}' on port {}",
                    host, ctx.listener_port
                )
                .into(),
            }),
        }
    }
}
