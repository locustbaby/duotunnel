use anyhow::Result;
use bytes::Bytes;
use quinn::{RecvStream, SendStream};

use super::h2::H2Peer;
use super::http::HttpPeer;
use super::tcp::{TcpPeer, TlsTcpPeer};

/// Static-dispatch peer selector.
///
/// Replaces `Box<dyn UpstreamPeer>` on the hot path.  Every variant is a
/// concrete type so the compiler can mono-morphise and inline `connect()`,
/// eliminating the vtable indirection and the heap allocation that
/// `async_trait` requires for `Box<dyn Future>`.
///
/// `Dyn` is an escape hatch for rare one-off peers (e.g. the MITM H2 peer
/// in the client).  It costs one heap allocation; all common paths use the
/// concrete variants.
pub enum PeerKind {
    Tcp(TcpPeer),
    Tls(TlsTcpPeer),
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
            Self::Tls(p) => p.connect_inner(send, recv, initial_data).await,
            Self::Http(p) => p.connect_inner(send, recv, initial_data).await,
            Self::H2(p) => p.connect_inner(send, recv, initial_data).await,
            Self::Dyn(p) => p.connect_boxed(send, recv, initial_data).await,
        }
    }
}

/// Escape hatch for one-off peer types that cannot be expressed as a
/// `PeerKind` variant.  Prefer adding a variant to `PeerKind` instead.
pub trait UpstreamPeer: Send + Sync {
    fn connect_boxed<'a>(
        &'a self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}
