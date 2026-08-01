use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tracing::{debug, warn};

use duotunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use duotunnel_lib::{ErrorKind, OpenStreamRequest, ProxyError};

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
        stream: duotunnel_lib::PrefixedReadWrite<tokio::net::TcpStream>,
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

        let routing_info = duotunnel_lib::RoutingInfo {
            proxy_name: proxy_name.clone(),
            src_addr: ctx.peer_addr.ip(),
            src_port: ctx.peer_addr.port(),
            protocol: duotunnel_lib::proxy::core::Protocol::Tcp,
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

            let open_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);
            match selected
                .handle
                .open_stream(OpenStreamRequest {
                    routing_info: routing_info.clone(),
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
                        "failed to open QUIC stream on selected passthrough TCP connection, retrying"
                    );
                    if connection_lost {
                        self.registry.unregister(&selected.conn_id);
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
        duotunnel_lib::proxy::forward_prefixed_to_client(send, recv, stream, ctx.relay_buf_size)
            .await
    }
}
