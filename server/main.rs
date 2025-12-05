use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, error};
use tunnel_lib::quic_transport::QuicServer;
use tunnel_lib::proto::tunnel::{Rule, Upstream, UpstreamServer};

mod config;
mod types;
mod connection;
mod control;
mod data_stream;
mod egress_forwarder;
mod ingress_handlers;

use types::{ServerState, IngressRule};
use connection::accept_loop;
use ingress_handlers::{HttpIngressHandler, GrpcIngressHandler, WssIngressHandler};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../config/server.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    
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

    // Extract egress routing rules and upstreams
    let (egress_rules_http, egress_rules_grpc, egress_upstreams) = if let Some(ref egress) = config.server_egress_upstream {
        let rules_http: Vec<types::EgressRule> = egress.rules.http.iter().map(|r| types::EgressRule {
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        }).collect();
        
        let rules_grpc: Vec<types::GrpcEgressRule> = egress.rules.grpc.iter().map(|r| types::GrpcEgressRule {
            match_host: r.match_host.clone(),
            match_service: r.match_service.clone(),
            action_upstream: r.action_upstream.clone(),
        }).collect();
        
        let mut upstreams_map = HashMap::new();
        for (name, upstream_config) in &egress.upstreams {
            if let Some(server) = upstream_config.servers.first() {
                let address = server.address.clone();
                let is_ssl = address.starts_with("https://");
                upstreams_map.insert(name.clone(), types::EgressUpstream {
                    name: name.clone(),
                    address: address.clone(),
                    is_ssl,
                });
            }
        }
        
        (rules_http, rules_grpc, upstreams_map)
    } else {
        (Vec::new(), Vec::new(), HashMap::new())
    };

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

    // Create unified egress connection pool
    let egress_pool = Arc::new(tunnel_lib::egress_pool::EgressPool::new());

    // Initialize server state
    let state = Arc::new(ServerState {
        clients: dashmap::DashMap::new(),
        client_groups: dashmap::DashMap::new(),
        group_clients: dashmap::DashMap::new(),
        addr_to_client: dashmap::DashMap::new(),
        data_stream_semaphore: Arc::new(tokio::sync::Semaphore::new(10000)),
        ingress_rules,
        egress_rules_http,
        egress_rules_grpc,
        egress_upstreams,
        client_configs,
        config_version: config.server.config_version.clone(),
        sessions: Arc::new(dashmap::DashMap::new()),
        egress_pool,
    });

    // Warmup connection pools for egress upstreams
    if let Some(ref egress) = config.server_egress_upstream {
        let rules: Vec<Rule> = egress.rules.http.iter().enumerate()
            .map(|(idx, r)| Rule {
                rule_id: format!("egress_http_{}", idx),
                r#type: "http".to_string(),
                match_host: r.match_host.clone(),
                match_path_prefix: String::new(),
                match_header: std::collections::HashMap::new(),
                action_proxy_pass: r.action_upstream.clone(),
                action_set_host: String::new(),
            })
            .collect();
        
        let upstreams: Vec<Upstream> = egress.upstreams.iter().map(|(name, u)| Upstream {
            name: name.clone(),
            servers: u.servers.iter().map(|s| UpstreamServer {
                address: s.address.clone(),
                resolve: s.resolve,
            }).collect(),
            lb_policy: u.lb_policy.clone(),
        }).collect();
        
        let egress_pool_clone = state.egress_pool.clone();
        tokio::spawn(async move {
            egress_pool_clone.warmup_upstreams(&upstreams).await;
        });
    }

    // 1. Start QUIC Server (always required)
    let server_addr: std::net::SocketAddr = bind_addr.parse()?;
    info!("Binding QUIC server on {}...", server_addr);
    let quic_server = QuicServer::bind(server_addr).await?;
    info!("✓ QUIC server successfully bound and listening on {}", server_addr);
    
    // Spawn QUIC acceptor
    let state_clone = state.clone();
    let quic_handle = tokio::spawn(async move {
        accept_loop(quic_server, state_clone).await;
    });
    info!("✓ QUIC acceptor loop started");

    // Start ingress listeners using ListenerManager
    let http_handler = config.server.http_entry_port.map(|_| {
        Arc::new(HttpIngressHandler::new(state.clone()))
    });
    
    let grpc_handler = config.server.grpc_entry_port.map(|_| {
        Arc::new(GrpcIngressHandler::new(state.clone()))
    });
    
    let wss_handler = config.server.wss_entry_port.map(|_| {
        Arc::new(WssIngressHandler::new(state.clone()))
    });
    
    let listener_manager = tunnel_lib::listener::ListenerManager::new(
        config.server.http_entry_port,
        config.server.grpc_entry_port,
        config.server.wss_entry_port,
    );
    
    let listener_handles = listener_manager.start_all(
        http_handler,
        grpc_handler,
        wss_handler,
    ).await?;

    info!("=== Tunnel Server Started Successfully ===");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("=== Received shutdown signal, initiating graceful shutdown ===");

    // Step 1: Stop accepting new connections (shutdown listeners)
    info!("Step 1/4: Shutting down ingress listeners");
    let listener_shutdown_timeout = std::time::Duration::from_secs(3);
    for (idx, handle) in listener_handles.into_iter().enumerate() {
        match tokio::time::timeout(listener_shutdown_timeout, handle).await {
            Ok(Ok(())) => {
                info!("  ✓ Listener {} shutdown successfully", idx + 1);
            }
            Ok(Err(e)) => {
                error!("  ⚠ Listener {} shutdown with error: {}", idx + 1, e);
            }
            Err(_) => {
                error!("  ⚠ Listener {} shutdown timeout after {:?}", idx + 1, listener_shutdown_timeout);
            }
        }
    }

    // Step 2: Close all client connections gracefully
    info!("Step 2/4: Closing client connections");
    let client_count = state.clients.len();
    for entry in state.clients.iter() {
        let client_id = entry.key();
        let conn = entry.value();
        conn.close(0u32.into(), b"Server shutdown");
        tracing::debug!("  Closed connection for client: {}", client_id);
    }
    if client_count > 0 {
        info!("  ✓ Closed {} client connections", client_count);
    }

    // Step 3: Wait for QUIC acceptor to finish
    info!("Step 3/4: Waiting for QUIC acceptor to complete");
    let quic_shutdown_timeout = std::time::Duration::from_secs(5);
    
    // Give the acceptor a moment to process the connection closures
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // Abort the acceptor loop (it's an infinite loop)
    quic_handle.abort();
    
    match tokio::time::timeout(quic_shutdown_timeout, quic_handle).await {
        Ok(Ok(())) => {
            info!("  ✓ QUIC acceptor shutdown successfully");
        }
        Ok(Err(e)) if e.is_cancelled() => {
            info!("  ✓ QUIC acceptor aborted (expected)");
        }
        Ok(Err(e)) => {
            error!("  ⚠ QUIC acceptor task panicked: {}", e);
        }
        Err(_) => {
            error!("  ⚠ QUIC acceptor shutdown timeout after {:?}", quic_shutdown_timeout);
        }
    }

    // Step 4: Final cleanup
    info!("Step 4/4: Performing final cleanup");
    
    // Clear all state
    let session_count = state.sessions.len();
    state.sessions.clear();
    if session_count > 0 {
        info!("  ✓ Cleared {} active sessions", session_count);
    }
    
    state.clients.clear();
    state.client_groups.clear();
    state.group_clients.clear();
    info!("  ✓ Cleared all client registrations");

    info!("=== Tunnel server shutdown complete ===");
    Ok(())
}
