use anyhow::{Result, Context};
use clap::Parser;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{info, error, debug, warn};
use rustls;
use tokio_rustls;
use webpki_roots;
use futures::future;
use tunnel_lib::quic_transport::QuicClient;
use tunnel_lib::protocol::{
    write_control_message, read_control_message, write_data_stream_header,
    read_data_stream_header,
    ControlMessage, ConfigSyncRequest, Rule, Upstream,
};
use uuid::Uuid;

mod config;
mod http_handler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../config/client.yaml")]
    config: String,
}

/// Pooled connection entry
enum PooledConnection {
    Tcp(TcpStream),
    Tls(tokio_rustls::client::TlsStream<TcpStream>),
}

/// Connection pool entry
struct ConnectionPoolEntry {
    connections: Vec<(PooledConnection, Instant)>, // (connection, created_at)
    max_size: usize,
    idle_timeout: std::time::Duration,
}

impl ConnectionPoolEntry {
    fn new(max_size: usize, idle_timeout: std::time::Duration) -> Self {
        Self {
            connections: Vec::new(),
            max_size,
            idle_timeout,
        }
    }
    
    /// Get a connection from pool, or return None if pool is empty
    fn get(&mut self) -> Option<PooledConnection> {
        // Clean up idle connections first
        let now = Instant::now();
        self.connections.retain(|(_, created_at)| {
            now.duration_since(*created_at) < self.idle_timeout
        });
        
        // Return the first available connection
        if let Some((conn, _)) = self.connections.pop() {
            Some(conn)
        } else {
            None
        }
    }
    
    /// Return a connection to the pool
    fn put(&mut self, conn: PooledConnection) {
        // Only keep connections if pool is not full
        if self.connections.len() < self.max_size {
            self.connections.push((conn, Instant::now()));
        }
        // Otherwise, drop the connection (pool is full)
    }
}

struct ClientState {
    /// Current rules from server
    rules: Arc<DashMap<String, Rule>>,
    /// Current upstreams from server
    upstreams: Arc<DashMap<String, Upstream>>,
    /// Current config version
    config_version: Arc<tokio::sync::RwLock<String>>,
    /// TLS connector (reused for all HTTPS connections)
    tls_connector: Arc<tokio_rustls::TlsConnector>,
    /// Connection pool: key is "hostname:port:is_ssl", value is connection pool
    connection_pool: Arc<DashMap<String, Arc<Mutex<ConnectionPoolEntry>>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::ClientConfig::load(&args.config)?;
    
    // Initialize logging with configured log level
    let log_level = config.log_level.to_lowercase();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(&log_level)
        });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Starting tunnel client...");
    info!("Config file: {}", args.config);
    info!("Log level: {}", config.log_level);
    let server_addr_str = config.server_addr();
    info!("Server address: {}", server_addr_str);

    // 1. Connect to QUIC Server
    let client = QuicClient::new()?;
    let server_addr: SocketAddr = server_addr_str.parse()?;
    let connection = client.connect(server_addr, "localhost").await?;
    let connection = Arc::new(connection);

    // Create TLS connector (reused for all HTTPS connections)
    let mut root_certs = rustls::RootCertStore::empty();
    root_certs.add_trust_anchors(
        webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        })
    );
    
    let tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();
    
    let tls_connector = Arc::new(tokio_rustls::TlsConnector::from(Arc::new(tls_config)));
    
    // Initialize state
    let state = Arc::new(ClientState {
        rules: Arc::new(DashMap::new()),
        upstreams: Arc::new(DashMap::new()),
        config_version: Arc::new(tokio::sync::RwLock::new("0".to_string())),
        tls_connector,
        connection_pool: Arc::new(DashMap::new()),
    });

    // 2. Establish Control Stream and send ConfigSyncRequest
    let conn_for_control = connection.clone();
    let state_for_control = state.clone();
    let client_id = config.client_id();
    let client_group_id = config.client_group_id.clone();
    tokio::spawn(async move {
        if let Err(e) = handle_control_stream(conn_for_control.clone(), client_id, client_group_id, state_for_control.clone()).await {
            error!("Control stream error: {}", e);
        }
    });

    // 2.5. Handle reverse streams (from Server to Client)
    let conn_for_reverse = connection.clone();
    let state_for_reverse = state.clone();
    tokio::spawn(async move {
        if let Err(e) = handle_reverse_streams(conn_for_reverse, state_for_reverse).await {
            error!("Reverse stream handler error: {}", e);
        }
    });

    // 3. Start HTTP listener if configured
    if let Some(http_port) = config.http_entry_port {
        info!("Starting HTTP listener on port {}", http_port);
        let conn = connection.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_http_listener(http_port, conn, state_clone).await {
                error!("HTTP listener error: {}", e);
            }
        });
    }

    // 4. Start gRPC listener if configured
    if let Some(grpc_port) = config.grpc_entry_port {
        info!("Starting gRPC listener on port {}", grpc_port);
        let conn = connection.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_listener(grpc_port, conn, state_clone).await {
                error!("gRPC listener error: {}", e);
            }
        });
    }

    // 5. Start WSS listener if configured
    if let Some(wss_port) = config.wss_entry_port {
        info!("Starting WSS listener on port {}", wss_port);
        let conn = connection.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_wss_listener(wss_port, conn, state_clone).await {
                error!("WSS listener error: {}", e);
            }
        });
    }

    // 6. Start Forward Tunnels (Legacy - Listen Local -> Forward to Server)
    for tunnel in &config.forward_tunnels {
        let conn = connection.clone();
        let state = state.clone();
        let local_port = tunnel.local_port;
        tokio::spawn(async move {
            if let Err(e) = run_forward_tunnel(local_port, conn, state).await {
                error!("Forward tunnel error: {}", e);
            }
        });
    }

    // Keep main alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

