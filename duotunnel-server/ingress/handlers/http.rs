use crate::runtime::metrics;
use crate::ServerState;
use anyhow::Result;
use duotunnel_lib::plugin::{IngressDispatcher, ServerCtx, Timeouts};
use duotunnel_lib::{run_accept_worker, AcceptedConn};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

pub async fn run_http_accept_loop(
    listener: Arc<TcpListener>,
    state: Arc<ServerState>,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = listener.local_addr()?;
    let emfile_backoff = state.emfile_backoff();
    info!(addr = %addr, "http accept loop started");

    let dispatcher = Arc::new(IngressDispatcher::new(
        state.plugin_registry().clone(),
        port,
    ));
    let metrics_sink = state.plugin_registry().metrics_sink.clone();
    let connection_quiesce = cancel.clone();

    run_accept_worker(listener, cancel, emfile_backoff, "http", move |accepted| {
        let state = state.clone();
        let dispatcher = dispatcher.clone();
        let metrics_sink = metrics_sink.clone();
        let quiesce = connection_quiesce.clone();
        async move {
            let AcceptedConn {
                stream,
                peer_addr,
                accepted_at,
                ..
            } = accepted;
            let Some(runtime_generation) = state.admit_runtime_generation() else {
                debug!("rejecting public HTTP connection while server is not ready");
                return;
            };
            if let Err(e) = state.tcp_params().apply(&stream) {
                debug!(error = %e, "tcp_params.apply failed");
                return;
            }
            metrics::tcp_connection_opened();

            let svc = crate::ingress::tunnel_service::DefaultTunnelService;
            let timeouts = Timeouts {
                open_stream_ms: state.open_stream_timeout().as_millis() as u64,
                sniff_ms: state.sniff_timeout().as_millis() as u64,
                ..Timeouts::default()
            };
            let mut ctx = ServerCtx::new(
                peer_addr,
                metrics_sink,
                Arc::new(state.tcp_params().clone()),
                state.overload_limits().clone(),
                timeouts,
                port,
                state.relay_buf_size(),
                runtime_generation.sequence(),
                quiesce,
            );
            ctx.timing.accepted_at = accepted_at;

            let result = dispatcher.dispatch(stream, &svc, &mut ctx).await;

            if let Err(e) = &result {
                debug!(error = %e, "entry connection error");
                metrics::request_completed("http", "error");
            } else {
                metrics::request_completed("http", "success");
            }
            metrics::tcp_connection_closed();
        }
    })
    .await;
    Ok(())
}
