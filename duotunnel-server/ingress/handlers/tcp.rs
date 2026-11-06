use crate::runtime::metrics;
use crate::ServerState;
use anyhow::Result;
use duotunnel_lib::{
    proxy, run_accept_worker, ErrorKind, GroupId, OpenBiOutcome, OpenStreamRequest, ProxyName,
};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

pub async fn run_tcp_accept_loop(
    listener: Arc<TcpListener>,
    state: Arc<ServerState>,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = listener.local_addr()?;
    let emfile_backoff = state.emfile_backoff();
    info!(addr = %addr, port, "TCP accept loop started");
    run_accept_worker(listener, cancel, emfile_backoff, "tcp", move |accepted| {
        let state = state.clone();
        async move {
            let stream = accepted.stream;
            let Some(_conn_permit) = state.try_acquire_ingress_conn() else {
                debug!("rejecting public TCP connection: max_ingress_connections reached");
                ::metrics::counter!(
                    "duotunnel_ingress_connections_rejected_overloaded_total",
                    "protocol" => "tcp"
                )
                .increment(1);
                return;
            };
            let Some(generation) = state.admit_runtime_generation() else {
                debug!("rejecting public TCP connection while server is not ready");
                return;
            };
            let Some((group_id, proxy_name)) = generation.routing().tcp_route(port) else {
                debug!(
                    port,
                    generation = generation.sequence(),
                    "rejecting TCP connection without a route in the pinned generation"
                );
                return;
            };
            if let Err(e) = state.tcp_params().apply(&stream) {
                debug!(error = %e, "tcp_params.apply failed");
                return;
            }
            metrics::tcp_connection_opened();
            let result = handle_tcp_connection(state, stream, proxy_name, group_id).await;
            if let Err(e) = &result {
                debug!(error = %e, "TCP connection error");
                metrics::request_failed("tcp", e);
            } else {
                metrics::request_completed("tcp", "success");
            }
            metrics::tcp_connection_closed();
        }
    })
    .await;
    Ok(())
}
fn observe_wait_metrics(elapsed: std::time::Duration, outcome: OpenBiOutcome) {
    metrics::open_bi_observe_wait_ms(elapsed.as_secs_f64() * 1000.0);
    if matches!(outcome, OpenBiOutcome::TimedOut) {
        metrics::open_bi_timed_out();
    } else if matches!(outcome, OpenBiOutcome::RejectedOverloaded) {
        metrics::open_bi_rejected_overloaded();
    }
}

async fn handle_tcp_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    proxy_name: ProxyName,
    group_id: GroupId,
) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    // A fixed TCP route must connect before reading: the upstream may speak first.
    let routing_info = duotunnel_lib::RoutingInfo {
        proxy_name,
        src_addr: peer_addr.ip(),
        src_port: peer_addr.port(),
        protocol: duotunnel_lib::proxy::core::Protocol::Tcp,
        host: None,
    };

    let mut attempts = 0;
    let max_attempts = 3;
    let opened = loop {
        attempts += 1;
        let selected = state
            .registry()
            .select_client_for_group(&group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

        let open_timeout = state.open_stream_timeout();
        let _open_bi_guard = metrics::open_bi_begin(&selected.conn_id);
        match selected
            .handle
            .open_stream(OpenStreamRequest {
                routing_info: routing_info.clone(),
                initial_bytes: None,
                stream_timeout: open_timeout,
                on_wait_done: Some(observe_wait_metrics),
            })
            .await
        {
            Ok(opened) => break opened,
            Err(e) => {
                let connection_lost = selected.handle.close_reason().is_some()
                    || matches!(
                        e.kind,
                        ErrorKind::QuicConnectionLost | ErrorKind::QuicConnectionFatal
                    );
                warn!(
                    conn_id = %selected.conn_id,
                    group_id = %group_id,
                    attempt = attempts,
                    error = %e,
                    connection_lost,
                    "failed to open QUIC stream on selected TCP proxy connection, retrying"
                );
                if connection_lost {
                    state.registry().unregister(&selected.conn_id);
                }
                if attempts >= max_attempts {
                    return Err(e.into());
                }
            }
        }
    };

    let send = opened.send;
    let recv = opened.recv;
    let _inflight_guard = opened.inflight;
    proxy::forward_to_client(send, recv, stream, state.relay_buf_size()).await
}
