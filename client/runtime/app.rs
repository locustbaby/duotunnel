use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::bootstrap::cli::Args;
use crate::bootstrap::ClientBootstrap;
use crate::egress::listener::EntryListenerConfig;
use crate::egress::udp_listener::{UdpEgressListenerService, UdpListenerRegistry};
use crate::runtime::engine::RuntimeEngine;
use crate::runtime::spawn_task;
use crate::tunnel::conn_pool::EntryConnPool;

const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct ClientApp {
    args: Args,
}

impl ClientApp {
    pub(crate) fn new(args: Args) -> Self {
        Self { args }
    }

    pub(crate) async fn run(self) -> Result<()> {
        let bootstrap = ClientBootstrap::from_args(&self.args)?;
        crate::runtime::init_observability(bootstrap.log_level());
        info!("Starting DuoTunnel Client");
        info!(server = %bootstrap.config().server_address(), config = %bootstrap.config_path(), "Configuration loaded");
        run_client_process(bootstrap).await
    }
}

async fn run_client_process(bootstrap: ClientBootstrap) -> Result<()> {
    let config = bootstrap.config().clone();
    let endpoint = crate::tunnel::endpoint::build_quic_endpoint(&config).await?;
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    spawn_task(async move {
        wait_for_shutdown_signal().await;
        info!("Received shutdown signal, shutting down...");
        cancel_clone.cancel();
    });
    let ready = Arc::new(AtomicBool::new(false));
    if let Some(port) = config.metrics_port {
        let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install prometheus recorder");
        crate::metrics::set_handle(handle);
        spawn_task(run_healthz_server(port, ready.clone(), cancel.clone()));
    }
    let resolved_connections = tunnel_lib::resolve_connection_count(config.quic.connections);
    let shard_count =
        tunnel_lib::resolve_shard_count(config.quic.shards, Some(resolved_connections as usize));
    let accept_workers = tunnel_lib::resolve_accept_workers(config.entry.accept_workers);
    info!(
        configured_connections = config.quic.connections,
        resolved_connections = resolved_connections,
        shards = shard_count,
        accept_workers = accept_workers,
        configured_worker_threads = tunnel_lib::configured_worker_threads(),
        cpu_parallelism = tunnel_lib::available_parallelism(),
        cgroup_cpu_limit = ?tunnel_lib::cgroup_cpu_limit(),
        effective_parallelism = tunnel_lib::effective_runtime_parallelism(),
        "client QUIC ownership topology resolved"
    );
    let overload_limits = config.overload.resolve(config.quic.max_concurrent_streams);
    info!(
        mode = ?overload_limits.mode,
        yield_threshold = overload_limits.inflight_yield_threshold,
        sleep_threshold = overload_limits.inflight_sleep_threshold,
        max_concurrent_streams = config.quic.max_concurrent_streams,
        "overload protection resolved"
    );
    let entry_pool = EntryConnPool::new(
        config.quic.max_concurrent_streams,
        overload_limits.max_pending_streams,
        resolved_connections,
        shard_count,
    );
    let udp_registry = Arc::new(UdpListenerRegistry::default());
    let mut engine = RuntimeEngine::new(cancel.clone());

    if let Some(entry_port) = config.entry.port {
        let entry_tcp_params = tunnel_lib::TcpParams::from(&config.tcp);
        let peek_buf_size = config.proxy_buffers.peek_buf_size;
        let open_stream_timeout = Duration::from_millis(config.reconnect.open_stream_timeout_ms);

        let entry_cfg = EntryListenerConfig {
            port: entry_port,
            tcp_params: entry_tcp_params,
            peek_buf_size,
            open_stream_timeout,
            accept_workers,
            overload: Arc::new(overload_limits),
            sniff_timeout: Duration::from_millis(config.proxy_buffers.sniff_timeout_ms),
        };

        engine.add_service(Arc::new(crate::egress::listener::EgressListenerService {
            entry_cfg,
            pool: entry_pool.clone(),
        }));
    }

    for udp_entry in &config.udp_entries {
        engine.add_service(Arc::new(UdpEgressListenerService {
            entry: udp_entry.clone(),
            pool: entry_pool.clone(),
            registry: udp_registry.clone(),
        }));
    }

    engine.add_service(Arc::new(crate::tunnel::TunnelPoolService {
        config,
        endpoint,
        entry_pool,
        ready,
        udp_registry,
        resolved_connections,
    }));

    let result = engine.run_until_shutdown().await;
    if cancel.is_cancelled() {
        wait_for_shutdown_drain("client").await;
    }
    result
}

async fn run_healthz_server(port: u16, ready: Arc<AtomicBool>, shutdown: CancellationToken) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            warn!(addr = %addr, error = %e, "failed to bind healthz server");
            return;
        }
    };
    info!(addr = %addr, "healthz server started");
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                info!(addr = %addr, "healthz server stopping");
                return;
            }
            accept = listener.accept() => {
                let Ok((mut stream, _)) = accept else {
                    continue;
                };
                let ready = ready.clone();
                spawn_task(async move {
                    let mut buf = [0u8; 256];
                    let n = stream.read(&mut buf).await.unwrap_or(0);
                    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
                    let (status, content_type, body) = if req.starts_with("GET /healthz") {
                        if ready.load(Ordering::Acquire) {
                            ("200 OK", "text/plain", "ok\n".to_string())
                        } else {
                            ("503 Service Unavailable", "text/plain", "not ready\n".to_string())
                        }
                    } else if req.starts_with("GET /metrics") {
                        ("200 OK", "text/plain; charset=utf-8", crate::metrics::encode())
                    } else {
                        ("400 Bad Request", "text/plain", "bad request\n".to_string())
                    };
                    let response = format!(
                        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                        status,
                        content_type,
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        }
    }
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
        warn!(
            role,
            active_connections = active,
            pending_streams = pending,
            drain_timeout_secs = SHUTDOWN_DRAIN_TIMEOUT.as_secs(),
            "shutdown drain timed out; forcing exit"
        );
    }
}
