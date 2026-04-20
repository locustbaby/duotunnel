use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::net::TcpStream;
use tracing::debug;

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};

use crate::registry::SharedRegistry;

/// Passthrough TCP handler: forwards raw bytes without protocol inspection.
///
/// Used for opaque TLS or any unrecognised protocol (ProtocolKind::Tcp).
pub struct TcpPassHandler {
    pub registry: SharedRegistry,
}

#[async_trait]
impl IngressProtocolHandler for TcpPassHandler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::Tcp
    }

    async fn handle(&self, stream: TcpStream, route: Route, ctx: &ServerCtx) -> Result<()> {
        let hint = ctx.hint.as_ref();
        let host = hint.and_then(|h| h.sni.clone().or_else(|| h.authority.clone()));
        let initial_data = hint
            .map(|h| h.raw_preface.to_vec())
            .unwrap_or_default();

        debug!(
            host = ?host,
            initial_len = initial_data.len(),
            "TCP passthrough"
        );

        let (group_id, proxy_name) = (route.group_id, route.proxy_name);
        let selected = self
            .registry
            .select_client_for_group(&group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

        let routing_info = tunnel_lib::RoutingInfo {
            proxy_name: proxy_name.to_string(),
            src_addr: ctx.peer_addr.ip().to_string(),
            src_port: ctx.peer_addr.port(),
            protocol: tunnel_lib::proxy::core::Protocol::Tcp,
            host,
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
        tunnel_lib::proxy::forward_to_client(
            send,
            recv,
            stream,
            tunnel_lib::proxy::ProxyBufferParams::default().relay_buf_size,
        )
        .await
    }
}
