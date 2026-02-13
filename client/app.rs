use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use anyhow::{Result, anyhow, Context};
use async_trait::async_trait;
use bytes::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use hyper_rustls::HttpsConnectorBuilder;
use tracing::{debug, info};
use tunnel_lib::ClientConfig;
use tunnel_lib::proxy::core::{ProxyApp, Context as ProxyContext, Protocol};
use tunnel_lib::proxy::peers::UpstreamPeer;
use tunnel_lib::proxy::http::HttpPeer;

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

    fn first(&self) -> Option<&String> {
        self.servers.first()
    }
}

pub struct LocalProxyMap {
    upstreams: HashMap<String, UpstreamGroup>,
    http_rules: HashMap<String, String>,
    pub https_client: HttpsClient,
}

impl LocalProxyMap {
    pub fn from_config(config: &ClientConfig) -> Self {
        let mut upstreams = HashMap::new();
        let mut http_rules = HashMap::new();

        for upstream in &config.upstreams {
            let servers: Vec<String> = upstream.servers.iter()
                .map(|s| s.address.clone())
                .collect();
            upstreams.insert(upstream.name.clone(), UpstreamGroup::new(servers));
        }

        for rule in &config.rules {
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

    pub fn get_local_address(&self, proxy_name: &str, host: Option<&str>, is_websocket: bool) -> Option<String> {
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

        // Fallback: match by proxy name
        for (name, group) in &self.upstreams {
            if name == proxy_name || name.contains(proxy_name) {
                return group.first().cloned();
            }
        }

        // Last resort: return first available server
        self.upstreams.values().next()
            .and_then(|group| group.first().cloned())
    }
}

pub struct ClientApp {
    map: Arc<LocalProxyMap>,
}

impl ClientApp {
    pub fn new(map: Arc<LocalProxyMap>) -> Self {
        Self { map }
    }
}

#[async_trait]
impl ProxyApp for ClientApp {
    async fn upstream_peer(&self, context: &mut ProxyContext) -> Result<Box<dyn UpstreamPeer>> {
        let routing = context.routing_info.as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;

        let is_websocket = matches!(context.protocol, Protocol::WebSocket);
        
        let upstream_addr = self.map.get_local_address(
            &routing.proxy_name,
            routing.host.as_deref(),
            is_websocket,
        ).ok_or_else(|| anyhow::anyhow!("no upstream address for proxy: {}", routing.proxy_name))?;

        use tunnel_lib::proxy::tcp::UpstreamScheme;

        let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
        let is_https = scheme.requires_tls();

        match context.protocol {

            Protocol::H1 => {
                let scheme = if is_https { "https".to_string() } else { "http".to_string() };
                Ok(Box::new(HttpPeer {
                    client: self.map.https_client.clone(),
                    target_host: connect_addr_str,
                    scheme,
                }))
            },
            Protocol::H2 => {
                let scheme = if is_https { "https".to_string() } else { "http".to_string() };
                Ok(Box::new(tunnel_lib::proxy::h2::H2Peer {
                    target_host: connect_addr_str,
                    scheme,
                    client: self.map.https_client.clone(),
                }))
            },
            Protocol::WebSocket => {
                 info!("WebSocket protocol detected, using TCP relay");
                 let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>() {
                     addr
                 } else {
                     let mut addrs = tokio::net::lookup_host(&connect_addr_str).await
                         .map_err(|e| anyhow!("failed to resolve upstream address {}: {}", connect_addr_str, e))?;
                     addrs.next().ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?
                 };

                 if is_https {
                      Ok(Box::new(tunnel_lib::proxy::tcp::TlsTcpPeer {
                          target_addr,
                          tls_host: tls_host.ok_or_else(|| anyhow!("TLS host required for WSS"))?,
                          alpn: None,
                      }))
                 } else {
                      Ok(Box::new(tunnel_lib::proxy::tcp::TcpPeer {
                          target_addr,
                      }))
                 }
            },
            Protocol::Tcp => {
                 info!("TCP protocol detected (opaque TLS)");
                 
                 let target_addr = if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>() {
                     addr
                 } else {
                     let mut addrs = tokio::net::lookup_host(&connect_addr_str).await
                         .map_err(|e| anyhow!("failed to resolve upstream address {}: {}", connect_addr_str, e))?;
                     addrs.next().ok_or_else(|| anyhow!("no resolved IP for {}", connect_addr_str))?
                 };

                 // If upstream is HTTPS/WSS, we should terminate the ingress TLS (if any) and re-encrypt needed.
                 // For Protocol::Tcp, the ingress IS TLS (with wrong SNI).
                 // So we act as a MITM: Accept ingress TLS -> Plaintext -> Connect egress TLS (correct SNI).
                 if is_https {
                     let host_for_cert = tls_host.as_deref().unwrap_or("localhost");
                     info!("Terminating ingress TLS to fix SNI for upstream {}", host_for_cert);

                     // 1. Generate self-signed cert for the MITM (cached by hostname)
                     let (certs, key) = tunnel_lib::infra::pki::generate_self_signed_cert_for_host(host_for_cert)?;
                     let server_config = rustls::ServerConfig::builder()
                        .with_no_client_auth()
                        .with_single_cert(certs, key)?;
                     let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(server_config));

                     struct MitmH2Peer {
                         acceptor: tokio_rustls::TlsAcceptor,
                         target_addr: std::net::SocketAddr,
                         tls_host: String,
                         https_client: HttpsClient,
                     }

                     #[async_trait::async_trait]
                     impl tunnel_lib::proxy::peers::UpstreamPeer for MitmH2Peer {
                         async fn connect(
                             &self, 
                             send: quinn::SendStream, 
                             recv: quinn::RecvStream, 
                             initial_data: Option<bytes::Bytes>
                         ) -> anyhow::Result<()> {
                             use crate::utils::{QuinnStream, PrefixedReadWrite};
                             use hyper::server::conn::http2::Builder as H2Builder;
                             use hyper::service::service_fn;
                             use hyper::{Request, Response};
                             use http_body_util::{BodyExt, Full};
                             use hyper_util::rt::TokioIo;

                             let stream = QuinnStream { send, recv };
                             let stream = if let Some(init) = initial_data {
                                 PrefixedReadWrite::new(stream, init)
                             } else {
                                 PrefixedReadWrite::new(stream, bytes::Bytes::new())
                             };

                             let accepted_stream = self.acceptor.accept(stream).await
                                 .context("failed to accept ingress TLS for MITM")?;
                             
                             info!("MITM H2: TLS handshake accepted, starting H2 server for {}", self.tls_host);

                             let target_host = self.tls_host.clone();
                             let client = self.https_client.clone();

                             let service = service_fn(move |mut req: Request<hyper::body::Incoming>| {
                                 let target_host = target_host.clone();
                                 let client = client.clone();
                                 async move {
                                     let (mut parts, body) = req.into_parts();
                                     
                                     let mut uri_parts = parts.uri.clone().into_parts();
                                     uri_parts.scheme = Some(hyper::http::uri::Scheme::HTTPS);
                                     uri_parts.authority = Some(target_host.parse().unwrap());
                                     parts.uri = hyper::Uri::from_parts(uri_parts).unwrap();
                                     
                                     parts.headers.insert(hyper::header::HOST, target_host.parse().unwrap());
                                     
                                     debug!("MITM H2: forwarding request to {} {}", parts.method, parts.uri);
                                     
                                     let boxed_body = body.map_err(|e| std::io::Error::other(e)).boxed_unsync();
                                     let upstream_req = Request::from_parts(parts, boxed_body);
                                     
                                     match client.request(upstream_req).await {
                                         Ok(resp) => {
                                             let (parts, body) = resp.into_parts();
                                             let boxed = body.map_err(|e| std::io::Error::other(e)).boxed_unsync();
                                             Ok::<_, hyper::Error>(Response::from_parts(parts, boxed))
                                         }
                                         Err(e) => {
                                             tracing::error!("MITM H2: upstream request failed: {}", e);
                                             Ok(Response::builder()
                                                 .status(502)
                                                 .body(Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                                                 .unwrap())
                                         }
                                     }
                                 }
                             });

                             let io = TokioIo::new(accepted_stream);
                             H2Builder::new(hyper_util::rt::TokioExecutor::new())
                                 .serve_connection(io, service)
                                 .await
                                 .map_err(|e| anyhow!("H2 connection error: {}", e))?;
                             
                             Ok(())
                         }
                     }
                     
                     Ok(Box::new(MitmH2Peer {
                         acceptor,
                         target_addr,
                         tls_host: tls_host.unwrap_or_else(|| "".to_string()),
                         https_client: self.map.https_client.clone(),
                     }))
                 } else {
                     // Always use plain TCPPeer for non-HTTPS upstream
                     Ok(Box::new(tunnel_lib::proxy::tcp::TcpPeer {
                         target_addr,
                     }))
                 }
            },
            Protocol::Unknown => {
                 let scheme = if is_https { "https".to_string() } else { "http".to_string() };
                 Ok(Box::new(HttpPeer {
                    client: self.map.https_client.clone(),
                    target_host: connect_addr_str,
                    scheme,
                }))
            },
            _ => Err(anyhow!("unsupported protocol")),
        }
    }
}
