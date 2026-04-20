//! Plugin system for duotunnel.
//!
//! Provides a trait-based, phase-ordered pipeline that decouples ingress
//! handlers and egress dialers from concrete symbols.  See `docs/plugins.md`
//! for the full design.
//!
//! ## Module layout
//!
//! | Module | Contents |
//! |---|---|
//! | `ctx` | `PhaseResult`, `ServerCtx`, `EgressCtx`, `Route`, `Timeouts`, … |
//! | `metrics` | `MetricsSink` trait + `NoopSink` |
//! | `ingress` | `IngressProtocolHandler`, `ProtocolHint`, `ProtocolKind` |
//! | `module` | `ConnectionModule` (cross-cutting) |
//! | `service` | `TunnelService` (service-level strategy) |
//! | `egress` | `LoadBalancer`, `UpstreamDialer`, `Resolver`, `SystemResolver` |
//! | `registry` | `PluginRegistry` |
//! | `dispatcher` | `IngressDispatcher` (runs the server-side phase pipeline) |

pub mod ctx;
pub mod dispatcher;
pub mod egress;
pub mod ingress;
pub mod metrics;
pub mod module;
pub mod registry;
pub mod route;
pub mod service;

// Convenience re-exports for the most commonly used types.
pub use ctx::{
    AdmissionReq, ConnectInfo, DialCtx, EgressCtx, PhaseOutcome, PhaseTiming, PhaseResult,
    PickCtx, Route, RouteCtx, ServerCtx, Target, Timeouts,
};
pub use dispatcher::IngressDispatcher;
pub use egress::{Connected, LoadBalancer, Resolver, SystemResolver, UpstreamDialer};
pub use ingress::{IngressProtocolHandler, ProtocolHint, ProtocolKind};
pub use metrics::{MetricsSink, NoopSink};
pub use module::ConnectionModule;
pub use registry::PluginRegistry;
pub use route::{NoRouteResolver, RouteResolver};
pub use service::TunnelService;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::Arc;

    // ── Minimal TunnelService impl for smoke tests ────────────────────────────

    struct AlwaysAllowService;

    #[async_trait]
    impl TunnelService for AlwaysAllowService {
        async fn admission(&self, _req: &AdmissionReq) -> Result<PhaseResult> {
            Ok(PhaseResult::Continue(()))
        }

        fn logging(&self, _ctx: &ServerCtx, _outcome: &PhaseOutcome) {}
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[test]
    fn noop_sink_compiles_as_trait_object() {
        let sink: Arc<dyn MetricsSink> = Arc::new(NoopSink);
        sink.incr("connections_total", &[("proto", "tcp")]);
        sink.observe("latency_ms", 1.5, &[]);
    }

    #[test]
    fn plugin_registry_default_has_safe_values() {
        let reg = PluginRegistry::new();
        assert!(reg.ingress_handlers.is_empty());
        assert!(reg.modules.is_empty());
        assert!(reg.dialers.is_empty());
        assert!(reg.lb.is_none());
    }

    #[test]
    fn module_order_is_sorted_on_insert() {
        let mut reg = PluginRegistry::new();

        struct OrderedModule(i32);
        #[async_trait]
        impl ConnectionModule for OrderedModule {
            fn order(&self) -> i32 { self.0 }
        }

        reg.add_module(Arc::new(OrderedModule(30)));
        reg.add_module(Arc::new(OrderedModule(10)));
        reg.add_module(Arc::new(OrderedModule(20)));

        let orders: Vec<i32> = reg.modules.iter().map(|m| m.order()).collect();
        assert_eq!(orders, vec![10, 20, 30]);
    }

    #[test]
    fn phase_result_is_continue() {
        let r: PhaseResult<i32> = PhaseResult::Continue(42);
        assert!(r.is_continue());
    }

    #[test]
    fn phase_result_reject_is_not_continue() {
        let r: PhaseResult = PhaseResult::Reject {
            status: 403,
            message: bytes::Bytes::from_static(b"forbidden"),
        };
        assert!(!r.is_continue());
    }

    #[tokio::test]
    async fn always_allow_service_passes_admission() {
        let svc = AlwaysAllowService;
        let req = AdmissionReq {
            peer_addr: "127.0.0.1:1234".parse().unwrap(),
            hint: None,
            token: None,
        };
        let result = svc.admission(&req).await.unwrap();
        assert!(result.is_continue());
    }

    #[test]
    fn validate_for_ingress_rejects_empty_registry() {
        let reg = PluginRegistry::new();
        let err = reg.validate_for_ingress().unwrap_err().to_string();
        assert!(err.contains("no ingress handlers"), "actual: {err}");
    }

    #[test]
    fn validate_for_ingress_rejects_missing_route_resolver() {
        let mut reg = PluginRegistry::new();

        struct DummyHandler;
        #[async_trait]
        impl IngressProtocolHandler for DummyHandler {
            fn protocol_kind(&self) -> ProtocolKind { ProtocolKind::Tcp }
            async fn handle(
                &self,
                _stream: tokio::net::TcpStream,
                _route: Option<Route>,
                _ctx: &ServerCtx,
            ) -> Result<()> { Ok(()) }
        }
        reg.register_ingress_handler(Arc::new(DummyHandler));

        let err = reg.validate_for_ingress().unwrap_err().to_string();
        assert!(err.contains("no route resolver"), "actual: {err}");
    }

    #[tokio::test]
    async fn no_route_resolver_rejects_with_404() {
        let resolver = NoRouteResolver;
        let hint = ProtocolHint::new(ProtocolKind::Http1, bytes::Bytes::new());
        let ctx = RouteCtx {
            listener_port: 8080,
            client_addr: "127.0.0.1:1234".parse().unwrap(),
            hint,
        };
        let result = resolver.resolve(&ctx).await.unwrap();
        match result {
            PhaseResult::Reject { status, .. } => assert_eq!(status, 404),
            _ => panic!("expected Reject"),
        }
    }
}
