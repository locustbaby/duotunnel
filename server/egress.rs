use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
use hyper_util::rt::TokioExecutor;
use hyper_rustls::HttpsConnectorBuilder;
use std::time::Duration;

use crate::config::ServerConfigFile;

type HttpsClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>>;

/// Upstream group with round-robin load balancing
struct UpstreamGroup {
    servers: Vec<String>,
    counter: AtomicUsize,
}

impl UpstreamGroup {
    fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            counter: AtomicUsize::new(0),
        }
    }

    fn next(&self) -> Option<&String> {
        if self.servers.is_empty() {
            return None;
        }
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % self.servers.len();
        self.servers.get(idx)
    }
}

pub struct ServerEgressMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: HashMap<String, String>,
    https_client: HttpsClient,
}

impl ServerEgressMap {
    pub fn from_config(config: &ServerConfigFile) -> Self {
        let mut upstreams = HashMap::new();
        let mut http_rules = HashMap::new();

        for (name, upstream_def) in &config.server_egress_upstream.upstreams {
            let servers: Vec<String> = upstream_def.servers.iter()
                .map(|s| s.address.clone())
                .collect();
            upstreams.insert(name.clone(), UpstreamGroup::new(servers));
        }

        for rule in &config.server_egress_upstream.rules.http {
            http_rules.insert(rule.match_host.clone(), rule.action_upstream.clone());
        }

        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let https_client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build(https);

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
                    Ok(Box::new(TlsTcpPeer {
                        target_addr,
                        tls_host: tls_host.unwrap_or_default(),
                        alpn: scheme.alpn(),
                    }))
                } else {
                    Ok(Box::new(TcpPeer { target_addr }))
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
