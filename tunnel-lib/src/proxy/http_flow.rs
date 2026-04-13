use super::core::Protocol;
use super::{RuleMatchContext, RuleSet};
use anyhow::Result;
use async_trait::async_trait;
use hyper::Request;

pub struct HttpRequestContext {
    pub authority: Option<String>,
    pub route_host: Option<String>,
    pub protocol: Protocol,
    pub route_key: Option<String>,
    pub selected_endpoint: Option<String>,
    pub target_host: Option<String>,
}

impl HttpRequestContext {
    pub fn from_authority(authority: Option<String>, protocol: Protocol) -> Self {
        let route_host = authority.as_deref().map(normalize_route_host);
        Self {
            authority,
            route_host,
            protocol,
            route_key: None,
            selected_endpoint: None,
            target_host: None,
        }
    }

    pub fn from_request<B>(req: &Request<B>, protocol: Protocol) -> Self {
        let authority = authority_from_request(req);
        Self::from_authority(authority, protocol)
    }

    pub fn set_authority(&mut self, authority: impl Into<String>) {
        let authority = authority.into();
        self.route_host = Some(normalize_route_host(&authority));
        self.authority = Some(authority);
    }

    pub fn set_route_key(&mut self, value: impl Into<String>) {
        self.route_key = Some(value.into());
    }

    pub fn set_selected_endpoint(&mut self, value: impl Into<String>) {
        self.selected_endpoint = Some(value.into());
    }

    pub fn set_target_host(&mut self, value: impl Into<String>) {
        self.target_host = Some(value.into());
    }
}

#[derive(Clone)]
pub struct RouteDecision<R> {
    pub route: R,
    pub route_key: Option<String>,
    pub target_host: Option<String>,
}

#[derive(Clone)]
pub struct EndpointDecision<E> {
    pub endpoint: E,
    pub selected_endpoint: Option<String>,
}

pub struct ResolvedHttpTarget<R, E> {
    pub context: HttpRequestContext,
    pub route: RouteDecision<R>,
    pub endpoint: EndpointDecision<E>,
}

#[derive(Clone)]
pub struct FixedHttpFlowResolver {
    target_host: String,
    route_key: Option<String>,
    selected_endpoint: Option<String>,
}

impl FixedHttpFlowResolver {
    pub fn new(target_host: impl Into<String>) -> Self {
        let target_host = target_host.into();
        Self {
            route_key: Some(target_host.clone()),
            selected_endpoint: Some(target_host.clone()),
            target_host,
        }
    }

    pub fn with_route_key(mut self, route_key: impl Into<String>) -> Self {
        self.route_key = Some(route_key.into());
        self
    }

    pub fn with_selected_endpoint(mut self, selected_endpoint: impl Into<String>) -> Self {
        self.selected_endpoint = Some(selected_endpoint.into());
        self
    }
}

#[async_trait]
pub trait HttpFlowResolver: Send + Sync {
    type Route: Clone + Send + Sync;
    type Endpoint: Clone + Send + Sync;

    async fn request_filter(&self, _ctx: &mut HttpRequestContext) -> Result<()> {
        Ok(())
    }

    fn select_route(
        &self,
        port: u16,
        ctx: &HttpRequestContext,
    ) -> Result<Option<RouteDecision<Self::Route>>>;

    async fn select_endpoint(
        &self,
        ctx: &HttpRequestContext,
        route: &RouteDecision<Self::Route>,
    ) -> Result<EndpointDecision<Self::Endpoint>>;

    async fn connected_to_upstream(
        &self,
        _ctx: &HttpRequestContext,
        _route: &RouteDecision<Self::Route>,
        _endpoint: &EndpointDecision<Self::Endpoint>,
    ) -> Result<()> {
        Ok(())
    }

    async fn response_filter(
        &self,
        _ctx: &HttpRequestContext,
        _parts: &mut http::response::Parts,
    ) -> Result<()> {
        Ok(())
    }

    fn fail_to_connect(&self, err: anyhow::Error) -> anyhow::Error {
        err
    }
}

#[async_trait]
impl HttpFlowResolver for FixedHttpFlowResolver {
    type Route = ();
    type Endpoint = ();

    fn select_route(
        &self,
        _port: u16,
        _ctx: &HttpRequestContext,
    ) -> Result<Option<RouteDecision<Self::Route>>> {
        Ok(Some(RouteDecision {
            route: (),
            route_key: self.route_key.clone(),
            target_host: Some(self.target_host.clone()),
        }))
    }

    async fn select_endpoint(
        &self,
        _ctx: &HttpRequestContext,
        _route: &RouteDecision<Self::Route>,
    ) -> Result<EndpointDecision<Self::Endpoint>> {
        Ok(EndpointDecision {
            endpoint: (),
            selected_endpoint: self.selected_endpoint.clone(),
        })
    }
}

pub struct HttpFlow<R: HttpFlowResolver> {
    resolver: R,
}

impl<R: HttpFlowResolver> HttpFlow<R> {
    pub fn new(resolver: R) -> Self {
        Self { resolver }
    }

    pub fn resolver(&self) -> &R {
        &self.resolver
    }

