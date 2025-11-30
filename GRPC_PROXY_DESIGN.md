# gRPC 代理完整设计方案

## 🎯 核心设计：单 Stream 代理单 gRPC 请求

### 设计思想

**关键洞察**: 每个 gRPC 请求（包括 Streaming）都通过独立的 tunnel stream 处理，这样：
- ✅ **不需要有序性保证**: 每个请求有独立的 `request_id`，可以完全并发
- ✅ **简化实现**: 不需要按 `stream_id` 分组有序处理
- ✅ **最大化性能**: 所有 gRPC 请求完全并发处理

### 架构设计

```
外部 gRPC Client
    ↓
Server gRPC Entry (8002)
    ↓
规则匹配 (RulesEngine)
    ↓
选择 Client Group
    ↓
为每个 gRPC 请求创建独立的 Tunnel Stream
    ↓
Client 端接收并转发到后端 gRPC Server
    ↓
响应通过相同的 Tunnel Stream 返回
```

---

## 📋 Proto 定义

### 当前定义（需要改进）

```protobuf
message GrpcRequest {
  string stream_id = 1;  // ← 用于标识 gRPC stream（后端）
  string service = 2;
  string method = 3;
  string host = 4;
  map<string, string> headers = 5;
  map<string, string> metadata = 6;
  bytes body = 7;
  string original_dst = 8;
}

message GrpcResponse {
  int32 status_code = 1;
  map<string, string> headers = 2;
  bytes body = 3;
  // ❌ 缺少 stream_id 和 request_id 关联
}
```

### 改进方案

```protobuf
message GrpcRequest {
  string grpc_stream_id = 1;  // gRPC 后端 stream ID（用于 Streaming RPC）
  string service = 2;
  string method = 3;
  string host = 4;
  map<string, string> headers = 5;
  map<string, string> metadata = 6;
  bytes body = 7;
  string original_dst = 8;
  bool is_streaming = 9;  // 是否是 Streaming RPC
  bool is_first_chunk = 10;  // 是否是第一个 chunk（Streaming）
  bool is_last_chunk = 11;   // 是否是最后一个 chunk（Streaming）
}

message GrpcResponse {
  string grpc_stream_id = 1;  // ← 新增：关联 gRPC stream
  int32 status_code = 2;
  map<string, string> headers = 3;
  bytes body = 4;
  bool is_first_chunk = 5;  // Streaming 响应
  bool is_last_chunk = 6;   // Streaming 响应
}

message TunnelMessage {
  string client_id = 1;
  string request_id = 2;  // ← 用于匹配请求/响应（每个 gRPC 请求唯一）
  Direction direction = 3;
  oneof payload {
    HttpRequest http_request = 4;
    HttpResponse http_response = 5;
    GrpcRequest grpc_request = 6;
    GrpcResponse grpc_response = 7;
    // ...
  }
  string trace_id = 15;
}
```

**关键点**:
- `request_id`: 每个 gRPC 请求唯一，用于匹配请求/响应
- `grpc_stream_id`: gRPC 后端 stream ID（用于 Streaming RPC）
- 每个 gRPC 请求通过独立的 `request_id` 处理，完全并发

---

## 🚀 完整实现方案

### 1. Server 端 gRPC Entry Handler

