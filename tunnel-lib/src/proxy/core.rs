use super::peers::PeerSpec;
use crate::infra::peek_buf::PeekBufPool;
use crate::models::msg::RoutingInfo;
use crate::sniff::{default_proxyengine_detectors, SniffPolicy, SniffRuntime};
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use quinn::{RecvStream, SendStream};
use std::net::SocketAddr;
use std::sync::OnceLock;
#[derive(Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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
    ) -> impl std::future::Future<Output = std::result::Result<PeerSpec, ProxyError>> + Send;

    fn connect_peer(
        &self,
        peer: PeerSpec,
        downstream_protocol: Protocol,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> impl std::future::Future<Output = std::result::Result<(), ProxyError>> + Send;
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
        let (protocol, initial_bytes) = if let Some(p) = routing_info
            .as_ref()
            .map(|ri| ri.protocol)
            .filter(|&p| p != Protocol::Unknown)
        {
            (p, None)
        } else {
            let pool = stream_peek_pool();
            let runtime =
                SniffRuntime::new(SniffPolicy::default(), default_proxyengine_detectors());
            let sniffed = runtime.sniff(&mut recv, pool).await?;
            if sniffed.bytes_read > 0 {
                (sniffed.hint.protocol, Some(sniffed.prefix.into_bytes()))
            } else {
                (Protocol::Unknown, None)
            }
        };

        let mut ctx = Context {
            client_addr,
            protocol,
            initial_bytes,
            routing_info,
        };
        let peer = self.app.upstream_peer(&mut ctx).await?;
        self.app
            .connect_peer(peer, ctx.protocol, send, recv, ctx.initial_bytes)
            .await?;
        Ok(())
    }
}
