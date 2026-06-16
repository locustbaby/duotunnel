use anyhow::Result;
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub(crate) mod cli;
pub(crate) mod config;

use self::cli::Cli;
use self::config::{
    ConfigSource, DbSource, FileSource, IngressListener, IngressMode, MergedSource,
    ServerConfigFile, ServerEgressUpstream, TunnelManagement,
};
use crate::control::local_auth;
use crate::control::null_stores::{NullConfigSource, NullRuleStore};
use crate::egress;
use crate::ingress::listener_mgr;
use crate::ingress::plugins;
use crate::ingress::registry::{new_shared_registry, SharedRegistry};
use tunnel_lib::{HttpClientParams, RouteTarget, VhostRouter};
use tunnel_store::{AuthStore, RuleStore};

pub(crate) struct ServerBootstrap {
    config_path: String,
    config: ServerConfigFile,
    mode: ServerMode,
}

#[derive(Clone)]
pub(crate) enum ServerMode {
    Standalone,
    Managed {
        ctld_addr: String,
        ctld_token: Option<String>,
    },
}

impl ServerBootstrap {
    pub(crate) fn from_cli(cli: &Cli) -> Result<Self> {
        let config = ServerConfigFile::load(&cli.config)?;
        let mode = match cli.ctld_addr.clone() {
            Some(ctld_addr) => ServerMode::Managed {
                ctld_addr,
                ctld_token: cli.resolved_ctld_token(),
            },
            None => ServerMode::Standalone,
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

    pub(crate) fn pki(&self) -> &tunnel_lib::PkiParams {
        &self.config.server.pki
    }

    pub(crate) fn mode(&self) -> &ServerMode {
        &self.mode
    }

    pub(crate) fn is_ctld_managed(&self) -> bool {
        matches!(self.mode, ServerMode::Managed { .. })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.is_ctld_managed() {
            return Ok(());
        }
        self::config::validate_server_config(&self.config)
    }
}

pub(crate) type HttpRouterMap = Arc<HashMap<u16, Arc<VhostRouter<RouteTarget>>>>;

pub(crate) struct RoutingSnapshot {
    http_routers: HttpRouterMap,
    tunnel_management: Arc<TunnelManagement>,
    egress_map: Arc<egress::ServerEgressMap>,
    egress_rules: Vec<tunnel_lib::EgressVhostRuleDef>,
}

impl RoutingSnapshot {
    pub(crate) fn ingress_listeners(&self) -> &[IngressListener] {
        &self.tunnel_management.server_ingress_routing.listeners
    }

    pub(crate) fn client_config_for_group(
        &self,
        group_id: &str,
    ) -> Option<tunnel_lib::ClientConfig> {
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
}

struct IngressRuntime {
    config: Arc<ServerConfigFile>,
    tcp_params: tunnel_lib::TcpParams,
    proxy_buffer_params: tunnel_lib::ProxyBufferParams,
    peek_buf_pool: crate::PeekBufPool,
    routing: Arc<ArcSwap<RoutingSnapshot>>,
    listeners: listener_mgr::ListenerManager,
    overload_limits: tunnel_lib::OverloadLimits,
    plugin_registry: Arc<tunnel_lib::plugin::PluginRegistry>,
}

struct ConnectionRuntime {
    registry: SharedRegistry,
}

struct ControlRuntime {
    auth_store: Arc<dyn AuthStore>,
    rule_store: Arc<dyn RuleStore>,
    config_source: Arc<dyn ConfigSource>,
    revocation_tx: tokio::sync::broadcast::Sender<String>,
    local_token_cache: Option<Arc<local_auth::LocalTokenCache>>,
}

pub(crate) struct ServerState {
    ingress: IngressRuntime,
    connection: ConnectionRuntime,
    control: ControlRuntime,
}

impl ServerState {
    pub(crate) fn http_client_params(&self) -> HttpClientParams {
        HttpClientParams::from(&self.ingress.config.server.http_pool)
    }

    pub(crate) fn accept_workers(&self) -> usize {
        self.ingress
            .config
            .server
            .accept_workers
            .unwrap_or(tunnel_lib::DEFAULT_ACCEPT_WORKERS)
            .max(1)
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

    pub(crate) fn quic_transport_params(&self) -> tunnel_lib::QuicTransportParams {
        tunnel_lib::QuicTransportParams::from(&self.ingress.config.server.quic)
    }

    pub(crate) fn login_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.ingress.config.server.login_timeout_secs)
    }

    pub(crate) fn routing_snapshot(&self) -> arc_swap::Guard<Arc<RoutingSnapshot>> {
        self.ingress.routing.load()
    }

    pub(crate) fn ingress_listeners(&self) -> Vec<IngressListener> {
        self.routing_snapshot().ingress_listeners().to_vec()
    }

    pub(crate) fn egress_map(&self) -> Arc<egress::ServerEgressMap> {
        self.routing_snapshot().egress_map()
    }

    pub(crate) fn client_config_for_group(&self, group_id: &str) -> tunnel_lib::ClientConfig {
        self.routing_snapshot()
            .client_config_for_group(group_id)
            .unwrap_or_default()
    }

    pub(crate) fn replace_routing(&self, snapshot: RoutingSnapshot) {
        self.ingress.routing.store(Arc::new(snapshot));
    }

    pub(crate) fn tcp_params(&self) -> &tunnel_lib::TcpParams {
        &self.ingress.tcp_params
    }

    pub(crate) fn relay_buf_size(&self) -> usize {
        self.ingress.proxy_buffer_params.relay_buf_size
    }

    pub(crate) fn sniff_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.ingress.proxy_buffer_params.sniff_timeout_ms)
    }

