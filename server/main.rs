use anyhow::{Result, Context};
use clap::Parser;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::io;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use rustls::ClientConfig;
use tracing::{info, error, debug, warn};
use tunnel_lib::quic_transport::QuicServer;
use tunnel_lib::protocol::{
    write_control_message, read_control_message, read_data_stream_header,
    write_data_stream_header,
    ControlMessage, ConfigSyncResponse,
};
use tunnel_lib::proto::tunnel::{Rule, Upstream, UpstreamServer, DataStreamHeader};
use uuid::Uuid;
use bytes::BytesMut;
use httparse::Request;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../config/server.yaml")]
    config: String,
}

struct ServerState {
    /// Map client_id -> Connection
    clients: DashMap<String, quinn::Connection>,
    /// Map client_id -> client_group
    client_groups: DashMap<String, String>,
    /// Map client_group -> Vec<client_id>
    group_clients: DashMap<String, Vec<String>>,
    /// Server ingress routing rules (for routing external requests to client groups)
    ingress_rules: Vec<IngressRule>,
    /// Client-specific configurations: Map client_group -> (rules, upstreams, config_version)
    client_configs: HashMap<String, (Vec<Rule>, Vec<Upstream>, String)>,
    /// Config version
    config_version: String,
}

#[derive(Debug, Clone)]
struct IngressRule {
    match_host: String,
    action_client_group: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::ServerConfig::load(&args.config)?;
    
    // Initialize logging with configured log level
    let log_level = config.server.log_level.to_lowercase();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(&log_level)
        });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Starting tunnel server...");
    info!("Config file: {}", args.config);
    info!("Log level: {}", config.server.log_level);
    let bind_addr = config.bind_addr();
    info!("QUIC tunnel bind address: {}", bind_addr);

    // Extract ingress routing rules
    let ingress_rules: Vec<IngressRule> = config.tunnel_management
        .as_ref()
        .and_then(|tm| tm.server_ingress_routing.as_ref())
        .map(|sir| {
            sir.rules.http.iter().map(|r| IngressRule {
                match_host: r.match_host.clone(),
                action_client_group: r.action_client_group.clone(),
            }).collect()
        })
        .unwrap_or_default();

    // Extract client-specific configurations (client_egress_routings)
    let mut client_configs: HashMap<String, (Vec<Rule>, Vec<Upstream>, String)> = HashMap::new();
    
    if let Some(ref tm) = config.tunnel_management {
        if let Some(ref client_configs_map) = tm.client_configs {
            for (group_id, client_routing) in &client_configs_map.client_egress_routings {
                // Convert client rules
                let client_rules: Vec<Rule> = client_routing.rules.http.iter().enumerate()
                    .map(|(idx, r)| Rule {
                        rule_id: format!("client_{}_{}", group_id, idx),
                        r#type: "http".to_string(),
                        match_host: r.match_host.clone(),
                        match_path_prefix: String::new(),
                        match_header: HashMap::new(),
                        action_proxy_pass: r.action_upstream.clone(),
                        action_set_host: String::new(),
                    })
                    .collect();
                
                // Convert client upstreams
                let client_upstreams: Vec<Upstream> = client_routing.upstreams.iter()
                    .map(|(name, u)| Upstream {
                        name: name.clone(),
                        servers: u.servers.iter().map(|s| UpstreamServer {
                            address: s.address.clone(),
                            resolve: s.resolve,
                        }).collect(),
                        lb_policy: u.lb_policy.clone(),
                    })
                    .collect();
                
                let rules_count = client_rules.len();
                let upstreams_count = client_upstreams.len();
                client_configs.insert(
                    group_id.clone(),
                    (client_rules, client_upstreams, client_routing.config_version.clone())
                );
                
                info!("Loaded client config for group '{}': {} rules, {} upstreams", 
                    group_id, rules_count, upstreams_count);
            }
        }
    }

    // Initialize server state
    let state = Arc::new(ServerState {
        clients: DashMap::new(),
        client_groups: DashMap::new(),
        group_clients: DashMap::new(),
        ingress_rules,
        client_configs,
        config_version: config.server.config_version.clone(),
    });

    // 1. Start QUIC Server (always required)
    let server_addr: SocketAddr = bind_addr.parse()?;
    info!("Binding QUIC server on {}...", server_addr);
    let quic_server = QuicServer::bind(server_addr).await?;
    info!("✓ QUIC server successfully bound and listening on {}", server_addr);
    
    // Spawn QUIC acceptor
    let state_clone = state.clone();
    tokio::spawn(async move {
        accept_loop(quic_server, state_clone).await;
    });
    info!("✓ QUIC acceptor loop started");

    // 2. Start HTTP listener if configured
    if let Some(http_port) = config.server.http_entry_port {
        info!("Starting HTTP listener on port {}...", http_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_http_listener(http_port, state_clone).await {
                error!("HTTP listener error: {}", e);
            }
        });
        info!("✓ HTTP listener task spawned (will listen on 0.0.0.0:{})", http_port);
    } else {
        info!("HTTP listener not configured (http_entry_port not set)");
    }

    // 3. Start gRPC listener if configured
    if let Some(grpc_port) = config.server.grpc_entry_port {
        info!("Starting gRPC listener on port {}", grpc_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_listener(grpc_port, state_clone).await {
                error!("gRPC listener error: {}", e);
            }
        });
    }

    // 4. Start WSS listener if configured
    if let Some(wss_port) = config.server.wss_entry_port {
        info!("Starting WSS listener on port {}", wss_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_wss_listener(wss_port, state_clone).await {
                error!("WSS listener error: {}", e);
            }
        });
    }

    // Keep main alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

