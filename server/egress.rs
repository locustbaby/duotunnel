use crate::config::ServerEgressUpstream;
use bytes::Bytes;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use tunnel_lib::proxy::core::{Context, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http_connector::SharedHttpConnector;
use tunnel_lib::proxy::peers::{BasicPeerSpec, HttpPeerSpec, PeerSpec, TlsPeerSpec};
use tunnel_lib::proxy::tcp::UpstreamScheme;
use tunnel_lib::{HttpClientParams, ProxyError, UpstreamGroup};
pub struct ServerEgressMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: HashMap<String, String>,
    http_connector: SharedHttpConnector,
    dns_cache: tunnel_lib::EgressDnsCache,
}
impl ServerEgressMap {
    pub fn from_config(egress: &ServerEgressUpstream, http_params: &HttpClientParams) -> Self {
        let mut upstreams = HashMap::new();
        let mut http_rules = HashMap::new();
        for (name, upstream_def) in &egress.upstreams {
            let servers: Vec<String> = upstream_def
                .servers
                .iter()
                .map(|s| s.address.clone())
                .collect();
            upstreams.insert(name.clone(), UpstreamGroup::new(servers));
        }
        for rule in &egress.rules.vhost {
            let host_key = rule
                .match_host
                .split(':')
                .next()
                .unwrap_or(&rule.match_host)
                .to_string();
            http_rules.insert(host_key, rule.action_upstream.clone());
        }
        let https_client = tunnel_lib::create_https_client_with(http_params);
        let h2c_client = tunnel_lib::create_h2c_client_with(http_params);
        let http_connector =
            tunnel_lib::proxy::http_connector::HttpConnector::new(https_client, h2c_client);
        Self {
            upstreams,
            http_rules,
            http_connector,
            dns_cache: tunnel_lib::EgressDnsCache::new(std::time::Duration::from_secs(30)),
        }
    }
    pub fn get_upstream_address(&self, host: &str) -> Option<String> {
        if let Some(upstream_name) = self.http_rules.get(host) {
            if let Some(group) = self.upstreams.get(upstream_name) {
                if let Some(server) = group.next_healthy() {
                    debug!(
                        host = % host, upstream = % upstream_name, server = %
                        server, "matched egress rule (round-robin)"
                    );
                    return Some(server.clone());
                }
            }
        }
        warn!(host = % host, "no egress rule matched");
        None
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
        let host = host_raw.split(':').next().unwrap_or(host_raw);
        let upstream_name = self.http_rules.get(host).cloned().ok_or_else(|| {
            ProxyError::route_not_found(format!("host={host}"))
        })?;
        let group = self.upstreams.get(&upstream_name).ok_or_else(|| {
            ProxyError::route_not_found(format!("upstream_group={upstream_name}"))
        })?;
        let upstream_addr = group.next_healthy().ok_or_else(|| {
            ProxyError::route_not_found(format!("no healthy backends for host={host}"))
        })?.clone();
        
        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();
        match context.protocol {
            Protocol::WebSocket => {
                info!("WebSocket egress, using TCP forwarding");
                let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>()
                {
                    addr
                } else {
                    let mut parts = connect_addr_str.rsplitn(2, ':');
                    let port_str = parts.next().ok_or_else(|| {
                        ProxyError::resolve_upstream(format!("missing port in {}", connect_addr_str))
                    })?;
                    let host_str = parts.next().ok_or_else(|| {
                        ProxyError::resolve_upstream(format!("missing host in {}", connect_addr_str))
                    })?;
                    let port = port_str.parse::<u16>().map_err(|_| {
                        ProxyError::resolve_upstream(format!("invalid port {} in {}", port_str, connect_addr_str))
                    })?;
                    self.dns_cache.resolve(host_str, port)
                        .await
                        .map_err(|e| {
                            ProxyError::resolve_upstream(format!("{connect_addr_str}: {e}"))
                        })?
                };
                let spec = BasicPeerSpec {
                    target_addr,
                    tls: is_https.then(|| TlsPeerSpec {
                        host: tls_host.unwrap_or_default(),
                        alpn: scheme.alpn(),
                    }),
                    upstream_name: Some(upstream_name),
                    upstream_addr_str: Some(upstream_addr),
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
                let (tcp_stream, final_tls) = if let (Some(upstream_name), Some(failed_addr)) = (&spec.upstream_name, &spec.upstream_addr_str) {
                    let group = self.upstreams.get(upstream_name).ok_or_else(|| {
                        ProxyError::route_not_found(format!("upstream={upstream_name}"))
                    })?;
                    let mut last_err = None;
                    let mut current_failed = failed_addr.clone();
                    let mut current_spec = spec.clone();
                    
                    let mut connected = None;
                    for _ in 0..group.servers.len().max(1) {
                        let tcp_peer = current_spec.clone()
                            .into_tcp_peer(tunnel_lib::TcpParams::default())
                            .map_err(|e| ProxyError::upstream_connect(e.to_string()))?;
                        
                        match tokio::net::TcpStream::connect(tcp_peer.target_addr).await {
                            Ok(stream) => {
                                group.mark_healthy(&current_failed);
                                connected = Some((stream, tcp_peer.tls));
                                break;
                            }
                            Err(e) => {
                                warn!(server = %current_failed, error = %e, "connection to upstream failed, marking unhealthy");
                                group.mark_unhealthy(&current_failed);
                                last_err = Some(e);
                                
                                if let Some(next_addr) = group.next_healthy() {
                                    current_failed = next_addr.clone();
                                    let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&current_failed);
                                    let is_https = scheme.requires_tls();
                                    
                                    match if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>() {
                                        Ok(addr)
                                    } else {
                                        let mut parts = connect_addr_str.rsplitn(2, ':');
                                        let port_str = parts.next().ok_or_else(|| {
                                            ProxyError::resolve_upstream(format!("missing port in {}", connect_addr_str))
                                        })?;
                                        let host_str = parts.next().ok_or_else(|| {
                                            ProxyError::resolve_upstream(format!("missing host in {}", connect_addr_str))
                                        })?;
                                        let port = port_str.parse::<u16>().map_err(|_| {
                                            ProxyError::resolve_upstream(format!("invalid port {} in {}", port_str, connect_addr_str))
                                        })?;
                                        self.dns_cache.resolve(host_str, port)
                                            .await
                                            .map_err(|e| {
                                                ProxyError::resolve_upstream(format!("{connect_addr_str}: {e}"))
                                            })
                                    } {
                                        Ok(addr) => {
                                            current_spec.target_addr = addr;
                                            current_spec.tls = is_https.then(|| TlsPeerSpec {
                                                host: tls_host.unwrap_or_default(),
                                                alpn: scheme.alpn(),
                                            });
                                            current_spec.upstream_addr_str = Some(current_failed.clone());
                                        }
                                        Err(resolve_err) => {
                                            last_err = Some(std::io::Error::other(resolve_err.to_string()));
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    
                    let (stream, tls) = connected.ok_or_else(|| {
                        ProxyError::upstream_connect(last_err.map(|e| e.to_string()).unwrap_or_default())
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
                tcp_params.apply(&tcp_stream).map_err(|e| ProxyError::upstream_connect(e.to_string()))?;
                
                match final_tls {
                    None => {
                        tunnel_lib::engine::bridge::relay_with_first_data(recv, send, tcp_stream, initial_data.as_deref())
                            .await
                            .map_err(|e| ProxyError::upstream_forward(e.to_string()))?;
                    }
                    Some(tls) => {
                        let server_name = rustls::pki_types::ServerName::try_from(tls.host.clone())
                            .map_err(|e| ProxyError::upstream_connect(format!("invalid TLS server name: {}", e)))?;
                        let tls_stream = tls.connector.connect(server_name, tcp_stream)
                            .await
                            .map_err(|e| ProxyError::tls_handshake(e.to_string()))?;
                        
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
