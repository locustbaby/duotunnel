use super::peers::PeerSpec;
use crate::infra::peek_buf::PeekBufPool;
use crate::models::msg::RoutingInfo;
use crate::protocol::sniff::{default_proxyengine_detectors, SniffPolicy, SniffRuntime};
use crate::proxy::buffer_params::ProxyBufferParams;
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use quinn::{RecvStream, SendStream};
use std::net::SocketAddr;
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
    peek_buf_pool: PeekBufPool,
}

impl<A: UpstreamResolver> ProxyEngine<A> {
    pub fn new(app: A) -> Self {
        Self {
            app,
            peek_buf_pool: PeekBufPool::new(ProxyBufferParams::default().peek_buf_size),
        }
    }

    pub fn new_with_peek_buf_size(app: A, peek_buf_size: usize) -> Self {
        Self {
            app,
            peek_buf_pool: PeekBufPool::new(peek_buf_size),
        }
    }

    pub fn new_with_buffer_params(
        app: A,
        buffer_params: crate::proxy::buffer_params::ProxyBufferParams,
    ) -> Self {
        Self::new_with_peek_buf_size(app, buffer_params.peek_buf_size)
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
            let runtime =
                SniffRuntime::new(SniffPolicy::default(), default_proxyengine_detectors());
            let sniffed = match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                runtime.sniff(&mut recv, &self.peek_buf_pool),
            )
            .await
            {
                Ok(res) => res?,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "protocol sniffing timed out on QUIC stream"
                    ));
                }
            };
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