/// Handle Control Stream: persistent bidirectional stream for config sync
async fn handle_control_stream(
    connection: Arc<quinn::Connection>,
    client_id: String,
    client_group_id: String,
    state: Arc<ClientState>,
) -> Result<()> {
    info!("Establishing control stream (persistent bidirectional)...");
    
    // Open a bidirectional stream for control (persistent)
    let (mut send, mut recv) = connection.open_bi().await?;
    info!("Control stream opened");
    
    // Send initial ConfigSyncRequest
    let current_version = state.config_version.read().await.clone();
    let config_sync = ConfigSyncRequest {
        client_id: client_id.clone(),
        group: client_group_id.clone(),
        config_version: current_version,
    };
    
    let control_msg = ControlMessage {
        payload: Some(tunnel_lib::proto::tunnel::control_message::Payload::ConfigSync(
            config_sync,
        )),
    };
    
    write_control_message(&mut send, &control_msg).await?;
    info!("Sent initial ConfigSyncRequest for client_id: {}", client_id);
    
    // Keep the stream open and continuously receive config updates
    loop {
        match read_control_message(&mut recv).await {
            Ok(msg) => {
                if let Some(payload) = msg.payload {
                    match payload {
                        tunnel_lib::proto::tunnel::control_message::Payload::ConfigSyncResponse(resp) => {
                            let current_version = state.config_version.read().await.clone();
                            if resp.config_version != current_version {
                                info!("Received ConfigSyncResponse, version: {} (was: {})", resp.config_version, current_version);
                                
                                // Update rules
                                state.rules.clear();
                                for rule in resp.rules {
                                    state.rules.insert(rule.rule_id.clone(), rule);
                                }
                                
                                // Update upstreams
                                state.upstreams.clear();
                                for upstream in resp.upstreams {
                                    state.upstreams.insert(upstream.name.clone(), upstream);
                                }
                                
                                // Update config version
                                *state.config_version.write().await = resp.config_version.clone();
                                
                                info!("Updated {} rules and {} upstreams (version: {})", 
                                    state.rules.len(), state.upstreams.len(), resp.config_version);
                            } else {
                                debug!("ConfigSyncResponse received but version unchanged: {}", resp.config_version);
                            }
                        }
                        tunnel_lib::proto::tunnel::control_message::Payload::ErrorMessage(err) => {
                            error!("Received error from server: {} - {}", err.code, err.message);
                            // Don't exit on error, keep listening for more messages
                        }
                        _ => {
                            warn!("Received unexpected control message");
                        }
                    }
                }
            }
            Err(e) => {
                // Check if the underlying connection is actually closed
                let connection_closed = connection.close_reason().is_some();
                
                if connection_closed {
                    error!("Connection closed, exiting control stream handler (client_id: {}, group: {})", 
                        client_id, client_group_id);
                    break;
                }
                
                // Stream read error, but connection might still be alive
                // This could be a transient error or stream closed but connection alive
                warn!("Control stream read error (connection still alive, will retry): {} (client_id: {}, group: {})", 
                    e, client_id, client_group_id);
                
                // Wait a bit and check connection status again
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                if connection.close_reason().is_some() {
                    error!("Connection closed during retry, exiting control stream handler");
                    break;
                }
                
                // Connection is still alive, continue waiting for messages
                // Note: The stream might be closed, but we can't reopen it from client side
                // The server should keep the stream open, so if we get here repeatedly,
                // it might indicate a problem with the server's control stream handler
                continue;
            }
        }
    }
    
    warn!("Control stream closed, exiting control handler");
    Ok(())
}

