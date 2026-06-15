use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tracing::{debug, warn};

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use tunnel_lib::ProxyError;

use crate::ingress::registry::SharedRegistry;

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

    async fn handle(
        &self,
        stream: tunnel_lib::PrefixedReadWrite<tokio::net::TcpStream>,
        route: Option<Route>,
        ctx: &ServerCtx,
    ) -> Result<()> {
        let route = route.ok_or_else(ProxyError::routing_missing_info)?;
        let hint = ctx.hint.as_ref();
        let host = hint.and_then(|h| h.sni.clone().or_else(|| h.authority.clone()));
        let initial_len = hint.map(|h| h.raw_preface.len()).unwrap_or(0);

        debug!(
            host = ?host,
            initial_len = initial_len,
            "TCP passthrough"
        );

        let (group_id, proxy_name) = (route.group_id, route.proxy_name);

        let routing_info = tunnel_lib::RoutingInfo {
            proxy_name: proxy_name.clone(),
            src_addr: ctx.peer_addr.ip().to_string(),
            src_port: ctx.peer_addr.port(),
            protocol: tunnel_lib::proxy::core::Protocol::Tcp,
            host,
        };

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
                &ctx.overload,
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
                        "failed to open QUIC stream on selected passthrough TCP connection, unregistering and retrying"
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
        tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
        tunnel_lib::proxy::forward_prefixed_to_client(send, recv, stream, ctx.relay_buf_size).await
    }
}
