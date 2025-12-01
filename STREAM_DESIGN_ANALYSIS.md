# Stream 设计分析：控制流 vs 数据流

## 📋 当前实现分析

### 1. Proto 定义

```proto
service TunnelService {
  rpc ControlStream(stream TunnelMessage) returns (stream TunnelMessage);
  rpc Proxy(stream TunnelMessage) returns (stream TunnelMessage);
  rpc ConfigSync(ConfigSyncRequest) returns (ConfigSyncResponse);
}
```

**Proto 定义了两种 stream：**
- `ControlStream` - 控制流（用于控制消息）
- `Proxy` - 代理流（用于数据消息）

### 2. 实际使用情况

#### Client 端

```102:102:client/tunnel_client.rs
            let response = grpc_client.proxy(Request::new(outbound)).await;
```

**Client 只使用 `Proxy` stream**，在同一个 stream 中发送：

1. **控制消息：**
   - `StreamOpen` - 注册 stream 和心跳（每30秒）
   ```81:96:client/tunnel_client.rs
            let connect_msg = TunnelMessage {
                client_id: (*self.client_id).clone(),
                request_id: Uuid::new_v4().to_string(),
                direction: Direction::ClientToServer as i32,
                payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::StreamOpen(
                    StreamOpenRequest {
                        client_id: (*self.client_id).clone(),
                        group: (*self.group_id).clone(),
                        version: "v1.0.0".to_string(),
                        stream_id: stream_id.clone(),
                        stream_type: stream_type as i32,
                        timestamp: chrono::Utc::now().timestamp(),
                    }
                )),
                trace_id: String::new(),
            };
   ```

   - `ConfigSync` - 配置同步（每30秒）
   ```219:231:client/tunnel_client.rs
                    let config_sync_msg = TunnelMessage {
                        client_id: (*self.client_id).clone(),
                        request_id: Uuid::new_v4().to_string(),
                        direction: Direction::ClientToServer as i32,
                        payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::ConfigSync(
                            ConfigSyncRequest {
                                client_id: (*self.client_id).clone(),
                                group: (*group_id).clone(),
                                config_version: "".to_string(),
                            }
                        )),
                        trace_id: String::new(),
                    };
   ```

2. **数据消息：**
   - `HttpRequest` - HTTP 请求数据
   - `HttpResponse` - HTTP 响应数据

#### Server 端

**`control_stream` 方法存在但未使用：**

```67:96:server/tunnel_server.rs
    async fn control_stream(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(10000);
        let client_registry = self.client_registry.clone();
        let rules_engine = self.rules_engine.clone();
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(tunnel_msg) => {
                        match tunnel_msg.payload {
                            Some(tunnel_lib::tunnel::tunnel_message::Payload::ConfigSync(config_req)) => {
                                // 只处理 ConfigSync，实现不完整
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
```

**`proxy` 方法处理所有消息类型：**

```98:266:server/tunnel_server.rs
    async fn proxy(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        // ...
        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    match msg.payload {
                        Some(tunnel_message::Payload::StreamOpen(ref req)) => {
                            // 处理控制消息：StreamOpen
                        }
                        Some(tunnel_message::Payload::ConfigSync(config_req)) => {
                            // 处理控制消息：ConfigSync
                        }
                        Some(tunnel_message::Payload::HttpRequest(ref req)) => {
                            // 处理数据消息：HttpRequest
                        }
                        Some(tunnel_message::Payload::HttpResponse(resp)) => {
                            // 处理数据消息：HttpResponse
                        }
                        _ => {}
                    }
                }
            }
        }
    }
```

---

## ✅ 结论

**是的，当前设计是控制请求和数据请求都在一条 Proxy stream 里。**

### 实际情况：

1. ✅ **Client 端**：只使用 `Proxy` stream，所有消息（控制+数据）都通过它发送
2. ✅ **Server 端**：`proxy` 方法处理所有消息类型
3. ❌ **`ControlStream`**：虽然 proto 定义了，但实际**没有被使用**

### 在同一个 Proxy stream 中传输的消息：

