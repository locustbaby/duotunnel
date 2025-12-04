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
    tokio::spawn(async move {
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
    
    let _listener_handles = listener_manager.start_all(
        http_handler,
        grpc_handler,
        wss_handler,
    ).await?;

    // Keep main alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
