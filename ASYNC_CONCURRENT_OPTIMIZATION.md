# 异步/并发优化方案

> **目标**: 将当前同步等待的实现改为异步并发处理，提升吞吐量和性能

---

## 📊 当前实现分析

### 1. Server 端：顺序处理导致阻塞

**问题位置：** `server/tunnel_server.rs:114-247`

```rust
while let Some(message) = stream.next().await {
    match message {
        Ok(msg) => {
            match msg.payload {
                Some(tunnel_message::Payload::HttpRequest(ref req)) => {
                    // ❌ 同步等待，阻塞后续消息
                    response = forward_http_to_backend(
                        req,
                        &backend,
                        https_client.clone(),
                        set_host
                    ).await;  // ← 这里会阻塞
                    
                    // 发送响应
                    let response_msg = TunnelMessage { ... };
                    if let Some((tx, _, _)) = client_registry.get_stream_info(...) {
                        let _ = tx.send(response_msg).await;  // ← 这里也会阻塞
                    }
                }
            }
        }
    }
}
```

**问题：**
- ❌ 每个消息顺序处理，一个慢请求会阻塞后续所有请求
- ❌ `forward_http_to_backend().await` 会阻塞等待后端响应
- ❌ `tx.send().await` 会阻塞等待 channel 有空间

### 2. Client 端：虽然有并发，但受限于接收顺序

**当前实现：** `client/tunnel_client.rs:111-138`

```rust
while let Some(message) = inbound.next().await {
    match message {
        Ok(msg) => {
            // ✅ 已经有并发处理
            let permit = semaphore.clone().acquire_owned().await?;
            tokio::spawn(async move {
                client.handle_tunnel_message(msg, &tx).await;
                drop(permit);
            });
        }
    }
}
```

**问题：**
- ⚠️ 消息接收仍然是顺序的（`inbound.next().await`）
- ⚠️ 虽然处理是并发的，但接收是瓶颈

### 3. HTTP 转发：使用 oneshot 等待响应

**当前实现：** `tunnel-lib/src/http_forward.rs:54-86`

```rust
let (tx, rx) = oneshot::channel();
pending_map.insert(request_id.clone(), tx);
tunnel_sender.send(tunnel_msg).await;  // ← 可能阻塞

match timeout(Duration::from_secs(30), rx).await {  // ← 等待响应
    Ok(Ok(resp)) => { /* 处理响应 */ }
}
```

**问题：**
- ⚠️ `tunnel_sender.send().await` 可能阻塞（如果 channel 满了）
- ✅ `rx.await` 是异步等待，这是正确的

---

## 🚀 优化方案

### 方案 1：Server 端完全并发处理（推荐）

**核心思想：**
- 使用无界 channel 解耦消息接收和处理
- 每个消息并发处理，不阻塞接收循环

**实现：**

