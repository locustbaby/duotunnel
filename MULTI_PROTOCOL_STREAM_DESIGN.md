# 单 Stream 多协议代理详细设计

## 🎯 核心问题

**问题**: 如何在一个 gRPC bidirectional stream 中同时代理：
- HTTP（短请求，请求-响应）
- WebSocket（长连接，双向通信）
- gRPC（Unary 短请求 + Streaming 长请求）

**关键挑战**:
1. 如何区分不同的请求？
2. 如何保证长连接不阻塞短请求？
3. 如何保证 Streaming 的正确性？
4. **如何识别 WebSocket 的消息？**（多个连接，多个帧）
5. **如何准确识别和拆分 gRPC Streaming 内的请求？**（多个请求/响应）

---

## ✅ 解决方案：request_id 隔离 + 完全并发

### 核心设计思想

**关键洞察**: 
- ✅ 每个请求（无论协议类型）都有唯一的 `request_id`
- ✅ 通过 `request_id` 匹配请求/响应，完全并发处理
- ✅ 长连接通过 `request_id` 维护状态，但消息本身并发处理
- ✅ 不需要有序性保证，因为每个请求独立

---

## 📋 详细设计

### 1. 消息格式设计

```protobuf
message TunnelMessage {
  string client_id = 1;
  string request_id = 2;  // ← 关键：每个请求唯一标识
  Direction direction = 3;
  ProtocolType protocol_type = 4;  // ← 协议类型
  oneof payload {
    HttpRequest http_request = 5;
    HttpResponse http_response = 6;
    GrpcRequest grpc_request = 7;
    GrpcResponse grpc_response = 8;
    WebSocketRequest ws_request = 9;
    WebSocketResponse ws_response = 10;
    WebSocketFrame ws_frame = 11;
    // ...
  }
  string trace_id = 17;
}

enum ProtocolType {
  PROTOCOL_UNSPECIFIED = 0;
  HTTP = 1;
  GRPC = 2;
  WEBSOCKET = 3;
  CONTROL = 4;
}
```

### 2. 请求生命周期管理

#### HTTP 请求（短请求）

```
时间线：
T1: Server 收到 HTTP 请求
    → 生成 request_id = "http-req-1"
    → 发送 TunnelMessage { request_id: "http-req-1", protocol_type: HTTP, payload: HttpRequest }
    
T2: Client 收到消息
    → 通过 request_id = "http-req-1" 识别是 HTTP 请求
    → 转发到后端 HTTP 服务
    → 等待响应
    
T3: Client 收到后端响应
    → 发送 TunnelMessage { request_id: "http-req-1", protocol_type: HTTP, payload: HttpResponse }
    
T4: Server 收到响应
    → 通过 request_id = "http-req-1" 匹配到 pending request
    → 返回给客户端
    → 清理 pending request（完成）
```

**特点**:
- ✅ 短请求，请求-响应模式
- ✅ 通过 `request_id` 匹配，完全并发
- ✅ 处理完成后立即清理

#### gRPC Unary（短请求）

```
时间线：
T1: Server 收到 gRPC Unary 请求
    → 生成 request_id = "grpc-req-1"
    → 发送 TunnelMessage { request_id: "grpc-req-1", protocol_type: GRPC, payload: GrpcRequest }
    
T2: Client 收到消息
    → 通过 request_id = "grpc-req-1" 识别是 gRPC 请求
    → 转发到后端 gRPC 服务
    → 等待响应
    
T3: Client 收到后端响应
    → 发送 TunnelMessage { request_id: "grpc-req-1", protocol_type: GRPC, payload: GrpcResponse }
    
T4: Server 收到响应
    → 通过 request_id = "grpc-req-1" 匹配到 pending request
    → 返回给客户端
    → 清理 pending request（完成）
```

**特点**:
- ✅ 与 HTTP 类似，短请求
- ✅ 通过 `request_id` 匹配，完全并发

#### gRPC Streaming（长请求）

