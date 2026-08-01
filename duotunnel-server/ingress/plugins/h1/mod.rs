use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};

use duotunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use duotunnel_lib::{ErrorKind, OpenStreamRequest, ProxyError};

use crate::ingress::registry::SharedRegistry;

/// Handles HTTP/1.x and WebSocket connections using byte-level forwarding.
///
/// Replays the peeked preface bytes into the QUIC tunnel so the client sees
/// the full original request.
pub struct H1Handler {
    pub registry: SharedRegistry,
}

#[async_trait]
impl IngressProtocolHandler for H1Handler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::Http1
    }

    async fn handle(
        &self,
        stream: duotunnel_lib::PrefixedReadWrite<tokio::net::TcpStream>,
        route: Option<Route>,
        ctx: &ServerCtx,
    ) -> Result<()> {
        let route = route.ok_or_else(ProxyError::routing_missing_info)?;
        let hint = ctx
            .hint
            .as_ref()
            .ok_or_else(ProxyError::routing_missing_info)?;

        let host = hint
            .authority
            .clone()
            .ok_or_else(ProxyError::routing_missing_host)?;

        let protocol = match hint.protocol {
            duotunnel_lib::proxy::core::Protocol::WebSocket => {
                duotunnel_lib::proxy::core::Protocol::WebSocket
            }
            _ => duotunnel_lib::proxy::core::Protocol::H1,
        };

        debug!(host = %host, protocol = ?protocol, "plaintext H1/WS, byte-level forwarding");

        let (group_id, proxy_name) = (route.group_id, route.proxy_name);

        let mut attempts = 0;
        let max_attempts = 3;
        let opened = loop {
            attempts += 1;
            let selected = match self.registry.select_client_for_group(&group_id) {
                Some(client) => client,
                None => {
                    let e = ProxyError::no_client_available(group_id.to_string());
                    let error_msg = format!("ProxyError: {}", e);
                    let response = format!(
                        "HTTP/1.1 502 Bad Gateway\r\n\
                         Content-Type: text/plain\r\n\
                         Content-Length: {}\r\n\
                         Connection: close\r\n\
                         \r\n\
                         {}",
                        error_msg.len(),
                        error_msg
                    );
                    let mut stream = stream;
                    let _ = stream.write_all(response.as_bytes()).await;
                    return Err(e.into());
                }
            };

            let open_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);
            let routing_info = duotunnel_lib::RoutingInfo {
                proxy_name: proxy_name.clone(),
                src_addr: ctx.peer_addr.ip(),
                src_port: ctx.peer_addr.port(),
                protocol,
                host: Some(host.clone()),
            };
            match selected
                .handle
                .open_stream(OpenStreamRequest {
                    routing_info,
                    initial_bytes: None,
                    stream_timeout: open_timeout,
                    on_wait_done: None,
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
                        "failed to open QUIC stream on selected H1 connection, retrying"
                    );
                    if connection_lost {
                        self.registry.unregister(&selected.conn_id);
                    }
                    if attempts >= max_attempts {
                        let error_msg = format!("ProxyError: failed to open QUIC stream: {}", e);
                        let response = format!(
                            "HTTP/1.1 502 Bad Gateway\r\n\
                             Content-Type: text/plain\r\n\
                             Content-Length: {}\r\n\
                             Connection: close\r\n\
                             \r\n\
                             {}",
                            error_msg.len(),
                            error_msg
                        );
                        let mut stream = stream;
                        let _ = stream.write_all(response.as_bytes()).await;
                        return Err(e.into());
                    }
                }
            }
        };

        let send = opened.send;
        let recv = opened.recv;
        let _inflight_guard = opened.inflight;
        duotunnel_lib::proxy::forward_prefixed_to_client(send, recv, stream, ctx.relay_buf_size)
            .await
    }
}