| 消息类型 | 用途 | 频率 |
|---------|------|------|
| `StreamOpen` | 注册 stream、心跳 | 每30秒 |
| `ConfigSync` | 配置同步 | 每30秒 |
| `HttpRequest` | HTTP 请求数据 | 按需 |
| `HttpResponse` | HTTP 响应数据 | 按需 |

---

## ⚠️ 潜在问题

### 1. 队头阻塞（Head-of-Line Blocking）

**问题：**
- 控制消息（心跳、配置同步）和数据消息混在一起
- 如果某个数据消息处理慢，会阻塞后续的控制消息
- 控制消息延迟可能导致连接被认为断开

**影响：**
- 心跳延迟可能导致误判连接断开
- 配置同步延迟可能导致配置更新不及时

### 1.1 重新评估：配置同步是否需要分离？

**配置同步的特点：**
- ✅ 频率很低：每30秒一次
- ✅ 数据量小：只是配置信息
- ✅ 延迟容忍度高：即使延迟几秒影响也不大

**结论：**
- ❌ **配置同步不需要分离**：频率低，即使被阻塞影响也不大
- ✅ **心跳已经在数据 stream 里**：每个 stream 自己维护心跳
- ⚠️ **真正的问题**：心跳是否会被数据消息阻塞？

**如果心跳也在数据 stream 里：**
- 心跳和数据消息在同一个 stream
- 如果数据消息处理慢，心跳可能被阻塞
- 心跳延迟可能导致连接被误判为断开

**但实际情况：**
- 心跳通过 `StreamOpen` 消息发送，每30秒一次
- Server 端处理 `StreamOpen` 很快（只是更新 `last_heartbeat`）
- 即使数据消息处理慢，心跳消息本身处理很快，不会被长时间阻塞

### 2. 心跳机制的正确理解

**当前实现的实际情况：**

从代码分析可以看出，当前实现中：
- ✅ 每个 stream 通过发送 `StreamOpen` 消息来维护自己的心跳
- ✅ Server 端收到 `StreamOpen` 后，更新对应 `stream_id` 的 `last_heartbeat`
- ✅ 心跳是**每个 stream 独立维护**的，不是通过 control stream 心跳其他 stream

**关键代码：**

```179:212:client/tunnel_client.rs
    async fn heartbeat_task(cancel_token: CancellationToken, tx: mpsc::Sender<TunnelMessage>, client_id: Arc<String>, group_id: Arc<String>, stream_id: String) {
        // 心跳任务发送 StreamOpen 消息，包含自己的 stream_id
        let heartbeat = TunnelMessage {
            payload: Some(Payload::StreamOpen(StreamOpenRequest {
                stream_id: stream_id.clone(), // 自己的 stream_id
                stream_type: StreamType::Http as i32,
                // ...
            })),
        };
        tx.send(heartbeat).await; // 通过自己的 tx 发送
    }
```

```119:132:server/tunnel_server.rs
                            Some(tunnel_message::Payload::StreamOpen(ref req)) => {
                                // Server 收到 StreamOpen，更新该 stream_id 的心跳时间
                                stream_id = req.stream_id.clone();
                                client_registry.sync_stream(
                                    &req.client_id,
                                    &req.group,
                                    &req.stream_id, // 更新这个 stream_id 的 last_heartbeat
                                    stream_type,
                                    ctx.clone(),
                                    token.clone(),
                                );
                            }
```

**结论：**
- ✅ 当前实现**已经是每个 stream 独立心跳**
- ✅ 如果分离成 control stream 和 data stream，**每个都需要独立心跳**
- ❌ Control stream **不能**用来心跳 data stream，因为 control stream 自己也需要心跳

### 2. 优先级问题

**问题：**
- 控制消息和数据消息没有优先级区分
- 大量数据请求可能淹没控制消息

**影响：**
- 心跳可能被延迟，导致连接被误判为断开
- 配置同步可能被延迟

### 3. 资源竞争

**问题：**
- 控制消息和数据消息共享同一个 channel 容量（128）
- 数据消息量大时，控制消息可能被阻塞

---

## 🚀 优化建议

### 方案 1：分离控制流和数据流（推荐，但需要修正）

