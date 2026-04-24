use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};
use tunnel_lib::plugin::{LoadBalancer, PickCtx, Resolver, Target};
use tunnel_lib::proxy::core::{Context as ProxyContext, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http_connector::SharedHttpConnector;
use tunnel_lib::proxy::peers::{
    BasicPeerSpec, HttpPeerSpec, MitmPeerSpec, PeerSpec, TlsPeerSpec,
};
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
    pub http_connector: SharedHttpConnector,
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
        let http_connector =
            tunnel_lib::proxy::http_connector::HttpConnector::new(https_client, h2c_client);
        Self {
            upstreams,
            resolver,
            http_connector,
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
    async fn upstream_peer(&self, context: &mut ProxyContext) -> Result<PeerSpec> {
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
            Protocol::H1 | Protocol::Unknown => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: http_scheme.to_string(),
                    protocol: context.protocol,
                };
                Ok(PeerSpec::Http(spec))
            }
            Protocol::H2 => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: http_scheme.to_string(),
                    protocol: context.protocol,
                };
                Ok(PeerSpec::Http(spec))
            }
            Protocol::WebSocket => {
                info!("WebSocket protocol detected, using TCP relay");
                let target_addr = self.map.resolve_addr(&connect_addr_str).await?;
                let spec = BasicPeerSpec {
                    target_addr,
                    tls: if is_https {
                        Some(TlsPeerSpec {
                            host: tls_host.ok_or_else(|| anyhow!("TLS host required for WSS"))?,
                            alpn: None,
                        })
                    } else {
                        None
                    },
                };
                Ok(PeerSpec::Tcp(spec))
            }
            Protocol::Tcp => {
                info!("TCP protocol detected (opaque TLS)");
                if is_https {
                    let spec = MitmPeerSpec {
                        tls_host: tls_host.as_deref().unwrap_or("localhost").to_string(),
                    };
                    info!(
                        "Terminating ingress TLS to fix SNI for upstream {}",
                        spec.tls_host
                    );
                    Ok(PeerSpec::MitmH2(spec))
                } else {
                    let target_addr = self.map.resolve_addr(&connect_addr_str).await?;
                    let spec = BasicPeerSpec {
                        target_addr,
                        tls: None,
                    };
                    Ok(PeerSpec::Tcp(spec))
                }
            }
            _ => Err(anyhow!("unsupported protocol")),
        }
    }

    async fn connect_peer(
        &self,
        peer: PeerSpec,
        send: quinn::SendStream,
        recv: quinn::RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        match peer {
            PeerSpec::Tcp(spec) => spec
                .into_tcp_peer(self.tcp_params.clone())?
                .connect_inner(send, recv, initial_data)
                .await,
            PeerSpec::Http(spec) => self
                .map
                .http_connector
                .connect(spec, send, recv, initial_data)
                .await,
            PeerSpec::MitmH2(spec) => {
                let server_config = tunnel_lib::get_or_create_server_config(&spec.tls_host)?;
                let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
                let stream = tunnel_lib::QuinnStream { send, recv };
                let stream = if let Some(init) = initial_data {
                    tunnel_lib::PrefixedReadWrite::new(stream, init)
                } else {
                    tunnel_lib::PrefixedReadWrite::new(stream, bytes::Bytes::new())
                };
                let accepted_stream = acceptor
                    .accept(stream)
                    .await
                    .context("failed to accept ingress TLS for MITM")?;
                info!(
                    "MITM H2: TLS handshake accepted, starting H2 server for {}",
                    spec.tls_host
                );
                self.map
                    .http_connector
                    .serve_h2(
                        accepted_stream,
                        HttpPeerSpec {
                            target_host: spec.tls_host,
                            scheme: "https".to_string(),
                            protocol: Protocol::H2,
                        },
                    )
                    .await
            }
        }
    }
}
