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
/// TLS configuration for an upstream TCP connection.
/// Present when the upstream requires TLS; absent for plain TCP.
pub struct TlsConfig {
    pub host: String,
    pub connector: Arc<TlsConnector>,
}

pub struct TcpPeer {
    pub target_addr: SocketAddr,
    /// `None` = plain TCP, `Some` = TLS with the given config.
    pub tls: Option<TlsConfig>,
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

impl TcpPeer {
    /// Construct a plain TCP peer.
    pub fn new(target_addr: SocketAddr, tcp_params: crate::transport::tcp_params::TcpParams) -> Self {
        Self { target_addr, tls: None, tcp_params }
    }

    /// Construct a TLS peer. `alpn` is optional; pass `None` for the default TLS config.
    pub fn new_tls(
        target_addr: SocketAddr,
        tls_host: String,
        alpn: Option<Vec<Vec<u8>>>,
    ) -> Result<Self> {
        Self::new_tls_with_params(
            target_addr,
            tls_host,
            alpn,
            crate::transport::tcp_params::TcpParams::default(),
        )
    }

    /// Construct a TLS peer with explicit TCP params.
    pub fn new_tls_with_params(
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
            tls: Some(TlsConfig { host: tls_host, connector }),
            tcp_params,
        })
    }

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

        match self.tls {
            None => {
                info!(target = %self.target_addr, "starting bidirectional relay (raw tcp)");
                bridge::relay_with_first_data(recv, send, tcp_stream, initial_data.as_deref()).await?;
            }
            Some(tls) => {
                debug!("TLS upstream: {} (SNI: {})", self.target_addr, tls.host);
                let server_name = ServerName::try_from(tls.host.clone())
                    .map_err(|_| anyhow::anyhow!("invalid TLS server name: {}", tls.host))?;
                let tls_stream = tls
                    .connector
                    .connect(server_name, tcp_stream)
                    .await
                    .context("TLS handshake failed")?;
                info!(target = %self.target_addr, tls_host = %tls.host, "starting TLS bidirectional relay");
                relay_with_initial(recv, send, tls_stream, initial_data.as_deref().unwrap_or(&[])).await?;
            }
        }
        Ok(())
    }
}