**关键设计原则：**
- ✅ **每个 stream 独立心跳**：每个 stream 通过发送自己的 StreamOpen 消息来维护心跳
- ✅ **Control stream 只用于配置同步**：不用于心跳其他 stream
- ✅ **数据 stream 独立心跳**：通过发送 StreamOpen 消息维护自己的心跳

**优点：**
- ✅ 控制消息和数据消息完全隔离
- ✅ 每个 stream 独立维护连接状态
- ✅ 一个 stream 断开不影响其他 stream
- ✅ 更好的优先级管理

**实现：**

```rust
// Client 端
impl TunnelClient {
    pub async fn connect(&self) -> Result<()> {
        // 1. 建立控制流（用于配置同步，自己也需要心跳）
        let control_stream_id = Uuid::new_v4().to_string();
        let control_tx = self.connect_control_stream(control_stream_id.clone()).await?;
        
        // 2. 建立数据流（用于 HTTP 请求/响应，自己也需要心跳）
        let data_stream_id = Uuid::new_v4().to_string();
        let data_tx = self.connect_data_stream(data_stream_id.clone()).await?;
        
        // 3. 启动控制流心跳任务（使用控制流自己的 tx）
        tokio::spawn(Self::heartbeat_task(
            control_tx.clone(),
            control_stream_id.clone(),
            StreamType::Control,
        ));
        
        // 4. 启动数据流心跳任务（使用数据流自己的 tx）
        tokio::spawn(Self::heartbeat_task(
            data_tx.clone(),
            data_stream_id.clone(),
            StreamType::Http,
        ));
        
        // 5. 启动配置同步任务（使用控制流）
        tokio::spawn(Self::config_sync_task(control_tx.clone()));
        
        Ok(())
    }
    
    async fn connect_control_stream(&self, stream_id: String) -> Result<mpsc::Sender<TunnelMessage>> {
        let grpc_client = TunnelServiceClient::connect(self.server_addr.clone()).await?;
        let (tx, rx) = mpsc::channel(128);
        
        // 发送 StreamOpen 注册控制流
        let stream_open = TunnelMessage {
            client_id: (*self.client_id).clone(),
            request_id: Uuid::new_v4().to_string(),
            direction: Direction::ClientToServer as i32,
            payload: Some(Payload::StreamOpen(StreamOpenRequest {
                client_id: (*self.client_id).clone(),
                group: (*self.group_id).clone(),
                version: "v1.0.0".to_string(),
                stream_id: stream_id.clone(),
                stream_type: StreamType::Control as i32,
                timestamp: chrono::Utc::now().timestamp(),
            })),
            trace_id: String::new(),
        };
        tx.send(stream_open).await?;
        
        let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
        let response = grpc_client.control_stream(Request::new(outbound)).await?;
        // 处理 inbound stream
        Ok(tx)
    }
    
    async fn connect_data_stream(&self, stream_id: String) -> Result<mpsc::Sender<TunnelMessage>> {
        let grpc_client = TunnelServiceClient::connect(self.server_addr.clone()).await?;
        let (tx, rx) = mpsc::channel(10000); // 更大的容量
        
        // 发送 StreamOpen 注册数据流
        let stream_open = TunnelMessage {
            client_id: (*self.client_id).clone(),
            request_id: Uuid::new_v4().to_string(),
            direction: Direction::ClientToServer as i32,
            payload: Some(Payload::StreamOpen(StreamOpenRequest {
                client_id: (*self.client_id).clone(),
                group: (*self.group_id).clone(),
                version: "v1.0.0".to_string(),
                stream_id: stream_id.clone(),
                stream_type: StreamType::Http as i32,
                timestamp: chrono::Utc::now().timestamp(),
            })),
            trace_id: String::new(),
        };
        tx.send(stream_open).await?;
        
        let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
        let response = grpc_client.proxy(Request::new(outbound)).await?;
        // 处理 inbound stream
        Ok(tx)
    }
    
    // 心跳任务：每个 stream 独立心跳
    async fn heartbeat_task(
        tx: mpsc::Sender<TunnelMessage>,
        stream_id: String,
        stream_type: StreamType,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let heartbeat = TunnelMessage {
                client_id: (*self.client_id).clone(),
                request_id: Uuid::new_v4().to_string(),
                direction: Direction::ClientToServer as i32,
                payload: Some(Payload::StreamOpen(StreamOpenRequest {
                    client_id: (*self.client_id).clone(),
                    group: (*self.group_id).clone(),
                    version: "v1.0.0".to_string(),
                    stream_id: stream_id.clone(),
                    stream_type: stream_type as i32,
                    timestamp: chrono::Utc::now().timestamp(),
                })),
                trace_id: String::new(),
            };
            if tx.send(heartbeat).await.is_err() {
                break; // Stream 断开，退出心跳任务
            }
        }
    }
}
```

