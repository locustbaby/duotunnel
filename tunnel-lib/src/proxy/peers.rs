use super::http_connector::SharedHttpConnector;
use crate::transport::tcp_params::TcpParams;
use super::core::Protocol;
use super::h2::H2Peer;
use super::http::HttpPeer;
use super::tcp::TcpPeer;
use anyhow::Result;
use bytes::Bytes;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum PeerSpec {
    Tcp(BasicPeerSpec),
    Http(HttpPeerSpec),
    MitmH2(MitmPeerSpec),
}

#[derive(Debug, Clone)]
pub struct TlsPeerSpec {
    pub host: String,
    pub alpn: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Clone)]
pub struct BasicPeerSpec {
    pub target_addr: SocketAddr,
    pub tls: Option<TlsPeerSpec>,
}

impl BasicPeerSpec {
    pub fn into_tcp_peer(self, tcp_params: TcpParams) -> Result<TcpPeer> {
        match self.tls {
            Some(tls) => TcpPeer::new_tls_with_params(
                self.target_addr,
                tls.host,
                tls.alpn,
                tcp_params,
            ),
            None => Ok(TcpPeer::new(self.target_addr, tcp_params)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpPeerSpec {
    pub target_host: String,
    pub scheme: String,
    pub protocol: Protocol,
}

impl HttpPeerSpec {
    pub fn into_h1_peer(self, connector: SharedHttpConnector) -> HttpPeer {
        HttpPeer {
            connector,
            spec: self,
        }
    }

    pub fn into_h2_peer(self, connector: SharedHttpConnector) -> H2Peer {
        H2Peer {
            connector,
            spec: self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MitmPeerSpec {
    pub tls_host: String,
}

pub enum PeerKind {
    Tcp(TcpPeer),
    Http(Box<HttpPeer>),
    H2(Box<H2Peer>),
}
impl PeerKind {
    #[inline]
    pub async fn connect(
        self,
        send: quinn::SendStream,
        recv: quinn::RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        match self {
            Self::Tcp(p) => p.connect_inner(send, recv, initial_data).await,
            Self::Http(p) => p.connect_inner(send, recv, initial_data).await,
            Self::H2(p) => p.connect_inner(send, recv, initial_data).await,
        }
    }
}
