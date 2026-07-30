use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::bootstrap::cli::{Cli, Commands};
use crate::bootstrap::{build_server_state, ServerBootstrap, ServerState};
use crate::ingress::{handlers, shutdown_all_listeners, sync_current_listeners};
use crate::runtime;
use crate::runtime::metrics;
use crate::runtime::supervisor::{
    BackgroundComponent, ComponentContext, MetricsComponent, ServerSupervisor,
};

const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct ServerApp {
    cli: Cli,
}

struct ServerRuntime {
    state: Arc<ServerState>,
    shutdown: CancellationToken,
    proxy_handle: tokio::runtime::Handle,
}

impl ServerApp {
    pub(crate) fn new(cli: Cli) -> Self {
        Self { cli }
    }

    pub(crate) async fn run(self) -> Result<()> {
        let bootstrap = ServerBootstrap::from_cli(&self.cli)?;
        runtime::init_observability(bootstrap.log_level());
        match self.cli.command {
            Some(Commands::Run) | None => run_server(bootstrap).await,
        }
    }
}

impl ServerRuntime {
    async fn build(bootstrap: &ServerBootstrap) -> Result<Self> {
        Ok(Self {
            state: build_server_state(bootstrap).await?,
            shutdown: CancellationToken::new(),
            proxy_handle: tokio::runtime::Handle::current(),
        })
    }
}

async fn run_server(bootstrap: ServerBootstrap) -> Result<()> {
    bootstrap.validate()?;
    info!("Starting DuoTunnel Server");
    info!(
        tunnel_port = %bootstrap.tunnel_port(),
        "Configuration loaded"
    );
    duotunnel_lib::init_cert_cache(bootstrap.pki());
    if bootstrap.metrics_port().is_some() {
        use metrics_exporter_prometheus::PrometheusBuilder;

        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install prometheus recorder");
        metrics::set_handle(handle);
    }

    let runtime = ServerRuntime::build(&bootstrap).await?;
    let bootstrap = Arc::new(bootstrap);
    let supervisor = start_supervisor(bootstrap.clone(), &runtime);
    let health = runtime.state.health().clone();
    let mut proxy = Box::pin(proxy_main(runtime.state.clone(), runtime.shutdown.clone()));
    let mut signal = Box::pin(wait_for_shutdown_signal());
    let graceful_shutdown;
    let result = tokio::select! {
        result = &mut proxy => {
            graceful_shutdown = true;
            result
        },
        failure = health.wait_for_failure() => {
            graceful_shutdown = true;
            enter_draining(&runtime.state, &runtime.shutdown);
            let _ = proxy.await;
            Err(anyhow::anyhow!("critical component failed: {failure}"))
        }
        _ = &mut signal => {
            graceful_shutdown = true;
            info!("Received shutdown signal, draining server...");
            enter_draining(&runtime.state, &runtime.shutdown);
            proxy.await
        }
    };
    enter_draining(&runtime.state, &runtime.shutdown);
    if graceful_shutdown {
        wait_for_shutdown_drain("server").await;
    }
    supervisor.shutdown();
    result
}

fn start_supervisor(bootstrap: Arc<ServerBootstrap>, runtime: &ServerRuntime) -> ServerSupervisor {
    let mut components: Vec<Box<dyn crate::runtime::supervisor::ServerComponent>> =
        vec![Box::new(BackgroundComponent)];
    if bootstrap.metrics_port().is_some() {
        components.push(Box::new(MetricsComponent));
    }
    let ctx = ComponentContext {
        state: runtime.state.clone(),
        bootstrap,
        shutdown: runtime.shutdown.clone(),
        proxy_handle: runtime.proxy_handle.clone(),
    };
    ServerSupervisor::start(components, ctx)
}

async fn proxy_main(state: Arc<ServerState>, shutdown: CancellationToken) -> Result<()> {
    let mut listeners_shutdown = false;
    let result = async {
        sync_current_listeners(&state).await?;

        let quic_state = state.clone();
        let quic_shutdown = shutdown.clone();
        let mut quic_handle = runtime::spawn_task(async move {
            handlers::quic::run_quic_server(quic_state, quic_shutdown).await
        });

        tokio::select! {
            result = &mut quic_handle => {
                let result = match result {
                    Ok(result) => result,
                    Err(error) => Err(anyhow::anyhow!("QUIC server task failed: {error}")),
                };
                match result {
                    Ok(()) if shutdown.is_cancelled() => Ok(()),
                    Ok(()) => anyhow::bail!("QUIC server exited unexpectedly"),
                    Err(error) => Err(error),
                }
            }
            _ = shutdown.cancelled() => {
                // Stop public accepts before waiting for QUIC connections to drain.
                shutdown_all_listeners(&state).await;
                listeners_shutdown = true;
                match quic_handle.await {
                    Ok(result) => result,
                    Err(error) => Err(anyhow::anyhow!("QUIC server task failed: {error}")),
                }
            }
        }
    }
    .await;

    enter_draining(&state, &shutdown);
    if !listeners_shutdown {
        shutdown_all_listeners(&state).await;
    }
    result
}

fn enter_draining(state: &ServerState, shutdown: &CancellationToken) {
    state.health().mark_quic_bound(false);
    shutdown.cancel();
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut terminate =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn wait_for_shutdown_drain(role: &'static str) {
    let drained = duotunnel_lib::wait_for_resource_drain(SHUTDOWN_DRAIN_TIMEOUT).await;
    let active = duotunnel_lib::METRICS.active_connections();
    let pending = duotunnel_lib::METRICS.pending_streams();
    if drained {
        info!(
            role,
            active_connections = active,
            pending_streams = pending,
            "shutdown drain completed"
        );
    } else {
        info!(
            role,
            active_connections = active,
            pending_streams = pending,
            drain_timeout_secs = SHUTDOWN_DRAIN_TIMEOUT.as_secs(),
            "shutdown drain timed out; forcing exit"
        );
    }
}