**Server 端：**

```rust
impl TunnelService for TunnelServer {
    async fn control_stream(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        // 处理控制流消息：
        // - StreamOpen（注册/心跳控制流自己）
        // - ConfigSync（配置同步）
        // - StreamOpenResponse（响应）
        // - ConfigSyncResponse（响应）
        // 不处理数据消息（HttpRequest/HttpResponse）
    }
    
    async fn proxy(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        // 处理数据流消息：
        // - StreamOpen（注册/心跳数据流自己）
        // - HttpRequest（数据消息）
        // - HttpResponse（数据消息）
        // - StreamOpenResponse（响应）
        // 不处理控制消息（ConfigSync）
    }
}
```

**关键点：**
- ✅ 每个 stream 通过发送 `StreamOpen` 消息来维护自己的心跳
- ✅ Server 端收到 `StreamOpen` 后，更新对应 `stream_id` 的 `last_heartbeat`
- ✅ Control stream 只用于配置同步，不用于心跳其他 stream
- ✅ 每个 stream 独立维护连接状态，互不影响

### 方案 2：优先级队列（如果必须使用单 stream）

**优点：**
- ✅ 不需要修改 proto
- ✅ 保持单 stream 设计

**实现：**

```rust
use priority_queue::PriorityQueue;

struct MessageQueue {
    control_queue: Arc<Mutex<VecDeque<TunnelMessage>>>, // 高优先级
    data_queue: Arc<Mutex<VecDeque<TunnelMessage>>>,    // 低优先级
}

impl MessageQueue {
    async fn send(&self, msg: TunnelMessage) {
        match msg.payload {
            Some(Payload::StreamOpen(_)) | Some(Payload::ConfigSync(_)) => {
                // 控制消息：高优先级
                self.control_queue.lock().await.push_back(msg);
            }
            Some(Payload::HttpRequest(_)) | Some(Payload::HttpResponse(_)) => {
                // 数据消息：低优先级
                self.data_queue.lock().await.push_back(msg);
            }
            _ => {}
        }
    }
    
    async fn next(&self) -> Option<TunnelMessage> {
        // 优先处理控制消息
        if let Some(msg) = self.control_queue.lock().await.pop_front() {
            return Some(msg);
        }
        // 然后处理数据消息
        self.data_queue.lock().await.pop_front()
    }
}
```

### 方案 3：使用独立的控制 channel（轻量级方案）

**优点：**
- ✅ 不需要修改 proto
- ✅ 实现简单

**实现：**

```rust
// Client 端：使用独立的 gRPC 连接处理控制消息
impl TunnelClient {
    pub async fn connect(&self) -> Result<()> {
        // 数据流：使用 Proxy，需要独立心跳
        let data_stream_id = Uuid::new_v4().to_string();
        let data_tx = self.connect_proxy_stream(data_stream_id.clone()).await?;
        tokio::spawn(Self::heartbeat_task(
            data_tx.clone(),
            data_stream_id.clone(),
            StreamType::Http,
        ));
        
        // 控制流：使用独立的 gRPC 连接 + ControlStream，也需要独立心跳
        let control_stream_id = Uuid::new_v4().to_string();
        let control_tx = self.connect_control_stream(control_stream_id.clone()).await?;
        tokio::spawn(Self::heartbeat_task(
            control_tx.clone(),
            control_stream_id.clone(),
            StreamType::Control,
        ));
        
        // 配置同步使用控制流
        tokio::spawn(Self::config_sync_task(control_tx.clone()));
        
        Ok(())
    }
}
```

