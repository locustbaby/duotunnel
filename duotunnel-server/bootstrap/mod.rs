use anyhow::Result;
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub(crate) mod cli;
pub(crate) mod config;

use self::cli::Cli;
use self::config::{
    IngressListener, IngressMode, ServerConfigFile, ServerEgressUpstream, TunnelManagement,
};
use crate::control::local_auth;
use crate::egress;
use crate::ingress::listener_mgr;
use crate::ingress::plugins;
use crate::ingress::registry::{new_shared_registry, SharedRegistry};
use duotunnel_lib::{AuthError, AuthResult, HttpClientParams, RouteTarget, VhostRouter};

pub(crate) struct ServerBootstrap {
    config_path: String,
    config: ServerConfigFile,
    mode: ServerMode,
}

#[derive(Clone)]
pub(crate) enum ServerMode {
    ControlPlane {
        ctld_addr: String,
        ctld_token: Option<String>,
    },
}

impl ServerBootstrap {
    pub(crate) fn from_cli(cli: &Cli) -> Result<Self> {
        let config = ServerConfigFile::load(&cli.config)?;
        let mode = ServerMode::ControlPlane {
            ctld_addr: cli.ctld_addr.clone(),
            ctld_token: cli.resolved_ctld_token(),
        };
        Ok(Self {
            config_path: cli.config.clone(),
            config,
            mode,
        })
    }

    pub(crate) fn log_level(&self) -> &str {
        self.config.server.log_level.as_deref().unwrap_or("info")
    }

    pub(crate) fn config_path(&self) -> &str {
        &self.config_path
    }

    pub(crate) fn tunnel_port(&self) -> u16 {
        self.config.server.tunnel_port
    }

    pub(crate) fn metrics_port(&self) -> Option<u16> {
        self.config.server.metrics_port
    }

    pub(crate) fn pki(&self) -> &duotunnel_lib::PkiParams {
        &self.config.server.pki
    }

    pub(crate) fn connection_registry_capacity(&self) -> usize {
        self.config.server.connection_registry_capacity
    }

    pub(crate) fn mode(&self) -> &ServerMode {
        &self.mode
    }

