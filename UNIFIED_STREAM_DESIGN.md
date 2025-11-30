# 统一 Tunnel Stream 设计：单 Stream 代理所有协议

## 🎯 设计目标

**核心需求**: 使用一条 client → server 的 gRPC bidirectional stream，代理所有 HTTP、WebSocket、gRPC 请求。

**概念澄清**:
- **gRPC Channel**: 底层 TCP 连接，可以复用
- **gRPC Stream**: 在 channel 上的双向流（bidirectional streaming RPC）
- **当前设计**: 一个 client 创建一个 channel，在这个 channel 上创建一个 Proxy stream，在这个 stream 上传输所有协议的消息

**优势**:
- ✅ **统一管理**: 一条 stream 管理所有协议
- ✅ **资源高效**: 减少 stream 数量，降低资源消耗
- ✅ **简化架构**: 统一的流管理逻辑
- ✅ **高性能**: 通过 `request_id` 完全并发处理

**注意**: 虽然一个 gRPC channel 可以创建多个 stream，但**推荐单 Stream 多协议**方案（性能最优，资源高效）。

详见: `STREAM_ARCHITECTURE_CLARIFICATION.md` - Channel vs Stream 详细说明

---

## 📋 架构设计

### 整体架构

```
外部客户端
    ↓
Server Entry (HTTP/WebSocket/gRPC)
    ↓
规则匹配 (RulesEngine)
    ↓
选择 Client Group
    ↓
统一的 Tunnel Stream (gRPC Bidirectional Streaming)
    ↓
Client 端接收并转发到后端服务
    ↓
响应通过相同的 Tunnel Stream 返回
```

### 关键设计点

1. **单 Stream 多协议**: 一条 tunnel stream 承载所有协议类型的消息
2. **request_id 隔离**: 每个请求有唯一的 `request_id`，完全并发处理
3. **协议标识**: 通过 `protocol_type` 和 `payload` 区分协议类型
4. **流式支持**: 支持 HTTP、WebSocket、gRPC 的流式传输
5. **长短请求共存**: HTTP 短请求、gRPC Streaming 长请求、WebSocket 长连接可以同时存在

### 如何同时代理长短请求？

**核心机制**: 通过 `request_id` 完全并发处理

- ✅ **HTTP 短请求**: `request_id` 匹配请求/响应，快速完成
- ✅ **gRPC Streaming 长请求**: 通过 `request_id` 维护 stream state，多个 chunk 并发处理
- ✅ **WebSocket 长连接**: 通过 `request_id` 维护连接状态，双向帧并发转发

**关键**: 所有请求通过 `request_id` 独立处理，**完全并发，互不阻塞**！

详见: `MULTI_PROTOCOL_STREAM_DESIGN.md` - 详细设计说明

---

## 🔧 Proto 定义

### 当前定义（需要扩展）

```protobuf
message TunnelMessage {
  string client_id = 1;
  string request_id = 2;  // ← 每个请求唯一标识
  Direction direction = 3;
  oneof payload {
    HttpRequest http_request = 4;
    HttpResponse http_response = 5;
    GrpcRequest grpc_request = 6;
    GrpcResponse grpc_response = 7;
    WebSocketRequest ws_request = 8;      // ← 需要添加
    WebSocketResponse ws_response = 9;    // ← 需要添加
    WebSocketFrame ws_frame = 10;         // ← 需要添加
    ConfigSyncRequest config_sync = 8;
    ConfigSyncResponse config_sync_response = 9;
    StreamOpenRequest stream_open = 10;
    StreamOpenResponse stream_open_response = 11;
    ErrorMessage error_message = 12;
  }
  string trace_id = 15;
}
```

### 改进方案