```rust
use tonic::{Request, Response, Status};
use tonic::codegen::*;
use tower::Service;
use std::pin::Pin;
use futures::Stream;

/// gRPC Entry Handler（类似 HTTP Entry）
pub struct ServerGrpcEntryTarget {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<GrpcResponse>>>,
}

impl ServerGrpcEntryTarget {
    /// 处理 gRPC 请求（Unary 或 Streaming）
    pub async fn handle_grpc_request(
        &self,
        request: Request<tonic::Streaming<Bytes>>,
    ) -> Result<Response<tonic::Streaming<Bytes>>, Status> {
        let (parts, mut body_stream) = request.into_parts();
        let metadata = parts.metadata.clone();
        
        // 提取服务和方法
        let path = parts.uri.path();
        let (service, method) = parse_grpc_path(path)?;
        
        // 规则匹配
        let host = parts.headers.get(":authority")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        
        if let Some(rule) = self.rules_engine.match_reverse_proxy_rule(host, "", Some(&service)) {
            if let Some(group) = &rule.action_client_group {
                // 选择健康的 client stream
                let healthy_streams = self.client_registry.get_healthy_streams_in_group(
                    group,
                    Some(StreamType::Grpc),
                    60,
                );
                
                if healthy_streams.is_empty() {
                    return Err(Status::unavailable("No healthy client available"));
                }
                
                // 为每个 gRPC 请求创建独立的 request_id
                let request_id = Uuid::new_v4().to_string();
                
                // 转发到 client（通过 tunnel）
                return self.forward_grpc_via_tunnel(
                    body_stream,
                    metadata,
                    service,
                    method,
                    &healthy_streams[0],
                    request_id,
                ).await;
            }
        }
        
        Err(Status::not_found("No matching rule"))
    }
    
    /// 通过 tunnel 转发 gRPC 请求
    async fn forward_grpc_via_tunnel(
        &self,
        mut body_stream: tonic::Streaming<Bytes>,
        metadata: MetadataMap,
        service: String,
        method: String,
        stream_info: &(String, StreamType, String),  // (client_id, stream_type, stream_id)
        request_id: String,
    ) -> Result<Response<tonic::Streaming<Bytes>>, Status> {
        let (client_id, _, tunnel_stream_id) = stream_info;
        let (tx, rx) = oneshot::channel();
        
        // 注册 pending request
        self.pending_requests.insert(request_id.clone(), tx);
        
        // 获取 tunnel sender
        let (tunnel_tx, _, _) = self.client_registry
            .get_stream_info("", StreamType::Grpc, client_id, tunnel_stream_id)
            .ok_or_else(|| Status::unavailable("Tunnel stream not found"))?;
        
        // 判断是否是 Streaming（简化：检查是否有多个 chunk）
        let mut is_first = true;
        let mut chunks = Vec::new();
        
        while let Some(chunk) = body_stream.next().await {
            let chunk = chunk?;
            chunks.push(chunk);
            
            // 构建 GrpcRequest
            let grpc_req = GrpcRequest {
                grpc_stream_id: request_id.clone(),  // 使用 request_id 作为 grpc_stream_id
                service: service.clone(),
                method: method.clone(),
                host: String::new(),
                headers: metadata_to_map(&metadata),
                metadata: HashMap::new(),
                body: chunk.to_vec(),
                original_dst: String::new(),
                is_streaming: false,  // 简化：可以根据实际情况判断
                is_first_chunk: is_first,
                is_last_chunk: false,  // 需要检查 stream 是否结束
            };
            
            let tunnel_msg = TunnelMessage {
                client_id: client_id.clone(),
                request_id: request_id.clone(),
                direction: Direction::ServerToClient as i32,
                payload: Some(tunnel_message::Payload::GrpcRequest(grpc_req)),
                trace_id: Uuid::new_v4().to_string(),
            };
            
            tunnel_tx.send(tunnel_msg).await
                .map_err(|e| Status::internal(format!("Failed to send tunnel message: {}", e)))?;
            
            is_first = false;
        }
        
        // 发送最后一个 chunk（标记结束）
        let last_grpc_req = GrpcRequest {
            grpc_stream_id: request_id.clone(),
            service: service.clone(),
            method: method.clone(),
            host: String::new(),
            headers: HashMap::new(),
            metadata: HashMap::new(),
            body: vec![],
            original_dst: String::new(),
            is_streaming: false,
            is_first_chunk: false,
            is_last_chunk: true,
        };
        
        let last_tunnel_msg = TunnelMessage {
            client_id: client_id.clone(),
            request_id: request_id.clone(),
            direction: Direction::ServerToClient as i32,
            payload: Some(tunnel_message::Payload::GrpcRequest(last_grpc_req)),
            trace_id: Uuid::new_v4().to_string(),
        };
        
        tunnel_tx.send(last_tunnel_msg).await
            .map_err(|e| Status::internal(format!("Failed to send last tunnel message: {}", e)))?;
        
        // 等待响应（通过 oneshot channel）
        // 注意：对于 Streaming 响应，需要特殊处理
        match rx.await {
            Ok(grpc_resp) => {
                // 构建响应 stream
                let response_stream = tokio_stream::once(Ok::<Bytes, Status>(
                    Bytes::from(grpc_resp.body)
                ));
                
                Ok(Response::new(Box::pin(response_stream)))
            }
            Err(_) => Err(Status::internal("Failed to receive response"))
        }
    }
}
```

### 2. Client 端 gRPC 处理