```
时间线：
T1: Server 收到 gRPC Streaming 请求
    → 生成 request_id = "grpc-stream-1"
    → 发送 TunnelMessage { request_id: "grpc-stream-1", protocol_type: GRPC, 
                            payload: GrpcRequest { is_first_chunk: true } }
    
T2-T5: 多个数据 chunk
    → TunnelMessage { request_id: "grpc-stream-1", payload: GrpcRequest { is_first_chunk: false, is_last_chunk: false } }
    → TunnelMessage { request_id: "grpc-stream-1", payload: GrpcRequest { is_first_chunk: false, is_last_chunk: false } }
    → ...
    
T6: 最后一个 chunk
    → TunnelMessage { request_id: "grpc-stream-1", payload: GrpcRequest { is_last_chunk: true } }
    
T7-T10: 多个响应 chunk
    → TunnelMessage { request_id: "grpc-stream-1", payload: GrpcResponse { is_first_chunk: true } }
    → TunnelMessage { request_id: "grpc-stream-1", payload: GrpcResponse { is_first_chunk: false, is_last_chunk: false } }
    → ...
    
T11: 最后一个响应 chunk
    → TunnelMessage { request_id: "grpc-stream-1", payload: GrpcResponse { is_last_chunk: true } }
    → 清理 pending request（完成）
```

**关键点**:
- ✅ 同一个 `request_id` 的多个 chunk 通过 `is_first_chunk` 和 `is_last_chunk` 标识
- ✅ **不需要有序性保证**：每个 chunk 独立处理，通过 `request_id` 组装
- ✅ Client 端维护 `request_id -> stream state` 映射，组装完整的流

#### WebSocket（长连接）

```
时间线：
T1: Server 收到 WebSocket 握手请求
    → 生成 request_id = "ws-conn-1"
    → 发送 TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                            payload: WebSocketRequest }
    
T2: Client 收到握手请求
    → 通过 request_id = "ws-conn-1" 识别是 WebSocket 请求
    → 转发到后端 WebSocket 服务
    → 建立 WebSocket 连接
    
T3: Client 收到握手响应
    → 发送 TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                            payload: WebSocketResponse }
    → **保持连接**（不清理 pending request）
    
T4-T100: 双向帧转发（长时间）
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketFrame { direction: CLIENT_TO_SERVER } }
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketFrame { direction: SERVER_TO_CLIENT } }
    → ...（持续转发，直到连接关闭）
    
T101: 连接关闭
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketFrame { opcode: CLOSE } }
    → 清理 pending request（完成）
```

**关键点**:
- ✅ WebSocket 是长连接，`request_id` 在整个连接生命周期内保持不变
- ✅ 通过 `request_id` 维护连接状态
- ✅ 双向帧转发，完全并发（不同 `request_id` 的 WebSocket 连接并发）

---

## 🚀 完整实现方案

### Server 端统一处理

```rust
async fn proxy(&self, request: Request<tonic::Streaming<TunnelMessage>>) -> Result<Response<Self::ProxyStream>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(10000);
    
    let pending_requests = self.pending_requests.clone();
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
            
            tokio::spawn(async move {
                match message {
                    Ok(msg) => {
                        // 根据协议类型和 payload 类型分发处理
                        match (msg.protocol_type, &msg.payload) {
                            (ProtocolType::Http, Some(Payload::HttpResponse(resp))) => {
                                // HTTP 响应：通过 request_id 匹配
                                if let Some((_, pending)) = pending_requests.remove(&msg.request_id) {
                                    if let PendingRequest::Http(sender) = pending {
                                        let _ = sender.send(resp);
                                    }
                                }
                            }
                            (ProtocolType::Http, Some(Payload::HttpRequest(req))) => {
                                // HTTP 请求：处理并转发
                                self.handle_http_request(req, msg.request_id).await;
                            }
                            (ProtocolType::Grpc, Some(Payload::GrpcResponse(resp))) => {
                                // gRPC 响应：通过 request_id 匹配
                                if let Some((_, pending)) = pending_requests.remove(&msg.request_id) {
                                    if let PendingRequest::Grpc(sender) = pending {
                                        let _ = sender.send(resp);
                                    }
                                }
                            }
                            (ProtocolType::Grpc, Some(Payload::GrpcRequest(req))) => {
                                // gRPC 请求：处理并转发
                                self.handle_grpc_request(req, msg.request_id).await;
                            }
                            (ProtocolType::Websocket, Some(Payload::WebSocketResponse(resp))) => {
                                // WebSocket 握手响应
                                if let Some((_, pending)) = pending_requests.get(&msg.request_id) {
                                    if let PendingRequest::WebSocket(sender) = pending {
                                        // 握手成功，开始转发帧
                                        // 注意：不 remove，保持连接
                                    }
                                }
                            }
                            (ProtocolType::Websocket, Some(Payload::WebSocketFrame(frame))) => {
                                // WebSocket 数据帧：通过 request_id 找到连接
                                if let Some((_, pending)) = pending_requests.get(&msg.request_id) {
                                    if let PendingRequest::WebSocket(sender) = pending {
                                        let _ = sender.send(frame).await;
                                    }
                                }
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
```