```protobuf
message TunnelMessage {
  string client_id = 1;
  string request_id = 2;  // 每个请求唯一标识（UUID）
  Direction direction = 3;
  ProtocolType protocol_type = 4;  // ← 新增：协议类型标识
  oneof payload {
    HttpRequest http_request = 5;
    HttpResponse http_response = 6;
    GrpcRequest grpc_request = 7;
    GrpcResponse grpc_response = 8;
    WebSocketRequest ws_request = 9;
    WebSocketResponse ws_response = 10;
    WebSocketFrame ws_frame = 11;
    ConfigSyncRequest config_sync = 12;
    ConfigSyncResponse config_sync_response = 13;
    StreamOpenRequest stream_open = 14;
    StreamOpenResponse stream_open_response = 15;
    ErrorMessage error_message = 16;
  }
  string trace_id = 17;
}

enum ProtocolType {
  PROTOCOL_UNSPECIFIED = 0;
  HTTP = 1;
  GRPC = 2;
  WEBSOCKET = 3;
  CONTROL = 4;  // 控制消息（心跳、配置同步等）
}

// HTTP 消息（保持现有定义）
message HttpRequest {
  string method = 1;
  string url = 2;
  string host = 3;
  string path = 4;
  string query = 5;
  map<string, string> headers = 6;
  bytes body = 7;
  bool is_first_chunk = 8;   // 流式传输标记
  bool is_last_chunk = 9;
}

message HttpResponse {
  int32 status_code = 1;
  map<string, string> headers = 2;
  bytes body = 3;
  bool is_first_chunk = 4;
  bool is_last_chunk = 5;
}

// gRPC 消息（改进）
message GrpcRequest {
  string service = 1;
  string method = 2;
  string host = 3;
  map<string, string> headers = 4;
  map<string, string> metadata = 5;
  bytes body = 6;
  bool is_first_chunk = 7;
  bool is_last_chunk = 8;
}

message GrpcResponse {
  int32 status_code = 1;
  map<string, string> headers = 2;
  bytes body = 3;
  bool is_first_chunk = 4;
  bool is_last_chunk = 5;
}

// WebSocket 消息（新增）
message WebSocketRequest {
  string url = 1;
  string host = 2;
  map<string, string> headers = 3;
  string subprotocol = 4;  // WebSocket 子协议
}

message WebSocketResponse {
  int32 status_code = 1;
  map<string, string> headers = 2;
  string subprotocol = 3;
  bool accepted = 4;  // 是否接受连接
}

message WebSocketFrame {
  bool fin = 1;        // FIN 标志
  uint32 opcode = 2;  // Opcode (0x1=text, 0x2=binary, 0x8=close, etc.)
  bool masked = 3;     // 是否掩码
  bytes payload = 4;   // 帧数据
}
```

---

## 🚀 完整实现方案

### 1. Server 端统一入口处理

