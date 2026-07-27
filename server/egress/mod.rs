use crate::bootstrap::config::ServerEgressUpstream;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use tunnel_lib::proxy::core::{Context, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http_connector::SharedHttpConnector;
use tunnel_lib::proxy::peers::{BasicPeerSpec, HttpPeerSpec, PeerSpec, TlsPeerSpec};
use tunnel_lib::proxy::tcp::UpstreamScheme;
use tunnel_lib::proxy::upstream::UpstreamHealthRegistry;
use tunnel_lib::{HttpClientParams, ProxyError, UpstreamGroup, VhostRouter};

pub struct ServerEgressMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: VhostRouter<String>,
    http_connector: SharedHttpConnector,
    dns_cache: tunnel_lib::EgressDnsCache,
}
impl ServerEgressMap {
    pub fn from_config_with_health(
        egress: &ServerEgressUpstream,
        http_params: &HttpClientParams,
        health: Arc<UpstreamHealthRegistry>,
    ) -> anyhow::Result<Self> {
        let mut upstreams = HashMap::new();
        let http_rules = VhostRouter::new();
        for (name, upstream_def) in &egress.upstreams {
            let servers: Vec<String> = upstream_def
                .servers
                .iter()
                .map(|s| s.address.clone())
                .collect();
            upstreams.insert(
                name.clone(),
                UpstreamGroup::with_scoped_health_registry(
                    servers,
                    Arc::<str>::from(name.as_str()),
                    health.clone(),
                ),
            );
        }
        for rule in &egress.rules.vhost {
            http_rules.add_route(&rule.match_host, rule.action_upstream.clone());
        }
        let https_client = tunnel_lib::create_https_client_with(http_params)?;
        let h2c_client = tunnel_lib::create_h2c_client_with(http_params);
        let http_connector = tunnel_lib::proxy::http_connector::HttpConnector::new_with_health(
            https_client,
            h2c_client,
            Some(health),
        );
        Ok(Self {
            upstreams,
            http_rules,
            http_connector,
            dns_cache: tunnel_lib::EgressDnsCache::new(std::time::Duration::from_secs(30)),
        })
    }

    pub async fn resolve_udp_target(
        &self,
        upstream_name: &str,
    ) -> Result<std::net::SocketAddr, ProxyError> {
        let group = self.upstreams.get(upstream_name).ok_or_else(|| {
            ProxyError::route_not_found(format!("upstream_group={upstream_name}"))
        })?;
        self.resolve_healthy_target(upstream_name, group)
            .await
            .map(|(_, target)| target)
    }

    async fn resolve_healthy_target(
        &self,
        upstream_name: &str,
        group: &UpstreamGroup,
    ) -> Result<(String, std::net::SocketAddr), ProxyError> {
        let mut last_error = None;
        for _ in 0..group.servers.len() {
            let Some(upstream_addr) = group.next_healthy().cloned() else {
                break;
            };
            let parsed = tunnel_lib::transport::addr::parse_upstream(&upstream_addr);
            match self.resolve_target_addr(&parsed.connect_addr).await {
                Ok(target) => return Ok((upstream_addr, target)),
                Err(error) => {
                    group.mark_unhealthy(&upstream_addr);
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| {
            ProxyError::route_not_found(format!("no healthy backends for upstream={upstream_name}"))
        }))
    }

    async fn resolve_target_addr(
        &self,
        connect_addr_str: &str,
    ) -> Result<std::net::SocketAddr, ProxyError> {
        if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>() {
            return Ok(addr);
        }
        let mut parts = connect_addr_str.rsplitn(2, ':');
        let port_str = parts.next().ok_or_else(|| {
            ProxyError::resolve_upstream(format!("missing port in {}", connect_addr_str))
        })?;
        let host_str = parts.next().ok_or_else(|| {
            ProxyError::resolve_upstream(format!("missing host in {}", connect_addr_str))
        })?;
        let port = port_str.parse::<u16>().map_err(|_| {
            ProxyError::resolve_upstream(format!(
                "invalid port {} in {}",
                port_str, connect_addr_str
            ))
        })?;
        self.dns_cache
            .resolve(host_str, port)
            .await
            .map_err(|e| ProxyError::resolve_upstream(format!("{connect_addr_str}: {e}")))
    }
}
impl UpstreamResolver for ServerEgressMap {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerSpec, ProxyError> {
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(ProxyError::routing_missing_info)?;
        let host_raw = routing
            .host
            .as_deref()
            .ok_or_else(ProxyError::routing_missing_host)?;
        let upstream_name = self
            .http_rules
            .get(host_raw)
            .ok_or_else(|| ProxyError::route_not_found(format!("host={host_raw}")))?;
        let group = self.upstreams.get(&upstream_name).ok_or_else(|| {
            ProxyError::route_not_found(format!("upstream_group={upstream_name}"))
        })?;
        let backend_ref = group.next_healthy_ref();
        let upstream_addr = backend_ref
            .as_ref()
            .map(|r| r.address().to_string())
            .or_else(|| group.next_healthy().cloned())
            .ok_or_else(|| {
                ProxyError::route_not_found(format!("no healthy backends for host={host_raw}"))
            })?;