    pub(crate) fn uses_control_plane(&self) -> bool {
        matches!(self.mode, ServerMode::ControlPlane { .. })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub(crate) type HttpRouterMap = Arc<HashMap<u16, Arc<VhostRouter<RouteTarget>>>>;

pub(crate) struct RoutingSnapshot {
    http_routers: HttpRouterMap,
    tunnel_management: Arc<TunnelManagement>,
    egress_map: Arc<egress::ServerEgressMap>,
    egress_rules: Vec<duotunnel_lib::EgressVhostRuleDef>,
}

impl RoutingSnapshot {
    pub(crate) fn ingress_listeners(&self) -> &[IngressListener] {
        &self.tunnel_management.server_ingress_routing.listeners
    }

    pub(crate) fn client_config_for_group(
        &self,
        group_id: &str,
    ) -> Option<duotunnel_lib::ClientConfig> {
        self::config::build_client_config_for_group(
            &self.tunnel_management,
            &self.egress_rules,
            group_id,
        )
    }

    pub(crate) fn route_target(&self, listener_port: u16, host: &str) -> Option<RouteTarget> {
        self.http_routers
            .get(&listener_port)
            .and_then(|router| router.get(host))
    }

    pub(crate) fn egress_map(&self) -> Arc<egress::ServerEgressMap> {
        self.egress_map.clone()
    }

    pub(crate) fn tcp_route(
        &self,
        listener_port: u16,
    ) -> Option<(duotunnel_lib::GroupId, duotunnel_lib::ProxyName)> {
        self.tunnel_management
            .server_ingress_routing
            .listeners
            .iter()
            .find(|listener| listener.port == listener_port)
            .and_then(|listener| match &listener.mode {
                IngressMode::Tcp(config) => {
                    Some((config.client_group.clone(), config.proxy_name.clone()))
                }
                IngressMode::Http(_) => None,
            })
    }
}

pub(crate) struct RuntimeGeneration {
    epoch: Arc<str>,
    sequence: u64,
    content_hash: Arc<str>,
    routing: RoutingSnapshot,
    token_map: Arc<local_auth::TokenMap>,
}

impl RuntimeGeneration {
    pub(crate) fn local(routing: RoutingSnapshot, token_map: Arc<local_auth::TokenMap>) -> Self {
        Self {
            epoch: Arc::from("local"),
            sequence: 0,
            content_hash: Arc::from("local"),
            routing,
            token_map,
        }
    }

    pub(crate) fn from_control_plane(
        epoch: impl Into<Arc<str>>,
        sequence: u64,
        content_hash: impl Into<Arc<str>>,
        routing: RoutingSnapshot,
        token_map: Arc<local_auth::TokenMap>,
    ) -> Self {
        Self {
            epoch: epoch.into(),
            sequence,
            content_hash: content_hash.into(),
            routing,
            token_map,
        }
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) fn epoch(&self) -> &str {
        &self.epoch
    }

    pub(crate) fn content_hash(&self) -> &str {
        &self.content_hash
    }

    pub(crate) fn routing(&self) -> &RoutingSnapshot {
        &self.routing
    }

    pub(crate) fn token_map(&self) -> &local_auth::TokenMap {
        &self.token_map
    }
}

struct IngressRuntime {
    config: Arc<ServerConfigFile>,
    tcp_params: duotunnel_lib::TcpParams,
    proxy_buffer_params: duotunnel_lib::ProxyBufferParams,
    peek_buf_pool: crate::PeekBufPool,
    generation: Arc<ArcSwap<RuntimeGeneration>>,
    upstream_health: Arc<duotunnel_lib::proxy::upstream::UpstreamHealthRegistry>,
    listeners: listener_mgr::ListenerManager,
    plugin_registry: Arc<duotunnel_lib::plugin::PluginRegistry>,
    /// Listeners must always run on the multi-threaded proxy runtime, never on
    /// whichever runtime happened to apply a config update: the control-plane
    /// runtime is single-threaded and outlived by the listeners it would
    /// otherwise own.
    proxy_handle: tokio::runtime::Handle,
}

struct ConnectionRuntime {
    registry: SharedRegistry,
}

pub(crate) struct ServerState {
    ingress: IngressRuntime,
    connection: ConnectionRuntime,
    health: Arc<crate::runtime::health::ServerHealthFacts>,
    security_apply_gate: tokio::sync::RwLock<()>,
    listener_apply_gate: tokio::sync::Mutex<()>,
}

impl ServerState {
    pub(crate) fn http_client_params(&self) -> HttpClientParams {
        HttpClientParams::from(&self.ingress.config.server.http_pool)
    }

    pub(crate) fn proxy_handle(&self) -> &tokio::runtime::Handle {
        &self.ingress.proxy_handle
    }

    pub(crate) fn upstream_health(
        &self,
    ) -> Arc<duotunnel_lib::proxy::upstream::UpstreamHealthRegistry> {
        self.ingress.upstream_health.clone()
    }

    pub(crate) fn accept_workers(&self) -> usize {
        duotunnel_lib::resolve_accept_workers(self.ingress.config.server.accept_workers)
    }

    pub(crate) fn emfile_backoff(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.ingress.config.server.overload.emfile_backoff_ms)
    }

    pub(crate) fn open_stream_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.ingress.config.server.open_stream_timeout_ms)
    }

    pub(crate) fn tunnel_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([0, 0, 0, 0], self.ingress.config.server.tunnel_port))
    }

    pub(crate) fn quic_transport_params(&self) -> duotunnel_lib::QuicTransportParams {
        duotunnel_lib::QuicTransportParams::from(&self.ingress.config.server.quic)
    }

    pub(crate) fn login_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.ingress.config.server.login_timeout_secs)
    }

    pub(crate) fn max_unauthenticated_connections(&self) -> usize {
        self.ingress.config.server.max_unauthenticated_connections
    }

    pub(crate) fn runtime_generation(&self) -> Arc<RuntimeGeneration> {
        self.ingress.generation.load_full()
    }

    pub(crate) fn admit_runtime_generation(&self) -> Option<Arc<RuntimeGeneration>> {
        if !self.health.admits_new_work() {
            return None;
        }
        let generation = self.runtime_generation();
        self.health.admits_new_work().then_some(generation)
    }

    pub(crate) fn routing_snapshot(&self) -> Arc<RuntimeGeneration> {
        self.runtime_generation()
    }

    pub(crate) fn ingress_listeners(&self) -> Vec<IngressListener> {
        self.routing_snapshot()
            .routing()
            .ingress_listeners()
            .to_vec()
    }

    pub(crate) fn client_config_for_generation(
        &self,
        generation: &RuntimeGeneration,
        group_id: &str,
    ) -> duotunnel_lib::ClientConfig {
        generation
            .routing()
            .client_config_for_group(group_id)
            .unwrap_or_default()
    }

    pub(crate) fn publish_generation(&self, generation: Arc<RuntimeGeneration>) {
        info!(
            epoch = generation.epoch(),
            sequence = generation.sequence(),
            content_hash = generation.content_hash(),
            "publishing runtime generation"
        );
        self.ingress.generation.store(generation);
    }

    pub(crate) async fn authenticate_pinned(
        &self,
        raw_token: &str,
    ) -> std::result::Result<(AuthResult, Arc<RuntimeGeneration>), AuthError> {
        let generation = self.runtime_generation();
        local_auth::authenticate(generation.token_map(), raw_token)
            .map(|result| (result, generation))
    }

    pub(crate) fn tcp_params(&self) -> &duotunnel_lib::TcpParams {
        &self.ingress.tcp_params
    }

    pub(crate) fn relay_buf_size(&self) -> usize {
        self.ingress.proxy_buffer_params.relay_buf_size
    }

    pub(crate) fn proxy_buffer_params(&self) -> &duotunnel_lib::ProxyBufferParams {
        &self.ingress.proxy_buffer_params
    }

    pub(crate) fn sniff_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.ingress.proxy_buffer_params.sniff_timeout_ms)
    }

    pub(crate) fn peek_buf_pool(&self) -> &crate::PeekBufPool {
        &self.ingress.peek_buf_pool
    }

    pub(crate) fn plugin_registry(&self) -> &Arc<duotunnel_lib::plugin::PluginRegistry> {
        &self.ingress.plugin_registry
    }

    pub(crate) fn registry(&self) -> &SharedRegistry {
        &self.connection.registry
    }

    pub(crate) fn listeners(&self) -> &listener_mgr::ListenerManager {
        &self.ingress.listeners
    }

    pub(crate) fn health(&self) -> &Arc<crate::runtime::health::ServerHealthFacts> {
        &self.health
    }

    pub(crate) fn security_apply_gate(&self) -> &tokio::sync::RwLock<()> {
        &self.security_apply_gate
    }

    pub(crate) fn listener_apply_gate(&self) -> &tokio::sync::Mutex<()> {
        &self.listener_apply_gate
    }
}