async fn accept_loop(server: QuicServer, state: Arc<ServerState>) {
    loop {
        if let Some(conn) = server.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(conn, state).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(conn: quinn::Connection, state: Arc<ServerState>) -> Result<()> {
    let remote_addr = conn.remote_address();
    info!("New connection from {}", remote_addr);
    
    // Store connection for data stream handling (separate from control stream)
    let conn_for_data_streams = conn.clone();

    // 1. Handle Control Stream: expect bidirectional stream for persistent control
    let (mut control_send, mut control_recv) = match conn.accept_bi().await {
        Ok(streams) => streams,
        Err(e) => {
            warn!("Failed to accept control stream: {}", e);
            return Ok(());
        }
    };
    
    // 2. Spawn control stream handler IMMEDIATELY to handle the initial ConfigSyncRequest
    // CRITICAL: Move control_send and control_recv into the spawned task to keep them alive
    // If we don't move them, they will be dropped when handle_connection continues,
    // causing the stream to close and client to see "connection lost"
    let state_for_control = state.clone();
    let conn_for_control_check = conn.clone();
    let remote_addr_for_control = remote_addr;
    
    // We need to read the initial ConfigSyncRequest first to get client_group
    // But we can't move control_recv yet, so we'll read it in the spawned task
    tokio::spawn(async move {
        info!("Control stream handler started");
        
        // Read initial ConfigSyncRequest
        let control_msg = match read_control_message(&mut control_recv).await {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to read initial ConfigSyncRequest: {}", e);
                return;
            }
        };
        
        let (requested_client_id, client_group) = match control_msg.payload {
            Some(tunnel_lib::proto::tunnel::control_message::Payload::ConfigSync(req)) => {
                info!("Received ConfigSyncRequest from client_id: {}, group: {}, version: {}", 
                    req.client_id, req.group, req.config_version);
                (req.client_id.clone(), req.group.clone())
            }
            _ => {
                warn!("Expected ConfigSyncRequest, got something else");
                return;
            }
        };
        
        // Generate unique client_id for this connection
        let unique_client_id = format!("{}-{}-{}", 
            client_group, 
            remote_addr_for_control.ip(), 
            Uuid::new_v4().to_string().split('-').next().unwrap_or("")
        );
        let client_id = unique_client_id;
        
        // Register client with group mapping
        info!("Registering client: unique_client_id='{}', requested_client_id='{}', group='{}'", 
            client_id, requested_client_id, client_group);
        
        if state_for_control.clients.contains_key(&client_id) {
            warn!("Client ID '{}' already exists, removing old connection", client_id);
            cleanup_client_registration(&state_for_control, &client_id);
        }
        
        state_for_control.clients.insert(client_id.clone(), conn_for_control_check.clone());
        state_for_control.client_groups.insert(client_id.clone(), client_group.clone());
        state_for_control.group_clients.entry(client_group.clone()).or_insert_with(Vec::new).push(client_id.clone());
        
        info!("Client registered successfully. Total clients in group '{}': {}", 
            client_group, state_for_control.group_clients.get(&client_group).map(|v| v.len()).unwrap_or(0));
        
        // Send initial ConfigSyncResponse
        let (client_rules_init, client_upstreams_init, client_config_version_init) = state_for_control.client_configs
            .get(&client_group)
            .map(|(r, u, v)| (r.clone(), u.clone(), v.clone()))
            .unwrap_or_else(|| {
                warn!("No client config found for group '{}', sending empty config", client_group);
                (Vec::new(), Vec::new(), state_for_control.config_version.clone())
            });
        
        info!("Sending config to client group '{}': {} rules, {} upstreams, version: {}", 
            client_group, client_rules_init.len(), client_upstreams_init.len(), client_config_version_init);
        
        let response_init = ConfigSyncResponse {
            config_version: client_config_version_init.clone(),
            rules: client_rules_init.clone(),
            upstreams: client_upstreams_init.clone(),
        };
        
        let response_msg_init = ControlMessage {
            payload: Some(tunnel_lib::proto::tunnel::control_message::Payload::ConfigSyncResponse(
                response_init,
            )),
        };
        
        if let Err(e) = write_control_message(&mut control_send, &response_msg_init).await {
            error!("Failed to send initial ConfigSyncResponse: {}", e);
            return;
        }
        info!("Sent initial ConfigSyncResponse to client_id: {}", client_id);
        
        // Keep control stream open and handle future config sync requests
        // Note: control_send and control_recv MUST be moved into this task
        loop {
            match read_control_message(&mut control_recv).await {
                Ok(msg) => {
                    if let Some(payload) = msg.payload {
                        match payload {
                            tunnel_lib::proto::tunnel::control_message::Payload::ConfigSync(req) => {
                                debug!("Received ConfigSyncRequest from client_id: {}, version: {}", 
                                    req.client_id, req.config_version);
                                
                                // Get client group for this client
                                let client_group_for_sync = state_for_control.client_groups
                                    .get(&req.client_id)
                                    .map(|g| g.value().clone())
                                    .unwrap_or_else(|| {
                                        warn!("Client '{}' not found in groups, using requested group", req.client_id);
                                        req.group.clone()
                                    });
                                
                                // Get client-specific config
                                let (client_rules, client_upstreams, client_config_version) = state_for_control.client_configs
                                    .get(&client_group_for_sync)
                                    .map(|(r, u, v)| (r.clone(), u.clone(), v.clone()))
                                    .unwrap_or_else(|| {
                                        warn!("No client config found for group '{}'", client_group_for_sync);
                                        (Vec::new(), Vec::new(), state_for_control.config_version.clone())
                                    });
                                
                                // Send updated ConfigSyncResponse
                                let response = ConfigSyncResponse {
                                    config_version: client_config_version,
                                    rules: client_rules,
                                    upstreams: client_upstreams,
                                };
                                
                                let response_msg = ControlMessage {
                                    payload: Some(tunnel_lib::proto::tunnel::control_message::Payload::ConfigSyncResponse(
                                        response,
                                    )),
                                };
                                
                                if let Err(e) = write_control_message(&mut control_send, &response_msg).await {
                                    error!("Failed to send ConfigSyncResponse: {}", e);
                                    // Check if connection is still alive
                                    if conn_for_control_check.close_reason().is_some() {
                                        info!("Connection closed, exiting control stream handler");
                                        break;
                                    }
                                    // Otherwise, continue - might be transient error
                                    continue;
                                }
                                debug!("Sent ConfigSyncResponse to client_id: {}", req.client_id);
                            }
                            _ => {
                                warn!("Received unexpected control message on control stream");
                            }
                        }
                    }
                }
                Err(e) => {
                    // Check if connection is actually closed
                    if conn_for_control_check.close_reason().is_some() {
                        info!("Control stream closed because connection closed for client {}", client_id);
                        break;
                    }
                    // Stream read error, but connection might still be alive
                    // This could be a transient error or stream closed but connection alive
                    warn!("Control stream read error for client {}: {} (connection still alive, will retry)", 
                        client_id, e);
                    // Don't break immediately - wait a bit and check connection status
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    if conn_for_control_check.close_reason().is_some() {
                        info!("Connection closed during retry, exiting control stream handler");
                        break;
                    }
                    // Continue loop - connection is still alive
                }
            }
        }
        warn!("Control stream handler exited for client {}", client_id);
    });
    
    // 5. Handle incoming data streams (Forward Tunnels from Client)
    // This loop keeps the connection alive and handles data streams from client
    // Note: client_id is registered in the control stream handler, so we need to find it by connection
    loop {
        match conn_for_data_streams.accept_bi().await {
            Ok((send, recv)) => {
                // Find client_id by connection (reverse lookup)
                let client_id_opt = state.clients.iter()
                    .find(|entry| entry.value().remote_address() == remote_addr)
                    .map(|entry| entry.key().clone());
                
                if let Some(ref client_id) = client_id_opt {
                    debug!("Accepted bidirectional data stream from client {}", client_id);
                } else {
                    debug!("Accepted bidirectional data stream from {} (client not yet registered)", remote_addr);
                }
                
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_data_stream(send, recv, state).await {
                        error!("Data stream error: {}", e);
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                // Find client_id and cleanup
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    info!("Connection closed gracefully for client {}: ApplicationClosed", client_id);
                    cleanup_client_registration(&state, &client_id);
                } else {
                    info!("Connection closed gracefully for {}", remote_addr);
                }
                break;
            }
            Err(quinn::ConnectionError::ConnectionClosed(e)) => {
                // Find client_id and cleanup
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    warn!("Connection closed unexpectedly for client {}: {}", client_id, e);
                    cleanup_client_registration(&state, &client_id);
                } else {
                    warn!("Connection closed unexpectedly for {}: {}", remote_addr, e);
                }
                break;
            }
            Err(quinn::ConnectionError::TimedOut) => {
                // Connection timeout - but connection might still be alive
                // Don't break, keep waiting for streams
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    warn!("Connection timeout for client {}, but keeping connection alive", client_id);
                    // Check if connection is still valid
                    if conn_for_data_streams.close_reason().is_some() {
                        info!("Connection actually closed for client {}", client_id);
                        cleanup_client_registration(&state, &client_id);
                        break;
                    }
                } else {
                    warn!("Connection timeout for {}, but keeping connection alive", remote_addr);
                    if conn_for_data_streams.close_reason().is_some() {
                        break;
                    }
                }
                // Otherwise, continue waiting
                continue;
            }
            Err(e) => {
                // Other errors - log but don't break immediately
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    warn!("Error accepting data stream for client {}: {} (will retry)", client_id, e);
                    // Check if connection is actually closed
                    if conn_for_data_streams.close_reason().is_some() {
                        info!("Connection closed for client {}: {}", client_id, e);
                        cleanup_client_registration(&state, &client_id);
                        break;
                    }
                } else {
                    warn!("Error accepting data stream for {}: {} (will retry)", remote_addr, e);
                    if conn_for_data_streams.close_reason().is_some() {
                        break;
                    }
                }
                // Wait a bit before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    Ok(())
}

async fn handle_data_stream(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    _state: Arc<ServerState>,
) -> Result<()> {
    // 1. Read DataStreamHeader
    let header = read_data_stream_header(&mut recv).await?;
    debug!(
        "Received DataStreamHeader: request_id={}, type={}, host={}",
        header.request_id, header.r#type, header.host
    );
    
    // Note: This is a forward tunnel (Client -> Server -> Target)
    // In the new design, Client sends type and host, Server needs to:
    // 1. Match rules based on type and host to find upstream
    // 2. Connect to upstream
    // 3. Forward data
    
    // For now, this is a placeholder - forward tunnels need rule matching on server side
    warn!("Forward tunnel handler: rule matching on server side not yet implemented");
    
    // TODO: Implement server-side rule matching for forward tunnels
    // This would require server to have rules configured for matching client requests
    // and routing them to upstreams
    
    Ok(())
}

/// Parse target address into hostname and port
/// Supports formats: "hostname:port", "http://hostname:port", "https://hostname:port"
fn parse_target_addr(addr: &str) -> Result<(String, u16)> {
    let addr = addr.trim();
    
    // Remove scheme if present
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
    
    // Split hostname and port
    if let Some(pos) = addr.rfind(':') {
        let hostname = addr[..pos].to_string();
        let port = addr[pos + 1..].parse::<u16>()
            .context(format!("Invalid port in address: {}", addr))?;
        Ok((hostname, port))
    } else {
        // Default port based on scheme (but we don't have scheme info here)
        // Assume port 80 for HTTP, 443 for HTTPS
        // For now, default to 80
        Ok((addr.to_string(), 80))
    }
}

/// Start HTTP listener
async fn start_http_listener(port: u16, state: Arc<ServerState>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP listener listening on 0.0.0.0:{}", port);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                debug!("Accepted HTTP connection from {}", peer_addr);
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_http_request(stream, peer_addr, state).await {
                        error!("HTTP request handling error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept HTTP connection: {}", e);
            }
        }
    }
}