```rust
use tonic::{Request, Response, Status};
use tokio::sync::{mpsc, oneshot};
use dashmap::DashMap;
use uuid::Uuid;

pub struct UnifiedTunnelServer {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    // 统一的 pending requests（所有协议共享）
    pub pending_requests: Arc<DashMap<String, PendingRequest>>,
}

#[derive(Clone)]
pub enum PendingRequest {
    Http(oneshot::Sender<HttpResponse>),
    Grpc(oneshot::Sender<GrpcResponse>),
    WebSocket(mpsc::Sender<WebSocketFrame>),  // WebSocket 是流式的
}

impl UnifiedTunnelServer {
    /// 统一的 proxy stream（处理所有协议）
    async fn proxy(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(10000);
        
        let pending_requests = self.pending_requests.clone();
        let rules_engine = self.rules_engine.clone();
        let client_registry = self.client_registry.clone();
        let semaphore = Arc::new(Semaphore::new(10000));
        
        // 消息接收任务（快速接收，不阻塞）
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                if tx.send(message).await.is_err() {
                    break;
                }
            }
        });
        
        // 完全并发处理任务池（所有协议统一处理）
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let pending_requests = pending_requests.clone();
                let rules_engine = rules_engine.clone();
                let client_registry = client_registry.clone();
                
                tokio::spawn(async move {
                    match message {
                        Ok(msg) => {
                            // 根据协议类型分发处理
                            match msg.protocol_type {
                                ProtocolType::Http => {
                                    self.handle_http_message(msg, &pending_requests).await;
                                }
                                ProtocolType::Grpc => {
                                    self.handle_grpc_message(msg, &pending_requests).await;
                                }
                                ProtocolType::Websocket => {
                                    self.handle_websocket_message(msg, &pending_requests).await;
                                }
                                ProtocolType::Control => {
                                    self.handle_control_message(msg).await;
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            tracing::error!("Stream error: {}", e);
                        }
                    }
                    drop(permit);
                });
            }
        });
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
    
    /// 处理 HTTP 消息
    async fn handle_http_message(
        &self,
        msg: TunnelMessage,
        pending_requests: &Arc<DashMap<String, PendingRequest>>,
    ) {
        match msg.payload {
            Some(Payload::HttpResponse(resp)) => {
                // 通过 request_id 匹配，完全并发
                if let Some((_, pending)) = pending_requests.remove(&msg.request_id) {
                    if let PendingRequest::Http(sender) = pending {
                        let _ = sender.send(resp);
                    }
                }
            }
            Some(Payload::HttpRequest(req)) => {
                // 处理 HTTP 请求
                self.process_http_request(req, msg.request_id).await;
            }
            _ => {}
        }
    }
    
    /// 处理 gRPC 消息
    async fn handle_grpc_message(
        &self,
        msg: TunnelMessage,
        pending_requests: &Arc<DashMap<String, PendingRequest>>,
    ) {
        match msg.payload {
            Some(Payload::GrpcResponse(resp)) => {
                // 通过 request_id 匹配，完全并发
                if let Some((_, pending)) = pending_requests.remove(&msg.request_id) {
                    if let PendingRequest::Grpc(sender) = pending {
                        let _ = sender.send(resp);
                    }
                }
            }
            Some(Payload::GrpcRequest(req)) => {
                // 处理 gRPC 请求
                self.process_grpc_request(req, msg.request_id).await;
            }
            _ => {}
        }
    }
    
    /// 处理 WebSocket 消息
    async fn handle_websocket_message(
        &self,
        msg: TunnelMessage,
        pending_requests: &Arc<DashMap<String, PendingRequest>>,
    ) {
        match msg.payload {
            Some(Payload::WebSocketResponse(resp)) => {
                // WebSocket 握手响应
                if let Some((_, pending)) = pending_requests.remove(&msg.request_id) {
                    if let PendingRequest::WebSocket(sender) = pending {
                        // WebSocket 连接建立，开始转发帧
                        // 这里需要特殊处理
                    }
                }
            }
            Some(Payload::WebSocketFrame(frame)) => {
                // WebSocket 数据帧（双向）
                // 通过 request_id 找到对应的 WebSocket 连接
                if let Some((_, pending)) = pending_requests.get(&msg.request_id) {
                    if let PendingRequest::WebSocket(sender) = pending {
                        let _ = sender.send(frame).await;
                    }
                }
            }
            Some(Payload::WebSocketRequest(req)) => {
                // 处理 WebSocket 握手请求
                self.process_websocket_request(req, msg.request_id).await;
            }
            _ => {}
        }
    }
}
```

### 2. Server Entry Handlers（统一使用 Tunnel Stream）

#### HTTP Entry Handler