```rust
// server/tunnel_server.rs
async fn proxy(
    &self,
    request: Request<tonic::Streaming<TunnelMessage>>,
) -> Result<Response<Self::ProxyStream>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(10000);
    
    // 1. 消息接收任务（快速，不阻塞）
    let pending_requests = self.pending_requests.clone();
    let rules_engine = self.rules_engine.clone();
    let client_registry = self.client_registry.clone();
    let https_client = self.https_client.clone();
    
    // 使用无界 channel 解耦接收和处理
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<TunnelMessage>();
    
    // 消息接收任务（只负责接收，不处理）
    tokio::spawn(async move {
        let mut client_id = String::new();
        let mut group = String::new();
        let mut stream_id = String::new();
        let mut stream_type = StreamType::Unspecified;
        
        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    // 快速分发到处理 channel，不阻塞
                    if let Err(_) = msg_tx.send(msg) {
                        break; // Channel 关闭
                    }
                }
                Err(_) => {
                    // 错误处理：清理资源
                    if let Some((_, token, _)) = client_registry.get_stream_info(&group, stream_type, &client_id, &stream_id) {
                        token.cancel();
                    }
                    break;
                }
            }
        }
    });
    
    // 2. 并发处理任务池
    let semaphore = Arc::new(Semaphore::new(1000)); // 限制并发数
    let pending_requests_clone = pending_requests.clone();
    let rules_engine_clone = rules_engine.clone();
    let client_registry_clone = client_registry.clone();
    let https_client_clone = https_client.clone();
    
    tokio::spawn(async move {
        let mut client_id = String::new();
        let mut group = String::new();
        let mut stream_id = String::new();
        let mut stream_type = StreamType::Unspecified;
        
        while let Some(msg) = msg_rx.recv().await {
            // 获取信号量许可（限制并发数）
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            
            // 克隆必要的资源
            let pending_requests = pending_requests_clone.clone();
            let rules_engine = rules_engine_clone.clone();
            let client_registry = client_registry_clone.clone();
            let https_client = https_client_clone.clone();
            let tx_clone = tx.clone();
            
            // 并发处理消息
            tokio::spawn(async move {
                // 处理消息
                match msg.payload {
                    Some(tunnel_message::Payload::StreamOpen(ref req)) => {
                        // 更新状态
                        client_id = req.client_id.clone();
                        group = req.group.clone();
                        stream_id = req.stream_id.clone();
                        stream_type = StreamType::from_i32(req.stream_type).unwrap_or(StreamType::Unspecified);
                        
                        // 注册 stream
                        let (ctx, mut crx) = mpsc::channel::<TunnelMessage>(128);
                        client_registry.sync_stream(
                            &req.client_id,
                            &req.group,
                            &req.stream_id,
                            stream_type,
                            ctx.clone(),
                            token.clone(),
                        );
                        
                        // 启动转发任务
                        let tx_clone2 = tx_clone.clone();
                        tokio::spawn(async move {
                            while let Some(tunnel_msg) = crx.recv().await {
                                if let Err(_) = tx_clone2.send(Ok(tunnel_msg)).await {
                                    break;
                                }
                            }
                        });
                        
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
                        // 使用 try_send 避免阻塞
                        let _ = ctx.try_send(response);
                    }
                    
                    Some(tunnel_message::Payload::HttpRequest(ref req)) => {
                        // ✅ 并发处理 HTTP 请求，不阻塞
                        let host = req.host.as_str();
                        let path = req.url.split('?').next().unwrap_or("/");
                        let request_id = msg.request_id.clone();
                        let client_id_clone = msg.client_id.clone();
                        let trace_id = msg.trace_id.clone();
                        
                        tokio::spawn(async move {
                            let mut response = error_response(
                                ProxyErrorKind::NoMatchRules,
                                None,
                                Some(&trace_id),
                                Some(&request_id),
                                Some(&client_id_clone),
                            );
                            
                            if let Some(rule) = rules_engine.match_forward_rule(host, path, None) {
                                if let Some(ref upstream_name) = rule.action_upstream {
                                    if let Some(upstream) = rules_engine.get_upstream(upstream_name) {
                                        if let Some(backend) = pick_backend(upstream) {
                                            let set_host = rule.action_set_host.as_deref().unwrap_or("");
                                            // ✅ 异步等待后端响应，不阻塞其他请求
                                            response = forward_http_to_backend(
                                                req,
                                                &backend,
                                                https_client.clone(),
                                                set_host
                                            ).await;
                                        }
                                    }
                                }
                            }
                            
                            // 构建响应消息
                            let response_msg = TunnelMessage {
                                client_id: client_id_clone.clone(),
                                request_id: request_id.clone(),
                                direction: Direction::ServerToClient as i32,
                                payload: Some(tunnel_message::Payload::HttpResponse(response)),
                                trace_id: trace_id.clone(),
                            };
                            
                            // ✅ 使用 try_send 避免阻塞，如果失败则记录日志
                            if let Some((tx, _, _)) = client_registry.get_stream_info(&group, stream_type, &client_id_clone, &stream_id) {
                                if let Err(e) = tx.try_send(response_msg) {
                                    tracing::error!("Failed to send response: {}", e);
                                    // 可以考虑使用 send().await，但需要确保不会阻塞
                                }
                            }
                        });
                    }
                    
                    Some(tunnel_message::Payload::HttpResponse(resp)) => {
                        // ✅ 快速处理响应，不阻塞
                        if msg.direction == Direction::ClientToServer as i32 {
                            if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
                                let _ = sender.send(resp); // oneshot 不会阻塞
                            }
                        }
                    }
                    
                    Some(tunnel_message::Payload::ConfigSync(config_req)) => {
                        // ✅ 并发处理配置同步
                        tokio::spawn(async move {
                            // 处理配置同步...
                        });
                    }
                    
                    _ => {}
                }
                
                // 释放信号量许可
                drop(permit);
            });
        }
    });
    
    Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
}
```

