use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::debug;

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};

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
        let route = route.ok_or_else(|| anyhow::anyhow!("H1Handler: missing Route"))?;
        let hint = ctx
            .hint
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("H1Handler: missing ProtocolHint"))?;

        let host = hint
            .authority
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no Host header in plaintext request"))?;

        let initial_data: Vec<u8> = hint.raw_preface.to_vec();
        let protocol = match hint.protocol {
            tunnel_lib::proxy::core::Protocol::WebSocket => {
                tunnel_lib::proxy::core::Protocol::WebSocket
            }
            _ => tunnel_lib::proxy::core::Protocol::H1,
        };

        debug!(host = %host, protocol = ?protocol, "plaintext H1/WS, byte-level forwarding");

        let (group_id, proxy_name) = (route.group_id, route.proxy_name);
        let selected = self
            .registry
            .select_client_for_group(&group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

        // Consume the peeked bytes from the stream before handing it to relay.
        let mut discard = vec![0u8; initial_data.len()];
        stream.read_exact(&mut discard).await?;

        let routing_info = tunnel_lib::RoutingInfo {
            proxy_name: proxy_name.to_string(),
            src_addr: ctx.peer_addr.ip().to_string(),
            src_port: ctx.peer_addr.port(),
            protocol,
            host: Some(host),
        };

        let open_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);
        let opened = tunnel_lib::open_bi_guarded(
            &selected.conn,
            &selected.inflight,
            &ctx.overload,
            open_timeout,
            |_elapsed, _outcome| {},
        )
        .await?;

        let mut send = opened.send;
        let recv = opened.recv;
        let _inflight_guard = opened.inflight;
        tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
        tunnel_lib::proxy::forward_with_initial_data(
            send,
            recv,
            stream,
            &initial_data,
            ctx.relay_buf_size,
        )
        .await
    }
}