    pub(crate) fn peek_buf_pool(&self) -> &crate::PeekBufPool {
        &self.ingress.peek_buf_pool
    }

    pub(crate) fn overload_limits(&self) -> &tunnel_lib::OverloadLimits {
        &self.ingress.overload_limits
    }

    pub(crate) fn plugin_registry(&self) -> &Arc<tunnel_lib::plugin::PluginRegistry> {
        &self.ingress.plugin_registry
    }

    pub(crate) fn registry(&self) -> &SharedRegistry {
        &self.connection.registry
    }

    pub(crate) fn auth_store(&self) -> &Arc<dyn AuthStore> {
        &self.control.auth_store
    }

    pub(crate) fn rule_store(&self) -> &Arc<dyn RuleStore> {
        &self.control.rule_store
    }

    pub(crate) fn config_source(&self) -> &Arc<dyn ConfigSource> {
        &self.control.config_source
    }

    pub(crate) fn revocation_tx(&self) -> &tokio::sync::broadcast::Sender<String> {
        &self.control.revocation_tx
    }

    pub(crate) fn local_token_cache(&self) -> Option<&Arc<local_auth::LocalTokenCache>> {
        self.control.local_token_cache.as_ref()
    }

    pub(crate) fn listeners(&self) -> &listener_mgr::ListenerManager {
        &self.ingress.listeners
    }
}