```rust
pub struct ServerHttpEntryTarget {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub pending_requests: Arc<DashMap<String, PendingRequest>>,
}

impl HttpEntryProxyTarget for ServerHttpEntryTarget {
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        _ctx: &HttpTunnelContext,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        // 规则匹配
        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
        let path = req.uri().path();
        
        if let Some(rule) = self.rules_engine.match_reverse_proxy_rule(host, path, None) {
            if let Some(group) = &rule.action_client_group {
                // 选择健康的 client stream（统一的 stream）
                let healthy_streams = self.client_registry.get_healthy_streams_in_group(
                    group,
                    None,  // 不限制 stream_type，使用统一的 stream
                    60,
                );
                
                if healthy_streams.is_empty() {
                    return Ok(resp_502());
                }
                
                let (client_id, _, stream_id) = &healthy_streams[0];
                
                // 获取统一的 tunnel stream sender
                if let Some((tx, _, _)) = self.client_registry.get_stream_info(
                    group,
                    StreamType::Unspecified,  // 使用统一的 stream
                    client_id,
                    stream_id,
                ) {
                    let request_id = Uuid::new_v4().to_string();
                    let (resp_tx, resp_rx) = oneshot::channel();
                    
                    // 注册 pending request
                    self.pending_requests.insert(
                        request_id.clone(),
                        PendingRequest::Http(resp_tx),
                    );
                    
                    // 构建 HTTP 请求消息
                    let tunnel_msg = TunnelMessage {
                        client_id: client_id.clone(),
                        request_id: request_id.clone(),
                        direction: Direction::ServerToClient as i32,
                        protocol_type: ProtocolType::Http as i32,
                        payload: Some(Payload::HttpRequest(build_http_request(req))),
                        trace_id: Uuid::new_v4().to_string(),
                    };
                    
                    // 发送到统一的 tunnel stream
                    if tx.send(tunnel_msg).await.is_err() {
                        self.pending_requests.remove(&request_id);
                        return Ok(resp_502());
                    }
                    
                    // 等待响应
                    match timeout(Duration::from_secs(30), resp_rx).await {
                        Ok(Ok(resp)) => {
                            // 构建 HTTP 响应
                            build_hyper_response(resp)
                        }
                        _ => Ok(resp_502()),
                    }
                } else {
                    Ok(resp_502())
                }
            } else {
                Ok(resp_404())
            }
        } else {
            Ok(resp_404())
        }
    }
}
```

#### gRPC Entry Handler

```rust
pub struct ServerGrpcEntryTarget {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub pending_requests: Arc<DashMap<String, PendingRequest>>,
}

impl ServerGrpcEntryTarget {
    pub async fn handle_grpc_request(
        &self,
        request: Request<tonic::Streaming<Bytes>>,
    ) -> Result<Response<tonic::Streaming<Bytes>>, Status> {
        let (parts, mut body_stream) = request.into_parts();
        let (service, method) = parse_grpc_path(parts.uri.path())?;
        
        // 规则匹配
        let host = parts.headers.get(":authority")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        
        if let Some(rule) = self.rules_engine.match_reverse_proxy_rule(host, "", Some(&service)) {
            if let Some(group) = &rule.action_client_group {
                // 选择统一的 tunnel stream
                let healthy_streams = self.client_registry.get_healthy_streams_in_group(
                    group,
                    None,
                    60,
                );
                
                if healthy_streams.is_empty() {
                    return Err(Status::unavailable("No healthy client"));
                }
                
                let (client_id, _, stream_id) = &healthy_streams[0];
                let request_id = Uuid::new_v4().to_string();
                
                // 获取统一的 tunnel stream sender
                if let Some((tx, _, _)) = self.client_registry.get_stream_info(
                    group,
                    StreamType::Unspecified,
                    client_id,
                    stream_id,
                ) {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    
                    // 注册 pending request
                    self.pending_requests.insert(
                        request_id.clone(),
                        PendingRequest::Grpc(resp_tx),
                    );
                    
                    // 发送 gRPC 请求（流式）
                    let mut is_first = true;
                    while let Some(chunk) = body_stream.next().await {
                        let chunk = chunk?;
                        let tunnel_msg = TunnelMessage {
                            client_id: client_id.clone(),
                            request_id: request_id.clone(),
                            direction: Direction::ServerToClient as i32,
                            protocol_type: ProtocolType::Grpc as i32,
                            payload: Some(Payload::GrpcRequest(GrpcRequest {
                                service: service.clone(),
                                method: method.clone(),
                                body: chunk.to_vec(),
                                is_first_chunk: is_first,
                                is_last_chunk: false,
                                // ...
                            })),
                            trace_id: Uuid::new_v4().to_string(),
                        };
                        
                        tx.send(tunnel_msg).await?;
                        is_first = false;
                    }
                    
                    // 等待响应
                    match resp_rx.await {
                        Ok(resp) => {
                            // 构建 gRPC 响应 stream
                            let response_stream = tokio_stream::once(Ok::<Bytes, Status>(
                                Bytes::from(resp.body)
                            ));
                            Ok(Response::new(Box::pin(response_stream)))
                        }
                        Err(_) => Err(Status::internal("Failed to receive response")),
                    }
                } else {
                    Err(Status::unavailable("Tunnel stream not found"))
                }
            } else {
                Err(Status::not_found("No matching rule"))
            }
        } else {
            Err(Status::not_found("No matching rule"))
        }
    }
}
```

