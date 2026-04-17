use super::peers::PeerKind;
use crate::infra::peek_buf::PeekBufPool;
use crate::models::msg::RoutingInfo;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::OnceLock;
#[derive(
    Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
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
    pub initial_bytes: Option<bytes::Bytes>,
    pub routing_info: Option<RoutingInfo>,
}
pub trait UpstreamResolver: Send + Sync {
    fn upstream_peer(
        &self,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<PeerKind>> + Send;
}
pub struct ProxyEngine<A: UpstreamResolver> {
    app: A,
}

static STREAM_PEEK_POOL: OnceLock<PeekBufPool> = OnceLock::new();
fn stream_peek_pool() -> &'static PeekBufPool {
    STREAM_PEEK_POOL.get_or_init(|| PeekBufPool::new(4096))
}

impl<A: UpstreamResolver> ProxyEngine<A> {
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
        let pool = stream_peek_pool();
        let mut buf = pool.take();
        let n = recv.read(&mut buf[..]).await?.unwrap_or(0);

        let initial_bytes: bytes::Bytes = if n > 0 {
            let b = bytes::Bytes::copy_from_slice(&buf[..n]);
            pool.put(buf);
            b
        } else {
            pool.put(buf);
            bytes::Bytes::new()
        };

        let protocol = detect_protocol(n, &initial_bytes, routing_info.as_ref());
        let mut ctx = Context {
            client_addr,
            protocol,
            initial_bytes: if n > 0 { Some(initial_bytes) } else { None },
            routing_info,
        };
        let peer = self.app.upstream_peer(&mut ctx).await?;
        peer.connect(send, recv, ctx.initial_bytes).await?;
        Ok(())
    }
}

/// Determine protocol from routing_info hint or from the first bytes of the stream.
/// Consolidates the websocket upgrade check into a single httparse pass.
fn detect_protocol(n: usize, data: &[u8], routing_info: Option<&RoutingInfo>) -> Protocol {
    if let Some(ri) = routing_info {
        match ri.protocol {
            Protocol::Unknown => {}
            p => return p,
        }
    }
    if n == 0 {
        return Protocol::Unknown;
    }
    // Single httparse pass: detect both WebSocket upgrade and H1 together.
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    if req.parse(data).is_ok() {
        for h in req.headers {
            if h.name.eq_ignore_ascii_case("Upgrade")
                && std::str::from_utf8(h.value)
                    .unwrap_or("")
                    .eq_ignore_ascii_case("websocket")
            {
                return Protocol::WebSocket;
            }
        }
        return Protocol::H1;
    }
    Protocol::H1
}