**关键优化点：**
1. ✅ **无界 channel 解耦**：`mpsc::unbounded_channel()` 解耦接收和处理
2. ✅ **并发处理**：每个消息使用 `tokio::spawn` 并发处理
3. ✅ **信号量限制**：使用 `Semaphore` 限制并发数（防止资源耗尽）
4. ✅ **非阻塞发送**：使用 `try_send` 避免阻塞，或使用 `send().await` 但确保不阻塞接收循环

### 方案 2：Client 端优化消息接收

**当前实现已经有并发处理，但可以优化接收：**

```rust
// client/tunnel_client.rs
pub async fn connect_with_retry_with_token(
    &self,
    mut grpc_client: TunnelServiceClient<tonic::transport::Channel>,
    rx: &mut mpsc::Receiver<TunnelMessage>,
    token: CancellationToken,
) -> anyhow::Result<()> {
    loop {
        // ... 建立连接 ...
        
        let mut inbound = resp.into_inner();
        
        // ✅ 使用无界 channel 解耦接收和处理
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel();
        
        // 消息接收任务（快速，不阻塞）
        let client_clone = self.clone();
        let tx_clone = self.tx.clone();
        tokio::spawn(async move {
            while let Some(message) = inbound.next().await {
                match message {
                    Ok(msg) => {
                        // 快速分发，不阻塞
                        if let Err(_) = msg_tx.send(msg) {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        // 并发处理任务池
        let semaphore = Arc::new(Semaphore::new(1000));
        let client_clone2 = self.clone();
        let tx_clone2 = self.tx.clone();
        
        while let Some(msg) = msg_rx.recv().await {
            let permit = semaphore.clone().acquire_owned().await?;
            let client = client_clone2.clone();
            let tx = tx_clone2.clone();
            
            tokio::spawn(async move {
                client.handle_tunnel_message(msg, &tx).await;
                drop(permit);
            });
        }
    }
}
```

### 方案 3：优化 HTTP 转发（避免 channel 阻塞）

**当前实现：** `tunnel-lib/src/http_forward.rs`

