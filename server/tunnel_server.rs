use crate::rules::RulesEngine;
use crate::registry::ManagedClientRegistry;
use crate::utils::pick_backend;
use tunnel_lib::tunnel::tunnel_service_server::TunnelService;
use tunnel_lib::tunnel::*;
use tunnel_lib::tunnel::Rule as ProtoRule;
use tunnel_lib::http_forward::forward_http_to_backend;
use tunnel_lib::response::{self, error_response, ProxyErrorKind};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Client as HyperClient, Body};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status};
use async_trait::async_trait;
use futures::StreamExt;
use tunnel_lib::tunnel::StreamType;
use lazy_static;

#[derive(Clone)]
pub struct TunnelServer {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub connected_streams: Arc<DashMap<(String, String), mpsc::Sender<TunnelMessage>>>,
    pub token_map: Arc<DashMap<String, CancellationToken>>,
    pub http_client: Arc<HyperClient<hyper::client::HttpConnector>>,
}

impl TunnelServer {
    pub fn new_with_config(config: &crate::config::ServerConfig, http_client: Arc<HyperClient<hyper::client::HttpConnector>>, pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>, token_map: Arc<DashMap<String, CancellationToken>>) -> Self {
        let rules_engine = RulesEngine::new(config.clone());
        Self {
            client_registry: Arc::new(ManagedClientRegistry::new()),
            pending_requests,
            connected_streams: Arc::new(DashMap::new()),
            token_map,
            rules_engine: Arc::new(rules_engine),
            http_client,
        }
    }
}

impl Default for TunnelServer {
    fn default() -> Self {
        let http_client = Arc::new(HyperClient::new());
        let pending_requests = Arc::new(DashMap::new());
        let token_map = Arc::new(DashMap::new());
        Self::new_with_config(&crate::config::ServerConfig::load("../config/server.toml").unwrap(), http_client, pending_requests, token_map)
    }
}

lazy_static::lazy_static! {
    static ref STREAMS: DashMap<(String, String, StreamType), mpsc::Sender<Result<TunnelMessage, Status>>> = DashMap::new();
}

#[tonic::async_trait]
impl TunnelService for TunnelServer {
    type ControlStreamStream = tokio_stream::wrappers::ReceiverStream<Result<TunnelMessage, Status>>;
    type ProxyStream = tokio_stream::wrappers::ReceiverStream<Result<TunnelMessage, Status>>;

