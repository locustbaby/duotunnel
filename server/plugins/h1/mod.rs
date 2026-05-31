use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::{debug, warn};

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use tunnel_lib::ProxyError;

use crate::registry::SharedRegistry;

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
        mut stream: TcpStream,
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

        let initial_data = &hint.raw_preface;
        let protocol = match hint.protocol {
            tunnel_lib::proxy::core::Protocol::WebSocket => {
                tunnel_lib::proxy::core::Protocol::WebSocket
            }
            _ => tunnel_lib::proxy::core::Protocol::H1,
        };

        debug!(host = %host, protocol = ?protocol, "plaintext H1/WS, byte-level forwarding");

        let (group_id, proxy_name) = (route.group_id, route.proxy_name);

        let mut discard = [0u8; tunnel_lib::plugin::SNIFF_LIMIT];
        stream
            .read_exact(&mut discard[..initial_data.len()])
            .await?;

        let mut attempts = 0;
        let max_attempts = 3;
        let opened = loop {
            attempts += 1;
            let selected = self
                .registry
                .select_client_for_group(&group_id)
                .ok_or_else(|| ProxyError::no_client_available(group_id.to_string()))?;

            tunnel_lib::maybe_slow_path(&selected.inflight_table, selected.slot_id, &ctx.overload)
                .await;

            let open_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);
            match tunnel_lib::open_bi_guarded(
                &selected.conn,
                &selected.inflight_table,
                selected.slot_id,
                open_timeout,
                |_elapsed, _outcome| {},
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
                        "failed to open QUIC stream on selected H1 connection, unregistering and retrying"
                    );
                    self.registry.unregister(&selected.conn_id);
                    if attempts >= max_attempts {
                        return Err(e.into());
                    }
                }
            }
        };

        let mut send = opened.send;
        let recv = opened.recv;
        let _inflight_guard = opened.inflight;
        let routing_info = tunnel_lib::RoutingInfo {
            proxy_name: proxy_name.to_string(),
            src_addr: ctx.peer_addr.ip().to_string(),
            src_port: ctx.peer_addr.port(),
            protocol,
            host: Some(host),
        };
        tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
        tunnel_lib::proxy::forward_with_initial_data(
            send,
            recv,
            stream,
            initial_data,
            ctx.relay_buf_size,
        )
        .await
    }
}