/// Handle reverse streams initiated by Server (Server -> Client -> Upstream)
async fn handle_reverse_streams(
    connection: Arc<quinn::Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    info!("Reverse stream handler started, waiting for server-initiated streams...");
    loop {
        match connection.accept_bi().await {
            Ok((send, recv)) => {
                info!("Accepted reverse bidirectional stream from server");
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_reverse_stream(send, recv, state).await {
                        error!("Reverse stream error: {}", e);
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                // Connection closed gracefully, this is normal
                info!("Connection closed, reverse stream handler exiting");
                break;
            }
            Err(quinn::ConnectionError::TimedOut) => {
                // Connection timeout, but keep trying
                warn!("Connection timeout, but continuing to wait for streams...");
                continue;
            }
            Err(e) => {
                // Other errors might be transient, log but continue
                warn!("Reverse stream accept error (will retry): {}", e);
                // Don't break on other errors, keep the handler alive
                // The connection might recover
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    
    Ok(())
}

/// Handle a single reverse stream from Server
async fn handle_reverse_stream(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: Arc<ClientState>,
) -> Result<()> {
    // 1. Read DataStreamHeader
    info!("Reading DataStreamHeader from reverse stream...");
    let header_start = std::time::Instant::now();
    let header = read_data_stream_header(&mut recv).await?;
    info!(
        "[{}] Received reverse DataStreamHeader: type={}, host={} (read in {:?})",
        header.request_id, header.r#type, header.host, header_start.elapsed()
    );
    
    // 2. Match rules based on type and host
    let rules: Vec<Rule> = state.rules.iter().map(|r| r.value().clone()).collect();
    let upstreams: Vec<Upstream> = state.upstreams.iter().map(|u| u.value().clone()).collect();
    
    info!("[{}] Matching rules for type={}, host={} (total rules: {}, total upstreams: {})", 
        header.request_id, header.r#type, header.host, rules.len(), upstreams.len());
    info!("[{}] Available rules: {:?}", 
        header.request_id, 
        rules.iter().map(|r| format!("{}:{}->{}", r.r#type, r.match_host, r.action_proxy_pass)).collect::<Vec<_>>());
    info!("[{}] Available upstreams: {:?}", 
        header.request_id,
        upstreams.iter().map(|u| {
            let addresses: Vec<String> = u.servers.iter().map(|s| s.address.clone()).collect();
            format!("{}:[{}]", u.name, addresses.join(","))
        }).collect::<Vec<_>>());
    
    let matched_rule = match_rule_by_type_and_host(&rules, &header.r#type, &header.host)?;
    
    let (final_target_addr, is_target_ssl) = if let Some(rule) = matched_rule {
        info!("[{}] Matched rule: {} -> {} (action_proxy_pass)", 
            header.request_id, header.host, rule.action_proxy_pass);
        
        // Resolve upstream or use direct address
        let resolved = resolve_upstream(&rule.action_proxy_pass, &upstreams)?;
        info!("[{}] Resolved upstream '{}' to address: {} (SSL: {})", 
            header.request_id, rule.action_proxy_pass, resolved.0, resolved.1);
        resolved
    } else {
        error!("[{}] No matching rule for type={}, host={} (available rules: {:?})", 
            header.request_id, header.r#type, header.host, 
            rules.iter().map(|r| format!("{}:{}", r.r#type, r.match_host)).collect::<Vec<_>>());
        return Err(anyhow::anyhow!("No matching rule for type={}, host={}", header.r#type, header.host));
    };
    
    info!("[{}] Forwarding to upstream: {} (SSL: {})", 
        header.request_id, final_target_addr, is_target_ssl);
    
    // 3. Connect to upstream (preserve protocol, don't parse HTTP)
    // Parse address to extract hostname and port
    let (hostname, port) = parse_target_addr(&final_target_addr, is_target_ssl)?;
    
    // 4. Get or create connection from pool
    let pool_key = format!("{}:{}:{}", hostname, port, is_target_ssl);
    let upstream_stream = get_or_create_connection(
        &state,
        &pool_key,
        &hostname,
        port,
        is_target_ssl,
        &header.request_id,
        &header.r#type,
    ).await?;
    
    // 5. Bidirectional copy: QUIC stream <-> Upstream stream
    // Note: Due to tokio::io::split consuming the stream, connections cannot be reused after use
    // The connection pool infrastructure is in place, but true reuse requires HTTP parsing
    // For now, connections are consumed after use
    match copy_bidirectional_and_return(recv, send, upstream_stream, &header.request_id, &state, &pool_key).await {
        Ok(_conn) => {
            // This branch is currently unreachable due to split consuming the connection
            // Future: If we implement HTTP parsing, we can detect request/response boundaries
            // and reuse connections without splitting
            unreachable!("Connection reuse not yet implemented")
        }
        Err(e) => {
            // Connection consumed or error occurred
            // Note: Even successful copies consume the connection due to split
            if e.to_string().contains("consumed after split") {
                debug!("[{}] Connection consumed after use (expected)", header.request_id);
            } else {
                debug!("[{}] Connection error: {}", header.request_id, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

/// Get or create a connection from pool
async fn get_or_create_connection(
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
async fn return_connection_to_pool(
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

/// Bidirectional copy between QUIC stream and upstream stream (TLS or TCP)
/// Note: Due to tokio::io::split consuming the stream, we cannot reuse connections after split
/// This function will consume the connection, so connection pooling is limited
async fn copy_bidirectional_and_return(
    mut recv: quinn::RecvStream,
    mut send: quinn::SendStream,
    upstream_stream: PooledConnection,
    request_id: &str,
    _state: &ClientState,
    _pool_key: &str,
) -> Result<PooledConnection> {
    debug!("[{}] Starting bidirectional copy: QUIC stream <-> Upstream socket (raw data forwarding, no HTTP parsing)", 
        request_id);
    
    let copy_start = std::time::Instant::now();
    
    // Note: tokio::io::split consumes the stream, so we cannot reuse connections
    // For true connection reuse, we would need HTTP parsing to detect request/response boundaries
    // For now, connections are consumed after use
    
    match upstream_stream {
        PooledConnection::Tcp(tcp_stream) => {
            let (mut upstream_read, mut upstream_write) = tokio::io::split(tcp_stream);
            
            let request_id_clone = request_id.to_string();
            let quic_to_upstream = async move {
                debug!("[{}] Starting QUIC -> Upstream copy...", request_id_clone);
                let bytes_copied = io::copy(&mut recv, &mut upstream_write).await?;
                debug!("[{}] QUIC -> Upstream: copied {} bytes in {:?}", 
                    request_id_clone, bytes_copied, copy_start.elapsed());
                Ok::<_, anyhow::Error>(())
            };
            
            let request_id_clone2 = request_id.to_string();
            let upstream_to_quic = async move {
                debug!("[{}] Starting Upstream -> QUIC copy...", request_id_clone2);
                let bytes_copied = io::copy(&mut upstream_read, &mut send).await?;
                debug!("[{}] Upstream -> QUIC: copied {} bytes in {:?}", 
                    request_id_clone2, bytes_copied, copy_start.elapsed());
                send.finish().await?;
                debug!("[{}] QUIC stream send side finished", request_id_clone2);
                Ok::<_, anyhow::Error>(())
            };
            
            tokio::try_join!(quic_to_upstream, upstream_to_quic)?;
            info!("[{}] Bidirectional copy completed in {:?}", request_id, copy_start.elapsed());
            
            // Connection consumed after split, cannot reuse
            Err(anyhow::anyhow!("Connection consumed after split, cannot reuse"))
        }
        PooledConnection::Tls(tls_stream) => {
            let (mut upstream_read, mut upstream_write) = tokio::io::split(tls_stream);
            
            let request_id_clone = request_id.to_string();
            let quic_to_upstream = async move {
                debug!("[{}] Starting QUIC -> Upstream copy...", request_id_clone);
                let bytes_copied = io::copy(&mut recv, &mut upstream_write).await?;
                debug!("[{}] QUIC -> Upstream: copied {} bytes in {:?}", 
                    request_id_clone, bytes_copied, copy_start.elapsed());
                Ok::<_, anyhow::Error>(())
            };
            
            let request_id_clone2 = request_id.to_string();
            let upstream_to_quic = async move {
                debug!("[{}] Starting Upstream -> QUIC copy...", request_id_clone2);
                let bytes_copied = io::copy(&mut upstream_read, &mut send).await?;
                debug!("[{}] Upstream -> QUIC: copied {} bytes in {:?}", 
                    request_id_clone2, bytes_copied, copy_start.elapsed());
                send.finish().await?;
                debug!("[{}] QUIC stream send side finished", request_id_clone2);
                Ok::<_, anyhow::Error>(())
            };
            
            tokio::try_join!(quic_to_upstream, upstream_to_quic)?;
            info!("[{}] Bidirectional copy completed in {:?}", request_id, copy_start.elapsed());
            
            // Connection consumed after split, cannot reuse
            Err(anyhow::anyhow!("Connection consumed after split, cannot reuse"))
        }
    }
}

/// Connect to upstream server with TCP connection (IPv4 only)
/// Returns TCP stream ready for use (or TLS upgrade)
/// Explicitly resolves DNS and filters IPv4 addresses to avoid IPv6 connection delays
async fn connect_to_upstream(
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

/// Match rule by type and host
fn match_rule_by_type_and_host<'a>(rules: &'a [Rule], rule_type: &str, host: &str) -> Result<Option<&'a Rule>> {
    for rule in rules {
        // Match type
        if rule.r#type != rule_type {
            continue;
        }
        
        // Match host
        if !rule.match_host.is_empty() && rule.match_host.eq_ignore_ascii_case(host) {
            return Ok(Some(rule));
        }
    }
    
    Ok(None)
}

/// Resolve upstream name to actual address (preserving protocol)
/// Returns (full_address_with_protocol, is_ssl)
fn resolve_upstream(
    action_proxy_pass: &str,
    upstreams: &[Upstream],
) -> Result<(String, bool)> {
    let address = if let Some(upstream) = upstreams.iter().find(|u| u.name == action_proxy_pass) {
        if upstream.servers.is_empty() {
            anyhow::bail!("Upstream '{}' has no servers", action_proxy_pass);
        }
        
        // Simple round-robin: just pick the first server
        let server = &upstream.servers[0];
        server.address.clone()
    } else {
        // It's a direct address
        action_proxy_pass.to_string()
    };
    
    // Determine if SSL is needed based on address scheme
    let is_ssl = address.trim().starts_with("https://");
    
    // Return full address with protocol preserved
    Ok((address.trim().to_string(), is_ssl))
}

/// Parse target address into hostname and port
/// Returns (hostname, port) with default ports based on protocol
fn parse_target_addr(addr: &str, is_ssl: bool) -> Result<(String, u16)> {
    let addr = addr.trim();
    
    // Remove protocol prefix if present
    let addr = if let Some(pos) = addr.find("://") {
        &addr[pos + 3..]
    } else {
        addr
    };
    
    // Remove path if present
    let addr = if let Some(pos) = addr.find('/') {
        &addr[..pos]
    } else {
        addr
    };
    
    // Parse hostname and port
    // Use rfind(':') to handle IPv6 addresses like [::1]:8080
    if let Some(pos) = addr.rfind(':') {
        // Check if it's an IPv6 address in brackets
        if addr.starts_with('[') {
            // IPv6 with port: [::1]:8080
            let bracket_end = addr.find(']').ok_or_else(|| anyhow::anyhow!("Invalid IPv6 address: {}", addr))?;
            let hostname = addr[1..bracket_end].to_string();
            let port = addr[bracket_end + 2..].parse::<u16>()
                .context(format!("Invalid port in address: {}", addr))?;
            Ok((hostname, port))
        } else {
            // Regular hostname:port or IPv4:port
            let hostname = addr[..pos].to_string();
            let port = addr[pos + 1..].parse::<u16>()
                .context(format!("Invalid port in address: {}", addr))?;
            Ok((hostname, port))
        }
    } else {
        // No port specified, use default based on protocol
        let default_port = if is_ssl {
            443  // HTTPS default port
        } else {
            80   // HTTP default port
        };
        Ok((addr.to_string(), default_port))
    }
}

async fn run_forward_tunnel(
    local_port: u16,
    connection: Arc<quinn::Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], local_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on 0.0.0.0:{}", local_port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        debug!("Accepted connection from {}", peer_addr);
        let connection = connection.clone();
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_forward_connection(socket, peer_addr, connection, state).await {
                error!("Connection handling error: {}", e);
            }
        });
    }
}

async fn handle_forward_connection(
    mut socket: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    connection: Arc<quinn::Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    // 1. Parse HTTP headers and modify if needed
    let rules: Vec<Rule> = state.rules.iter().map(|r| r.value().clone()).collect();
    let upstreams: Vec<Upstream> = state.upstreams.iter().map(|u| u.value().clone()).collect();
    
    let (modified_request_bytes, original_host, modified_host, final_target_addr, is_target_ssl) =
        http_handler::parse_and_modify_http_request(
            &mut socket,
            &rules,
            &upstreams,
            peer_addr.ip().to_string(),
        )
        .await
        .context("Failed to parse HTTP request")?;
    
    debug!("Final target: {}, SSL: {}", final_target_addr, is_target_ssl);
    
    // 2. Open a new QUIC bidirectional stream
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // 3. Create and send DataStreamHeader
    let request_id = Uuid::new_v4().to_string();
    let header = http_handler::create_data_stream_header(
        request_id.clone(),
        modified_host.unwrap_or_else(|| original_host.unwrap_or_default()),
    );
    
    write_data_stream_header(&mut send, &header).await?;
    debug!("Sent DataStreamHeader for request_id: {}", request_id);
    
    // 4. Send modified HTTP request bytes
    send.write_all(&modified_request_bytes).await?;
    
    // 5. Bidirectional copy: remaining data from socket <-> QUIC stream
    let (mut ri, mut wi) = socket.split();
    
    let client_to_server = async {
        tokio::io::copy(&mut ri, &mut send).await?;
        send.finish().await?;
        Ok::<_, anyhow::Error>(())
    };
    
    let server_to_client = async {
        tokio::io::copy(&mut recv, &mut wi).await?;
        Ok::<_, anyhow::Error>(())
    };
    
    tokio::try_join!(client_to_server, server_to_client)?;
    
    Ok(())
}

/// Start HTTP listener (uses existing HTTP handler)
async fn start_http_listener(
    port: u16,
    connection: Arc<quinn::Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP listener listening on 0.0.0.0:{}", port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        debug!("Accepted HTTP connection from {}", peer_addr);
        let connection = connection.clone();
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_forward_connection(socket, peer_addr, connection, state).await {
                error!("HTTP connection handling error: {}", e);
            }
        });
    }
}

/// Start gRPC listener (placeholder - to be implemented)
async fn start_grpc_listener(
    _port: u16,
    _connection: Arc<quinn::Connection>,
    _state: Arc<ClientState>,
) -> Result<()> {
    // TODO: Implement gRPC listener
    warn!("gRPC listener not yet implemented");
    Ok(())
}

/// Start WSS listener (placeholder - to be implemented)
async fn start_wss_listener(
    _port: u16,
    _connection: Arc<quinn::Connection>,
    _state: Arc<ClientState>,
) -> Result<()> {
    // TODO: Implement WSS listener
    warn!("WSS listener not yet implemented");
    Ok(())
}
