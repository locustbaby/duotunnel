use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};
pub use tunnel_lib::egress::http::{H2cClient, HttpsClient};
use tunnel_lib::plugin::{LoadBalancer, PickCtx, Resolver, Target};
use tunnel_lib::proxy::core::{Context as ProxyContext, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http::HttpPeer;
use tunnel_lib::proxy::peers::PeerKind;
use tunnel_lib::proxy::tcp::UpstreamScheme;
use tunnel_lib::{ClientConfig, HttpClientParams};

/// One named upstream group. `raw` is the source-of-truth address list in
/// `scheme://host:port` form. `targets` is a parallel view built at config
/// load time for the `LoadBalancer` plugin; the LB returns an index into the
/// slice which we use directly into `raw`.
struct UpstreamGroup {
    raw: Vec<String>,
    targets: Vec<Target>,
    lb: Arc<dyn LoadBalancer>,
}

pub struct LocalProxyMap {
    upstreams: HashMap<String, UpstreamGroup>,
    resolver: Arc<dyn Resolver>,
    pub https_client: HttpsClient,
    pub h2c_client: H2cClient,
}

impl LocalProxyMap {
    pub fn from_config(
        config: &ClientConfig,
        http_params: &HttpClientParams,
        lb: Arc<dyn LoadBalancer>,
        resolver: Arc<dyn Resolver>,
    ) -> Self {
        let mut upstreams = HashMap::new();
        for upstream in &config.upstreams {
            let raw: Vec<String> = upstream.servers.iter().map(|s| s.address.clone()).collect();
            let targets: Vec<Target> = raw
                .iter()
                .map(|addr| {
                    let (scheme, _connect_addr, _tls_host) = UpstreamScheme::from_address(addr);
                    let parsed = tunnel_lib::transport::addr::parse_upstream(addr);
                    Target {
                        host: parsed.host,
                        port: parsed.port,
                        scheme,
                    }
                })
                .collect();
            upstreams.insert(
                upstream.name.clone(),
                UpstreamGroup {
                    raw,
                    targets,
                    lb: lb.clone(),
                },
            );
        }
        let https_client = tunnel_lib::create_https_client_with(http_params);
        let h2c_client = tunnel_lib::create_h2c_client_with(http_params);
        Self {
            upstreams,
            resolver,
            https_client,
            h2c_client,
        }
    }

    /// Returns the raw upstream address (still in `scheme://host:port` form)
    /// chosen by the `LoadBalancer` plugin for `proxy_name`.
    pub fn get_local_address(&self, proxy_name: &str, client_addr: SocketAddr) -> Option<String> {
        let group = self.upstreams.get(proxy_name)?;
        let ctx = PickCtx { client_addr };
        let idx = group.lb.pick(&group.targets, &ctx)?;
        let addr = group.raw.get(idx).cloned()?;
        debug!(proxy_name = %proxy_name, server = %addr, "upstream selected");
        Some(addr)
    }

    /// Resolve `connect_addr_str` (either an IP literal `host:port` or a
    /// hostname `host:port`) to a single `SocketAddr` via the `Resolver`
    /// plugin.
    pub async fn resolve_addr(&self, connect_addr_str: &str) -> Result<SocketAddr> {
        if let Ok(addr) = connect_addr_str.parse::<SocketAddr>() {
            return Ok(addr);
        }
        let parsed = tunnel_lib::transport::addr::parse_upstream(connect_addr_str);
        let addrs = self.resolver.resolve(&parsed.host, parsed.port).await?;
        addrs
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))
    }
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
impl UpstreamResolver for ClientApp {
    async fn upstream_peer(&self, context: &mut ProxyContext) -> Result<PeerKind> {
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;
        let upstream_addr = self
            .map
            .get_local_address(&routing.proxy_name, context.client_addr)
            .ok_or_else(|| anyhow::anyhow!("no upstream for proxy_name: {}", routing.proxy_name))?;
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
                    Ok(PeerKind::Tcp(
                        tunnel_lib::proxy::tcp::TcpPeer::new_tls_with_params(
                            target_addr,
                            tls_host.ok_or_else(|| anyhow!("TLS host required for WSS"))?,
                            None,
                            self.tcp_params.clone(),
                        )?,
                    ))
                } else {
                    Ok(PeerKind::Tcp(tunnel_lib::proxy::tcp::TcpPeer::new(
                        target_addr,
                        self.tcp_params.clone(),
                    )))
                }
            }
            Protocol::Tcp => {
                info!("TCP protocol detected (opaque TLS)");
                if is_https {
                    let host_for_cert = tls_host.as_deref().unwrap_or("localhost").to_string();
                    info!(
                        "Terminating ingress TLS to fix SNI for upstream {}",
                        host_for_cert
                    );
                    let server_config = tunnel_lib::get_or_create_server_config(&host_for_cert)?;
                    let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
                    struct MitmH2Peer {
                        acceptor: tokio_rustls::TlsAcceptor,
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
                        tls_host: host_for_cert,
                        https_client: self.map.https_client.clone(),
                        h2c_client: self.map.h2c_client.clone(),
                    })))
                } else {
                    let target_addr = self.map.resolve_addr(&connect_addr_str).await?;
                    Ok(PeerKind::Tcp(tunnel_lib::proxy::tcp::TcpPeer::new(
                        target_addr,
                        self.tcp_params.clone(),
                    )))
                }
            }
            _ => Err(anyhow!("unsupported protocol")),
        }
    }
}
