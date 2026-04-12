use crate::engine::{bridge, relay::relay_with_initial};
use crate::transport::addr::parse_upstream;
use anyhow::{Context, Result};
use bytes::Bytes;
use quinn::{RecvStream, SendStream};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tracing::{debug, info};
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
    /// Parse an upstream address string into a scheme, a `host:port` connect address,
    /// and an optional TLS SNI hostname.
    ///
    /// Delegates address/port/host extraction to [`parse_upstream`] so there is a
    /// single source of truth for URL parsing in `transport::addr`.
    pub fn from_address(addr: &str) -> (Self, String, Option<String>) {
        let parsed = parse_upstream(addr);
        let scheme = if addr.starts_with("https://") {
            Self::Https
        } else if addr.starts_with("http://") {
            Self::Http
        } else if addr.starts_with("wss://") {
            Self::Wss
        } else if addr.starts_with("ws://") {
            Self::Ws
        } else if parsed.is_https {
            Self::Tls
        } else {
            Self::Tcp
        };
        let tls_host = if scheme.requires_tls() {
            Some(parsed.host)
        } else {
            None
        };
        (scheme, parsed.connect_addr, tls_host)
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
    pub tcp_params: crate::transport::tcp_params::TcpParams,
}
pub struct TlsTcpPeer {
    pub target_addr: SocketAddr,
    pub tls_host: String,
    pub connector: Arc<TlsConnector>,
    pub tcp_params: crate::transport::tcp_params::TcpParams,
}
/// Shared base TLS config with system roots loaded once.
/// ALPN is set per-connection by cloning and mutating before wrapping in TlsConnector.
fn base_tls_config() -> Arc<rustls::ClientConfig> {
    use std::sync::OnceLock;
    static BASE: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();
    BASE.get_or_init(|| {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        )
    })
    .clone()
}

impl TlsTcpPeer {
    pub fn new(
        target_addr: SocketAddr,
        tls_host: String,
        alpn: Option<Vec<Vec<u8>>>,
    ) -> Result<Self> {
        Self::new_with_params(
            target_addr,
            tls_host,
            alpn,
            crate::transport::tcp_params::TcpParams::default(),
        )
    }
    pub fn new_with_params(
        target_addr: SocketAddr,
        tls_host: String,
        alpn: Option<Vec<Vec<u8>>>,
        tcp_params: crate::transport::tcp_params::TcpParams,
    ) -> Result<Self> {
        let connector = if let Some(alpn) = alpn {
            // Clone only when ALPN overrides are needed — avoids copying the CA store.
            let mut cfg = (*base_tls_config()).clone();
            cfg.alpn_protocols = alpn;
            Arc::new(TlsConnector::from(Arc::new(cfg)))
        } else {
            Arc::new(TlsConnector::from(base_tls_config()))
        };
        Ok(Self {
            target_addr,
            tls_host,
            connector,
            tcp_params,
        })
    }
}
impl TcpPeer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!("connecting to tcp upstream: {}", self.target_addr);
        let tcp_stream = TcpStream::connect(self.target_addr)
            .await
            .context("failed to connect to tcp upstream")?;
        self.tcp_params
            .apply(&tcp_stream)
            .context("failed to apply TCP params to upstream")?;
        let initial_slice = initial_data.as_deref();
        info!(
            target = % self.target_addr,
            "starting bidirectional relay (raw tcp, into_split)"
        );
        bridge::relay_with_first_data(recv, send, tcp_stream, initial_slice).await?;
        Ok(())
    }
}
impl TlsTcpPeer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!(
            "connecting to TLS tcp upstream: {} (SNI: {})",
            self.target_addr, self.tls_host
        );
        let tcp_stream = TcpStream::connect(self.target_addr)
            .await
            .context("failed to connect to tcp upstream")?;
        self.tcp_params
            .apply(&tcp_stream)
            .context("failed to apply TCP params to TLS upstream")?;
        let server_name = ServerName::try_from(self.tls_host.clone())
            .map_err(|_| anyhow::anyhow!("invalid TLS server name: {}", self.tls_host))?;
        let tls_stream = self
            .connector
            .connect(server_name, tcp_stream)
            .await
            .context("TLS handshake failed")?;
        let initial_slice = initial_data.as_deref();
        info!(
            target = % self.target_addr, tls_host = % self.tls_host,
            "starting TLS bidirectional relay"
        );
        relay_with_initial(recv, send, tls_stream, initial_slice.unwrap_or(&[])).await?;
        Ok(())
    }
}