### Client 端统一处理

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
                // HTTP 请求：转发到后端
                let resp = self.forward_http_to_backend(req).await;
                
                // 发送响应
                let response_msg = TunnelMessage {
                    client_id: msg.client_id.clone(),
                    request_id: msg.request_id.clone(),  // ← 使用相同的 request_id
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
    
    /// 处理 gRPC 消息（支持 Streaming）
    async fn handle_grpc_message(
        &self,
        msg: TunnelMessage,
        tx: &mpsc::Sender<TunnelMessage>,
    ) {
        match msg.payload {
            Some(Payload::GrpcRequest(req)) => {
                // gRPC 请求：转发到后端
                if req.is_first_chunk {
                    // 第一个 chunk：建立 gRPC stream
                    self.start_grpc_stream(msg.request_id.clone(), req, tx).await;
                } else {
                    // 后续 chunk：转发到已建立的 stream
                    self.forward_grpc_chunk(msg.request_id.clone(), req, tx).await;
                }
            }
            Some(Payload::GrpcResponse(resp)) => {
                // gRPC 响应（从 server 返回给 client 的 gRPC entry）
                if resp.is_last_chunk {
                    // 最后一个 chunk：完成请求
                    if let Some((_, pending)) = self.pending_requests.remove(&msg.request_id) {
                        if let PendingRequest::Grpc(sender) = pending {
                            let _ = sender.send(resp);
                        }
                    }
                } else {
                    // 中间 chunk：继续转发
                    if let Some((_, pending)) = self.pending_requests.get(&msg.request_id) {
                        if let PendingRequest::GrpcStream(sender) = pending {
                            let _ = sender.send(resp).await;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    /// 处理 WebSocket 消息（长连接）
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
                    request_id: msg.request_id.clone(),  // ← 保持相同的 request_id
                    direction: Direction::ClientToServer as i32,
                    protocol_type: ProtocolType::Websocket as i32,
                    payload: Some(Payload::WebSocketResponse(resp)),
                    trace_id: msg.trace_id.clone(),
                };
                
                let _ = tx.send(response_msg).await;
                
                // 建立 WebSocket 连接后，开始双向转发帧
                self.start_websocket_forwarding(msg.request_id.clone(), tx).await;
            }
            Some(Payload::WebSocketFrame(frame)) => {
                // WebSocket 数据帧（双向转发）
                // 通过 request_id 找到对应的 WebSocket 连接
                if let Some((_, pending)) = self.pending_requests.get(&msg.request_id) {
                    if let PendingRequest::WebSocket(sender) = pending {
                        // 转发帧到前端或后端
                        self.forward_websocket_frame(msg.request_id.clone(), frame, sender, tx).await;
                    }
                }
            }
            _ => {}
        }
    }
}
```

---

## 🔑 关键设计点

### 1. request_id 的作用

**HTTP 短请求**:
```
request_id = "http-req-1"
  → 请求发送
  → 响应匹配（通过 request_id）
  → 清理（完成）
```

**gRPC Streaming 长请求**:
```
request_id = "grpc-stream-1"
  → 第一个 chunk（is_first_chunk = true）
  → 多个中间 chunk（is_first_chunk = false, is_last_chunk = false）
  → 最后一个 chunk（is_last_chunk = true）
  → 响应 chunk（通过 request_id 匹配）
  → 清理（完成）
```

**WebSocket 长连接**:
```
request_id = "ws-conn-1"
  → 握手请求
  → 握手响应
  → 多个数据帧（持续转发）
  → 关闭帧
  → 清理（完成）
```

### 2. 并发处理保证

**关键**: 所有消息通过 `request_id` 完全并发处理

```rust
// 场景：同时有 HTTP、gRPC、WebSocket 请求
T1: HTTP 请求 (request_id = "http-1")
T2: gRPC Streaming 请求 (request_id = "grpc-1")
T3: WebSocket 握手 (request_id = "ws-1")
T4: HTTP 响应 (request_id = "http-1") ← 先到达
T5: gRPC chunk (request_id = "grpc-1")
T6: WebSocket 帧 (request_id = "ws-1")
T7: HTTP 请求 (request_id = "http-2")
T8: gRPC chunk (request_id = "grpc-1")
```

**处理结果**:
- ✅ 所有消息并发处理，不阻塞
- ✅ 通过 `request_id` 正确匹配
- ✅ HTTP 短请求快速完成，不受长请求影响
- ✅ WebSocket 长连接持续转发，不影响其他请求

### 3. 状态管理

#### HTTP（无状态）
```rust
// 请求发送时注册
pending_requests.insert(request_id, PendingRequest::Http(sender));

// 响应到达时匹配并清理
if let Some((_, pending)) = pending_requests.remove(&request_id) {
    // 发送响应
    // 自动清理
}
```

#### gRPC Streaming（有状态，但通过 request_id 管理）
```rust
// 第一个 chunk：建立 stream state
grpc_streams.insert(request_id.clone(), GrpcStreamState {
    backend_stream: backend_grpc_stream,
    chunks: VecDeque::new(),
});

// 后续 chunk：添加到 state
if let Some(state) = grpc_streams.get_mut(&request_id) {
    state.chunks.push_back(chunk);
}

// 最后一个 chunk：完成并清理
if is_last_chunk {
    grpc_streams.remove(&request_id);
}
```

#### WebSocket（长连接状态）
```rust
// 握手时注册（不清理）
pending_requests.insert(request_id.clone(), PendingRequest::WebSocket(sender));

// 数据帧转发（持续使用）
if let Some((_, pending)) = pending_requests.get(&request_id) {
    // 转发帧
}

// 关闭时清理
if frame.opcode == CLOSE {
    pending_requests.remove(&request_id);
}
```

---

## ✅ 方案优势

### 1. 完全并发

- ✅ **短请求不阻塞**: HTTP 和 gRPC Unary 快速完成
- ✅ **长请求不阻塞**: WebSocket 和 gRPC Streaming 不影响其他请求
- ✅ **请求独立**: 每个 `request_id` 独立处理

### 2. 资源高效

- ✅ **单 Stream**: 只需要一个 tunnel stream
- ✅ **状态管理**: 通过 `request_id` 管理状态，内存占用可控
- ✅ **自动清理**: 请求完成后自动清理状态

### 3. 实现简单

- ✅ **统一处理**: 所有协议使用相同的并发模式
- ✅ **协议无关**: 通过 `protocol_type` 和 `payload` 区分协议
- ✅ **易于扩展**: 新增协议只需添加新的 payload 类型

---

## 📊 并发场景示例

### 场景：同时处理多种请求

```
时间线：
T1: HTTP 请求 A (request_id = "http-A")
T2: gRPC Streaming 请求 B (request_id = "grpc-B")
T3: WebSocket 连接 C (request_id = "ws-C")
T4: HTTP 请求 D (request_id = "http-D")
T5: HTTP 响应 A (request_id = "http-A") ← 快速返回
T6: gRPC chunk B-1 (request_id = "grpc-B")
T7: WebSocket 帧 C-1 (request_id = "ws-C")
T8: HTTP 响应 D (request_id = "http-D") ← 快速返回
T9: gRPC chunk B-2 (request_id = "grpc-B")
T10: WebSocket 帧 C-2 (request_id = "ws-C")
...
```

**处理结果**:
- ✅ HTTP 请求 A 和 D 快速完成（T5, T8）
- ✅ gRPC Streaming B 持续处理（T6, T9, ...）
- ✅ WebSocket C 持续转发（T7, T10, ...）
- ✅ **所有请求完全并发，互不影响**

---

## 🎯 总结

### 核心设计

**单 Stream 多协议代理**:
- ✅ 一个 gRPC bidirectional stream
- ✅ 通过 `request_id` 区分不同的请求
- ✅ 通过 `protocol_type` 区分协议类型
- ✅ **完全并发处理**，不需要有序性保证

### 关键保证

1. **短请求快速完成**: HTTP 和 gRPC Unary 不受长请求影响
2. **长请求持续处理**: WebSocket 和 gRPC Streaming 通过 `request_id` 维护状态
3. **完全并发**: 所有请求独立处理，互不阻塞
4. **资源高效**: 单 stream，状态管理可控

### 实现要点

- ✅ `request_id` 唯一性（UUID）
- ✅ `protocol_type` 协议标识
- ✅ `pending_requests` 统一管理（所有协议共享）
- ✅ 完全并发处理（通过 `request_id` 匹配）

---

**结论**: 单 Stream 完全可以同时代理 HTTP、WebSocket、gRPC（长短请求），通过 `request_id` 完全并发处理，性能最优！