pub(crate) async fn build_server_state(bootstrap: &ServerBootstrap) -> Result<Arc<ServerState>> {
    #[allow(clippy::type_complexity)]
    let (auth_store, rule_store, config_source, local_token_cache): (
        Arc<dyn AuthStore>,
        Arc<dyn RuleStore>,
        Arc<dyn ConfigSource>,
        Option<Arc<local_auth::LocalTokenCache>>,
    ) = if bootstrap.is_ctld_managed() {
        info!("running in ctld-managed mode; no local SQLite stores");
        let token_cache = Arc::new(local_auth::LocalTokenCache::new());
        let auth: Arc<dyn AuthStore> = token_cache.clone();
        (
            auth,
            Arc::new(NullRuleStore),
            Arc::new(NullConfigSource),
            Some(token_cache),
        )
    } else {
        let (auth_store, rule_store) = build_stores(&bootstrap.config.server.database_url).await?;
        info!(
            url = %bootstrap.config.server.database_url,
            "auth and rule stores initialized (shared pool)"
        );
        match rule_store.is_routing_empty().await {
            Ok(true) => {
                if let Err(e) =
                    self::config::sync_file_to_db(&bootstrap.config, rule_store.as_ref()).await
                {
                    tracing::warn!(error = %e, "failed to seed routing DB from YAML (non-fatal)");
                } else {
                    info!("routing rules seeded into DB from config file (first boot)");
                }
            }
            Ok(false) => info!("routing DB already populated, skipping YAML seed"),
            Err(e) => {
                tracing::warn!(error = %e, "could not check routing DB state, skipping YAML seed");
            }
        }
        let cs = build_config_source(&bootstrap.config_path, rule_store.clone());
        (auth_store, rule_store, cs, None)
    };

    let http_params = HttpClientParams::from(&bootstrap.config.server.http_pool);
    let (tm, egress) = config_source.load().await?;
    let initial_snapshot = build_routing_snapshot(&tm, &egress, &http_params);
    let (revocation_tx, _) = tokio::sync::broadcast::channel::<String>(64);
    let proxy_buffer_params =
        tunnel_lib::ProxyBufferParams::from(&bootstrap.config.server.proxy_buffers);
    let peek_buf_pool = crate::PeekBufPool::new(proxy_buffer_params.peek_buf_size);
    let max_streams =
        tunnel_lib::QuicTransportParams::from(&bootstrap.config.server.quic).max_concurrent_streams;
    let overload_limits = bootstrap.config.server.overload.resolve(max_streams);
    info!(
        mode = ?overload_limits.mode,
        yield_threshold = overload_limits.inflight_yield_threshold,
        sleep_threshold = overload_limits.inflight_sleep_threshold,
        max_concurrent_streams = max_streams,
        "overload protection resolved"
    );
    let shard_count = tunnel_lib::resolve_shard_count(bootstrap.config.server.quic.shards, None);
    info!(
        shards = shard_count,
        cpu_parallelism = tunnel_lib::available_parallelism(),
        "server QUIC ownership topology resolved"
    );
    let shared_registry = new_shared_registry(shard_count);
    let routing = Arc::new(ArcSwap::from_pointee(initial_snapshot));
    let plugin_registry = {
        use tunnel_lib::plugin::PluginRegistry;

        let mut reg = PluginRegistry::new();
        let route_resolver: Arc<dyn tunnel_lib::plugin::RouteResolver> =
            Arc::new(plugins::vhost::VhostPlugin {
                routing: routing.clone(),
            });
        reg.register_ingress_handler(Arc::new(plugins::tls::TlsHandler {
            registry: shared_registry.clone(),
        }));
        reg.register_ingress_handler(Arc::new(plugins::h2c::H2cHandler {
            registry: shared_registry.clone(),
            route_resolver: route_resolver.clone(),
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
            tcp_params: tunnel_lib::TcpParams::from(&bootstrap.config.server.tcp),
            proxy_buffer_params,
            peek_buf_pool,
            routing,
            listeners: listener_mgr::ListenerManager::new(),
            overload_limits,
            plugin_registry,
        },
        connection: ConnectionRuntime {
            registry: shared_registry,
        },
        control: ControlRuntime {
            auth_store,
            rule_store,
            config_source,
            revocation_tx,
            local_token_cache,
        },
    }))
}

pub(crate) fn build_routing_snapshot(
    tm: &TunnelManagement,
    egress: &ServerEgressUpstream,
    http_params: &HttpClientParams,
) -> RoutingSnapshot {
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
    let egress_map = egress::ServerEgressMap::from_config(egress, http_params);
    let egress_rules = egress
        .rules
        .vhost
        .iter()
        .map(|r| tunnel_lib::EgressVhostRuleDef {
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        })
        .collect();
    RoutingSnapshot {
        http_routers: Arc::new(http_routers),
        tunnel_management: Arc::new(tm.clone()),
        egress_map: Arc::new(egress_map),
        egress_rules,
    }
}

async fn build_stores(database_url: &str) -> Result<(Arc<dyn AuthStore>, Arc<dyn RuleStore>)> {
    let pool = tunnel_store::open_sqlite_pool(database_url, 16).await?;
    let auth_store = tunnel_store::sqlite::SqliteAuthStore::from_pool(pool.clone());
    auth_store.migrate().await?;
    let rule_store = tunnel_store::sqlite_rules::SqliteRuleStore::new(pool);
    rule_store.migrate().await?;
    Ok((Arc::new(auth_store), Arc::new(rule_store)))
}

fn build_config_source(config_path: &str, rule_store: Arc<dyn RuleStore>) -> Arc<dyn ConfigSource> {
    Arc::new(MergedSource::new(
        Box::new(DbSource::new(rule_store)),
        Box::new(FileSource::new(config_path)),
    ))
}
