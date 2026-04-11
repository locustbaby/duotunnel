use crate::{metrics, ServerState};
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::proxy;
pub async fn run_tcp_listener(
    state: Arc<ServerState>,
    port: u16,
    proxy_name: String,
    group_id: String,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = tunnel_lib::build_reuseport_listener(addr.parse()?)?;
    info!(
        addr = % addr, proxy = % proxy_name, group = % group_id, "TCP listener started"
    );
    loop {
        let (stream, peer_addr) = tokio::select! {
            _ = cancel.cancelled() => {
                info!(addr = %addr, "TCP listener shutting down");
                return Ok(());
            }
            result = listener.accept() => result?,
        };
        state.tcp_params.apply(&stream)?;
        debug!(peer_addr = % peer_addr, "new TCP connection");
        let permit = match state.tcp_semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(peer_addr = %peer_addr, "TCP connection rejected: max connections reached");
                metrics::connection_rejected("tcp");
                continue;
            }
        };
        let state = state.clone();
        let proxy_name = proxy_name.clone();
        let group_id = group_id.clone();
        crate::spawn_task(async move {
            let _permit = permit;
            metrics::tcp_connection_opened();
            let result = handle_tcp_connection(state, stream, proxy_name, group_id).await;
            if let Err(e) = &result {
                debug!(error = % e, "TCP connection error");
                metrics::request_completed("tcp", "error");
            } else {
                metrics::request_completed("tcp", "success");
            }
            metrics::tcp_connection_closed();
        });
    }
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
    debug!(protocol = % protocol, host = ? host, "detected protocol on tcp listener");
    let selected = state
        .registry
        .select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name,
        src_addr: peer_addr.ip().to_string(),
        src_port: peer_addr.port(),
        protocol: protocol.to_string(),
        host,
    };
    let open_timeout = Duration::from_millis(state.config.server.open_stream_timeout_ms);
    let _open_bi_guard = metrics::open_bi_begin(&selected.conn_id);
    let _inflight_guard = selected.begin_inflight();
    let wait_started = Instant::now();
    let (mut send, recv) = match tokio::time::timeout(open_timeout, selected.conn.open_bi()).await {
        Ok(Ok(streams)) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            streams
        }
        Ok(Err(e)) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            return Err(e.into());
        }
        Err(_) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            metrics::open_bi_timeout();
            return Err(anyhow::anyhow!("open_bi timed out after {:?}", open_timeout));
        }
    };
    tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
    proxy::forward_to_client(send, recv, stream, state.proxy_buffer_params.relay_buf_size).await
}
