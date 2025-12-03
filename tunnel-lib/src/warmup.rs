#[cfg(feature = "warmup")]
use std::str::FromStr;
#[cfg(feature = "warmup")]
use tracing::{info, debug, warn};
#[cfg(feature = "warmup")]
use crate::proto::tunnel::{Rule, Upstream};

#[cfg(feature = "warmup")]
/// Warm up connection pool for all upstream targets (HTTP, HTTPS, WSS, gRPC)
/// 
/// This function accepts a unified Hyper client for HTTP/HTTPS and handles WSS/gRPC internally
pub async fn warmup_connection_pools(
    client: &hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    rules: &[Rule],
    upstreams: &[Upstream],
) {
    
    // Collect all unique upstream addresses by protocol type
    let mut http_targets = std::collections::HashSet::new();
    let mut wss_targets = std::collections::HashSet::new();
    let mut grpc_targets = std::collections::HashSet::new();
    
    for rule in rules {
        // Resolve upstream address
        let (target_addr, is_ssl) = if let Some(upstream) = upstreams.iter().find(|u| u.name == rule.action_proxy_pass) {
            // Get first server for warmup (or all servers if needed)
            if upstream.servers.is_empty() {
                continue;
            }
            let server = &upstream.servers[0];
            (server.address.clone(), server.address.starts_with("https://"))
        } else {
            // Direct address
            let is_ssl = rule.action_proxy_pass.starts_with("https://");
            (rule.action_proxy_pass.clone(), is_ssl)
        };
        
        // Categorize by protocol type
        match rule.r#type.as_str() {
            "http" => {
                http_targets.insert((target_addr, is_ssl));
            }
            "wss" => {
                wss_targets.insert((target_addr, is_ssl));
            }
            "grpc" => {
                grpc_targets.insert((target_addr, is_ssl));
            }
            _ => {
                debug!("Skipping warmup for unknown protocol type: {}", rule.r#type);
            }
        }
    }
    
    let total_targets = http_targets.len() + wss_targets.len() + grpc_targets.len();
    if total_targets == 0 {
        debug!("No upstream targets to warmup");
        return;
    }
    
    info!("Warming up connection pools: {} HTTP, {} WSS, {} gRPC targets...", 
        http_targets.len(), wss_targets.len(), grpc_targets.len());
    
    // Warmup each protocol type asynchronously
    let mut warmup_tasks = Vec::new();
    
    // Warmup HTTP/HTTPS targets
    for (target_addr, is_ssl) in http_targets {
        let client_clone = client.clone();
        let target_addr = target_addr.clone();
        
        let task = tokio::spawn(async move {
            warmup_http_target(&client_clone, &target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    
    // Warmup WSS targets
    for (target_addr, is_ssl) in wss_targets {
        let client_clone = client.clone();
        let target_addr = target_addr.clone();
        
        let task = tokio::spawn(async move {
            warmup_wss_target(&client_clone, &target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    
    // Warmup gRPC targets
    for (target_addr, is_ssl) in grpc_targets {
        let target_addr = target_addr.clone();
        
        let task = tokio::spawn(async move {
            warmup_grpc_target(&target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    
    // Wait for all warmup tasks (with timeout)
    let timeout = tokio::time::Duration::from_secs(15); // Longer timeout for WSS/gRPC
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

#[cfg(feature = "warmup")]
/// Warmup HTTP/HTTPS target by sending a lightweight request
async fn warmup_http_target(
    client: &hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    target_addr: &str,
    _is_ssl: bool,  // No longer needed, client handles both
) {
    use hyper::{Body, Method, Request};
    
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
    
    // Send request using unified client (handles both HTTP and HTTPS)
    let result = client.request(request).await;
    
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

#[cfg(feature = "warmup")]
/// Warmup WSS target by establishing TCP/TLS connection
/// For WSS, we pre-establish the underlying TCP/TLS connection
/// The actual WebSocket handshake will happen on first real request
async fn warmup_wss_target(
    client: &hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    target_addr: &str,
    is_ssl: bool,
) {
    use hyper::{Body, Method, Request};
    
    if !is_ssl {
        warn!("WSS warmup requires HTTPS, skipping {}", target_addr);
        return;
    }
    
    // Parse URI
    let uri = match hyper::Uri::from_str(target_addr) {
        Ok(uri) => uri,
        Err(e) => {
            warn!("Invalid WSS target URI for warmup: {} ({})", target_addr, e);
            return;
        }
    };
    
    // For WSS warmup, we send a simple HTTP request to establish the TCP/TLS connection
    // The actual WebSocket Upgrade handshake will happen on first real request
    // This pre-establishes the connection in Hyper's connection pool
    let request = match Request::builder()
        .method(Method::GET)
        .uri(uri.clone())
        .body(Body::empty())
    {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to create WSS warmup request for {}: {}", target_addr, e);
            return;
        }
    };
    
    // Send request to establish connection in pool
    match client.request(request).await {
        Ok(response) => {
            // Consume response to complete connection establishment
            let body = response.into_body();
            use hyper::body::HttpBody;
            let mut body_stream = body;
            while let Some(chunk_result) = body_stream.data().await {
                match chunk_result {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("Error reading WSS warmup response body for {}: {}", target_addr, e);
                        break;
                    }
                }
            }
            debug!("Warmed up WSS connection pool for {} (TCP/TLS connection established)", target_addr);
        }
        Err(e) => {
            // Connection might still be established in pool even if request fails
            debug!("WSS warmup request failed for {} (connection may still be in pool): {}", target_addr, e);
        }
    }
}

#[cfg(feature = "warmup")]
/// Warmup gRPC target by establishing TCP/TLS connection
/// For gRPC, we pre-establish the underlying TCP/TLS connection
/// The actual gRPC channel will be created on first real request
async fn warmup_grpc_target(
    target_addr: &str,
    is_ssl: bool,
) {
    // Parse URI
    let uri = match hyper::Uri::from_str(target_addr) {
        Ok(uri) => uri,
        Err(e) => {
            warn!("Invalid gRPC target URI for warmup: {} ({})", target_addr, e);
            return;
        }
    };
    
    // For gRPC warmup, we establish a TCP/TLS connection
    // The actual gRPC channel will be created on first real request
    // This pre-establishes the underlying connection
    
    // Extract hostname and port
    let host = uri.host().unwrap_or("localhost");
    let port = uri.port_u16().unwrap_or(if is_ssl { 443 } else { 80 });
    
    // Establish TCP connection to warmup DNS resolution and routing
    // For gRPC, the actual channel (with TLS if needed) will be created on first real request
    // This pre-establishes DNS cache and routing information
    match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
        Ok(_stream) => {
            // Connection established - drop immediately as we just need DNS/routing warmup
            // The actual gRPC channel with TLS will be created on first real request
            // Stream will be closed when dropped
            debug!("Warmed up gRPC connection pool for {}:{} (DNS/routing cached)", host, port);
        }
        Err(e) => {
            debug!("gRPC warmup connection failed for {}:{} (will connect on first real request): {}", 
                host, port, e);
        }
    }
}

