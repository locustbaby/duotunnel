use anyhow::{anyhow, Context, Result};
use dashmap::DashMap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};
pub use tunnel_lib::egress::http::{H2cClient, HttpsClient};
use tunnel_lib::proxy::core::{Context as ProxyContext, Protocol, ProxyApp};
use tunnel_lib::proxy::http::HttpPeer;
use tunnel_lib::proxy::peers::PeerKind;
use tunnel_lib::{ClientConfig, HttpClientParams, UpstreamGroup};
struct CachedAddr {
    addr: SocketAddr,
    cached_at: Instant,
}
pub struct LocalProxyMap {
    upstreams: HashMap<String, UpstreamGroup>,
    /// Pre-computed (scheme, authority) keyed by raw server address string.
    /// Covers every server in every upstream group — preserves round-robin across all backends.
    h2_addrs: HashMap<String, (String, String)>,
    /// DNS cache for WebSocket/TCP upstreams: connect_addr_str → SocketAddr.
    /// Populated lazily on first connection; avoids repeated DNS lookups for fixed upstreams.
    dns_cache: DashMap<String, CachedAddr>,
    pub https_client: HttpsClient,
    pub h2c_client: H2cClient,
}
impl LocalProxyMap {
    pub fn from_config(config: &ClientConfig, http_params: &HttpClientParams) -> Self {
        let mut upstreams = HashMap::new();
        let mut h2_addrs = HashMap::new();
        for upstream in &config.upstreams {
            let servers: Vec<String> = upstream.servers.iter().map(|s| s.address.clone()).collect();
            // Pre-parse H2 (scheme, authority) for every server address.
            // Keyed by address string so round-robin via get_local_address() is preserved.
            for addr in &servers {
                if let Some(parsed) = parse_h2_addr(addr) {
                    h2_addrs.insert(addr.clone(), parsed);
                } else {
                    tracing::warn!(
                        upstream = %upstream.name, addr = %addr,
                        "could not parse H2 upstream address — H2 traffic to this server will fail"
                    );
                }
            }
            upstreams.insert(upstream.name.clone(), UpstreamGroup::new(servers));
        }
        let https_client = tunnel_lib::create_https_client_with(http_params);
        let h2c_client = tunnel_lib::create_h2c_client_with(http_params);
        Self {
            upstreams,
            h2_addrs,
            dns_cache: DashMap::new(),
            https_client,
            h2c_client,
        }
    }
    pub fn get_local_address(&self, proxy_name: &str) -> Option<String> {
        let group = self.upstreams.get(proxy_name)?;
        let server = group.next()?;
        debug!(proxy_name = %proxy_name, server = %server, "upstream selected");
        Some(server.clone())
    }
    /// Returns pre-computed (scheme, authority) for the given raw server address string.
    /// Call `get_local_address()` first (for round-robin), then pass the result here.
    pub fn get_h2_addr_for_server(&self, addr: &str) -> Option<(&str, &str)> {
        self.h2_addrs
            .get(addr)
            .map(|(s, a)| (s.as_str(), a.as_str()))
    }
    /// Resolve `connect_addr_str` to a `SocketAddr`, using a lazy DNS cache.
    /// IP addresses are parsed directly. Hostnames are looked up once and cached.
    pub async fn resolve_addr(&self, connect_addr_str: &str) -> Result<SocketAddr> {
        const DNS_CACHE_TTL: Duration = Duration::from_secs(30);
        if let Ok(addr) = connect_addr_str.parse::<SocketAddr>() {
            return Ok(addr);
        }
        if let Some(cached) = self.dns_cache.get(connect_addr_str) {
            if cached.cached_at.elapsed() < DNS_CACHE_TTL {
                return Ok(cached.addr);
            }
        }
        let mut addrs = tokio::net::lookup_host(connect_addr_str)
            .await
            .map_err(|e| {
                anyhow!(
                    "failed to resolve upstream address {}: {}",
                    connect_addr_str,
                    e
                )
            })?;
        let addr = addrs
            .next()
            .ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?;
        self.dns_cache.insert(
            connect_addr_str.to_string(),
            CachedAddr {
                addr,
                cached_at: Instant::now(),
            },
        );
        Ok(addr)
    }
}