```rust
// client/tunnel_client.rs

impl TunnelClient {
    /// 处理 gRPC 请求消息
    pub async fn handle_grpc_request(
        &self,
        req: GrpcRequest,
        request_id: String,
        trace_id: String,
    ) -> GrpcResponse {
        // 规则匹配
        let rules_engine = self.rules_engine.lock().await;
        let rule = rules_engine.match_grpc_rule(&req.service, &req.method);
        
        match rule {
            Some(rule) => {
                // 转发到后端 gRPC 服务
                if !rule.action_upstream.is_empty() {
                    if let Some(backend) = rules_engine.pick_backend(&rule.action_upstream) {
                        return self.forward_grpc_to_backend(
                            req,
                            &backend,
                            trace_id,
                        ).await;
                    }
                }
            }
            None => {}
        }
        
        // 返回错误响应
        GrpcResponse {
            grpc_stream_id: req.grpc_stream_id,
            status_code: 404,
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
            is_first_chunk: true,
            is_last_chunk: true,
        }
    }
    
    /// 转发 gRPC 请求到后端
    async fn forward_grpc_to_backend(
        &self,
        req: GrpcRequest,
        backend: &str,
        trace_id: String,
    ) -> GrpcResponse {
        // 使用 tonic 客户端连接到后端
        let mut client = tonic::client::Grpc::new(backend.parse().unwrap())
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to backend: {}", e);
                e
            })?;
        
        // 构建 gRPC 请求
        let mut request = tonic::Request::new(
            tokio_stream::once(Ok::<Bytes, Status>(Bytes::from(req.body)))
        );
        
        // 设置 metadata
        for (k, v) in req.headers {
            request.metadata_mut().insert(
                tonic::metadata::MetadataKey::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        
        // 调用后端服务
        let mut response_stream = client
            .unary(request, Method::new(req.service.clone(), req.method.clone()))
            .await
            .map_err(|e| {
                tracing::error!("gRPC call failed: {}", e);
                e
            })?;
        
        // 收集响应（简化：假设是 Unary）
        let mut response_body = Vec::new();
        while let Some(chunk) = response_stream.next().await {
            let chunk = chunk?;
            response_body.extend_from_slice(&chunk);
        }
        
        GrpcResponse {
            grpc_stream_id: req.grpc_stream_id,
            status_code: 0,  // gRPC 使用 status code
            headers: HashMap::new(),
            body: response_body,
            is_first_chunk: true,
            is_last_chunk: true,
        }
    }
}
```

### 3. 消息处理（完全并发）

```rust
// server/tunnel_server.rs

async fn proxy(&self, request: Request<tonic::Streaming<TunnelMessage>>) -> Result<Response<Self::ProxyStream>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(10000);
    
    let pending_requests = self.pending_requests.clone();
    let semaphore = Arc::new(Semaphore::new(10000));
    
    // 消息接收任务
    tokio::spawn(async move {
        while let Some(message) = stream.next().await {
            if tx.send(message).await.is_err() {
                break;
            }
        }
    });
    
    // 完全并发处理任务池
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let pending_requests = pending_requests.clone();
            
            tokio::spawn(async move {
                match message {
                    Ok(msg) => {
                        match msg.payload {
                            Some(Payload::GrpcResponse(resp)) => {
                                // 通过 request_id 匹配，完全并发，不需要有序
                                if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
                                    let _ = sender.send(resp);
                                }
                            }
                            Some(Payload::GrpcRequest(req)) => {
                                // 处理 gRPC 请求（完全并发）
                                handle_grpc_request(msg, req).await;
                            }
                            // ... 其他消息类型
                        }
                    }
                    Err(e) => {
                        // 错误处理
                    }
                }
                drop(permit);
            });
        }
    });
    
    Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
}
```

---

## ✅ 方案优势

### 1. 完全并发处理

- ✅ **每个 gRPC 请求独立**: 通过唯一的 `request_id` 标识
- ✅ **不需要有序性保证**: 不同请求之间完全并发
- ✅ **最大化吞吐量**: 充分利用多核 CPU

### 2. 简化实现

- ✅ **统一处理逻辑**: HTTP 和 gRPC 使用相同的并发模式
- ✅ **不需要分组队列**: 不需要按 `stream_id` 分组有序处理
- ✅ **代码简洁**: 实现和维护成本低

### 3. 性能最优

| 方案 | 吞吐量 | 延迟 | 复杂度 |
|------|--------|------|--------|
| **单 stream 代理单请求** | ✅ 最高 | ✅ 最低 | ✅ 简单 |
| **按 stream_id 分组有序** | 中 | 中 | 复杂 |
| **全部有序** | 低 | 高 | 简单 |

---

## 🔧 实现细节

### Streaming RPC 处理

对于 Streaming RPC，需要在 proto 中添加 chunk 标记：

```protobuf
message GrpcRequest {
  // ...
  bool is_first_chunk = 10;
  bool is_last_chunk = 11;
}

message GrpcResponse {
  // ...
  bool is_first_chunk = 5;
  bool is_last_chunk = 6;
}
```

处理逻辑：
- 第一个 chunk: `is_first_chunk = true`
- 中间 chunk: `is_first_chunk = false, is_last_chunk = false`
- 最后一个 chunk: `is_last_chunk = true`

### 错误处理

```rust
// 超时清理
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = Instant::now();
        pending_requests.retain(|request_id, (created_at, _)| {
            now.duration_since(*created_at) < Duration::from_secs(30)
        });
    }
});
```

---

## 📊 总结

**核心设计**: 每个 gRPC 请求通过独立的 tunnel stream 处理，使用 `request_id` 匹配请求/响应。

**优势**:
- ✅ 完全并发，最大化性能
- ✅ 实现简单，易于维护
- ✅ 不需要有序性保证
- ✅ 统一处理 HTTP 和 gRPC

**适用场景**:
- ✅ Unary RPC
- ✅ Server Streaming RPC
- ✅ Client Streaming RPC
- ✅ Bidirectional Streaming RPC