**关键点：**
- ✅ 每个 stream（control 和 data）都需要独立心跳
- ✅ Control stream 不能用来心跳 data stream
- ✅ 每个 stream 通过发送自己的 `StreamOpen` 消息维护心跳

---

## 📊 对比

| 方案 | 复杂度 | 性能 | 可靠性 | 推荐度 |
|------|--------|------|--------|--------|
| **方案1：分离流** | 中 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **方案2：优先级队列** | 高 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **方案3：独立连接** | 低 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

---

## 🎯 推荐方案

### 方案 A：单 Stream（推荐，如果只是配置同步）

**核心观点：**
- ✅ **配置同步频率低**（每30秒一次），即使被阻塞影响也不大
- ✅ **心跳已经在数据 stream 里**，每个 stream 自己维护
- ✅ **单 stream 更简单**，不需要管理多个 stream
- ✅ **心跳处理很快**，不会被数据消息长时间阻塞

**适用场景：**
- 配置同步频率低（30秒一次）
- 心跳处理快速（只是更新 `last_heartbeat`）
- 不需要复杂的控制消息

**实现：**
- 保持当前设计：所有消息（心跳、配置同步、数据）都在 `Proxy` stream 中
- 优化消息处理：确保心跳消息快速处理，不被数据消息阻塞

**优点：**
- ✅ 实现简单，不需要管理多个 stream
- ✅ 资源占用少，只需要一个 stream
- ✅ 代码维护成本低

**缺点：**
- ⚠️ 理论上心跳可能被数据消息阻塞（但实际影响很小）
- ⚠️ 配置同步可能被数据消息阻塞（但频率低，影响不大）

### 方案 B：分离控制流和数据流（如果未来需要更多控制消息）

**核心设计原则：**
1. ✅ **每个 stream 独立心跳**：每个 stream 通过发送 `StreamOpen` 消息维护自己的心跳
2. ✅ **Control stream 用于控制消息**：配置同步、错误处理、流管理（如果未来需要）
3. ✅ **数据 stream 独立心跳**：通过发送 `StreamOpen` 消息维护自己的心跳
4. ✅ **应用层隔离**：控制消息和数据消息使用不同的 stream（注意：TCP 层无隔离）

**适用场景：**
- 需要频繁的控制消息（不仅仅是配置同步）
- 需要应用层的隔离（虽然 TCP 层无隔离）
- 需要更细粒度的流管理

**限制：**
- ⚠️ **TCP 层无隔离**：TCP 连接问题影响所有 stream
- ⚠️ **收益有限**：如果只是配置同步，分离的收益很小
- ⚠️ **复杂度增加**：需要管理多个 stream

**理由：**
1. ✅ 应用层隔离控制消息和数据消息
2. ✅ 控制消息不会被数据消息阻塞（应用层）
3. ✅ 每个 stream 独立维护连接状态，互不影响（应用层）
4. ✅ 更好的资源管理（控制流可以用小容量，数据流用大容量）
5. ✅ 符合 proto 设计的初衷
6. ⚠️ **但不能解决 TCP 层的队头阻塞**

**心跳机制：**
- Control stream：每30秒发送 `StreamOpen`（stream_type=Control）维护自己的心跳
- Data stream：每30秒发送 `StreamOpen`（stream_type=Http）维护自己的心跳
- Server 端：收到 `StreamOpen` 后，更新对应 `stream_id` 的 `last_heartbeat`

**实施步骤：**
1. 完善 `ControlStream` 的实现，支持 StreamOpen 心跳
2. Client 端同时建立两个 stream（control + data）
3. 每个 stream 独立启动心跳任务
4. 控制消息（ConfigSync）使用 `ControlStream`
5. 数据消息（HttpRequest/HttpResponse）使用 `Proxy` stream
6. 逐步迁移现有代码

---

## 📊 方案对比

| 方案 | 复杂度 | 性能 | 可靠性 | TCP隔离 | 适用场景 | 推荐度 |
|------|--------|------|--------|---------|----------|--------|
| **方案A：单 Stream** | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ❌ 无 | 配置同步频率低 | ⭐⭐⭐⭐⭐ |
| **方案B：分离流** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ❌ 无 | 需要频繁控制消息 | ⭐⭐⭐ |

