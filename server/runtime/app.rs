use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::bootstrap::cli::{Cli, Commands, TokenAction};
use crate::bootstrap::config::ServerConfigFile;
use crate::bootstrap::{build_server_state, ServerBootstrap, ServerState};
use crate::ingress::{handlers, shutdown_all_listeners, sync_listeners};
use crate::runtime;
use crate::runtime::metrics;
use crate::runtime::supervisor::{
    BackgroundComponent, ComponentContext, MetricsComponent, ServerSupervisor,
};
use tunnel_store::AuthStore;

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
            Some(Commands::Token { action }) => {
                handle_token_command(bootstrap.config_path(), action).await
            }
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

async fn handle_token_command(config_path: &str, action: TokenAction) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;
    let pool = tunnel_store::open_sqlite_pool(&config.server.database_url, 16).await?;
    let auth = tunnel_store::sqlite::SqliteAuthStore::from_pool(pool);
    auth.migrate().await?;
    let store = Arc::new(auth) as Arc<dyn AuthStore>;
    match action {
        TokenAction::Create { name } => {
            let token = store.create_client(&name).await?;
            println!("{}", token);
        }
        TokenAction::List => {
            let entries = store.list_tokens().await?;
            println!(
                "{:<20} {:<10} {:<8} {:<10} {:<20} REVOKED",
                "NAME", "CLIENT", "TOKEN_ID", "STATUS", "CREATED"
            );
            for e in entries {
                println!(
                    "{:<20} {:<10} {:<8} {:<10} {:<20} {}",
                    e.client_name,
                    e.client_status,
                    e.token_id,
                    e.token_status
                        .map(|status| status.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    e.created_at,
                    e.revoked_at.as_deref().unwrap_or("-")
                );
            }
        }
        TokenAction::Revoke { name } => {
            store.revoke_token(&name).await?;
            println!("token revoked for '{}'", name);
        }
        TokenAction::Rotate { name } => {
            let token = store.rotate_token(&name).await?;
            println!("{}", token);
        }
    }
    Ok(())
}

async fn run_server(bootstrap: ServerBootstrap) -> Result<()> {
    bootstrap.validate()?;
    info!("Starting DuoTunnel Server");
    info!(
        tunnel_port = %bootstrap.tunnel_port(),
        "Configuration loaded"
    );
    tunnel_lib::init_cert_cache(bootstrap.pki());
    {
        use metrics_exporter_prometheus::PrometheusBuilder;

        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install prometheus recorder");
        metrics::set_handle(handle);
    }

    let runtime = ServerRuntime::build(&bootstrap).await?;
    let signal_shutdown = runtime.shutdown.clone();
    runtime::spawn_task(async move {
        wait_for_shutdown_signal().await;
        info!("Received shutdown signal, draining server...");
        signal_shutdown.cancel();
    });
    let bootstrap = Arc::new(bootstrap);
    let supervisor = start_supervisor(bootstrap.clone(), &runtime);
    let health = runtime.state.health().clone();
    let mut proxy = Box::pin(proxy_main(runtime.state.clone(), runtime.shutdown.clone()));
    let result = tokio::select! {
        result = &mut proxy => result,
        failure = health.wait_for_failure() => {
            runtime.shutdown.cancel();
            let _ = proxy.await;
            Err(anyhow::anyhow!("critical component failed: {failure}"))
        }
    };
    if runtime.shutdown.is_cancelled() {
        wait_for_shutdown_drain("server").await;
    }
    runtime.shutdown.cancel();
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
    let listeners = state.ingress_listeners();
    sync_listeners(&state, &listeners).await?;

    let quic_state = state.clone();
    let quic_shutdown = shutdown.clone();
    let mut quic_handle = runtime::spawn_task(async move {
        handlers::quic::run_quic_server(quic_state, quic_shutdown).await
    });

    tokio::select! {
        r = &mut quic_handle => {
            r??;
            shutdown_all_listeners(&state).await;
            if !shutdown.is_cancelled() {
                anyhow::bail!("QUIC server exited unexpectedly");
            }
            return Ok(());
        }
        _ = shutdown.cancelled() => {}
    }
    // Shutdown order: stop public accepts first so the drain counters can
    // reach zero while tunnel connections are still open; run_quic_server
    // then drains and closes connections before closing the endpoint.
    shutdown_all_listeners(&state).await;
    quic_handle.await??;
    Ok(())
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
    let drained = tunnel_lib::wait_for_resource_drain(SHUTDOWN_DRAIN_TIMEOUT).await;
    let active = tunnel_lib::METRICS.active_connections();
    let pending = tunnel_lib::METRICS.pending_streams();
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
