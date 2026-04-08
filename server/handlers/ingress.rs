use crate::config::IngressMode;
use crate::{metrics, RoutingSnapshot, ServerState};
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{build_reuseport_tcp_listener, proxy};

pub fn spawn_ingress_listeners(
    state: Arc<ServerState>,
    cancel: CancellationToken,
    routing: Arc<RoutingSnapshot>,
) {
    let listeners = routing.tunnel_management.server_ingress_routing.listeners.clone();
    for listener_cfg in listeners {
        let port = listener_cfg.port;
        match listener_cfg.mode {
            IngressMode::Http(_) => {
                let s = state.clone();
                let c = cancel.clone();
                tokio::spawn(async move {
                    if let Err(e) = run_http_ingress(s, port, c).await {
                        tracing::error!(port = %port, error = %e, "HTTP ingress failed");
                    }
                });
            }
            IngressMode::Tcp(cfg) => {
                let s = state.clone();
                let c = cancel.clone();
                let group_id = cfg.client_group.clone();
                let proxy_name = cfg.proxy_name.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        run_tcp_ingress(s, port, group_id, proxy_name, c).await
                    {
                        tracing::error!(port = %port, error = %e, "TCP ingress failed");
                    }
                });
            }
        }
    }
}

async fn run_http_ingress(
    state: Arc<ServerState>,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = build_reuseport_tcp_listener(addr)?;
    info!(port = %port, "HTTP ingress listener started");
    loop {
        let (stream, peer_addr) = tokio::select! {
            _ = cancel.cancelled() => {
                debug!(port = %port, "HTTP ingress listener cancelled");
                return Ok(());
            }
            result = listener.accept() => result?,
        };
        state.tcp_params.apply(&stream)?;
        let permit = match state.tcp_semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(peer_addr = %peer_addr, "HTTP ingress rejected: max connections reached");
                metrics::connection_rejected("http");
                continue;
            }
        };
        let routing = state.routing.load_full();
        let s = state.clone();
        tokio::spawn(async move {
            let _permit = permit;
            metrics::tcp_connection_opened();
            let result = handle_http_connection(s, routing, stream, peer_addr, port).await;
            if let Err(e) = &result {
                debug!(error = %e, "HTTP ingress connection error");
                metrics::request_completed("http", "error");
            } else {
                metrics::request_completed("http", "success");
            }
            metrics::tcp_connection_closed();
        });
    }
}

async fn handle_http_connection(
    state: Arc<ServerState>,
    routing: Arc<RoutingSnapshot>,
    stream: TcpStream,
    peer_addr: SocketAddr,
    port: u16,
) -> Result<()> {
    use tunnel_lib::extract_host_from_http;
    use tunnel_lib::protocol::detect::extract_tls_sni;
    use tunnel_lib::detect_protocol_and_host;

    let mut buf = vec![0u8; 16384];
    let n = stream.peek(&mut buf).await?;
    let is_tls = n > 0 && buf[0] == 0x16;

    if is_tls {
        let sni = extract_tls_sni(&buf[..n]);
        let host = sni.ok_or_else(|| anyhow::anyhow!("no SNI in TLS ClientHello"))?;
        let (group_id, proxy_name) = lookup_http_route(&routing, port, &host)
            .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;
        let client_conn = state.registry
            .select_client_for_group(&group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
        proxy::forward_to_client(
            &client_conn,
            tunnel_lib::RoutingInfo {
                proxy_name: proxy_name.to_string(),
                src_addr: peer_addr.ip().to_string(),
                src_port: peer_addr.port(),
                protocol: "tls".to_string(),
                host: Some(host),
            },
            stream,
        )
        .await
    } else {
        let (protocol, detected_host) = detect_protocol_and_host(&buf[..n]);
        let host = detected_host
            .or_else(|| extract_host_from_http(&buf[..n]))
            .ok_or_else(|| anyhow::anyhow!("no Host header in plaintext request"))?;
        let (group_id, proxy_name) = lookup_http_route(&routing, port, &host)
            .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;
        let client_conn = state.registry
            .select_client_for_group(&group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
        proxy::forward_to_client(
            &client_conn,
            tunnel_lib::RoutingInfo {
                proxy_name: proxy_name.to_string(),
                src_addr: peer_addr.ip().to_string(),
                src_port: peer_addr.port(),
                protocol: protocol.to_string(),
                host: Some(host),
            },
            stream,
        )
        .await
    }
}

fn lookup_http_route(
    routing: &RoutingSnapshot,
    port: u16,
    host: &str,
) -> Option<(Arc<str>, Arc<str>)> {
    routing.http_routers.get(&port)?.get(host)
}

async fn run_tcp_ingress(
    state: Arc<ServerState>,
    port: u16,
    group_id: String,
    proxy_name: String,
    cancel: CancellationToken,
) -> Result<()> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = build_reuseport_tcp_listener(addr)?;
    info!(port = %port, group = %group_id, proxy = %proxy_name, "TCP ingress listener started");
    loop {
        let (stream, peer_addr) = tokio::select! {
            _ = cancel.cancelled() => {
                debug!(port = %port, "TCP ingress listener cancelled");
                return Ok(());
            }
            result = listener.accept() => result?,
        };
        state.tcp_params.apply(&stream)?;
        let permit = match state.tcp_semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(peer_addr = %peer_addr, "TCP ingress rejected: max connections reached");
                metrics::connection_rejected("tcp");
                continue;
            }
        };
        let s = state.clone();
        let g = group_id.clone();
        let p = proxy_name.clone();
        tokio::spawn(async move {
            let _permit = permit;
            metrics::tcp_connection_opened();
            let result = handle_tcp_connection(s, stream, peer_addr, g, p).await;
            if let Err(e) = &result {
                debug!(error = %e, "TCP ingress connection error");
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
    peer_addr: SocketAddr,
    group_id: String,
    proxy_name: String,
) -> Result<()> {
    use tunnel_lib::detect_protocol_and_host;
    let mut buf = vec![0u8; 4096];
    let n = stream.peek(&mut buf).await?;
    let (protocol, host) = detect_protocol_and_host(&buf[..n]);
    let client_conn = state.registry
        .select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
    proxy::forward_to_client(
        &client_conn,
        tunnel_lib::RoutingInfo {
            proxy_name,
            src_addr: peer_addr.ip().to_string(),
            src_port: peer_addr.port(),
            protocol: protocol.to_string(),
            host,
        },
        stream,
    )
    .await
}