**关键点：**
- ⚠️ **两种方案在 TCP 层都没有隔离**：TCP 连接问题影响所有 stream
- ✅ **方案 A 更简单**：如果只是配置同步，单 stream 足够
- ⚠️ **方案 B 收益有限**：分离 stream 主要在应用层提供隔离，TCP 层无隔离

---

## 🔍 关键洞察：多 Stream 的隔离效果

### TCP 层面的限制

**重要发现：**
- ⚠️ **多 stream 底层都是同一条 TCP 连接**
- ⚠️ **TCP 层的队头阻塞**：如果 TCP 连接上的数据包丢失，TCP 层可能会阻塞所有 stream
- ⚠️ **TCP 连接问题**：如果 TCP 连接断开，所有 stream 都会受影响

**HTTP/2 层面的隔离：**
- ✅ HTTP/2 的 stream 之间是独立的，一个 stream 的阻塞不应该影响其他 stream
- ✅ HTTP/2 多路复用允许多个 stream 并发传输
- ⚠️ 但这是**应用层**的隔离，不是 TCP 层的隔离

**实际隔离效果：**

| 层面 | 隔离效果 | 说明 |
|------|---------|------|
| **TCP 层** | ❌ 无隔离 | TCP 连接问题影响所有 stream |
| **HTTP/2 层** | ✅ 有隔离 | Stream 之间独立，但受 TCP 限制 |
| **应用层** | ✅ 有隔离 | 消息处理可以独立，但受 TCP 限制 |

### 分离 Stream 的实际收益

**分离 Stream 的好处：**
1. ✅ **应用层隔离**：不同 stream 的消息处理逻辑分离
2. ✅ **优先级管理**：可以为不同 stream 设置不同的优先级
3. ✅ **资源管理**：可以为不同 stream 设置不同的 buffer 大小

**分离 Stream 的限制：**
1. ❌ **TCP 层无隔离**：TCP 连接问题影响所有 stream
2. ❌ **HTTP/2 流限制**：默认每个连接最多 100 个并发 stream
3. ❌ **复杂度增加**：需要管理多个 stream

### 重新评估分离的必要性

**如果底层都是同一条 TCP 连接：**
- ❌ **TCP 连接问题**：影响所有 stream，分离 stream 无法解决
- ✅ **应用层隔离**：分离 stream 可以提供应用层的隔离
- ⚠️ **收益有限**：如果只是配置同步，分离的收益很小

**结论：**
- ✅ **单 stream 更合理**：如果底层都是同一条 TCP 连接，分离 stream 的收益有限
- ✅ **优化重点**：应该优化应用层的消息处理，而不是分离 stream
- ✅ **真正的问题**：TCP 层的队头阻塞，分离 stream 无法解决

---

## 🎯 最终推荐

**当前推荐：方案 A（单 Stream）**

**理由：**
1. ✅ **配置同步频率低**（30秒一次），即使被阻塞影响也不大
2. ✅ **心跳处理快速**，不会被数据消息长时间阻塞
3. ✅ **实现简单**，不需要管理多个 stream
4. ✅ **资源占用少**，只需要一个 stream
5. ✅ **TCP 层无隔离**：分离 stream 无法解决 TCP 层的队头阻塞
6. ✅ **收益有限**：如果只是配置同步，分离的收益很小

**如果未来需要：**
- 频繁的控制消息（如流管理、错误处理等）
- 更细粒度的流控制
- 应用层的隔离（虽然 TCP 层无隔离）

**可以考虑方案 B（分离流）**

**但要注意：**
- ⚠️ 分离 stream **不能解决** TCP 层的队头阻塞
- ⚠️ 分离 stream 的收益主要在**应用层**的隔离
- ⚠️ 如果只是配置同步，分离的收益很小

**当前优化重点：**
- ✅ 优化消息处理，确保心跳消息快速处理
- ✅ 使用并发处理，避免数据消息阻塞心跳
- ✅ 监控心跳延迟，如果发现阻塞问题再考虑分离
- ✅ **关注 TCP 层性能**：优化 TCP 连接，减少丢包和重传

