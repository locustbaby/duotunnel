use crate::{metrics, ServerState};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use tunnel_lib::{open_bi_guarded, proxy, run_accept_worker, OpenBiOutcome};

pub async fn run_tcp_accept_loop(
    listener: Arc<TcpListener>,
    state: Arc<ServerState>,
    _port: u16,
    proxy_name: String,
    group_id: String,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = listener.local_addr()?;
    let emfile_backoff = Duration::from_millis(state.config.server.overload.emfile_backoff_ms);
    info!(addr = %addr, proxy = %proxy_name, group = %group_id, "TCP accept loop started");
    run_accept_worker(
        listener,
        cancel,
        emfile_backoff,
        "tcp",
        move |stream, _peer_addr| {
            let state = state.clone();
            let proxy_name = proxy_name.clone();
            let group_id = group_id.clone();
            tokio::task::spawn(async move {
                if let Err(e) = state.tcp_params.apply(&stream) {
                    debug!(error = %e, "tcp_params.apply failed");
                    return;
                }
                metrics::tcp_connection_opened();
                let result = handle_tcp_connection(state, stream, proxy_name, group_id).await;
                if let Err(e) = &result {
                    debug!(error = %e, "TCP connection error");
                    metrics::request_completed("tcp", "error");
                } else {
                    metrics::request_completed("tcp", "success");
                }
                metrics::tcp_connection_closed();
            });
        },
    )
    .await;
    Ok(())
}
async fn handle_tcp_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    proxy_name: String,
    group_id: String,
) -> Result<()> {
    use tunnel_lib::detect_protocol_and_host;
    let peer_addr = stream.peer_addr()?;
    let pool = &state.peek_buf_pool;
    let mut buf = pool.take();
    let n = stream.peek(&mut buf).await?;
    let (protocol, host) = detect_protocol_and_host(&buf[..n]);
    pool.put(buf);
    debug!(protocol = ? protocol, host = ? host, "detected protocol on tcp listener");
    let selected = state
        .registry
        .select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name,
        src_addr: peer_addr.ip().to_string(),
        src_port: peer_addr.port(),
        protocol,
        host,
    };
    let open_timeout = Duration::from_millis(state.config.server.open_stream_timeout_ms);
    let _open_bi_guard = metrics::open_bi_begin(&selected.conn_id);
    let opened = open_bi_guarded(
        &selected.conn,
        &selected.inflight,
        &state.overload_limits,
        open_timeout,
        |elapsed, outcome| {
            metrics::open_bi_observe_wait_ms(elapsed.as_secs_f64() * 1000.0);
            if matches!(outcome, OpenBiOutcome::Timeout) {
                metrics::open_bi_timeout();
            }
        },
    )
    .await?;
    let mut send = opened.send;
    let recv = opened.recv;
    let _inflight_guard = opened.inflight;
    tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
    proxy::forward_to_client(send, recv, stream, state.proxy_buffer_params.relay_buf_size).await
}