pub(crate) async fn build_server_state(bootstrap: &ServerBootstrap) -> Result<Arc<ServerState>> {
    info!("running with resident ctld control plane; no local SQLite stores");

    let http_params = HttpClientParams::from(&bootstrap.config.server.http_pool);
    let proxy_buffer_params =
        duotunnel_lib::ProxyBufferParams::from(&bootstrap.config.server.proxy_buffers);
    let tm = TunnelManagement::default();
    let egress = ServerEgressUpstream::default();
    let upstream_health =
        Arc::new(duotunnel_lib::proxy::upstream::UpstreamHealthRegistry::default());
    let initial_snapshot = build_routing_snapshot_with_health(
        &tm,
        &egress,
        &http_params,
        upstream_health.clone(),
        &proxy_buffer_params,
    )?;
    let peek_buf_pool = crate::PeekBufPool::new(proxy_buffer_params.peek_buf_size);
    let max_streams = duotunnel_lib::QuicTransportParams::from(&bootstrap.config.server.quic)
        .max_concurrent_streams;
    let overload_limits = bootstrap.config.server.overload.resolve(max_streams);
    info!(
        max_concurrent_streams = max_streams,
        max_pending_streams = overload_limits.max_pending_streams,
        "overload protection resolved"
    );
    let shard_count = duotunnel_lib::resolve_shard_count(bootstrap.config.server.quic.shards, None);
    let accept_workers =
        duotunnel_lib::resolve_accept_workers(bootstrap.config.server.accept_workers);
    let connection_registry_capacity = bootstrap.connection_registry_capacity();
    info!(
        shards = shard_count,
        accept_workers = accept_workers,
        connection_registry_capacity,
        configured_worker_threads = duotunnel_lib::configured_worker_threads(),
        cpu_parallelism = duotunnel_lib::available_parallelism(),
        cgroup_cpu_limit = ?duotunnel_lib::cgroup_cpu_limit(),
        effective_parallelism = duotunnel_lib::effective_runtime_parallelism(),
        "server QUIC ownership topology resolved"
    );
    let shared_registry = new_shared_registry(
        shard_count,
        max_streams,
        overload_limits.max_pending_streams,
        connection_registry_capacity,
    );
    let registry_capacity = shared_registry.capacity_snapshot();
    info!(
        active = registry_capacity.active,
        available = registry_capacity.available,
        exhausted = registry_capacity.exhausted,
        "server connection registry capacity initialized"
    );
    let generation = Arc::new(ArcSwap::from_pointee(RuntimeGeneration::local(
        initial_snapshot,
        Arc::new(HashMap::new()),
    )));
    let health = Arc::new(crate::runtime::health::ServerHealthFacts::new(
        bootstrap.uses_control_plane(),
    ));
    let plugin_registry = {
        use duotunnel_lib::plugin::PluginRegistry;

        let mut reg = PluginRegistry::new();
        let route_resolver: Arc<dyn duotunnel_lib::plugin::RouteResolver> =
            Arc::new(plugins::vhost::VhostPlugin {
                generation: generation.clone(),
            });
        reg.register_ingress_handler(Arc::new(plugins::tls::TlsHandler {
            registry: shared_registry.clone(),
            generation: generation.clone(),
            health: health.clone(),
        }));
        reg.register_ingress_handler(Arc::new(plugins::h2c::H2cHandler {
            registry: shared_registry.clone(),
            generation: generation.clone(),
            health: health.clone(),
            single_authority: bootstrap.config.server.h2_single_authority,
        }));
        reg.register_ingress_handler(Arc::new(plugins::h1::H1Handler {
            registry: shared_registry.clone(),
        }));
        reg.register_ingress_handler(Arc::new(plugins::tcp_pass::TcpPassHandler {
            registry: shared_registry.clone(),
        }));
        reg.set_route_resolver(route_resolver);
        reg.set_metrics_sink(Arc::new(plugins::prometheus::PrometheusSink));
        reg.validate_for_ingress()?;
        Arc::new(reg)
    };

    Ok(Arc::new(ServerState {
        ingress: IngressRuntime {
            config: Arc::new(bootstrap.config.clone()),
            tcp_params: duotunnel_lib::TcpParams::from(&bootstrap.config.server.tcp),
            proxy_buffer_params,
            peek_buf_pool,
            generation,
            upstream_health,
            listeners: listener_mgr::ListenerManager::new(),
            plugin_registry,
            proxy_handle: tokio::runtime::Handle::current(),
        },
        connection: ConnectionRuntime {
            registry: shared_registry,
        },
        health,
        security_apply_gate: tokio::sync::RwLock::new(()),
        listener_apply_gate: tokio::sync::Mutex::new(()),
    }))
}