```rust
pub async fn forward_http_via_tunnel(
    http_req: HyperRequest<Body>,
    client_id: &str,
    tunnel_sender: &mpsc::Sender<TunnelMessage>,
    pending_map: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    request_id: String,
    direction: Direction,
    stream_id: String,
) -> Result<hyper::Response<Body>, hyper::Error> {
    // ... 构建消息 ...
    
    let (tx, rx) = oneshot::channel();
    pending_map.insert(request_id.clone(), tx);
    
    // ✅ 使用 try_send 避免阻塞，如果失败则返回错误
    match tunnel_sender.try_send(tunnel_msg) {
        Ok(()) => {
            // 成功发送，等待响应
            match timeout(Duration::from_secs(30), rx).await {
                Ok(Ok(resp)) => {
                    // 转换为 HyperResponse
                    let mut builder = hyper::Response::builder().status(resp.status_code as u16);
                    for (k, v) in resp.headers {
                        builder = builder.header(k, v);
                    }
                    Ok(builder.body(Body::from(resp.body)).unwrap())
                }
                Ok(Err(_)) => {
                    pending_map.remove(&request_id);
                    Ok(hyper::Response::builder().status(502).body(Body::from("Tunnel response failed")).unwrap())
                }
                Err(_) => {
                    pending_map.remove(&request_id);
                    let err_resp = error_response(
                        ProxyErrorKind::Timeout,
                        None,
                        Some(&trace_id),
                        Some(&request_id),
                        Some(client_id),
                    );
                    let mut builder = hyper::Response::builder().status(err_resp.status_code as u16);
                    for (k, v) in err_resp.headers.iter() {
                        builder = builder.header(k, v);
                    }
                    Ok(builder.body(Body::from(err_resp.body)).unwrap())
                }
            }
        }
        Err(mpsc::error::TrySendError::Full(_)) => {
            // Channel 满了，返回 503 Service Unavailable
            pending_map.remove(&request_id);
            Ok(hyper::Response::builder()
                .status(503)
                .body(Body::from("Service temporarily unavailable"))
                .unwrap())
        }
        Err(mpsc::error::TrySendError::Closed(_)) => {
            // Channel 关闭，返回 502 Bad Gateway
            pending_map.remove(&request_id);
            Ok(hyper::Response::builder()
                .status(502)
                .body(Body::from("Tunnel connection closed"))
                .unwrap())
        }
    }
}
```

**或者使用无界 channel：**

```rust
// 如果使用无界 channel，send 不会阻塞
let tunnel_sender: &mpsc::UnboundedSender<TunnelMessage> = ...;

// send 不会阻塞
tunnel_sender.send(tunnel_msg).map_err(|_| {
    hyper::Error::from(std::io::Error::new(
        std::io::ErrorKind::ConnectionAborted,
        "Tunnel channel closed"
    ))
})?;
```

---

## 🔑 关键机制：Request-Response 匹配

### 核心原理：通过 `request_id` 匹配

**关键点：**
- ✅ 每个请求生成唯一的 `request_id` (UUID)
- ✅ 使用 `DashMap` 存储 `pending_requests: request_id -> oneshot::Sender`
- ✅ 响应包含相同的 `request_id`，通过它找到对应的 sender
- ✅ **完全并发安全**：即使响应乱序到达，也能正确匹配

### 匹配流程

#### 1. 请求发送时（注册）

```rust
// tunnel-lib/src/http_forward.rs
pub async fn forward_http_via_tunnel(...) -> Result<hyper::Response<Body>, hyper::Error> {
    // 1. 生成唯一的 request_id
    let request_id = Uuid::new_v4().to_string();
    
    // 2. 创建 oneshot channel
    let (tx, rx) = oneshot::channel();
    
    // 3. 将 sender 存入 pending_map（关键：使用 request_id 作为 key）
    pending_map.insert(request_id.clone(), tx);
    
    // 4. 构建 TunnelMessage，包含 request_id
    let tunnel_msg = TunnelMessage {
        request_id: request_id.clone(),  // ← 关键：请求包含 request_id
        payload: Some(Payload::HttpRequest(http_request)),
        // ...
    };
    
    // 5. 发送请求
    tunnel_sender.send(tunnel_msg).await?;
    
    // 6. 等待响应（异步等待，不阻塞其他请求）
    match timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(resp)) => { /* 处理响应 */ }
        // ...
    }
}
```

#### 2. 响应接收时（匹配）

```rust
// server/tunnel_server.rs 或 client/tunnel_client.rs
match msg.payload {
    Some(Payload::HttpResponse(resp)) => {
        // ✅ 通过 request_id 匹配（线程安全）
        if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
            // ✅ 发送响应到对应的 oneshot channel
            let _ = sender.send(resp);
            // 注意：oneshot::Sender::send 是线程安全的
            // 即使多个响应并发到达，也能正确匹配
        }
    }
}
```

