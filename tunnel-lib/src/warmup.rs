#[cfg(feature = "warmup")]
use std::str::FromStr;
#[cfg(feature = "warmup")]
use tracing::{info, debug, warn};
#[cfg(feature = "warmup")]
use crate::proto::tunnel::{Rule, Upstream};

#[cfg(feature = "warmup")]
pub async fn warmup_connection_pools(
    client: &hyper_util::client::legacy::Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, http_body_util::Full<bytes::Bytes>>,
    rules: &[Rule],
    upstreams: &[Upstream],
) {
    

    let mut http_targets = std::collections::HashSet::new();
    let mut wss_targets = std::collections::HashSet::new();
    let mut grpc_targets = std::collections::HashSet::new();
    
    for rule in rules {

        let (target_addr, is_ssl) = if let Some(upstream) = upstreams.iter().find(|u| u.name == rule.action_proxy_pass) {

            if upstream.servers.is_empty() {
                continue;
            }
            let server = &upstream.servers[0];
            (server.address.clone(), server.address.starts_with("https://"))
        } else {

            let is_ssl = rule.action_proxy_pass.starts_with("https://");
            (rule.action_proxy_pass.clone(), is_ssl)
        };
        

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
    

    let mut warmup_tasks = Vec::new();
    

    for (target_addr, is_ssl) in http_targets {
        let client_clone = client.clone();
        let target_addr = target_addr.clone();
        
        let task = tokio::spawn(async move {
            warmup_http_target(&client_clone, &target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    

    for (target_addr, is_ssl) in wss_targets {
        let client_clone = client.clone();
        let target_addr = target_addr.clone();
        
        let task = tokio::spawn(async move {
            warmup_wss_target(&client_clone, &target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    

    for (target_addr, is_ssl) in grpc_targets {
        let target_addr = target_addr.clone();
        
        let task = tokio::spawn(async move {
            warmup_grpc_target(&target_addr, is_ssl).await;
        });
        
        warmup_tasks.push(task);
    }
    

    let timeout = tokio::time::Duration::from_secs(15);
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
async fn warmup_http_target(
    client: &hyper_util::client::legacy::Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, http_body_util::Full<bytes::Bytes>>,
    target_addr: &str,
    _is_ssl: bool,
) {
    use hyper::{Method, Request};
    

    let uri = match hyper::Uri::from_str(target_addr) {
        Ok(uri) => uri,
        Err(e) => {
            warn!("Invalid target URI for warmup: {} ({})", target_addr, e);
            return;
        }
    };
    

    let request = match Request::builder()
        .method(Method::HEAD)
        .uri(uri.clone())
        .body(http_body_util::Full::new(bytes::Bytes::new()))
    {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to create warmup request for {}: {}", target_addr, e);
            return;
        }
    };
    

    let result = client.request(request).await;
    
    match result {
        Ok(response) => {

            let status = response.status();
            let body = response.into_body();

            use http_body_util::BodyExt;
            let mut body_stream = body;
            while let Some(chunk_result) = body_stream.frame().await {
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

            debug!("Warmup request failed for {} (will connect on first real request): {}", target_addr, e);
        }
    }
}

#[cfg(feature = "warmup")]
async fn warmup_wss_target(
    client: &hyper_util::client::legacy::Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, http_body_util::Full<bytes::Bytes>>,
    target_addr: &str,
    is_ssl: bool,
) {
    use hyper::{Method, Request};
    
    if !is_ssl {
        warn!("WSS warmup requires HTTPS, skipping {}", target_addr);
        return;
    }
    

    let uri = match hyper::Uri::from_str(target_addr) {
        Ok(uri) => uri,
        Err(e) => {
            warn!("Invalid WSS target URI for warmup: {} ({})", target_addr, e);
            return;
        }
    };
    



    let request = match Request::builder()
        .method(Method::GET)
        .uri(uri.clone())
        .body(http_body_util::Full::new(bytes::Bytes::new()))
    {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to create WSS warmup request for {}: {}", target_addr, e);
            return;
        }
    };
    

    match client.request(request).await {
        Ok(response) => {

            let body = response.into_body();
            use http_body_util::BodyExt;
            let mut body_stream = body;
            while let Some(chunk_result) = body_stream.frame().await {
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

            debug!("WSS warmup request failed for {} (connection may still be in pool): {}", target_addr, e);
        }
    }
}

#[cfg(feature = "warmup")]
async fn warmup_grpc_target(
    target_addr: &str,
    is_ssl: bool,
) {

    let uri = match hyper::Uri::from_str(target_addr) {
        Ok(uri) => uri,
        Err(e) => {
            warn!("Invalid gRPC target URI for warmup: {} ({})", target_addr, e);
            return;
        }
    };
    



    

    let host = uri.host().unwrap_or("localhost");
    let port = uri.port_u16().unwrap_or(if is_ssl { 443 } else { 80 });
    



    match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
        Ok(_stream) => {



            debug!("Warmed up gRPC connection pool for {}:{} (DNS/routing cached)", host, port);
        }
        Err(e) => {
            debug!("gRPC warmup connection failed for {}:{} (will connect on first real request): {}", 
                host, port, e);
        }
    }
}