/// Handle HTTP request from external client
async fn handle_http_request(
    mut stream: TcpStream,
    peer_addr: SocketAddr,
    state: Arc<ServerState>,
) -> Result<()> {
    // 1. Read HTTP request headers
    let mut buffer = BytesMut::new();
    let mut header_end = false;
    
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed before headers");
        }
        
        buffer.extend_from_slice(&buf[..n]);
        
        // Check for end of headers
        if buffer.len() >= 4 {
            for i in 0..=buffer.len().saturating_sub(4) {
                if &buffer[i..i+4] == b"\r\n\r\n" {
                    header_end = true;
                    break;
                }
            }
        }
        
        // Safety limit
        if buffer.len() > 8192 {
            anyhow::bail!("HTTP headers too large");
        }
    }
    
    // Find the end of headers
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = buffer.split_to(header_end_pos);
    let remaining_body = buffer;
    
    // Parse HTTP headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(&header_bytes)?;
    match parse_result {
        httparse::Status::Complete(_) => {
            debug!("Parsed HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
        }
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Extract Host header
    let host = req.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case("host"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .map(|s| s.to_string());
    
    let host = host.ok_or_else(|| anyhow::anyhow!("Missing Host header"))?;
    debug!("HTTP request Host: {}", host);
    
    // 2. Match ingress routing rule
    let matched_rule = state.ingress_rules.iter()
        .find(|r| r.match_host.eq_ignore_ascii_case(&host));
    
    let client_group = match matched_rule {
        Some(rule) => {
            debug!("Matched ingress rule: {} -> group {}", host, rule.action_client_group);
            rule.action_client_group.clone()
        }
        None => {
            warn!("No matching ingress rule for host: {}", host);
            return Err(anyhow::anyhow!("No matching ingress rule for host: {}", host));
        }
    };
    
    // 3. Select a client from the group
    info!("Selecting client from group '{}' for host '{}'", client_group, host);
    let client_id = select_client_from_group(&state, &client_group)?;
    
    // 4. Get Connection from registry
    let client_conn = state.clients.get(&client_id)
        .ok_or_else(|| {
            error!("Client '{}' not found in clients registry", client_id);
            anyhow::anyhow!("Client {} not found", client_id)
        })?
        .clone();
    
    info!("Selected client '{}' from group '{}', opening QUIC stream...", client_id, client_group);
    
    // 5. Open QUIC bidirectional stream
    let (mut send, mut recv) = match client_conn.open_bi().await {
        Ok(streams) => {
            info!("Successfully opened QUIC bidirectional stream to client '{}'", client_id);
            streams
        }
        Err(e) => {
            error!("Failed to open QUIC stream to client '{}': {}", client_id, e);
            return Err(anyhow::anyhow!("Failed to open QUIC stream: {}", e));
        }
    };
    
    // 6. Create and send DataStreamHeader
    let request_id = Uuid::new_v4().to_string();
    let header = DataStreamHeader {
        request_id: request_id.clone(),
        r#type: "http".to_string(),
        host: host.clone(),
        metadata: HashMap::new(),
    };
    
    let header_send_start = std::time::Instant::now();
    write_data_stream_header(&mut send, &header).await?;
    info!("[{}] Sent DataStreamHeader to client {}: request_id={}, host={}", 
        request_id, client_id, request_id, host);
    
    // 7. Send HTTP request headers and body
    info!("[{}] Sending HTTP headers ({} bytes) and body ({} bytes) to client...", 
        request_id, header_bytes.len(), remaining_body.len());
    send.write_all(&header_bytes).await?;
    if !remaining_body.is_empty() {
        send.write_all(&remaining_body).await?;
    }
    // Flush to ensure data is sent immediately
    send.flush().await?;
    info!("[{}] Sent HTTP headers and body to client in {:?}, starting bidirectional copy...", 
        request_id, header_send_start.elapsed());
    
    // 8. Bidirectional copy: External client <-> QUIC stream
    let copy_start = std::time::Instant::now();
    info!("[{}] Starting bidirectional copy: External client <-> QUIC stream", request_id);
    let (mut ri, mut wi) = stream.split();
    
    let request_id_clone = request_id.clone();
    let client_to_quic = async move {
        let bytes_copied = io::copy(&mut ri, &mut send).await?;
        info!("[{}] External client -> QUIC: copied {} bytes in {:?}", 
            request_id_clone, bytes_copied, copy_start.elapsed());
        send.finish().await?;
        info!("[{}] QUIC stream send side finished", request_id_clone);
        Ok::<_, anyhow::Error>(())
    };
    
    let request_id_clone2 = request_id.clone();
    let quic_to_client = async move {
        let bytes_copied = io::copy(&mut recv, &mut wi).await?;
        info!("[{}] QUIC -> External client: copied {} bytes in {:?}", 
            request_id_clone2, bytes_copied, copy_start.elapsed());
        Ok::<_, anyhow::Error>(())
    };
    
    tokio::try_join!(client_to_quic, quic_to_client)?;
    info!("[{}] Bidirectional copy completed in {:?}", request_id, copy_start.elapsed());
    
    Ok(())
}

/// Find client_id by remote address
fn find_client_id_by_addr(state: &ServerState, remote_addr: &SocketAddr) -> Option<String> {
    state.clients.iter()
        .find(|entry| entry.value().remote_address() == *remote_addr)
        .map(|entry| entry.key().clone())
}

/// Clean up client registration when connection closes
fn cleanup_client_registration(state: &ServerState, client_id: &str) {
    info!("Cleaning up client registration: client_id='{}'", client_id);
    
    // Remove from clients map
    state.clients.remove(client_id);
    
    // Get client group before removing (DashMap::remove returns Option<(K, V)>)
    if let Some((_, client_group)) = state.client_groups.remove(client_id) {
        // Remove from group_clients mapping
        if let Some(mut clients) = state.group_clients.get_mut(&client_group) {
            clients.retain(|id| id != client_id);
            if clients.is_empty() {
                drop(clients); // Release the reference before removing
                state.group_clients.remove(&client_group);
            }
        }
        info!("Client '{}' removed from group '{}'", client_id, client_group);
    }
}

/// Select a client from a group (simple round-robin)
fn select_client_from_group(
    state: &ServerState,
    group: &str,
) -> Result<String> {
    // Debug: log all available groups
    let available_groups: Vec<String> = state.group_clients.iter().map(|e| e.key().clone()).collect();
    debug!("Available client groups: {:?}", available_groups);
    debug!("Looking for group: {}", group);
    
    let clients = state.group_clients.get(group)
        .ok_or_else(|| {
            let available = available_groups.join(", ");
            anyhow::anyhow!("Client group '{}' not found. Available groups: [{}]", group, available)
        })?;
    
    if clients.is_empty() {
        anyhow::bail!("Client group {} has no clients", group);
    }
    
    // Simple round-robin: just pick the first available client
    // TODO: Implement proper load balancing
    for client_id in clients.iter() {
        if state.clients.contains_key(client_id) {
            return Ok(client_id.clone());
        }
    }
    
    anyhow::bail!("No available clients in group {}", group)
}

/// Start gRPC listener (placeholder - to be implemented)
async fn start_grpc_listener(_port: u16, _state: Arc<ServerState>) -> Result<()> {
    // TODO: Implement gRPC listener
    warn!("gRPC listener not yet implemented");
    Ok(())
}

/// Start WSS listener (placeholder - to be implemented)
async fn start_wss_listener(_port: u16, _state: Arc<ServerState>) -> Result<()> {
    // TODO: Implement WSS listener
    warn!("WSS listener not yet implemented");
    Ok(())
}
