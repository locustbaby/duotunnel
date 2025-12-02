use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Warm up connection pool for all upstream targets (HTTP, HTTPS, WSS, gRPC)
/// This is a wrapper around tunnel_lib::warmup::warmup_connection_pools
pub async fn warmup_connection_pools(
    http_client: &Client<HttpConnector, Body>,
    https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    rules: &[Rule],
    upstreams: &[Upstream],
) {
    tunnel_lib::warmup::warmup_connection_pools(http_client, https_client, rules, upstreams).await;
}

