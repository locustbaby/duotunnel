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
mod client_mgr;
mod http;
mod data_stream;
mod listeners;

use types::{ServerState, IngressRule};
use connection::accept_loop;
use http::start_http_listener;
use listeners::{start_grpc_listener, start_wss_listener};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../config/server.yaml")]
    config: String,
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
        clients: dashmap::DashMap::new(),
        client_groups: dashmap::DashMap::new(),
        group_clients: dashmap::DashMap::new(),
        ingress_rules,
        client_configs,
        config_version: config.server.config_version.clone(),
    });

    // 1. Start QUIC Server (always required)
    let server_addr: std::net::SocketAddr = bind_addr.parse()?;
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
