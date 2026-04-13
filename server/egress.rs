use crate::config::ServerEgressUpstream;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use tunnel_lib::proxy::core::{Context, FlowDirection, Protocol, UpstreamResolver};
use tunnel_lib::proxy::http::HttpPeer;
use tunnel_lib::proxy::peers::PeerKind;
use tunnel_lib::proxy::tcp::{TcpPeer, UpstreamScheme};
use tunnel_lib::{HttpClientParams, RuleMatchContext, RuleSet, UpstreamGroup};
type HttpsClient = Client<
    hyper_rustls::HttpsConnector<HttpConnector>,
    http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>,
>;

#[derive(Clone)]
pub struct EgressHttpEndpoint {
    connect_addr: String,
    scheme: UpstreamScheme,
}

pub struct ServerEgressMap {
    upstreams: HashMap<String, UpstreamGroup>,
    rules: RuleSet<String>,
    https_client: HttpsClient,
}
impl ServerEgressMap {
    pub fn from_config(egress: &ServerEgressUpstream, http_params: &HttpClientParams) -> Self {
        let mut upstreams = HashMap::new();
        let rules = RuleSet::new();
        for (name, upstream_def) in &egress.upstreams {
            let servers: Vec<String> = upstream_def
                .servers
                .iter()
                .map(|s| s.address.clone())
                .collect();
            upstreams.insert(name.clone(), UpstreamGroup::new(servers));
        }
        for rule in &egress.rules.vhost {
            rules.add_host_rule(&rule.match_host, rule.action_upstream.clone());
        }
        let https_client = tunnel_lib::create_https_client_with(http_params);
        Self {
            upstreams,
            rules,
            https_client,
        }
    }
    /// Look up the upstream route and selected server for the given host.
    pub fn resolve_target(&self, host: &str, protocol: Protocol) -> Option<(String, String)> {
        let upstream_name = self
            .rules
            .resolve(&RuleMatchContext::new(None, Some(host), Some(protocol)))?;
        if let Some(group) = self.upstreams.get(&upstream_name) {
            if let Some(server) = group.next() {
                debug!(
                    host = %host, upstream = %upstream_name, server = %server,
                    "matched egress rule (round-robin)"
                );
                return Some((upstream_name, server.clone()));
            }
        }
        warn!(host = %host, "no egress rule matched");
        None
    }

    async fn resolve_http_target(
        &self,
        authority: &str,
        protocol: Protocol,
    ) -> Result<Option<tunnel_lib::ResolvedHttpTarget<String, EgressHttpEndpoint>>> {
        let flow = tunnel_lib::HttpFlow::new(self);
        flow.resolve_target(
            0,
            tunnel_lib::HttpRequestContext::from_authority(Some(authority.to_string()), protocol),
        )
        .await
    }
}

#[async_trait]
impl tunnel_lib::HttpFlowResolver for &ServerEgressMap {
    type Route = String;
    type Endpoint = EgressHttpEndpoint;

    fn select_route(
        &self,
        _port: u16,
        ctx: &tunnel_lib::HttpRequestContext,
    ) -> Result<Option<tunnel_lib::RouteDecision<Self::Route>>> {
        let Some(route_host) = ctx.route_host.as_deref() else {
            return Ok(None);
        };
        let route = self
            .rules
            .resolve(&RuleMatchContext::new(None, Some(route_host), Some(ctx.protocol)));
        if route.is_none() {
            warn!(host = %route_host, "no egress rule matched");
        }
        Ok(route.map(|upstream_name| tunnel_lib::RouteDecision {
            route: upstream_name.clone(),
            route_key: Some(upstream_name),
            target_host: ctx.authority.clone(),
        }))
    }

    async fn select_endpoint(
        &self,
        _ctx: &tunnel_lib::HttpRequestContext,
        route: &tunnel_lib::RouteDecision<Self::Route>,
    ) -> Result<tunnel_lib::EndpointDecision<Self::Endpoint>> {
        let group = self
            .upstreams
            .get(&route.route)
            .ok_or_else(|| anyhow!("missing egress upstream group: {}", route.route))?;
        let upstream_addr = group
            .next()
            .cloned()
            .ok_or_else(|| anyhow!("no egress backend in upstream group: {}", route.route))?;
        let (scheme, connect_addr, _tls_host) = UpstreamScheme::from_address(&upstream_addr);
        debug!(
            upstream = %route.route,
            server = %upstream_addr,
            connect_addr = %connect_addr,
            "matched egress rule (round-robin)"
        );
        Ok(tunnel_lib::EndpointDecision {
            selected_endpoint: Some(connect_addr.clone()),
            endpoint: EgressHttpEndpoint {
                connect_addr,
                scheme,
            },
        })
    }
}

impl UpstreamResolver for ServerEgressMap {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerKind> {
        debug_assert_eq!(context.direction, FlowDirection::Egress);
        let routing = context
            .routing_info
            .as_ref()
            .ok_or_else(|| anyhow!("missing routing info in context"))?;
        let host_raw = routing
            .host
            .as_deref()
            .ok_or_else(|| anyhow!("no host in routing info"))?;
        let host = host_raw.split(':').next().unwrap_or(host_raw).to_string();
        match context.protocol {
            Protocol::WebSocket => {
                context.set_target_host(host.clone());
                let (route_key, upstream_addr) = self
                    .resolve_target(&host, context.protocol)
                    .ok_or_else(|| anyhow!("no egress route for host: {}", host))?;
                let (scheme, connect_addr_str, tls_host) = UpstreamScheme::from_address(&upstream_addr);
                let is_https = scheme.requires_tls();
                context.set_route_key(route_key);
                context.set_selected_endpoint(connect_addr_str.clone());
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
                let resolved = self
                    .resolve_http_target(host_raw, context.protocol)
                    .await?
                    .ok_or_else(|| anyhow!("no egress route for host: {}", host_raw))?;
                if let Some(target_host) = resolved.context.target_host.as_deref() {
                    context.set_target_host(target_host.to_string());
                } else {
                    context.set_target_host(host_raw.to_string());
                }
                if let Some(route_key) = resolved.context.route_key.as_deref() {
                    context.set_route_key(route_key.to_string());
                }
                if let Some(selected_endpoint) = resolved.context.selected_endpoint.as_deref() {
                    context.set_selected_endpoint(selected_endpoint.to_string());
                }
                let scheme_str = if resolved.endpoint.endpoint.scheme.requires_tls() {
                    "https"
                } else {
                    "http"
                }
                .to_string();
                Ok(PeerKind::Http(Box::new(HttpPeer {
                    client: self.https_client.clone(),
                    target_host: resolved.endpoint.endpoint.connect_addr,
                    scheme: scheme_str,
                })))
            }
            _ => Err(anyhow!("unsupported protocol for egress")),
        }
    }
}

/// Newtype around `Arc<ServerEgressMap>` so it can impl `UpstreamResolver` without
/// violating the orphan rule (`Arc` is a foreign type).
pub struct EgressProxy(pub std::sync::Arc<ServerEgressMap>);

impl UpstreamResolver for EgressProxy {
    async fn upstream_peer(&self, context: &mut Context) -> Result<PeerKind> {
        self.0.upstream_peer(context).await
    }
}
