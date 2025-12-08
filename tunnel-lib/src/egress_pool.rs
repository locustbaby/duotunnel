use std::sync::Arc;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use hyper_rustls::HttpsConnector;
use tracing::{info, debug};
use crate::proto::tunnel::Upstream;

pub struct EgressPool {

    client: Arc<Client<HttpsConnector<HttpConnector>, http_body_util::Full<bytes::Bytes>>>,
}

impl EgressPool {

    pub fn new() -> Self {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        
        // hyper-rustls automatically configures ALPN when enable_http2() is called
        let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .enable_http2()   // Automatically handles ALPN negotiation
            .build();
        
        let client = Arc::new(
            Client::builder(TokioExecutor::new())
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .pool_max_idle_per_host(10)
                .build(https_connector)
        );

        info!("Egress pool initialized with unified HTTP/HTTPS client (HTTP/1.1 + HTTP/2 support)");
        
        Self { client }
    }


    pub fn client(&self) -> Arc<Client<HttpsConnector<HttpConnector>, http_body_util::Full<bytes::Bytes>>> {
        self.client.clone()
    }


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

async fn warmup_connection(
    client: &Client<HttpsConnector<HttpConnector>, http_body_util::Full<bytes::Bytes>>,
    address: &str,
) -> anyhow::Result<()> {
    use hyper::{Method, Request};
    use std::str::FromStr;
    
    let uri = hyper::Uri::from_str(address)?;
    
    let request = Request::builder()
        .method(Method::HEAD)
        .uri(uri)
        .body(http_body_util::Full::new(bytes::Bytes::new()))?;
    
    let response = client.request(request).await?;
    
    use http_body_util::BodyExt;
    let mut body = response.into_body();
    while let Some(chunk) = body.frame().await {
        let _ = chunk?;
    }
    
    Ok(())
}
