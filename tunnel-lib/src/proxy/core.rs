use async_trait::async_trait;
use anyhow::Result;
use std::net::SocketAddr;
use bytes::Bytes;
use super::peers::UpstreamPeer;

use crate::models::msg::RoutingInfo;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    H1,
    H2,
    WebSocket,
    Tcp,
    Unknown,
}

pub struct Context {
    pub client_addr: SocketAddr,
    pub protocol: Protocol,
    pub initial_bytes: Option<Bytes>,
    pub routing_info: Option<RoutingInfo>,
}

#[async_trait]
pub trait ProxyApp: Send + Sync {
    /// Determine the upstream peer based on context
    async fn upstream_peer(&self, context: &mut Context) -> Result<Box<dyn UpstreamPeer>>;
}

pub struct ProxyEngine<A: ProxyApp> {
    app: A,
}

impl<A: ProxyApp> ProxyEngine<A> {
    pub fn new(app: A) -> Self {
        Self { app }
    }

    pub async fn run_stream(
        &self, 
        send: quinn::SendStream, 
        mut recv: quinn::RecvStream,
        client_addr: SocketAddr,
        routing_info: Option<RoutingInfo>,
    ) -> Result<()> {
        let mut buf = vec![0u8; 4096];
        let n = recv.read(&mut buf).await?.unwrap_or(0);
        let initial_bytes = Bytes::copy_from_slice(&buf[..n]);

        let protocol = if let Some(ref ri) = routing_info {
            match ri.protocol.as_str() {
                "h2" => Protocol::H2,
                "websocket" => Protocol::WebSocket,
                "h1" => Protocol::H1,
                "tcp" => Protocol::Tcp,
                _ => {
                    if n > 0 && is_websocket_upgrade(&initial_bytes) {
                        Protocol::WebSocket
                    } else if n > 0 {
                        Protocol::H1
                    } else {
                        Protocol::Unknown
                    }
                }
            }
        } else if n > 0 && is_websocket_upgrade(&initial_bytes) {
            Protocol::WebSocket
        } else if n > 0 {
            Protocol::H1
        } else {
            Protocol::Unknown
        };

        let mut ctx = Context {
            client_addr,
            protocol,
            initial_bytes: Some(initial_bytes),
            routing_info,
        };

        let peer = self.app.upstream_peer(&mut ctx).await?;
        peer.connect(send, recv, ctx.initial_bytes).await?;

        Ok(())
    }
}

fn is_websocket_upgrade(data: &[u8]) -> bool {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    if req.parse(data).is_ok() {
        for h in req.headers {
             if h.name.eq_ignore_ascii_case("Upgrade") && 
                std::str::from_utf8(h.value).unwrap_or("").eq_ignore_ascii_case("websocket") {
                 return true;
             }
        }
    }
    false
}
