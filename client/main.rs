use tunnel_lib::tunnel::tunnel_service_client::TunnelServiceClient;
use tunnel_lib::tunnel::*;
use tonic::Request;
use tokio_stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper::{Body, Request as HyperRequest, Method};
use hyper::service::{make_service_fn, service_fn};
use uuid::Uuid;
mod config;
use anyhow::Result;
use crate::config::{ClientConfig, Upstream};
use tracing::{info, error, debug};
use tracing_subscriber;
use chrono;
use tracing::Instrument;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
mod proxy;
use tunnel_lib::http_forward::set_host_header;
use tokio_util::sync::CancellationToken;
use std::sync::Arc as StdArc;
use tokio::sync::RwLock;
use hyper::Uri;
use url::Url;
use dashmap::DashMap;
use tunnel_lib::proxy::{HttpTunnelContext, http_entry_handler};
use proxy::ClientHttpEntryTarget;
mod rules_engine;
use crate::rules_engine::ClientRulesEngine;
mod tunnel_client;
use crate::tunnel_client::TunnelClient;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::load("../config/client.toml")?;
    let log_level = config.log_level.as_str();
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
    info!("Loaded client config: {:?}", config);

    let server_addr = format!("http://{}:{}", config.server_addr, config.server_port);
    let client_group_id = config.client_group_id.clone();
    let trace_enabled = config.trace_enabled.unwrap_or(false);
    let http_port = config.http_entry_port.unwrap_or(8003);
    let http_addr = format!("0.0.0.0:{}", http_port);

    // HTTP入口监听只启动一次，使用动态 tunnel_tx、pending_requests
    let tunnel_tx_holder = StdArc::new(RwLock::new(None));
    let pending_requests_holder = StdArc::new(RwLock::new(None));
    let client_id_holder = StdArc::new(RwLock::new(None));
    let token_holder = StdArc::new(RwLock::new(None::<CancellationToken>));

    // 优雅退出信号监听
    let shutdown_token = CancellationToken::new();
    let shutdown_token2 = shutdown_token.clone();
    tokio::spawn(async move {
        // 同时监听 ctrl-c 和 SIGTERM
        let ctrl_c = async {
            signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        };
        #[cfg(unix)]
        let sigterm = async {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
            sigterm.recv().await;
        };
        #[cfg(not(unix))]
        let sigterm = std::future::pending::<()>();
        tokio::select! {
            _ = ctrl_c => {},
            _ = sigterm => {},
        }
        shutdown_token2.cancel();
        info!("Received shutdown signal, will exit gracefully");
    });

    // 启动 HTTP 入口监听，只启动一次
    {
        let tunnel_tx_holder = tunnel_tx_holder.clone();
        let pending_requests_holder = pending_requests_holder.clone();
        let client_id_holder = client_id_holder.clone();
        let shutdown_token = shutdown_token.clone();
        tokio::spawn(async move {
            let tunnel_tx = tunnel_tx_holder.clone();
            let client_id = client_id_holder.clone();
            let target = Arc::new(ClientHttpEntryTarget {
                tunnel_tx,
                pending_requests: pending_requests_holder.clone(),
                client_id,
            });
            let ctx = HttpTunnelContext {
                client_id: "client".to_string(), // 仅作标识
                tunnel_tx: Arc::new(mpsc::channel(1).0), // dummy
                pending_requests: Arc::new(DashMap::new()),
                direction: tunnel_lib::tunnel::Direction::ClientToServer,
            };
            let make_svc = make_service_fn(move |_| {
                let ctx = ctx.clone();
                let target = target.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let ctx = ctx.clone();
                        let target = target.clone();
                        async move {
                            http_entry_handler(req, &ctx, &*target).await
                        }
                    }))
                }
            });
            let server = hyper::Server::bind(&http_addr.parse().unwrap()).serve(make_svc);
            info!("Client HTTP entry listening on http://{} (tunnel-lib handler)", http_addr);
            let graceful = server.with_graceful_shutdown(async {
                shutdown_token.cancelled().await;
            });
            if let Err(e) = graceful.await {
                error!("Client HTTP entry server error: {}", e);
            }
        });
    }

    // main loop，每次重建所有资源
    let mut backoff: u32 = 0;
    const MAX_BACKOFF_SECS: u64 = 10;
    loop {
        // 1. 新建 channel
        let (tx, rx) = mpsc::channel(128);
        // 2. 新建 TunnelClient
        let client_id = format!("client-{}", Uuid::new_v4());
        let group_id = client_group_id.clone();
        let tunnel_client = Arc::new(TunnelClient::new(client_id.clone(), group_id.clone(), server_addr.clone(), trace_enabled, tx.clone()));
        // 3. 更新 HTTP入口监听用的 tunnel_tx、pending_requests、client_id、token
        {
            let mut t = tunnel_tx_holder.write().await;
            *t = Some(tx.clone());
            let mut p = pending_requests_holder.write().await;
            *p = Some(tunnel_client.pending_requests.clone());
            let mut c = client_id_holder.write().await;
            *c = Some(client_id.clone());
            let mut token_w = token_holder.write().await;
            *token_w = Some(shutdown_token.child_token());
        }
        // 4. 打印 upstreams（可选）
        let rules_engine = tunnel_client.rules_engine.lock().await;
        rules_engine.debug_print_upstreams();
        drop(rules_engine);
        // 5. 建立 gRPC client
        let mut rx = rx;
        info!("==== [MAIN] Creating new gRPC client connection ====");
        let mut success = false;
        match TunnelServiceClient::connect(server_addr.clone()).await {
            Ok(grpc_client) => {
                // 新建 token
                let token = shutdown_token.child_token();
                {
                    let mut token_w = token_holder.write().await;
                    *token_w = Some(token.clone());
                }
                let tunnel_fut = tunnel_client.connect_with_retry_with_token(grpc_client, &mut rx, token.clone());
                tokio::select! {
                    res = tunnel_fut => {
                        if let Err(e) = res {
                            error!("Tunnel connection lost: {e}, will retry...");
                        } else {
                            success = true;
                        }
                    }
                    _ = shutdown_token.cancelled() => {
                        info!("Shutdown signal received, breaking main loop");
                        break;
                    }
                }
                backoff = 0; // 连接成功或正常断开都重置退避
            }
            Err(e) => {
                error!("Failed to connect to server: {e}, will retry...");
            }
        }
        // 清理 token/资源
        {
            let mut token_w = token_holder.write().await;
            *token_w = None;
            let mut t = tunnel_tx_holder.write().await;
            *t = None;
            let mut p = pending_requests_holder.write().await;
            if let Some(pending_requests_arc) = p.as_ref() {
                let client_id: String = client_id_holder.read().await.clone().unwrap();
                let keys: Vec<_> = pending_requests_arc.iter().map(|entry| entry.key().clone()).collect();
                for request_id in keys {
                    if let Some((_, sender)) = pending_requests_arc.remove(&request_id) {
                        let resp = tunnel_lib::response::resp_502(
                            Some("Tunnel closed"),
                            None,
                            Some(client_id.as_str()),
                        );
                        let _ = sender.send(resp);
                    }
                }
            }
            *p = None;
            let mut c = client_id_holder.write().await;
            *c = None;
        }
        // 指数退避：首次失败立即重试，之后 1s、2s、4s、8s、10s（最大）
        let sleep_secs = if backoff == 0 { 0 } else { 2u64.pow(backoff - 1).min(MAX_BACKOFF_SECS) };
        if sleep_secs > 0 {
            info!("Retrying in {}s...", sleep_secs);
            tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
        }
        if backoff < 4 { // 2^3=8, 2^4=16>10, 所以最多退避到10s
            backoff += 1;
        }
        if shutdown_token.is_cancelled() {
            info!("Shutdown token cancelled, exiting main loop");
            break;
        }
    }
    info!("Client exited gracefully");
    Ok(())
} 