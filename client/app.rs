use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use tunnel_lib::proxy::core::{Context as ProxyContext, Protocol, ProxyApp};
use tunnel_lib::proxy::http::HttpPeer;
use tunnel_lib::proxy::peers::PeerKind;
use tunnel_lib::{ClientConfig, HttpClientParams, UpstreamGroup};

pub use tunnel_lib::egress::http::{H2cClient, HttpsClient};

pub struct LocalProxyMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: HashMap<String, String>,
    pub https_client: HttpsClient,
    pub h2c_client: H2cClient,
}

impl LocalProxyMap {
    pub fn from_config(config: &ClientConfig, http_params: &HttpClientParams) -> Self {
        let mut upstreams = HashMap::new();
        let mut http_rules = HashMap::new();

        for upstream in &config.upstreams {
            let servers: Vec<String> = upstream.servers.iter().map(|s| s.address.clone()).collect();
            upstreams.insert(upstream.name.clone(), UpstreamGroup::new(servers));
        }

        for rule in &config.rules {
            http_rules.insert(rule.match_host.clone(), rule.action_upstream.clone());
        }

        let https_client = tunnel_lib::create_https_client_with(http_params);
        let h2c_client = tunnel_lib::create_h2c_client_with(http_params);

        Self {
            upstreams,
            http_rules,
            https_client,
            h2c_client,
        }
    }

    pub fn get_local_address(
        &self,
        proxy_name: &str,
        host: Option<&str>,
        is_websocket: bool,
    ) -> Option<String> {
        let rules = &self.http_rules;

        if let Some(host) = host {
            let host_without_port = host.split(':').next().unwrap_or(host);

            if host != host_without_port {
                debug!(original_host = %host, stripped_host = %host_without_port, "stripped port from host for routing lookup");
            }

            if let Some(upstream_name) = rules.get(host_without_port) {
                if let Some(group) = self.upstreams.get(upstream_name) {
                    if let Some(server) = group.next() {
                        debug!(host = %host_without_port, upstream = %upstream_name, server = %server, is_websocket = %is_websocket, "matched routing rule (round-robin)");
                        return Some(server.clone());
                    }
                }
            }
        }

        for (name, group) in &self.upstreams {
            if name == proxy_name || name.contains(proxy_name) {
                return group.first().cloned();
            }
        }

        self.upstreams
            .values()
            .next()
            .and_then(|group| group.first().cloned())
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

impl ProxyApp for ClientApp {
    async fn upstream_peer(&self, context: &mut ProxyContext) -> Result<PeerKind> {
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;

        let is_websocket = matches!(context.protocol, Protocol::WebSocket);

        let upstream_addr = self
            .map
            .get_local_address(&routing.proxy_name, routing.host.as_deref(), is_websocket)
            .ok_or_else(|| {
                anyhow::anyhow!("no upstream address for proxy: {}", routing.proxy_name)
            })?;

        use tunnel_lib::proxy::tcp::UpstreamScheme;

        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();

        #[allow(unreachable_patterns)]
        match context.protocol {
            Protocol::H1 => {
                let scheme = if is_https {
                    "https".to_string()
                } else {
                    "http".to_string()
                };
                Ok(PeerKind::Http(HttpPeer {
                    client: self.map.https_client.clone(),
                    target_host: connect_addr_str,
                    scheme,
                }))
            }
            Protocol::H2 => {
                let scheme = if is_https {
                    "https".to_string()
                } else {
                    "http".to_string()
                };
                Ok(PeerKind::H2(tunnel_lib::proxy::h2::H2Peer {
                    target_host: connect_addr_str,
                    scheme,
                    https_client: self.map.https_client.clone(),
                    h2c_client: self.map.h2c_client.clone(),
                }))
            }
            Protocol::WebSocket => {
                info!("WebSocket protocol detected, using TCP relay");
                let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>()
                {
                    addr
                } else {
                    let mut addrs =
                        tokio::net::lookup_host(&connect_addr_str)
                            .await
                            .map_err(|e| {
                                anyhow!(
                                    "failed to resolve upstream address {}: {}",
                                    connect_addr_str,
                                    e
                                )
                            })?;
                    addrs
                        .next()
                        .ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?
                };

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

                let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>()
                {
                    addr
                } else {
                    let mut addrs =
                        tokio::net::lookup_host(&connect_addr_str)
                            .await
                            .map_err(|e| {
                                anyhow!(
                                    "failed to resolve upstream address {}: {}",
                                    connect_addr_str,
                                    e
                                )
                            })?;
                    addrs
                        .next()
                        .ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?
                };

                if is_https {
                    let host_for_cert = tls_host.as_deref().unwrap_or("localhost");
                    info!(
                        "Terminating ingress TLS to fix SNI for upstream {}",
                        host_for_cert
                    );

                    let (certs, key) =
                        tunnel_lib::infra::pki::generate_self_signed_cert_for_host(host_for_cert)?;
                    let server_config = rustls::ServerConfig::builder()
                        .with_no_client_auth()
                        .with_single_cert(certs, key)?;
                    let acceptor =
                        tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(server_config));

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
                        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
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
                    }  // impl MitmH2Peer

                    Ok(PeerKind::Dyn(Box::new(MitmH2Peer {
                        acceptor,
                        target_addr,
                        tls_host: tls_host.unwrap_or_else(|| "".to_string()),
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
            Protocol::Unknown => {
                let scheme = if is_https {
                    "https".to_string()
                } else {
                    "http".to_string()
                };
                Ok(PeerKind::Http(HttpPeer {
                    client: self.map.https_client.clone(),
                    target_host: connect_addr_str,
                    scheme,
                }))
            }
            _ => Err(anyhow!("unsupported protocol")),
        }
    }
}