#### WebSocket Entry Handler

```rust
pub struct ServerWebSocketEntryTarget {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub pending_requests: Arc<DashMap<String, PendingRequest>>,
}

impl ServerWebSocketEntryTarget {
    pub async fn handle_websocket_upgrade(
        &self,
        req: HyperRequest<Body>,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        // 规则匹配
        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
        let path = req.uri().path();
        
        if let Some(rule) = self.rules_engine.match_reverse_proxy_rule(host, path, None) {
            if let Some(group) = &rule.action_client_group {
                // 选择统一的 tunnel stream
                let healthy_streams = self.client_registry.get_healthy_streams_in_group(
                    group,
                    None,
                    60,
                );
                
                if healthy_streams.is_empty() {
                    return Ok(resp_502());
                }
                
                let (client_id, _, stream_id) = &healthy_streams[0];
                let request_id = Uuid::new_v4().to_string();
                
                // 获取统一的 tunnel stream sender
                if let Some((tx, _, _)) = self.client_registry.get_stream_info(
                    group,
                    StreamType::Unspecified,
                    client_id,
                    stream_id,
                ) {
                    // 创建 WebSocket 帧 channel（双向）
                    let (ws_tx, mut ws_rx) = mpsc::channel::<WebSocketFrame>(100);
                    
                    // 注册 pending request（WebSocket 是长连接）
                    self.pending_requests.insert(
                        request_id.clone(),
                        PendingRequest::WebSocket(ws_tx.clone()),
                    );
                    
                    // 发送 WebSocket 握手请求
                    let tunnel_msg = TunnelMessage {
                        client_id: client_id.clone(),
                        request_id: request_id.clone(),
                        direction: Direction::ServerToClient as i32,
                        protocol_type: ProtocolType::Websocket as i32,
                        payload: Some(Payload::WebSocketRequest(WebSocketRequest {
                            url: req.uri().to_string(),
                            host: host.to_string(),
                            headers: extract_headers(&req),
                            subprotocol: String::new(),
                        })),
                        trace_id: Uuid::new_v4().to_string(),
                    };
                    
                    if tx.send(tunnel_msg).await.is_err() {
                        self.pending_requests.remove(&request_id);
                        return Ok(resp_502());
                    }
                    
                    // 等待握手响应
                    // 然后升级到 WebSocket 连接
                    // 实现 WebSocket 帧的双向转发
                    // ...
                }
            }
        }
        
        Ok(resp_404())
    }
}
```

### 3. Client 端统一处理

