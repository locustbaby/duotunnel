use std::collections::HashMap;
use std::sync::Arc;

use super::egress::{LoadBalancer, Resolver, SystemResolver, UpstreamDialer};
use super::ingress::{IngressProtocolHandler, ProtocolKind};
use super::metrics::{MetricsSink, NoopSink};
use super::module::ConnectionModule;

/// Central registry of all runtime plugin instances.
///
/// Constructed once at startup from configuration; then handed to
/// `IngressDispatcher` and the egress `ProxyEngine` as an `Arc<PluginRegistry>`.
///
/// All fields are `pub` so callers can read them directly — the registry is
/// immutable after construction.
pub struct PluginRegistry {
    /// Ingress protocol handlers keyed by `ProtocolKind` for O(1) dispatch.
    pub ingress_handlers: HashMap<ProtocolKind, Arc<dyn IngressProtocolHandler>>,

    /// Connection-level modules, sorted ascending by `order()`.
    pub modules: Vec<Arc<dyn ConnectionModule>>,

    /// Active metrics backend.  Defaults to `NoopSink`.
    pub metrics_sink: Arc<dyn MetricsSink>,

    /// Egress dialers tried in registration order until `matches_scheme` returns true.
    pub dialers: Vec<Arc<dyn UpstreamDialer>>,

    /// Active load-balancing strategy.
    pub lb: Option<Arc<dyn LoadBalancer>>,

    /// Active DNS resolver.  Defaults to `SystemResolver`.
    pub resolver: Arc<dyn Resolver>,
}

impl PluginRegistry {
    /// Create an empty registry with safe defaults (noop metrics, system resolver).
    pub fn new() -> Self {
        Self {
            ingress_handlers: HashMap::new(),
            modules: Vec::new(),
            metrics_sink: Arc::new(NoopSink),
            dialers: Vec::new(),
            lb: None,
            resolver: Arc::new(SystemResolver),
        }
    }

    // ── Builder-style helpers ─────────────────────────────────────────────────

    pub fn register_ingress_handler(&mut self, handler: Arc<dyn IngressProtocolHandler>) {
        self.ingress_handlers.insert(handler.protocol_kind(), handler);
    }

    pub fn add_module(&mut self, module: Arc<dyn ConnectionModule>) {
        self.modules.push(module);
        self.modules.sort_by_key(|m| m.order());
    }

    pub fn set_metrics_sink(&mut self, sink: Arc<dyn MetricsSink>) {
        self.metrics_sink = sink;
    }

    pub fn add_dialer(&mut self, dialer: Arc<dyn UpstreamDialer>) {
        self.dialers.push(dialer);
    }

    pub fn set_lb(&mut self, lb: Arc<dyn LoadBalancer>) {
        self.lb = Some(lb);
    }

    pub fn set_resolver(&mut self, resolver: Arc<dyn Resolver>) {
        self.resolver = resolver;
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
