use std::sync::Arc;
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use tracing::{info, debug};
use tunnel_lib::proto::tunnel::Upstream;

/// Egress connection pool manager
/// Manages HTTP/HTTPS clients for different upstream targets
pub struct EgressPool {
    /// Unified HTTPS-capable client (handles both HTTP and HTTPS)
    client: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
}

impl EgressPool {
    /// Create a new egress pool with a unified HTTP/HTTPS client
    pub fn new() -> Self {
        let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()  // Supports both HTTP and HTTPS
            .enable_http1()
            .enable_http2()
            .build();
        
        let client = Arc::new(
            hyper::Client::builder()
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .pool_max_idle_per_host(10)
                .build(https_connector)
        );

        info!("Egress pool initialized with unified HTTP/HTTPS client");
        
        Self { client }
    }

    /// Get the unified client (handles both HTTP and HTTPS)
    pub fn client(&self) -> Arc<Client<HttpsConnector<HttpConnector>, Body>> {
        self.client.clone()
    }

    /// Warmup connections to upstreams
    pub async fn warmup_upstreams(&self, upstreams: &[Upstream]) {
        if upstreams.is_empty() {
            debug!("No upstreams to warmup");
            return;
        }

        info!("Warming up connections to {} upstreams...", upstreams.len());
        
        let mut warmup_tasks = Vec::new();
        
        for upstream in upstreams {
            for server in &upstream.servers {
                let client = self.client.clone();
                let address = server.address.clone();
                let upstream_name = upstream.name.clone();
                
                let task = tokio::spawn(async move {
                    if let Err(e) = warmup_connection(&client, &address).await {
                        debug!("Warmup failed for upstream '{}' ({}): {}", 
                            upstream_name, address, e);
                    } else {
                        debug!("Warmed up connection to upstream '{}' ({})", 
                            upstream_name, address);
                    }
                });
                
                warmup_tasks.push(task);
            }
        }
        
        // Wait for all warmup tasks with timeout
        let timeout = tokio::time::Duration::from_secs(10);
        match tokio::time::timeout(timeout, futures::future::join_all(warmup_tasks)).await {
            Ok(_) => {
                info!("Egress pool warmup completed");
            }
            Err(_) => {
                debug!("Egress pool warmup timed out after {:?}", timeout);
            }
        }
    }
}

impl Default for EgressPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Warmup a single connection
async fn warmup_connection(
    client: &Client<HttpsConnector<HttpConnector>, Body>,
    address: &str,
) -> anyhow::Result<()> {
    use hyper::{Method, Request};
    use std::str::FromStr;
    
    // Parse URI
    let uri = hyper::Uri::from_str(address)?;
    
    // Create a lightweight HEAD request
    let request = Request::builder()
        .method(Method::HEAD)
        .uri(uri)
        .body(Body::empty())?;
    
    // Send request
    let response = client.request(request).await?;
    
    // Consume response body to complete connection
    use hyper::body::HttpBody;
    let mut body = response.into_body();
    while let Some(chunk) = body.data().await {
        let _ = chunk?;
    }
    
    Ok(())
}