    async fn control_stream(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(32);
        let client_registry = self.client_registry.clone();
        let rules_engine = self.rules_engine.clone();
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(tunnel_msg) => {
                        match tunnel_msg.payload {
                            Some(tunnel_lib::tunnel::tunnel_message::Payload::ConfigSync(config_req)) => {
                                let (rules, upstreams) = if let Some(group) = rules_engine.get_group(&config_req.group) {
                                    (group.rules.http.clone(), group.upstreams.clone())
                                } else {
                                    (Vec::new(), std::collections::HashMap::new())
                                };
                                // ...组装 ConfigSyncResponse 并通过 tx 发送回 client
                            }
                            _ => {}
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn proxy(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(128);
        let pending_requests = self.pending_requests.clone();
        let rules_engine = self.rules_engine.clone();
        let client_registry = self.client_registry.clone();
        let http_client = self.http_client.clone();
        let token_map = self.token_map.clone();
        let connected_streams = self.connected_streams.clone();
        let token = CancellationToken::new();
        tokio::spawn(async move {
            let mut client_id = String::new();
            let mut group = String::new();
            let mut stream_id = String::new();
            let mut stream_type = StreamType::Unspecified;
            let mut client_tx: Option<mpsc::Sender<TunnelMessage>> = None;
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        client_id = msg.client_id.clone();
                        if client_tx.is_none() {
                            let (ctx, mut crx) = mpsc::channel::<TunnelMessage>(128);
                            client_tx = Some(ctx.clone());
                            token_map.insert(client_id.clone(), token.clone());
                            connected_streams.insert((client_id.clone(), stream_id.clone()), ctx.clone());
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                while let Some(tunnel_msg) = crx.recv().await {
                                    if let Err(_) = tx_clone.send(Ok(tunnel_msg)).await {
                                        break;
                                    }
                                }
                            });
                        }
                        match msg.payload {
                            Some(tunnel_message::Payload::HttpResponse(resp)) => {
                                if msg.direction == Direction::ClientToServer as i32 {
                                    if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
                                        let _ = sender.send(resp);
                                    }
                                }
                            }
                            Some(tunnel_message::Payload::ConfigSync(config_req)) => {
                                if msg.direction == Direction::ClientToServer as i32 {
                                    if let Some(group) = rules_engine.get_group(&config_req.group) {
                                        let (rules, upstreams) = (group.rules.http.clone(), group.upstreams.clone());
                                        let proto_rules: Vec<ProtoRule> = rules.into_iter().map(|r| ProtoRule {
                                            rule_id: String::new(),
                                            r#type: String::new(),
                                            match_host: r.match_host.unwrap_or_default(),
                                            match_path_prefix: r.match_path_prefix.unwrap_or_default(),
                                            match_service: r.match_service.unwrap_or_default(),
                                            match_header: std::collections::HashMap::new(),
                                            action_proxy_pass: String::new(),
                                            action_set_host: r.action_set_host.unwrap_or_default(),
                                            action_upstream: r.action_upstream.unwrap_or_default(),
                                            action_ssl: false,
                                        }).collect();
                                        let proto_upstreams: Vec<Upstream> = upstreams.into_iter().map(|(name, u)| Upstream {
                                            name,
                                            servers: u.servers.into_iter().map(|s| UpstreamServer {
                                                address: s.address,
                                                resolve: s.resolve,
                                            }).collect(),
                                            lb_policy: u.lb_policy.unwrap_or_else(|| "round_robin".to_string()),
                                        }).collect();
                                        let config_response = ConfigSyncResponse {
                                            config_version: "v1.0.0".to_string(),
                                            rules: proto_rules,
                                            upstreams: proto_upstreams,
                                        };
                                        let response_msg = TunnelMessage {
                                            client_id: msg.client_id.clone(),
                                            request_id: msg.request_id,
                                            direction: Direction::ServerToClient as i32,
                                            payload: Some(tunnel_message::Payload::ConfigSyncResponse(config_response)),
                                            trace_id: String::new(),
                                        };
                                        if let Some(tx) = client_tx.as_ref() {
                                            let _ = tx.send(response_msg).await;
                                        }
                                    }
                                }
                            }
                            Some(tunnel_message::Payload::HttpRequest(ref req)) => {
                                let host = req.host.as_str();
                                let path = req.url.split('?').next().unwrap_or("/");
                                let mut response = error_response(
                                    ProxyErrorKind::NoMatchRules,
                                    None,
                                    Some(&msg.trace_id),
                                    Some(&msg.request_id),
                                    Some(&msg.client_id),
                                );
                                if let Some(rule) = rules_engine.match_forward_rule(host, path, None) {
                                    if let Some(ref upstream_name) = rule.action_upstream {
                                        if let Some(upstream) = rules_engine.get_upstream(upstream_name) {
                                            if let Some(backend) = pick_backend(upstream) {
                                                let set_host = rule.action_set_host.as_deref().unwrap_or("");
                                                response = forward_http_to_backend(
                                                    req,
                                                    &backend,
                                                    http_client.clone(),
                                                    set_host
                                                ).await;
                                            }
                                        }
                                    }
                                }
                                let response_msg = TunnelMessage {
                                    client_id: msg.client_id.clone(),
                                    request_id: msg.request_id.clone(),
                                    direction: Direction::ServerToClient as i32,
                                    payload: Some(tunnel_message::Payload::HttpResponse(response)),
                                    trace_id: msg.trace_id.clone(),
                                };
                                if let Some(tx) = client_tx.as_ref() {
                                    let _ = tx.send(response_msg).await;
                                }
                            }
                            Some(tunnel_message::Payload::StreamOpen(ref req)) => {
                                // 注册/更新 client 到 group
                                client_registry.register_client(&req.client_id, &req.group);
                                // 注册/更新 stream 到 connected_streams
                                stream_id = req.stream_id.clone();
                                group = req.group.clone();
                                stream_type = StreamType::from_i32(req.stream_type).unwrap_or(StreamType::Unspecified);
                                connected_streams.insert((req.client_id.clone(), req.stream_id.clone()), client_tx.as_ref().unwrap().clone());
                                // 更新多维健康检查
                                client_registry.update_heartbeat_multi(&req.client_id, &req.group, stream_type, &req.stream_id);
                                // 回复 StreamOpenResponse
                                let response = TunnelMessage {
                                    client_id: req.client_id.clone(),
                                    request_id: msg.request_id.clone(),
                                    direction: Direction::ServerToClient as i32,
                                    payload: Some(tunnel_message::Payload::StreamOpenResponse(StreamOpenResponse {
                                        success: true,
                                        message: "stream registered/heartbeat ok".to_string(),
                                        timestamp: chrono::Utc::now().timestamp(),
                                    })),
                                    trace_id: msg.trace_id.clone(),
                                };
                                if let Some(tx) = client_tx.as_ref() {
                                    let _ = tx.send(response).await;
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        if let Some(token) = token_map.get(client_id.as_str()) {
                            token.cancel();
                        }
                        connected_streams.remove(&(client_id.clone(), stream_id.clone()));
                        break;
                    }
                }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn config_sync(
        &self,
        request: Request<ConfigSyncRequest>,
    ) -> Result<Response<ConfigSyncResponse>, Status> {
        let req = request.into_inner();
        let (rules, upstreams) = if let Some(group) = self.rules_engine.get_group(&req.group) {
            (group.rules.http.clone(), group.upstreams.clone())
        } else {
            (Vec::new(), std::collections::HashMap::new())
        };
        let proto_rules: Vec<ProtoRule> = rules.into_iter().map(|r| ProtoRule {
            rule_id: String::new(),
            r#type: String::new(),
            match_host: r.match_host.unwrap_or_default(),
            match_path_prefix: r.match_path_prefix.unwrap_or_default(),
            match_service: r.match_service.unwrap_or_default(),
            match_header: std::collections::HashMap::new(),
            action_proxy_pass: String::new(),
            action_set_host: String::new(),
            action_upstream: r.action_upstream.unwrap_or_default(),
            action_ssl: false,
        }).collect();
        let proto_upstreams: Vec<Upstream> = upstreams.into_iter().map(|(name, u)| Upstream {
            name,
            servers: u.servers.into_iter().map(|s| UpstreamServer {
                address: s.address,
                resolve: s.resolve,
            }).collect(),
            lb_policy: u.lb_policy.unwrap_or_else(|| "round_robin".to_string()),
        }).collect();
        let response = ConfigSyncResponse {
            config_version: "v1.0.0".to_string(),
            rules: proto_rules,
            upstreams: proto_upstreams,
        };
        Ok(Response::new(response))
    }
} 