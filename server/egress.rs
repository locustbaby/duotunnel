use crate::config::ServerEgressUpstream;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use tunnel_lib::proxy::core::{Context, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http_connector::SharedHttpConnector;
use tunnel_lib::proxy::peers::{BasicPeerSpec, HttpPeerSpec, PeerSpec, TlsPeerSpec};
use tunnel_lib::proxy::tcp::UpstreamScheme;
use tunnel_lib::{HttpClientParams, UpstreamGroup};
pub struct ServerEgressMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: HashMap<String, String>,
    http_connector: SharedHttpConnector,
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
            // Strip port at insert time so lookup needs no split per request.
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
        }
    }
    /// Look up the upstream address for the given host (must be pre-stripped of port).
    pub fn get_upstream_address(&self, host: &str) -> Option<String> {
        if let Some(upstream_name) = self.http_rules.get(host) {
            if let Some(group) = self.upstreams.get(upstream_name) {
                if let Some(server) = group.next() {
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
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerSpec> {
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;
        let host_raw = routing
            .host
            .as_deref()
            .ok_or_else(|| anyhow!("no host in routing info"))?;
        let host = host_raw.split(':').next().unwrap_or(host_raw);
        let upstream_addr = self
            .get_upstream_address(host)
            .ok_or_else(|| anyhow!("no egress route for host: {}", host))?;
        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();
        match context.protocol {
            Protocol::WebSocket => {
                info!("WebSocket egress, using TCP forwarding");
                let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>()
                {
                    addr
                } else {
                    tokio::net::lookup_host(&connect_addr_str)
                        .await
                        .map_err(|e| anyhow!("failed to resolve: {}: {}", connect_addr_str, e))?
                        .next()
                        .ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?
                };
                let spec = BasicPeerSpec {
                    target_addr,
                    tls: is_https.then(|| TlsPeerSpec {
                        host: tls_host.unwrap_or_default(),
                        alpn: scheme.alpn(),
                    }),
                };
                Ok(PeerSpec::Tcp(spec))
            }
            Protocol::H1 | Protocol::Unknown => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: if is_https { "https" } else { "http" }.to_string(),
                    protocol: context.protocol,
                };
                Ok(PeerSpec::Http(spec))
            }
            Protocol::H2 => {
                let spec = HttpPeerSpec {
                    target_host: connect_addr_str,
                    scheme: if is_https { "https" } else { "http" }.to_string(),
                    protocol: context.protocol,
                };
                Ok(PeerSpec::Http(spec))
            }
            _ => Err(anyhow!("unsupported protocol for egress")),
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
            PeerSpec::Tcp(spec) => {
                spec.into_tcp_peer(tunnel_lib::TcpParams::default())?
                    .connect_inner(send, recv, initial_data)
                    .await
            }
            PeerSpec::Http(spec) => match spec.protocol {
                Protocol::H2 | Protocol::H1 | Protocol::Unknown => self
                    .http_connector
                    .connect(spec, send, recv, initial_data)
                    .await,
                _ => Err(anyhow!("unsupported http protocol variant")),
            },
            PeerSpec::MitmH2(_) => Err(anyhow!("server egress does not support MITM peer")),
        }
    }
}

/// Newtype around `Arc<ServerEgressMap>` so it can impl `UpstreamResolver` without
/// violating the orphan rule (`Arc` is a foreign type).
pub struct EgressProxy(pub std::sync::Arc<ServerEgressMap>);

impl UpstreamResolver for EgressProxy {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerSpec> {
        self.0.upstream_peer(context).await
    }

    async fn connect_peer(
        &self,
        peer: PeerSpec,
        send: quinn::SendStream,
        recv: quinn::RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        self.0.connect_peer(peer, send, recv, initial_data).await
    }
}
