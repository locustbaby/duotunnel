use crate::config::ServerEgressUpstream;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use tunnel_lib::proxy::core::{Context, Protocol, ProxyApp};
use tunnel_lib::proxy::http::HttpPeer;
use tunnel_lib::proxy::peers::PeerKind;
use tunnel_lib::proxy::tcp::{TcpPeer, UpstreamScheme};
use tunnel_lib::{HttpClientParams, UpstreamGroup};
type HttpsClient = Client<
    hyper_rustls::HttpsConnector<HttpConnector>,
    http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>,
>;
pub struct ServerEgressMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: HashMap<String, String>,
    https_client: HttpsClient,
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
        Self {
            upstreams,
            http_rules,
            https_client,
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
impl ProxyApp for ServerEgressMap {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerKind> {
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
                if is_https {
                    Ok(PeerKind::Tcp(TcpPeer::new_tls(
                        target_addr,
                        tls_host.unwrap_or_default(),
                        scheme.alpn(),
                    )?))
                } else {
                    Ok(PeerKind::Tcp(TcpPeer::new(
                        target_addr,
                        tunnel_lib::TcpParams::default(),
                    )))
                }
            }
            Protocol::H1 | Protocol::H2 | Protocol::Unknown => {
                let scheme_str = if is_https { "https" } else { "http" }.to_string();
                Ok(PeerKind::Http(Box::new(HttpPeer {
                    client: self.https_client.clone(),
                    target_host: connect_addr_str,
                    scheme: scheme_str,
                })))
            }
            _ => Err(anyhow!("unsupported protocol for egress")),
        }
    }
}

/// Newtype around `Arc<ServerEgressMap>` so it can impl `ProxyApp` without
/// violating the orphan rule (`Arc` is a foreign type).
pub struct EgressProxy(pub std::sync::Arc<ServerEgressMap>);

impl ProxyApp for EgressProxy {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerKind> {
        self.0.upstream_peer(context).await
    }
}
