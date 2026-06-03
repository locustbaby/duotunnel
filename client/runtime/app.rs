use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::bootstrap::cli::Args;
use crate::bootstrap::ClientBootstrap;
use crate::egress::listener::EntryListenerConfig;
use crate::runtime::engine::RuntimeEngine;
use crate::runtime::spawn_task;
use crate::tunnel::conn_pool::EntryConnPool;

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
    let endpoint = crate::tunnel::client::build_quic_endpoint(&config).await?;
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    spawn_task(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received Ctrl+C, shutting down...");
            cancel_clone.cancel();
        }
    });
    let ready = Arc::new(AtomicBool::new(false));
    if let Some(port) = config.metrics_port {
        spawn_task(run_healthz_server(port, ready.clone()));
    }
    let entry_pool =
        EntryConnPool::new(config.quic.max_concurrent_streams, config.quic.connections);
    let mut engine = RuntimeEngine::new(cancel.clone());

    if let Some(entry_port) = config.entry.port {
        let entry_tcp_params = tunnel_lib::TcpParams::from(&config.tcp);
        let peek_buf_size = config.proxy_buffers.peek_buf_size;
        let open_stream_timeout = Duration::from_millis(config.reconnect.open_stream_timeout_ms);
        let overload_limits = config.overload.resolve(config.quic.max_concurrent_streams);

        info!(
            mode = ?overload_limits.mode,
            yield_threshold = overload_limits.inflight_yield_threshold,
            sleep_threshold = overload_limits.inflight_sleep_threshold,
            max_concurrent_streams = config.quic.max_concurrent_streams,
            "overload protection resolved"
        );

        let entry_cfg = EntryListenerConfig {
            port: entry_port,
            tcp_params: entry_tcp_params,
            peek_buf_size,
            open_stream_timeout,
            accept_workers: config.entry.accept_workers.max(1),
            overload: Arc::new(overload_limits),
        };

        engine.add_service(Arc::new(crate::egress::listener::EgressListenerService {
            entry_cfg,
            pool: entry_pool.clone(),
        }));
    }

    engine.add_service(Arc::new(crate::tunnel::TunnelPoolService {
        config,
        endpoint,
        entry_pool,
        ready,
    }));

    engine.run_until_shutdown().await
}

async fn run_healthz_server(port: u16, ready: Arc<AtomicBool>) {
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
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let ready = ready.clone();
        spawn_task(async move {
            let mut buf = [0u8; 256];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
            let (status, body) = if req.starts_with("GET /healthz") {
                if ready.load(Ordering::Acquire) {
                    ("200 OK", "ok\n")
                } else {
                    ("503 Service Unavailable", "not ready\n")
                }
            } else {
                ("400 Bad Request", "bad request\n")
            };
            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
