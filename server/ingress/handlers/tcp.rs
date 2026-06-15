use crate::runtime::metrics;
use crate::ServerState;
use anyhow::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    maybe_slow_path, open_bi_guarded, proxy, run_accept_worker, GroupId, OpenBiOutcome, ProxyName,
};

pub async fn run_tcp_accept_loop(
    listener: Arc<TcpListener>,
    state: Arc<ServerState>,
    _port: u16,
    proxy_name: ProxyName,
    group_id: GroupId,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = listener.local_addr()?;
    let emfile_backoff = state.emfile_backoff();
    info!(addr = %addr, proxy = %proxy_name, group = %group_id, "TCP accept loop started");
    run_accept_worker(listener, cancel, emfile_backoff, "tcp", move |accepted| {
        let state = state.clone();
        let proxy_name = proxy_name.clone();
        let group_id = group_id.clone();
        async move {
            let stream = accepted.stream;
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
async fn handle_tcp_connection(
    state: Arc<ServerState>,
    mut stream: TcpStream,
    proxy_name: ProxyName,
    group_id: GroupId,
) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    let pool = state.peek_buf_pool();
    let runtime = tunnel_lib::SniffRuntime::new(
        tunnel_lib::SniffPolicy::default(),
        tunnel_lib::default_ingress_detectors(),
    );
    let sniffed =
        match tokio::time::timeout(state.sniff_timeout(), runtime.sniff(&mut stream, pool)).await {
            Ok(res) => res?,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "protocol sniffing timed out (Slowloris protection)"
                ))
            }
        };
    let protocol = if sniffed.bytes_read == 0 {
        tunnel_lib::proxy::core::Protocol::Unknown
    } else {
        sniffed.hint.protocol
    };
    let host = sniffed.hint.sni.clone().or(sniffed.hint.authority.clone());
    let prefixed_stream = sniffed.into_stream(stream);
    debug!(protocol = ? protocol, host = ? host, "detected protocol on tcp listener");

    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name,
        src_addr: peer_addr.ip().to_string(),
        src_port: peer_addr.port(),
        protocol,
        host,
    };

    let mut attempts = 0;
    let max_attempts = 3;
    let opened = loop {
        attempts += 1;
        let selected = state
            .registry()
            .select_client_for_group(&group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

        maybe_slow_path(
            &selected.inflight_table,
            selected.slot_id,
            state.overload_limits(),
        )
        .await;

        let open_timeout = state.open_stream_timeout();
        let _open_bi_guard = metrics::open_bi_begin(&selected.conn_id);
        match open_bi_guarded(
            &selected.conn,
            &selected.inflight_table,
            selected.slot_id,
            state.overload_limits(),
            open_timeout,
            |elapsed, outcome| {
                metrics::open_bi_observe_wait_ms(elapsed.as_secs_f64() * 1000.0);
                if matches!(outcome, OpenBiOutcome::TimedOut) {
                    metrics::open_bi_timed_out();
                } else if matches!(outcome, OpenBiOutcome::RejectedOverloaded) {
                    metrics::open_bi_rejected_overloaded();
                }
            },
        )
        .await
        {
            Ok(opened) => break opened,
            Err(e) => {
                warn!(
                    conn_id = %selected.conn_id,
                    group_id = %group_id,
                    attempt = attempts,
                    error = %e,
                    "failed to open QUIC stream on selected TCP proxy connection, unregistering and retrying"
                );
                state.registry().unregister(&selected.conn_id);
                if attempts >= max_attempts {
                    return Err(e.into());
                }
            }
        }
    };

    let mut send = opened.send;
    let recv = opened.recv;
    let _inflight_guard = opened.inflight;
    tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
    proxy::forward_prefixed_to_client(send, recv, prefixed_stream, state.relay_buf_size()).await
}