/// Normalise a raw upstream address string into (scheme, authority) for H2 use.
fn parse_h2_addr(addr: &str) -> Option<(String, String)> {
    let normalized = addr
        .replace("wss://", "https://")
        .replace("ws://", "http://");
    let with_scheme = if normalized.contains("://") {
        normalized
    } else {
        format!("http://{}", normalized)
    };
    let uri: hyper::Uri = with_scheme.parse().ok()?;
    let scheme = uri.scheme_str().unwrap_or("http").to_string();
    let authority = uri
        .authority()
        .map(|a| a.as_str().to_string())
        .or_else(|| Some(addr.to_string()))?;
    Some((scheme, authority))
}
pub struct ClientApp {
    map: Arc<LocalProxyMap>,
    tcp_params: tunnel_lib::TcpParams,
}
impl ClientApp {
    pub fn new(map: Arc<LocalProxyMap>, tcp_params: tunnel_lib::TcpParams) -> Self {
        Self { map, tcp_params }
    }
}
impl ProxyApp for ClientApp {
    async fn upstream_peer(&self, context: &mut ProxyContext) -> Result<PeerKind> {
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;
        let upstream_addr = self
            .map
            .get_local_address(&routing.proxy_name)
            .ok_or_else(|| anyhow::anyhow!("no upstream for proxy_name: {}", routing.proxy_name))?;
        use tunnel_lib::proxy::tcp::UpstreamScheme;
        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();
        let http_scheme = if is_https { "https" } else { "http" };
        #[allow(unreachable_patterns)]
        match context.protocol {
            Protocol::H1 | Protocol::Unknown => Ok(PeerKind::Http(Box::new(HttpPeer {
                client: self.map.https_client.clone(),
                target_host: connect_addr_str,
                scheme: http_scheme.to_string(),
            }))),
            Protocol::H2 => Ok(PeerKind::H2(Box::new(tunnel_lib::proxy::h2::H2Peer {
                target_host: connect_addr_str,
                scheme: http_scheme.to_string(),
                https_client: self.map.https_client.clone(),
                h2c_client: self.map.h2c_client.clone(),
            }))),
            Protocol::WebSocket => {
                info!("WebSocket protocol detected, using TCP relay");
                let target_addr = self.map.resolve_addr(&connect_addr_str).await?;
                if is_https {
                    Ok(PeerKind::Tls(
                        tunnel_lib::proxy::tcp::TlsTcpPeer::new_with_params(
                            target_addr,
                            tls_host.ok_or_else(|| anyhow!("TLS host required for WSS"))?,
                            None,
                            self.tcp_params.clone(),
                        )?,
                    ))
                } else {
                    Ok(PeerKind::Tcp(tunnel_lib::proxy::tcp::TcpPeer {
                        target_addr,
                        tcp_params: self.tcp_params.clone(),
                    }))
                }
            }
            Protocol::Tcp => {
                info!("TCP protocol detected (opaque TLS)");
                let target_addr = self.map.resolve_addr(&connect_addr_str).await?;
                if is_https {
                    let host_for_cert = tls_host.as_deref().unwrap_or("localhost").to_string();
                    info!(
                        "Terminating ingress TLS to fix SNI for upstream {}",
                        host_for_cert
                    );
                    // Reuse a cached ServerConfig (cert generated once per host, TTL 1h).
                    let server_config = tunnel_lib::get_or_create_server_config(&host_for_cert)?;
                    let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
                    #[allow(dead_code)]
                    struct MitmH2Peer {
                        acceptor: tokio_rustls::TlsAcceptor,
                        target_addr: std::net::SocketAddr,
                        tls_host: String,
                        https_client: HttpsClient,
                        h2c_client: H2cClient,
                    }
                    impl tunnel_lib::proxy::peers::UpstreamPeer for MitmH2Peer {
                        fn connect_boxed<'a>(
                            &'a self,
                            send: quinn::SendStream,
                            recv: quinn::RecvStream,
                            initial_data: Option<bytes::Bytes>,
                        ) -> std::pin::Pin<
                            Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>,
                        > {
                            Box::pin(self.do_connect(send, recv, initial_data))
                        }
                    }
                    impl MitmH2Peer {
                        async fn do_connect(
                            &self,
                            send: quinn::SendStream,
                            recv: quinn::RecvStream,
                            initial_data: Option<bytes::Bytes>,
                        ) -> anyhow::Result<()> {
                            let stream = tunnel_lib::QuinnStream { send, recv };
                            let stream = if let Some(init) = initial_data {
                                tunnel_lib::PrefixedReadWrite::new(stream, init)
                            } else {
                                tunnel_lib::PrefixedReadWrite::new(stream, bytes::Bytes::new())
                            };
                            let accepted_stream = self
                                .acceptor
                                .accept(stream)
                                .await
                                .context("failed to accept ingress TLS for MITM")?;
                            info!(
                                "MITM H2: TLS handshake accepted, starting H2 server for {}",
                                self.tls_host
                            );
                            tunnel_lib::proxy::h2::serve_h2_forward(
                                accepted_stream,
                                self.https_client.clone(),
                                self.h2c_client.clone(),
                                "https".to_string(),
                                self.tls_host.clone(),
                            )
                            .await
                        }
                    }
                    Ok(PeerKind::Dyn(Box::new(MitmH2Peer {
                        acceptor,
                        target_addr,
                        tls_host: host_for_cert,
                        https_client: self.map.https_client.clone(),
                        h2c_client: self.map.h2c_client.clone(),
                    })))
                } else {
                    Ok(PeerKind::Tcp(tunnel_lib::proxy::tcp::TcpPeer {
                        target_addr,
                        tcp_params: self.tcp_params.clone(),
                    }))
                }
            }
            _ => Err(anyhow!("unsupported protocol")),
        }
    }
}
