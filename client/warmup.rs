use hyper::{Body, Client, Method, Request};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use std::str::FromStr;
use tracing::{info, debug, warn};
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Warm up connection pool for all upstream targets
pub async fn warmup_connection_pools(
    http_client: &Client<HttpConnector, Body>,
    https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    rules: &[Rule],
    upstreams: &[Upstream],
) {
    // Collect all unique upstream addresses
    let mut targets = std::collections::HashSet::new();
    
    for rule in rules {
        if rule.r#type != "http" {
            continue; // Only warmup HTTP/HTTPS targets
        }
        
        // Resolve upstream address
        if let Some(upstream) = upstreams.iter().find(|u| u.name == rule.action_proxy_pass) {
            for server in &upstream.servers {
                targets.insert((server.address.clone(), server.address.starts_with("https://")));
            }
        } else {
            // Direct address
            let is_ssl = rule.action_proxy_pass.starts_with("https://");
            targets.insert((rule.action_proxy_pass.clone(), is_ssl));
        }
    }
    
    if targets.is_empty() {
        debug!("No upstream targets to warmup");
        return;
    }
    
    info!("Warming up connection pools for {} upstream targets...", targets.len());
    
    // Warmup each target asynchronously
    let mut warmup_tasks = Vec::new();
    
    for (target_addr, is_ssl) in targets {
        let http_clone = http_client.clone();
        let https_clone = https_client.clone();
        
        let task = tokio::spawn(async move {
            warmup_single_target(&http_clone, &https_clone, &target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    
    // Wait for all warmup tasks (with timeout)
    let timeout = tokio::time::Duration::from_secs(10);
    let warmup_result = tokio::time::timeout(timeout, futures::future::join_all(warmup_tasks)).await;
    
    match warmup_result {
        Ok(_) => {
            info!("Connection pool warmup completed for all targets");
        }
        Err(_) => {
            warn!("Connection pool warmup timed out after {:?}", timeout);
        }
    }
}

/// Warmup a single target by sending a lightweight request
async fn warmup_single_target(
    http_client: &Client<HttpConnector, Body>,
    https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    target_addr: &str,
    is_ssl: bool,
) {
    // Parse URI
    let uri = match hyper::Uri::from_str(target_addr) {
        Ok(uri) => uri,
        Err(e) => {
            warn!("Invalid target URI for warmup: {} ({})", target_addr, e);
            return;
        }
    };
    
    // Create a lightweight HEAD request (or GET if HEAD is not supported)
    let request = match Request::builder()
        .method(Method::HEAD)
        .uri(uri.clone())
        .body(Body::empty())
    {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to create warmup request for {}: {}", target_addr, e);
            return;
        }
    };
    
    // Send request using appropriate client
    let result = if is_ssl {
        https_client.request(request).await
    } else {
        http_client.request(request).await
    };
    
    match result {
        Ok(response) => {
            // Consume the response to complete the connection
            let status = response.status();
            let body = response.into_body();
            // Read body to ensure connection is established
            use hyper::body::HttpBody;
            let mut body_stream = body;
            while let Some(chunk_result) = body_stream.data().await {
                match chunk_result {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("Error reading warmup response body for {}: {}", target_addr, e);
                        break;
                    }
                }
            }
            debug!("Warmed up connection pool for {} (status: {})", target_addr, status);
        }
        Err(e) => {
            // Don't fail on warmup errors - connection will be established on first real request
            debug!("Warmup request failed for {} (will connect on first real request): {}", target_addr, e);
        }
    }
}

