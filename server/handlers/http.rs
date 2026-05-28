use crate::{metrics, ServerState};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use tunnel_lib::plugin::{IngressDispatcher, ServerCtx, Timeouts};
use tunnel_lib::run_accept_worker;

pub async fn run_http_accept_loop(
    listener: Arc<TcpListener>,
    state: Arc<ServerState>,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = listener.local_addr()?;
    let emfile_backoff = Duration::from_millis(state.config.server.overload.emfile_backoff_ms);
    info!(addr = %addr, "http accept loop started");

    let dispatcher = Arc::new(IngressDispatcher::new(state.plugin_registry.clone(), port));
    let metrics_sink = state.plugin_registry.metrics_sink.clone();

    run_accept_worker(
        listener,
        cancel,
        emfile_backoff,
        "http",
        move |stream, peer_addr| {
            let state = state.clone();
            let dispatcher = dispatcher.clone();
            let metrics_sink = metrics_sink.clone();
            tokio::task::spawn(async move {
                if let Err(e) = state.tcp_params.apply(&stream) {
                    debug!(error = %e, "tcp_params.apply failed");
                    return;
                }
                metrics::tcp_connection_opened();

                let svc = crate::tunnel_service::DefaultTunnelService;
                let timeouts = Timeouts {
                    open_stream_ms: state.config.server.open_stream_timeout_ms,
                    ..Timeouts::default()
                };
                let mut ctx = ServerCtx::new(
                    peer_addr,
                    metrics_sink,
                    Arc::new(state.tcp_params.clone()),
                    state.overload_limits.clone(),
                    timeouts,
                    port,
                    state.proxy_buffer_params.relay_buf_size,
                );
                ctx.timing.accepted_at = std::time::Instant::now();

                let result = dispatcher.dispatch(stream, &svc, &mut ctx).await;

                if let Err(e) = &result {
                    debug!(error = %e, "entry connection error");
                    metrics::request_completed("http", "error");
                } else {
                    metrics::request_completed("http", "success");
                }
                metrics::tcp_connection_closed();
            });
        },
    )
    .await;
    Ok(())
}