```rust
impl TunnelClient {
    /// 统一的消息处理（所有协议）
    pub async fn handle_tunnel_message(
        &self,
        msg: TunnelMessage,
        tx: &mpsc::Sender<TunnelMessage>,
    ) {
        match msg.protocol_type {
            ProtocolType::Http => {
                self.handle_http_message(msg, tx).await;
            }
            ProtocolType::Grpc => {
                self.handle_grpc_message(msg, tx).await;
            }
            ProtocolType::Websocket => {
                self.handle_websocket_message(msg, tx).await;
            }
            ProtocolType::Control => {
                self.handle_control_message(msg, tx).await;
            }
            _ => {}
        }
    }
    
    /// 处理 HTTP 消息
    async fn handle_http_message(
        &self,
        msg: TunnelMessage,
        tx: &mpsc::Sender<TunnelMessage>,
    ) {
        match msg.payload {
            Some(Payload::HttpRequest(req)) => {
                // 处理 HTTP 请求，转发到后端
                let resp = self.forward_http_to_backend(req).await;
                
                // 发送响应
                let response_msg = TunnelMessage {
                    client_id: msg.client_id.clone(),
                    request_id: msg.request_id.clone(),
                    direction: Direction::ClientToServer as i32,
                    protocol_type: ProtocolType::Http as i32,
                    payload: Some(Payload::HttpResponse(resp)),
                    trace_id: msg.trace_id.clone(),
                };
                
                let _ = tx.send(response_msg).await;
            }
            Some(Payload::HttpResponse(resp)) => {
                // HTTP 响应（从 server 返回给 client 的 HTTP entry）
                if let Some((_, pending)) = self.pending_requests.remove(&msg.request_id) {
                    if let PendingRequest::Http(sender) = pending {
                        let _ = sender.send(resp);
                    }
                }
            }
            _ => {}
        }
    }
    
    /// 处理 gRPC 消息
    async fn handle_grpc_message(
        &self,
        msg: TunnelMessage,
        tx: &mpsc::Sender<TunnelMessage>,
    ) {
        match msg.payload {
            Some(Payload::GrpcRequest(req)) => {
                // 处理 gRPC 请求，转发到后端
                let resp = self.forward_grpc_to_backend(req).await;
                
                // 发送响应
                let response_msg = TunnelMessage {
                    client_id: msg.client_id.clone(),
                    request_id: msg.request_id.clone(),
                    direction: Direction::ClientToServer as i32,
                    protocol_type: ProtocolType::Grpc as i32,
                    payload: Some(Payload::GrpcResponse(resp)),
                    trace_id: msg.trace_id.clone(),
                };
                
                let _ = tx.send(response_msg).await;
            }
            Some(Payload::GrpcResponse(resp)) => {
                // gRPC 响应
                if let Some((_, pending)) = self.pending_requests.remove(&msg.request_id) {
                    if let PendingRequest::Grpc(sender) = pending {
                        let _ = sender.send(resp);
                    }
                }
            }
            _ => {}
        }
    }
    
    /// 处理 WebSocket 消息
    async fn handle_websocket_message(
        &self,
        msg: TunnelMessage,
        tx: &mpsc::Sender<TunnelMessage>,
    ) {
        match msg.payload {
            Some(Payload::WebSocketRequest(req)) => {
                // WebSocket 握手请求
                let resp = self.handle_websocket_upgrade(req).await;
                
                // 发送握手响应
                let response_msg = TunnelMessage {
                    client_id: msg.client_id.clone(),
                    request_id: msg.request_id.clone(),
                    direction: Direction::ClientToServer as i32,
                    protocol_type: ProtocolType::Websocket as i32,
                    payload: Some(Payload::WebSocketResponse(resp)),
                    trace_id: msg.trace_id.clone(),
                };
                
                let _ = tx.send(response_msg).await;
                
                // 建立 WebSocket 连接后，开始双向转发帧
                // ...
            }
            Some(Payload::WebSocketFrame(frame)) => {
                // WebSocket 数据帧（双向转发）
                // 通过 request_id 找到对应的 WebSocket 连接
                // 转发帧到后端或前端
                // ...
            }
            _ => {}
        }
    }
}
```

