use anyhow::{Result, Context};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{info, debug};
use futures::future;
use crate::types::{ClientState, ConnectionPoolEntry, PooledConnection};

/// Get or create a connection from pool
pub async fn get_or_create_connection(
    state: &ClientState,
    pool_key: &str,
    hostname: &str,
    port: u16,
    is_ssl: bool,
    request_id: &str,
    protocol_type: &str,
) -> Result<PooledConnection> {
    // Get or create pool entry
    let pool = state.connection_pool
        .entry(pool_key.to_string())
        .or_insert_with(|| {
            Arc::new(Mutex::new(ConnectionPoolEntry::new(
                10, // max 10 connections per upstream
                std::time::Duration::from_secs(30), // 30s idle timeout
            )))
        })
        .clone();
    
    // Try to get connection from pool
    {
        let mut pool_guard = pool.lock().await;
        if let Some(conn) = pool_guard.get() {
            debug!("[{}] Reusing connection from pool for {}:{}", request_id, hostname, port);
            return Ok(conn);
        }
    }
    
    // No connection in pool, create new one
    debug!("[{}] Creating new connection for {}:{}", request_id, hostname, port);
    let tcp_stream = connect_to_upstream(request_id, protocol_type, hostname, port).await?;
    
    if is_ssl {
        let domain = rustls::ServerName::try_from(hostname)
            .map_err(|_| anyhow::anyhow!("Invalid server name: {}", hostname))?;
        let tls_stream = state.tls_connector.connect(domain, tcp_stream).await
            .context(format!("Failed to establish TLS connection to {}:{}", hostname, port))?;
        Ok(PooledConnection::Tls(tls_stream))
    } else {
        Ok(PooledConnection::Tcp(tcp_stream))
    }
}

/// Return connection to pool
pub async fn return_connection_to_pool(
    state: &ClientState,
    pool_key: &str,
    conn: PooledConnection,
) {
    if let Some(pool) = state.connection_pool.get(pool_key) {
        let mut pool_guard = pool.lock().await;
        pool_guard.put(conn);
        debug!("Returned connection to pool: {}", pool_key);
    }
}

/// Connect to upstream server with TCP connection (IPv4 only)
/// Returns TCP stream ready for use (or TLS upgrade)
/// Explicitly resolves DNS and filters IPv4 addresses to avoid IPv6 connection delays
pub async fn connect_to_upstream(
    request_id: &str,
    protocol_type: &str,
    hostname: &str,
    port: u16,
) -> Result<TcpStream> {
    debug!("[{}] Starting TCP connection to {}:{} (IPv4 only)...", request_id, hostname, port);
    
    // Determine connection timeout based on protocol type
    // HTTP: 5s (short connections), WSS/gRPC: 30s (long connections)
    let connect_timeout_secs = match protocol_type {
        "wss" | "grpc" => 30,
        _ => 5, // Default for HTTP and other protocols
    };
    
    let connect_start = std::time::Instant::now();
    
    // Resolve DNS and filter IPv4 addresses only to avoid IPv6 connection delays
    let dns_start = std::time::Instant::now();
    let addresses: Vec<SocketAddr> = tokio::net::lookup_host((hostname, port))
        .await
        .context(format!("DNS resolution failed for {}:{}", hostname, port))?
        .filter(|addr| addr.is_ipv4()) // Only use IPv4 addresses
        .collect();
    
    if addresses.is_empty() {
        return Err(anyhow::anyhow!("No IPv4 addresses found for {}:{}", hostname, port));
    }
    
    debug!("[{}] DNS resolution completed in {:?}, found {} IPv4 addresses", 
        request_id, dns_start.elapsed(), addresses.len());
    
    // Try connecting to all IPv4 addresses in parallel
    // Use futures::future::select_all to race all connections - first successful wins
    let connect_futures: Vec<_> = addresses.iter().map(|&addr| {
        let request_id = request_id.to_string();
        Box::pin(async move {
            debug!("[{}] Attempting TCP connection to IPv4 address {} (timeout: {}s)...", 
                request_id, addr, connect_timeout_secs);
            tokio::time::timeout(
                std::time::Duration::from_secs(connect_timeout_secs),
                TcpStream::connect(addr)
            ).await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, format!("Connection timeout to {}", addr)))?
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to connect to {}: {}", addr, e)))
            .map(|stream| (stream, addr))
        })
    }).collect();
    
    // Race all connection attempts - select_all returns the first completed (success or failure)
    // We loop until we find a successful connection or all fail
    let mut remaining = connect_futures;
    let mut last_error = None;
    
    while !remaining.is_empty() {
        let (result, _index, rest) = future::select_all(remaining).await;
        remaining = rest;
        
        match result {
            Ok((stream, addr)) => {
                // Success! Cancel remaining attempts by dropping them
                info!("[{}] TCP connection established to {} in {:?}", 
                    request_id, addr, connect_start.elapsed());
                return Ok(stream);
            }
            Err(e) => {
                debug!("[{}] Connection attempt failed: {}", request_id, e);
                last_error = Some(e);
            }
        }
    }
    
    Err(anyhow::anyhow!("Failed to connect to any IPv4 address for {}:{} (tried {} addresses): {:?}", 
        hostname, port, addresses.len(), last_error))
}