        let (scheme, connect_addr_str, _) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();
        match context.protocol {
            Protocol::WebSocket => {
                info!("WebSocket egress, using TCP forwarding");
                let (upstream_addr, target_addr) =
                    self.resolve_healthy_target(&upstream_name, group).await?;
                let (scheme, _, tls_host) = UpstreamScheme::from_address(&upstream_addr);
                let is_https = scheme.requires_tls();
                let spec = BasicPeerSpec {
                    target_addr,
                    tls: is_https.then(|| TlsPeerSpec {
                        host: tls_host.unwrap_or_default(),
                        alpn: scheme.alpn(),
                    }),
                    upstream_name: Some(upstream_name),
                    upstream_addr_str: Some(upstream_addr),
                    backend_ref,
                };
                Ok(PeerSpec::Tcp(spec))
            }
            Protocol::H1 | Protocol::Unknown => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: if is_https { "https" } else { "http" }.to_string(),
                    upstream_protocol: if is_https {
                        Protocol::Unknown
                    } else {
                        Protocol::H1
                    },
                    upstream_name: Some(upstream_name),
                    upstream_addr_str: Some(upstream_addr),
                    backend_ref,
                };
                Ok(PeerSpec::Http(spec))
            }
            Protocol::H2 => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: if is_https { "https" } else { "http" }.to_string(),
                    upstream_protocol: if is_https {
                        Protocol::Unknown
                    } else {
                        Protocol::H2
                    },
                    upstream_name: Some(upstream_name),
                    upstream_addr_str: Some(upstream_addr),
                    backend_ref,
                };
                Ok(PeerSpec::Http(spec))
            }
            p => Err(ProxyError::unsupported_protocol(format!(
                "server egress: {p:?}"
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
            PeerSpec::Tcp(spec) => {
                let mut final_health_identity = spec
                    .upstream_name
                    .clone()
                    .zip(spec.upstream_addr_str.clone());
                let (tcp_stream, final_tls) = if let (Some(upstream_name), Some(failed_addr)) =
                    (&spec.upstream_name, &spec.upstream_addr_str)
                {
                    let group = self.upstreams.get(upstream_name).ok_or_else(|| {
                        ProxyError::route_not_found(format!("upstream={upstream_name}"))
                    })?;
                    let mut last_err = None;
                    let mut current_failed = failed_addr.clone();
                    let mut current_spec = spec.clone();

                    let mut connected = None;
                    for _ in 0..group.servers.len().max(1) {
                        let tcp_peer = current_spec
                            .clone()
                            .into_tcp_peer(tunnel_lib::TcpParams::default())
                            .map_err(|e| ProxyError::upstream_connect(e.to_string()))?;

                        let stream_res = tokio::net::TcpStream::connect(tcp_peer.target_addr).await;

                        match stream_res {
                            Ok(stream) => {
                                final_health_identity =
                                    Some((upstream_name.clone(), current_failed.clone()));
                                connected = Some((stream, tcp_peer.tls));
                                break;
                            }
                            Err(e) => {
                                warn!(server = %current_failed, error = %e, "connection to upstream failed, marking unhealthy");
                                group.mark_unhealthy(&current_failed);
                                last_err = Some(e);

                                let mut replacement_found = false;
                                while let Some(next_addr) = group.next_healthy().cloned() {
                                    let (scheme, connect_addr_str, tls_host) =
                                        UpstreamScheme::from_address(&next_addr);
                                    let is_https = scheme.requires_tls();

                                    match self.resolve_target_addr(&connect_addr_str).await {
                                        Ok(addr) => {
                                            current_failed = next_addr;
                                            current_spec.target_addr = addr;
                                            current_spec.tls = is_https.then(|| TlsPeerSpec {
                                                host: tls_host.unwrap_or_default(),
                                                alpn: scheme.alpn(),
                                            });
                                            current_spec.upstream_addr_str =
                                                Some(current_failed.clone());
                                            replacement_found = true;
                                            break;
                                        }
                                        Err(resolve_err) => {
                                            group.mark_unhealthy(&next_addr);
                                            last_err = Some(std::io::Error::other(
                                                resolve_err.to_string(),
                                            ));
                                        }
                                    }
                                }
                                if !replacement_found {
                                    break;
                                }
                            }
                        }
                    }

                    let (stream, tls) = connected.ok_or_else(|| {
                        ProxyError::upstream_connect(
                            last_err.map(|e| e.to_string()).unwrap_or_default(),
                        )
                    })?;
                    (stream, tls)
                } else {
                    let tcp_peer = spec
                        .into_tcp_peer(tunnel_lib::TcpParams::default())
                        .map_err(|e| ProxyError::upstream_connect(e.to_string()))?;
                    let stream = tokio::net::TcpStream::connect(tcp_peer.target_addr)
                        .await
                        .map_err(|e| ProxyError::upstream_connect(e.to_string()))?;
                    (stream, tcp_peer.tls)
                };

                let tcp_params = tunnel_lib::TcpParams::default();
                tcp_params
                    .apply(&tcp_stream)
                    .map_err(|e| ProxyError::upstream_connect(e.to_string()))?;

                match final_tls {
                    None => {
                        if let Some((upstream_name, upstream_addr)) = &final_health_identity {
                            if let Some(group) = self.upstreams.get(upstream_name) {
                                group.mark_healthy(upstream_addr);
                            }
                        }
                        tunnel_lib::engine::bridge::relay_with_first_data(
                            recv,
                            send,
                            tcp_stream,
                            initial_data.as_deref(),
                        )
                        .await
                        .map_err(|e| ProxyError::upstream_forward(e.to_string()))?;
                    }
                    Some(tls) => {
                        let server_name = rustls::pki_types::ServerName::try_from(tls.host.clone())
                            .map_err(|e| {
                                ProxyError::upstream_connect(format!(
                                    "invalid TLS server name: {}",
                                    e
                                ))
                            })?;
                        let tls_stream = match tls.connector.connect(server_name, tcp_stream).await
                        {
                            Ok(stream) => stream,
                            Err(error) => {
                                if let Some((upstream_name, upstream_addr)) = &final_health_identity
                                {
                                    if let Some(group) = self.upstreams.get(upstream_name) {
                                        group.mark_unhealthy(upstream_addr);
                                    }
                                }
                                return Err(ProxyError::tls_handshake(error.to_string()));
                            }
                        };
                        if let Some((upstream_name, upstream_addr)) = &final_health_identity {
                            if let Some(group) = self.upstreams.get(upstream_name) {
                                group.mark_healthy(upstream_addr);
                            }
                        }

                        tunnel_lib::engine::relay::relay_with_initial(
                            recv,
                            send,
                            tls_stream,
                            initial_data.as_deref().unwrap_or(&[]),
                        )
                        .await
                        .map_err(|e| ProxyError::upstream_forward(e.to_string()))?;
                    }
                }
                Ok(())
            }
            PeerSpec::Http(spec) => self
                .http_connector
                .connect(spec, downstream_protocol, send, recv, initial_data)
                .await
                .map_err(|e| ProxyError::http_upstream_request(e.to_string())),
            PeerSpec::MitmH2(_) => Err(ProxyError::unsupported_protocol(
                "server egress does not support MITM peer",
            )),
        }
    }
}

/// Newtype around `Arc<ServerEgressMap>` so it can impl `UpstreamResolver` without
/// violating the orphan rule (`Arc` is a foreign type).
pub struct EgressProxy(pub std::sync::Arc<ServerEgressMap>);

impl UpstreamResolver for EgressProxy {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerSpec, ProxyError> {
        self.0.upstream_peer(context).await
    }

    async fn connect_peer(
        &self,
        peer: PeerSpec,
        downstream_protocol: Protocol,
        send: quinn::SendStream,
        recv: quinn::RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<(), ProxyError> {
        self.0
            .connect_peer(peer, downstream_protocol, send, recv, initial_data)
            .await
    }
}