    pub async fn resolve_target(
        &self,
        port: u16,
        mut ctx: HttpRequestContext,
    ) -> Result<Option<ResolvedHttpTarget<R::Route, R::Endpoint>>> {
        self.resolver.request_filter(&mut ctx).await?;
        let Some(route) = self.resolver.select_route(port, &ctx)? else {
            return Ok(None);
        };
        if let Some(route_key) = route.route_key.as_deref() {
            ctx.set_route_key(route_key.to_string());
        }
        if let Some(target_host) = route.target_host.as_deref() {
            ctx.set_target_host(target_host.to_string());
        }
        let endpoint = match self.resolver.select_endpoint(&ctx, &route).await {
            Ok(endpoint) => endpoint,
            Err(err) => return Err(self.resolver.fail_to_connect(err)),
        };
        if let Some(selected_endpoint) = endpoint.selected_endpoint.as_deref() {
            ctx.set_selected_endpoint(selected_endpoint.to_string());
        }
        self.resolver
            .connected_to_upstream(&ctx, &route, &endpoint)
            .await?;
        Ok(Some(ResolvedHttpTarget {
            context: ctx,
            route,
            endpoint,
        }))
    }

    pub fn resolve_target_for_request<'a, B>(
        &'a self,
        port: u16,
        req: &Request<B>,
        protocol: Protocol,
    ) -> impl std::future::Future<Output = Result<Option<ResolvedHttpTarget<R::Route, R::Endpoint>>>> + Send + 'a {
        let ctx = HttpRequestContext::from_request(req, protocol);
        async move { self.resolve_target(port, ctx).await }
    }

    pub async fn filter_response_parts(
        &self,
        target: &ResolvedHttpTarget<R::Route, R::Endpoint>,
        parts: &mut http::response::Parts,
    ) -> Result<()> {
        self.resolver.response_filter(&target.context, parts).await
    }
}

pub fn authority_from_request<B>(req: &Request<B>) -> Option<String> {
    req.uri().authority().map(|a| a.to_string()).or_else(|| {
        req.headers()
            .get(hyper::header::HOST)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    })
}

pub fn normalize_route_host(authority: &str) -> String {
    authority
        .split(':')
        .next()
        .unwrap_or(authority)
        .to_ascii_lowercase()
}

pub fn resolve_http_route<T: Clone + Send + Sync>(
    rules: &RuleSet<T>,
    port: u16,
    ctx: &HttpRequestContext,
) -> Option<T> {
    rules.resolve(&RuleMatchContext::new(
        Some(port),
        ctx.route_host.as_deref(),
        Some(ctx.protocol),
    ))
}

pub fn rewrite_request_authority(parts: &mut http::request::Parts, target_host: &str) {
    let mut uri_parts = parts.uri.clone().into_parts();
    if let Ok(authority) = target_host.parse() {
        uri_parts.authority = Some(authority);
    }
    parts.uri = hyper::Uri::from_parts(uri_parts).unwrap_or(parts.uri.clone());
    if let Ok(host_value) = target_host.parse() {
        parts.headers.insert(hyper::header::HOST, host_value);
    }
}

pub fn rewrite_request_upstream(
    parts: &mut http::request::Parts,
    scheme: &str,
    target_host: &str,
) {
    let target_uri: hyper::Uri = format!(
        "{}://{}{}",
        scheme,
        target_host,
        parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    )
    .parse()
    .unwrap();
    parts.uri = target_uri;
    if let Ok(host_value) = target_host.parse() {
        parts.headers.insert(hyper::header::HOST, host_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_route_host_without_port() {
        assert_eq!(normalize_route_host("Example.COM:8443"), "example.com");
    }

    #[test]
    fn resolves_http_route_from_authority() {
        let mut rules = RuleSet::new();
        rules.add_port_host_rule(443, "api.example.com", "upstream-a".to_string());
        let ctx = HttpRequestContext::from_authority(
            Some("api.example.com:443".to_string()),
            Protocol::H2,
        );
        let resolved = resolve_http_route(&rules, 443, &ctx);
        assert_eq!(resolved.as_deref(), Some("upstream-a"));
    }

    #[test]
    fn rewrites_request_upstream() {
        let mut parts = Request::builder()
            .uri("/v1/ping?x=1")
            .body(())
            .unwrap()
            .into_parts()
            .0;
        rewrite_request_upstream(&mut parts, "https", "api.example.com:8443");
        assert_eq!(
            parts.uri.to_string(),
            "https://api.example.com:8443/v1/ping?x=1"
        );
        assert_eq!(
            parts.headers
                .get(hyper::header::HOST)
                .and_then(|v| v.to_str().ok()),
            Some("api.example.com:8443")
        );
    }

    #[tokio::test]
    async fn fixed_resolver_sets_target_metadata() {
        let flow = HttpFlow::new(
            FixedHttpFlowResolver::new("backend.internal:9443")
                .with_route_key("backend-route")
                .with_selected_endpoint("backend.internal:9443"),
        );
        let req = Request::builder()
            .uri("https://api.example.com/v1/ping")
            .body(())
            .unwrap();
        let resolved = flow
            .resolve_target_for_request(0, &req, Protocol::H2)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved.context.route_key.as_deref(), Some("backend-route"));
        assert_eq!(
            resolved.context.selected_endpoint.as_deref(),
            Some("backend.internal:9443")
        );
        assert_eq!(
            resolved.context.target_host.as_deref(),
            Some("backend.internal:9443")
        );
    }
}
