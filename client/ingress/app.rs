use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::debug;
use tunnel_lib::plugin::{LoadBalancer, PickCtx, Resolver, Target};
use tunnel_lib::proxy::core::{Context as ProxyContext, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http_connector::SharedHttpConnector;
use tunnel_lib::proxy::peers::{BasicPeerSpec, HttpPeerSpec, MitmPeerSpec, PeerSpec, TlsPeerSpec};
use tunnel_lib::proxy::tcp::UpstreamScheme;
use tunnel_lib::{ClientConfig, HttpClientParams, ProxyError};

struct UpstreamGroup<L> {
    raw: Vec<String>,
    targets: Vec<Target>,
    lb: Arc<L>,
}

pub struct LocalProxyMap<L, R> {
    upstreams: HashMap<String, UpstreamGroup<L>>,
    resolver: Arc<R>,
    pub http_connector: SharedHttpConnector,
}

impl<L: LoadBalancer, R: Resolver> LocalProxyMap<L, R> {
    pub fn from_config(
        config: &ClientConfig,
        http_params: &HttpClientParams,
        lb: Arc<L>,
        resolver: Arc<R>,
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

    pub fn get_local_address(&self, proxy_name: &str, client_addr: SocketAddr) -> Option<String> {
        let group = self.upstreams.get(proxy_name)?;
        let ctx = PickCtx { client_addr };
        let idx = group.lb.pick(&group.targets, &ctx)?;
        let addr = group.raw.get(idx).cloned()?;
        debug!(proxy_name = %proxy_name, server = %addr, "upstream selected");
        Some(addr)
    }

    pub async fn resolve_addr(&self, connect_addr_str: &str) -> Result<SocketAddr> {
        if let Ok(addr) = connect_addr_str.parse::<SocketAddr>() {
            return Ok(addr);
        }
        let parsed = tunnel_lib::transport::addr::parse_upstream(connect_addr_str);
        let mut addrs = self.resolver.resolve(&parsed.host, parsed.port).await?;
        use rand::seq::SliceRandom;
        addrs.shuffle(&mut rand::rng());
        addrs
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))
    }
}

pub struct IngressClientApp<L, R> {
    map: Arc<LocalProxyMap<L, R>>,
    tcp_params: tunnel_lib::TcpParams,
}

impl<L, R> IngressClientApp<L, R> {
    pub fn new(map: Arc<LocalProxyMap<L, R>>, tcp_params: tunnel_lib::TcpParams) -> Self {
        Self { map, tcp_params }
    }
}

impl<L: LoadBalancer, R: Resolver> UpstreamResolver for IngressClientApp<L, R> {
    async fn upstream_peer(&self, context: &mut ProxyContext) -> Result<PeerSpec, ProxyError> {
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(ProxyError::routing_missing_info)?;
        let upstream_addr = self
            .map
            .get_local_address(&routing.proxy_name, context.client_addr)
            .ok_or_else(|| {
                ProxyError::route_not_found(format!("proxy_name={}", routing.proxy_name))
            })?;
        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();
        let http_scheme = if is_https { "https" } else { "http" };
        #[allow(unreachable_patterns)]
        match context.protocol {
            Protocol::H1 | Protocol::Unknown => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: http_scheme.to_string(),
                    upstream_protocol: if is_https {
                        Protocol::Unknown
                    } else {
                        Protocol::H1
                    },
                };
                Ok(PeerSpec::Http(spec))
            }
            Protocol::H2 => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: http_scheme.to_string(),
                    upstream_protocol: if is_https {
                        Protocol::Unknown
                    } else {
                        Protocol::H2
                    },
                };
                Ok(PeerSpec::Http(spec))
            }
            Protocol::WebSocket => {
                let target_addr = self
                    .map
                    .resolve_addr(&connect_addr_str)
                    .await
                    .map_err(|e| {
                        ProxyError::resolve_upstream(format!("{connect_addr_str}: {e}"))
                    })?;
                let spec = BasicPeerSpec {
                    target_addr,
                    tls: if is_https {
                        Some(TlsPeerSpec {
                            host: tls_host.ok_or_else(|| {
                                ProxyError::upstream_connect("TLS host required for WSS")
                            })?,
                            alpn: None,
                        })
                    } else {
                        None
                    },
                    upstream_name: None,
                    upstream_addr_str: None,
                };
                Ok(PeerSpec::Tcp(spec))
            }
            Protocol::Tcp => {
                if is_https {
                    let spec = MitmPeerSpec {
                        tls_host: tls_host.as_deref().unwrap_or("localhost").to_string(),
                    };
                    Ok(PeerSpec::MitmH2(spec))
                } else {
                    let target_addr =
                        self.map
                            .resolve_addr(&connect_addr_str)
                            .await
                            .map_err(|e| {
                                ProxyError::resolve_upstream(format!("{connect_addr_str}: {e}"))
                            })?;
                    let spec = BasicPeerSpec {
                        target_addr,
                        tls: None,
                        upstream_name: None,
                        upstream_addr_str: None,
                    };
                    Ok(PeerSpec::Tcp(spec))
                }
            }
            p => Err(ProxyError::unsupported_protocol(format!(
                "client app: {p:?}"
            ))),
        }
    }

    async fn connect_peer(
        &self,
        peer: PeerSpec,
        downstream_protocol: Protocol,
        send: quinn::SendStream,
        recv: quinn::RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<(), ProxyError> {
        match peer {
            PeerSpec::Tcp(spec) => spec
                .into_tcp_peer(self.tcp_params.clone())
                .map_err(|e| ProxyError::upstream_connect(e.to_string()))?
                .connect_inner(send, recv, initial_data)
                .await
                .map_err(|e| ProxyError::upstream_forward(e.to_string())),
            PeerSpec::Http(spec) => self
                .map
                .http_connector
                .connect(spec, downstream_protocol, send, recv, initial_data)
                .await
                .map_err(|e| ProxyError::http_upstream_request(e.to_string())),
            PeerSpec::MitmH2(spec) => {
                let server_config = tunnel_lib::get_or_create_server_config(&spec.tls_host)
                    .await
                    .map_err(|e| ProxyError::upstream_connect(e.to_string()))?;
                let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
                let stream = tunnel_lib::QuinnStream { send, recv };
                let stream = if let Some(init) = initial_data {
                    tunnel_lib::PrefixedReadWrite::new(stream, init)
                } else {
                    tunnel_lib::PrefixedReadWrite::new(stream, bytes::Bytes::new())
                };
                let accepted_stream = acceptor.accept(stream).await.map_err(|e| {
                    ProxyError::tls_handshake(format!("failed to accept ingress TLS for MITM: {e}"))
                })?;
                self.map
                    .http_connector
                    .serve_h2(
                        accepted_stream,
                        HttpPeerSpec {
                            target_host: spec.tls_host,
                            scheme: "https".to_string(),
                            upstream_protocol: Protocol::H2,
                        },
                    )
                    .await
                    .map_err(|e| ProxyError::http_upstream_request(e.to_string()))
            }
        }
    }
}