pub(crate) fn build_routing_snapshot_with_health(
    tm: &TunnelManagement,
    egress: &ServerEgressUpstream,
    http_params: &HttpClientParams,
    upstream_health: Arc<duotunnel_lib::proxy::upstream::UpstreamHealthRegistry>,
    proxy_buffer_params: &duotunnel_lib::ProxyBufferParams,
) -> Result<RoutingSnapshot> {
    let mut http_routers: HashMap<u16, Arc<VhostRouter<RouteTarget>>> = HashMap::new();
    for listener in &tm.server_ingress_routing.listeners {
        if let IngressMode::Http(cfg) = &listener.mode {
            let router: VhostRouter<RouteTarget> = VhostRouter::new();
            for rule in &cfg.vhost {
                router.add_route(
                    &rule.match_host,
                    RouteTarget {
                        group_id: rule.client_group.clone(),
                        proxy_name: rule.proxy_name.clone(),
                    },
                );
            }
            info!(
                port = listener.port,
                routes = router.len(),
                "http listener router initialized"
            );
            http_routers.insert(listener.port, Arc::new(router));
        }
    }
    let egress_map = egress::ServerEgressMap::from_config_with_health(
        egress,
        http_params,
        upstream_health,
        proxy_buffer_params,
    )?;
    let egress_rules = egress
        .rules
        .vhost
        .iter()
        .map(|r| duotunnel_lib::EgressVhostRuleDef {
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        })
        .collect();
    Ok(RoutingSnapshot {
        http_routers: Arc::new(http_routers),
        tunnel_management: Arc::new(tm.clone()),
        egress_map: Arc::new(egress_map),
        egress_rules,
    })
}
