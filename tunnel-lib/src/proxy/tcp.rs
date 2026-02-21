use super::peers::UpstreamPeer;
use crate::engine::{bridge, relay::relay_with_initial};
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
        } else if has_port_443(addr) {
            // Use precise port matching to avoid false positives like host:4430
            (Self::Tls, addr)
        } else {
            (Self::Tcp, addr)
        };

        let host = extract_host_part(rest);
        let addr_with_port = if has_explicit_port(rest) {
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

/// Extract host from an address string, handling IPv6 bracket notation.
fn extract_host_part(addr: &str) -> String {
    if let Some(rest) = addr.strip_prefix('[') {
        // IPv6: [::1]:8080 → "::1"
        rest.split(']').next().unwrap_or(addr).to_string()
    } else {
        addr.split(':').next().unwrap_or(addr).to_string()
    }
}

/// Returns true if the address has an explicit port component.
fn has_explicit_port(addr: &str) -> bool {
    if addr.starts_with('[') {
        addr.contains("]:") // IPv6 with port: [::1]:8080
    } else {
        addr.contains(':')
    }
}

/// Returns true only when the port is exactly 443 (not :4430, :44300, etc.)
fn has_port_443(addr: &str) -> bool {
    extract_port_number(addr) == Some(443)
}

fn extract_port_number(addr: &str) -> Option<u16> {
    if addr.starts_with('[') {
        // IPv6: [::1]:8080 — take everything after the last ':'
        addr.rsplit(':').next()?.parse().ok()
    } else {
        addr.split(':').nth(1)?.parse().ok()
    }
}

pub struct TcpPeer {
    pub target_addr: SocketAddr,
}

/// TLS peer with a pre-built connector shared across all `connect()` calls.
/// Building `rustls::ClientConfig` clones ~150 WebPKI root certs — doing it once
/// per peer instance (at construction time) instead of per connection eliminates
/// significant per-request CPU and memory overhead.
pub struct TlsTcpPeer {
    pub target_addr: SocketAddr,
    pub tls_host: String,
    /// Pre-built connector. `Arc<ClientConfig>` is reference-counted internally,
    /// so cloning the connector is O(1) and does not re-allocate the root store.
    pub connector: Arc<TlsConnector>,
}

impl TlsTcpPeer {
    pub fn new(
        target_addr: SocketAddr,
        tls_host: String,
        alpn: Option<Vec<Vec<u8>>>,
    ) -> Result<Self> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let mut tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        if let Some(alpn) = alpn {
            tls_config.alpn_protocols = alpn;
        }

        Ok(Self {
            target_addr,
            tls_host,
            connector: Arc::new(TlsConnector::from(Arc::new(tls_config))),
        })
    }
}

#[async_trait]
impl UpstreamPeer for TcpPeer {
    async fn connect(
        &self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!("connecting to tcp upstream: {}", self.target_addr);
        let tcp_stream = TcpStream::connect(self.target_addr).await
            .context("failed to connect to tcp upstream")?;
        tcp_stream.set_nodelay(true).context("failed to set TCP_NODELAY on tcp upstream")?;

        let initial_slice = initial_data.as_deref();

        info!(target = %self.target_addr, "starting bidirectional relay (raw tcp, into_split)");
        // Use bridge::relay_with_first_data which calls TcpStream::into_split() — zero-cost
        // split without Arc<Mutex> overhead of the generic tokio::io::split().
        bridge::relay_with_first_data(recv, send, tcp_stream, initial_slice).await?;

        Ok(())
    }
}

#[async_trait]
impl UpstreamPeer for TlsTcpPeer {
    async fn connect(
        &self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!("connecting to TLS tcp upstream: {} (SNI: {})", self.target_addr, self.tls_host);

        let tcp_stream = TcpStream::connect(self.target_addr).await
            .context("failed to connect to tcp upstream")?;
        tcp_stream.set_nodelay(true).context("failed to set TCP_NODELAY on TLS upstream")?;

        let server_name = ServerName::try_from(self.tls_host.clone())
            .map_err(|_| anyhow::anyhow!("invalid TLS server name: {}", self.tls_host))?;

        let tls_stream = self.connector.connect(server_name, tcp_stream).await
            .context("TLS handshake failed")?;

        let initial_slice = initial_data.as_deref();

        info!(target = %self.target_addr, tls_host = %self.tls_host, "starting TLS bidirectional relay");
        relay_with_initial(recv, send, tls_stream, initial_slice.unwrap_or(&[])).await?;

        Ok(())
    }
}