### 并发场景下的匹配示例

**场景：3个请求并发处理，响应乱序到达**

```
时间线：
T1: 请求A发送 (request_id = "req-A")
    → pending_map.insert("req-A", sender_A)
    
T2: 请求B发送 (request_id = "req-B")
    → pending_map.insert("req-B", sender_B)
    
T3: 请求C发送 (request_id = "req-C")
    → pending_map.insert("req-C", sender_C)

T4: 响应B到达 (request_id = "req-B") ← 先到达
    → pending_map.remove("req-B") → 找到 sender_B
    → sender_B.send(resp_B) → 唤醒等待请求B的 rx
    
T5: 响应A到达 (request_id = "req-A")
    → pending_map.remove("req-A") → 找到 sender_A
    → sender_A.send(resp_A) → 唤醒等待请求A的 rx
    
T6: 响应C到达 (request_id = "req-C")
    → pending_map.remove("req-C") → 找到 sender_C
    → sender_C.send(resp_C) → 唤醒等待请求C的 rx
```

**结果：**
- ✅ 所有请求都能正确匹配到对应的响应
- ✅ 即使响应乱序到达，也不影响匹配
- ✅ 完全并发安全

### 为什么这个机制是线程安全的？

#### 1. `request_id` 的唯一性

```rust
let request_id = Uuid::new_v4().to_string();
// UUID v4 保证全局唯一性，碰撞概率极低（约 5.3×10^-37）
```

#### 2. `DashMap` 的线程安全性

```rust
// DashMap 是线程安全的并发 HashMap
pub type PendingRequests = Arc<DashMap<String, oneshot::Sender<HttpResponse>>>;

// insert 和 remove 都是原子操作
pending_map.insert(request_id.clone(), tx);  // ✅ 线程安全
pending_map.remove(&request_id);             // ✅ 线程安全
```

#### 3. `oneshot::Sender` 的线程安全性

```rust
// oneshot::Sender::send 是线程安全的
// 可以安全地从多个线程调用
let _ = sender.send(resp);  // ✅ 线程安全
```

### 在并发优化后的匹配

**优化后的代码仍然使用相同的匹配机制：**

```rust
// Server 端并发处理
tokio::spawn(async move {
    match msg.payload {
        Some(Payload::HttpRequest(ref req)) => {
            // 处理请求...
            let response = forward_http_to_backend(...).await;
            
            // 构建响应消息，使用相同的 request_id
            let response_msg = TunnelMessage {
                request_id: msg.request_id.clone(),  // ← 关键：使用相同的 request_id
                payload: Some(Payload::HttpResponse(response)),
                // ...
            };
            
            // 发送响应（通过 request_id 匹配）
            tx.send(response_msg).await;
        }
        
        Some(Payload::HttpResponse(resp)) => {
            // ✅ 响应匹配：通过 request_id 找到对应的 sender
            if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
                let _ = sender.send(resp);  // ✅ 线程安全，并发安全
            }
        }
    }
});
```

**关键点：**
- ✅ 请求和响应都包含 `request_id`
- ✅ 响应处理时，通过 `request_id` 从 `pending_map` 中找到对应的 `sender`
- ✅ `DashMap::remove` 是原子操作，并发安全
- ✅ `oneshot::Sender::send` 是线程安全的
- ✅ **即使响应乱序到达，也能正确匹配**

### 完整的数据流

```
外部请求
    ↓
生成 request_id (UUID)
    ↓
创建 oneshot channel (tx, rx)
    ↓
pending_map.insert(request_id, tx)  ← 注册
    ↓
发送 TunnelMessage { request_id, HttpRequest }
    ↓
    ↓ (并发处理，可能乱序)
    ↓
接收 TunnelMessage { request_id, HttpResponse }
    ↓
pending_map.remove(request_id)  ← 匹配（原子操作）
    ↓
找到对应的 sender
    ↓
sender.send(resp)  ← 发送响应（线程安全）
    ↓
唤醒等待的 rx
    ↓
返回响应给外部请求
```