---

## ✅ 方案优势

### 1. 统一管理

- ✅ **单 Stream**: 一条 tunnel stream 承载所有协议
- ✅ **统一处理**: 相同的消息处理逻辑
- ✅ **简化架构**: 减少连接管理复杂度

### 2. 高性能

- ✅ **完全并发**: 通过 `request_id` 完全并发处理
- ✅ **无阻塞**: 不同协议的请求互不影响
- ✅ **资源高效**: 减少连接数，降低资源消耗

### 3. 易于扩展

- ✅ **协议无关**: 新增协议只需添加新的 payload 类型
- ✅ **统一接口**: 所有协议使用相同的 tunnel stream
- ✅ **灵活配置**: 支持不同协议的规则配置

---

## 📊 性能对比

| 方案 | 连接数 | 吞吐量 | 复杂度 | 资源消耗 |
|------|--------|--------|--------|----------|
| **多 Stream（每协议一个）** | 高 | 高 | 中 | 高 |
| **统一 Stream（单 Stream 多协议）** | **低** | **高** | **低** | **低** |

---

## 🔧 实现要点

### 1. Stream 管理

```rust
// 每个 client 只需要一个统一的 stream
pub struct ManagedClientRegistry {
    // group -> (client_id, stream_id) -> (tx, token, last_heartbeat)
    pub connected_streams: DashMap<String, DashMap<(String, String), StreamInfo>>,
}

// StreamInfo 包含统一的 tunnel stream sender
pub struct StreamInfo {
    pub tx: mpsc::Sender<TunnelMessage>,  // 统一的发送通道
    pub token: CancellationToken,
    pub last_heartbeat: u64,
}
```

### 2. 消息路由

```rust
// 根据 protocol_type 路由到不同的处理器
match msg.protocol_type {
    ProtocolType::Http => http_handler.handle(msg).await,
    ProtocolType::Grpc => grpc_handler.handle(msg).await,
    ProtocolType::Websocket => websocket_handler.handle(msg).await,
    _ => {}
}
```

### 3. 并发处理

```rust
// 所有协议的消息都通过 request_id 完全并发处理
// 不需要有序性保证，因为每个请求有唯一的 request_id
```

---

## 📝 实施步骤

### 阶段 1: Proto 定义（1 天）

1. 添加 `ProtocolType` 枚举
2. 添加 WebSocket 消息定义
3. 扩展 `TunnelMessage` 支持多协议

### 阶段 2: Server 端实现（2-3 天）

1. 实现统一的 `proxy` stream 处理
2. 实现 HTTP Entry Handler（使用统一 stream）
3. 实现 gRPC Entry Handler（使用统一 stream）
4. 实现 WebSocket Entry Handler（使用统一 stream）

### 阶段 3: Client 端实现（2-3 天）

1. 实现统一的消息处理分发
2. 实现各协议的后端转发逻辑
3. 实现 WebSocket 双向帧转发

### 阶段 4: 测试验证（1-2 天）

1. HTTP 请求/响应测试
2. gRPC 请求/响应测试
3. WebSocket 连接和帧转发测试
4. 并发性能测试

---

## 🎯 总结

**核心设计**: 使用一条 client → server 的 gRPC bidirectional stream，通过 `request_id` 和 `protocol_type` 区分不同的请求和协议类型，完全并发处理。

**关键优势**:
- ✅ 统一管理，简化架构
- ✅ 资源高效，性能最优
- ✅ 易于扩展，支持新协议
- ✅ 完全并发，无阻塞

**适用场景**:
- ✅ HTTP 请求/响应
- ✅ gRPC Unary 和 Streaming
- ✅ WebSocket 双向通信
- ✅ 未来其他协议扩展

