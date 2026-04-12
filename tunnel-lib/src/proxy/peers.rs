use super::h2::H2Peer;
use super::http::HttpPeer;
use super::tcp::TcpPeer;
use anyhow::Result;
use bytes::Bytes;
use quinn::{RecvStream, SendStream};
pub enum PeerKind {
    Tcp(TcpPeer),
    Http(Box<HttpPeer>),
    H2(Box<H2Peer>),
    Dyn(Box<dyn UpstreamPeer>),
}
impl PeerKind {
    #[inline]
    pub async fn connect(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        match self {
            Self::Tcp(p) => p.connect_inner(send, recv, initial_data).await,
            Self::Http(p) => p.connect_inner(send, recv, initial_data).await,
            Self::H2(p) => p.connect_inner(send, recv, initial_data).await,
            Self::Dyn(p) => p.connect_boxed(send, recv, initial_data).await,
        }
    }
}
pub trait UpstreamPeer: Send + Sync {
    fn connect_boxed<'a>(
        &'a self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}
