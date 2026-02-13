use super::peers::UpstreamPeer;
use crate::engine::relay::relay_with_initial;
use anyhow::{Result, Context};
use async_trait::async_trait;
use bytes::Bytes;
use quinn::{SendStream, RecvStream};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::{info, debug};
use tokio_rustls::TlsConnector;
use rustls::pki_types::ServerName;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum UpstreamScheme {
    Tcp,
    Tls,
    Http,
    Https,
    Ws,
    Wss,
}

impl UpstreamScheme {
    pub fn from_address(addr: &str) -> (Self, String, Option<String>) {
        let (scheme, rest) = if addr.starts_with("https://") {
            (Self::Https, addr.trim_start_matches("https://"))
        } else if addr.starts_with("http://") {
            (Self::Http, addr.trim_start_matches("http://"))
        } else if addr.starts_with("wss://") {
            (Self::Wss, addr.trim_start_matches("wss://"))
        } else if addr.starts_with("ws://") {
            (Self::Ws, addr.trim_start_matches("ws://"))
        } else if addr.contains(":443") {
            (Self::Tls, addr)
        } else {
            (Self::Tcp, addr)
        };

        let host = rest.split(':').next().unwrap_or(rest).to_string();
        let addr_with_port = if rest.contains(':') {
            rest.to_string()
        } else {
            match scheme {
                Self::Https | Self::Wss | Self::Tls => format!("{}:443", rest),
                _ => format!("{}:80", rest),
            }
        };

        let tls_host = match scheme {
            Self::Https | Self::Wss | Self::Tls => Some(host),
            _ => None,
        };

        (scheme, addr_with_port, tls_host)
    }

    pub fn requires_tls(&self) -> bool {
        matches!(self, Self::Tls | Self::Https | Self::Wss)
    }

    pub fn alpn(&self) -> Option<Vec<Vec<u8>>> {
        match self {
            Self::Https => Some(vec![b"h2".to_vec(), b"http/1.1".to_vec()]),
            Self::Wss => Some(vec![b"http/1.1".to_vec()]),
            _ => None,
        }
    }
}

pub struct TcpPeer {
    pub target_addr: SocketAddr,
}

pub struct TlsTcpPeer {
    pub target_addr: SocketAddr,
    pub tls_host: String,
    pub alpn: Option<Vec<Vec<u8>>>,
}

#[async_trait]
impl UpstreamPeer for TcpPeer {
    async fn connect(
        &self, 
        send: SendStream, 
        recv: RecvStream, 
        initial_data: Option<Bytes>
    ) -> Result<()> {
        debug!("connecting to tcp upstream: {}", self.target_addr);
        let tcp_stream = TcpStream::connect(self.target_addr).await
            .context("failed to connect to tcp upstream")?;
        
        let initial_slice = initial_data.as_deref();
        
        info!(target = %self.target_addr, "starting bidirectional relay (raw tcp)");
        relay_with_initial(recv, send, tcp_stream, initial_slice.unwrap_or(&[])).await?;
        
        Ok(())
    }
}

#[async_trait]
impl UpstreamPeer for TlsTcpPeer {
    async fn connect(
        &self, 
        send: SendStream, 
        recv: RecvStream, 
        initial_data: Option<Bytes>
    ) -> Result<()> {
        debug!("connecting to TLS tcp upstream: {} (SNI: {})", self.target_addr, self.tls_host);
        
        let tcp_stream = TcpStream::connect(self.target_addr).await
            .context("failed to connect to tcp upstream")?;

        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        
        let mut tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        
        if let Some(alpn) = &self.alpn {
            tls_config.alpn_protocols = alpn.clone();
        }
        
        let connector = TlsConnector::from(Arc::new(tls_config));
        let server_name = ServerName::try_from(self.tls_host.clone())
            .map_err(|_| anyhow::anyhow!("invalid TLS server name: {}", self.tls_host))?;
        
        let tls_stream = connector.connect(server_name, tcp_stream).await
            .context("TLS handshake failed")?;
        
        let initial_slice = initial_data.as_deref();
        
        info!(target = %self.target_addr, tls_host = %self.tls_host, "starting TLS bidirectional relay");
        relay_with_initial(recv, send, tls_stream, initial_slice.unwrap_or(&[])).await?;
        
        Ok(())
    }
}