### 注意事项

#### 1. request_id 必须唯一

```rust
// ✅ 正确：每次生成新的 UUID
let request_id = Uuid::new_v4().to_string();

// ❌ 错误：重复使用 request_id
let request_id = "fixed-id".to_string();  // 会导致匹配错误
```

#### 2. 响应必须包含相同的 request_id

```rust
// ✅ 正确：响应使用相同的 request_id
let response_msg = TunnelMessage {
    request_id: msg.request_id.clone(),  // ← 使用请求的 request_id
    payload: Some(Payload::HttpResponse(response)),
};

// ❌ 错误：响应使用不同的 request_id
let response_msg = TunnelMessage {
    request_id: Uuid::new_v4().to_string(),  // ← 错误！无法匹配
    payload: Some(Payload::HttpResponse(response)),
};
```

#### 3. 超时清理

```rust
// 如果响应超时，需要清理 pending_map
match timeout(Duration::from_secs(30), rx).await {
    Err(_) => {
        // 超时，清理 pending_map
        pending_map.remove(&request_id);
        // 返回超时错误
    }
}
```

---

## 📊 性能对比

### 当前实现（同步）

```
请求1 (慢，5秒) → 阻塞 → 请求2 → 阻塞 → 请求3
总时间：5 + 2 + 1 = 8秒
```

### 优化后（并发）

```
请求1 (慢，5秒) ─┐
请求2 (2秒)     ├─→ 并发处理
请求3 (1秒) ────┘
总时间：max(5, 2, 1) = 5秒
```

**性能提升：**
- ✅ 吞吐量提升：从顺序处理到并发处理
- ✅ 延迟降低：慢请求不再阻塞快请求
- ✅ 资源利用率提升：充分利用多核 CPU

---

## 🎯 实施步骤

### 步骤 1：Server 端优化（高优先级）

1. 修改 `server/tunnel_server.rs` 的 `proxy` 方法
2. 使用无界 channel 解耦接收和处理
3. 每个消息并发处理
4. 使用信号量限制并发数

### 步骤 2：Client 端优化（中优先级）

1. 优化 `client/tunnel_client.rs` 的消息接收
2. 使用无界 channel 解耦接收和处理
3. 保持现有的并发处理逻辑

### 步骤 3：HTTP 转发优化（中优先级）

1. 修改 `tunnel-lib/src/http_forward.rs`
2. 使用 `try_send` 或无界 channel
3. 处理 channel 满的情况

---

## ⚠️ 注意事项

### 1. 信号量限制

- ✅ 使用 `Semaphore` 限制并发数，防止资源耗尽
- ✅ 建议设置为 1000-10000，根据服务器资源调整

### 2. 错误处理

- ✅ 使用 `try_send` 时，需要处理 channel 满的情况
- ✅ 记录错误日志，便于排查问题

### 3. 资源清理

- ✅ 确保所有 spawn 的任务都能正确清理
- ✅ 使用 `CancellationToken` 优雅关闭

### 4. 内存管理

- ✅ 无界 channel 可能导致内存增长，需要监控
- ✅ 考虑使用有界 channel + backpressure 机制

---

## 📈 预期效果

### 性能提升

- ✅ **吞吐量**：提升 5-10 倍（取决于并发数）
- ✅ **延迟**：P99 延迟降低 50-80%
- ✅ **资源利用率**：CPU 利用率提升 3-5 倍

### 可靠性提升

- ✅ **故障隔离**：单个慢请求不影响其他请求
- ✅ **背压处理**：channel 满时返回 503，而不是阻塞
- ✅ **优雅降级**：高负载时自动限流

