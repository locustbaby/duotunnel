use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use bytes::Bytes;
use tracing::{debug, info, warn};
use tunnel_lib::proxy::core::{ProxyApp, Context, Protocol};
use tunnel_lib::proxy::peers::UpstreamPeer;
use tunnel_lib::proxy::tcp::{TcpPeer, TlsTcpPeer, UpstreamScheme};
use tunnel_lib::proxy::http::HttpPeer;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;

use crate::config::ServerEgressUpstream;
use tunnel_lib::{HttpClientParams, UpstreamGroup};

type HttpsClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>>;

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
            let servers: Vec<String> = upstream_def.servers.iter()
                .map(|s| s.address.clone())
                .collect();
            upstreams.insert(name.clone(), UpstreamGroup::new(servers));
        }

        for rule in &egress.rules.vhost {
            http_rules.insert(rule.match_host.clone(), rule.action_upstream.clone());
        }

        let https_client = tunnel_lib::create_https_client_with(http_params);

        Self { upstreams, http_rules, https_client }
    }

    pub fn get_upstream_address(&self, host: &str) -> Option<String> {
        let host_clean = host.split(':').next().unwrap_or(host);

        if let Some(upstream_name) = self.http_rules.get(host_clean) {
            if let Some(group) = self.upstreams.get(upstream_name) {
                if let Some(server) = group.next() {
                    debug!(host = %host_clean, upstream = %upstream_name, server = %server, "matched egress rule (round-robin)");
                    return Some(server.clone());
                }
            }
        }

        warn!(host = %host_clean, "no egress rule matched");
        None
    }
}

pub struct ServerEgressApp {
    map: Arc<ServerEgressMap>,
}

impl ServerEgressApp {
    pub fn new(map: Arc<ServerEgressMap>) -> Self {
        Self { map }
    }
}

#[async_trait]
impl ProxyApp for ServerEgressApp {
    async fn upstream_peer(&self, context: &mut Context) -> Result<Box<dyn UpstreamPeer>> {
        let routing = context.routing_info.as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;

        let host = routing.host.as_deref()
            .ok_or_else(|| anyhow!("no host in routing info"))?;

        let upstream_addr = self.map.get_upstream_address(host)
            .ok_or_else(|| anyhow!("no egress route for host: {}", host))?;

        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();

        match context.protocol {
            Protocol::WebSocket => {
                info!("WebSocket egress, using TCP forwarding");

                let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>() {
                    addr
                } else {
                    let mut addrs = tokio::net::lookup_host(&connect_addr_str).await
                        .map_err(|e| anyhow!("failed to resolve: {}: {}", connect_addr_str, e))?;
                    addrs.next().ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?
                };

                if is_https {
                    Ok(Box::new(TlsTcpPeer::new(
                        target_addr,
                        tls_host.unwrap_or_default(),
                        scheme.alpn(),
                    )?))
                } else {
                    Ok(Box::new(TcpPeer {
                        target_addr,
                        tcp_params: tunnel_lib::TcpParams::default(),
                    }))
                }
            },
            Protocol::H1 | Protocol::H2 | Protocol::Unknown => {
                let scheme_str = if is_https { "https".to_string() } else { "http".to_string() };
                Ok(Box::new(HttpPeer {
                    client: self.map.https_client.clone(),
                    target_host: connect_addr_str,
                    scheme: scheme_str,
                }))
            },
            _ => Err(anyhow!("unsupported protocol for egress")),
        }
    }
}
