mod config;
mod proxy;
mod registry;
mod rules;
mod utils;
mod tunnel_server;

use crate::config::ServerConfig;
use crate::rules::RulesEngine;
use crate::registry::ManagedClientRegistry;
use crate::tunnel_server::TunnelServer;
use std::sync::Arc;
use hyper::{Body, Response as HyperResponse, Server as HyperServer, Client as HyperClient};
use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;
use tokio::sync::{mpsc};
use tracing::{info, error, debug};
use tracing_subscriber;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use crate::utils::pick_backend;
use tokio::task::JoinSet;
use crate::proxy::ServerHttpEntryTarget;
use tunnel_lib::proxy::{HttpTunnelContext, http_entry_handler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::load("../config/server.toml")?;
    config.validate_rules()?;
    let log_level = config.server.log_level.as_str();
    let filter = match log_level {
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .json()
        .init();
    debug!("Loaded config: {:?}", config);
    info!("Starting tunnel server...");
    
    let rules_engine = Arc::new(RulesEngine::new(config.clone()));
    let http_client = Arc::new(HyperClient::new());
    let pending_requests = Arc::new(DashMap::new());
    let token_map = Arc::new(DashMap::new());
    let tunnel_server = TunnelServer::new_with_config(&config, http_client.clone(), pending_requests.clone(), token_map.clone());
    let rules_engine = tunnel_server.rules_engine.clone();
    let client_registry = tunnel_server.client_registry.clone();
    let http_client = tunnel_server.http_client.clone();

    // HTTP 入口监听
    let http_port = config.server.http_entry_port;
    let http_addr: SocketAddr = format!("0.0.0.0:{}", http_port).parse()?;
    let target = Arc::new(ServerHttpEntryTarget {
        rules_engine: rules_engine.clone(),
        client_registry: client_registry.clone(),
        connected_streams: tunnel_server.connected_streams.clone(),
        pending_requests: pending_requests.clone(),
        token_map: token_map.clone(),
    });
    let (dummy_tx, _dummy_rx) = mpsc::channel(1);
    let pending_requests = Arc::new(DashMap::new());
    let ctx = HttpTunnelContext {
        client_id: "server".to_string(),
        tunnel_tx: Arc::new(dummy_tx),
        pending_requests: pending_requests.clone(),
        direction: tunnel_lib::tunnel::Direction::ServerToClient,
    };
    let mut join_set = JoinSet::new();

    // HTTP 入口监听
    join_set.spawn(async move {
        let target = target.clone();
        let ctx = ctx.clone();
            let make_svc = make_service_fn(move |_| {
                let ctx = ctx.clone();
                let target = target.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let ctx = ctx.clone();
                        let target = target.clone();
                        async move {
                            http_entry_handler::<ServerHttpEntryTarget>(req, &ctx, &*target).await
                        }
                    }))
                }
            });
            let server = HyperServer::bind(&http_addr).serve(make_svc);
            info!("HTTP entry listening on http://{} (tunnel-lib handler)", http_addr);
            if let Err(e) = server.await {
                error!("HTTP entry server error: {}", e);
        }
    });

    // gRPC 入口监听
    let grpc_port = config.server.grpc_entry_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let rules_engine_grpc = rules_engine.clone();
    join_set.spawn(async move {
        let rules_engine = rules_engine_grpc.clone();
            let make_svc = make_service_fn(move |_| {
                let rules_engine = rules_engine.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let rules_engine = rules_engine.clone();
                        async move {
                            let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
                            let service = None;
                            if let Some(rule) = rules_engine.match_reverse_proxy_rule(host, "", service) {
                                return Ok::<_, hyper::Error>(HyperResponse::new(Body::from(format!("Reverse proxy to client group: {:?}", rule.action_client_group))));
                            }
                            if let Some(rule) = rules_engine.match_forward_rule(host, "", service) {
                                return Ok::<_, hyper::Error>(HyperResponse::new(Body::from(format!("Forward proxy to upstream: {:?}", rule.action_upstream))));
                            }
                            Ok::<_, hyper::Error>(HyperResponse::new(Body::from("No matching rule")))
                        }
                    }))
                }
            });
            let server = HyperServer::bind(&grpc_addr).serve(make_svc);
            info!("gRPC entry listening on http://{} (stub, not real gRPC)", grpc_addr);
            if let Err(e) = server.await {
                error!("gRPC entry server error: {}", e);
        }
    });

    // tunnel 控制端口监听
    let tunnel_port = config.server.tunnel_port;
    let tunnel_addr: SocketAddr = format!("0.0.0.0:{}", tunnel_port).parse()?;
    let svc = tunnel_lib::tunnel::tunnel_service_server::TunnelServiceServer::new(tunnel_server);
    info!("gRPC tunnel server listening on {} (tunnel control)", tunnel_addr);
    join_set.spawn(async move {
        if let Err(e) = tonic::transport::Server::builder().add_service(svc).serve(tunnel_addr).await {
            error!("Tunnel server error: {}", e);
        }
    });

    // 等待所有任务完成
    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            error!("A background task failed: {:?}", e);
        }
    }
    Ok(())
}