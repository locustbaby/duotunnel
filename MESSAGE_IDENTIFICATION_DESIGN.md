# 消息识别与拆分设计：WebSocket 和 gRPC Streaming

## 🎯 核心问题

**问题 1**: 如何识别 WebSocket 的消息？
- WebSocket 是长连接，会有多个数据帧
- 如何区分不同的 WebSocket 连接？
- 如何识别帧的边界和类型？

**问题 2**: gRPC Streaming 内的请求如何准确识别和拆分？
- gRPC Streaming 可能有多个请求/响应
- 如何区分不同的请求？
- 如何保证请求的完整性？

---

## ✅ 解决方案

### 核心设计：request_id + 消息边界标识

**关键思想**:
- ✅ **每个逻辑请求有唯一的 `request_id`**
- ✅ **WebSocket 连接**: 一个连接一个 `request_id`，整个生命周期保持不变
- ✅ **gRPC Streaming**: 每个 Streaming RPC 一个 `request_id`，通过 chunk 标记标识边界
- ✅ **消息边界**: 通过 `is_first_chunk` 和 `is_last_chunk` 标识消息边界

---

## 📋 WebSocket 消息识别

### 问题分析

**WebSocket 的特点**:
- 长连接，持续双向通信
- 多个数据帧（text/binary/close/ping/pong）
- 需要区分不同的 WebSocket 连接
- 需要识别帧的类型和边界

### 解决方案

#### 1. WebSocket 连接标识

```protobuf
message TunnelMessage {
  string request_id = 2;  // ← WebSocket 连接的唯一标识（整个生命周期不变）
  ProtocolType protocol_type = 4;
  oneof payload {
    WebSocketRequest ws_request = 9;      // 握手请求
    WebSocketResponse ws_response = 10;   // 握手响应
    WebSocketFrame ws_frame = 11;         // 数据帧
  }
}

message WebSocketRequest {
  string url = 1;
  string host = 2;
  map<string, string> headers = 3;
  string subprotocol = 4;
}

message WebSocketResponse {
  int32 status_code = 1;
  map<string, string> headers = 2;
  string subprotocol = 3;
  bool accepted = 4;
}

message WebSocketFrame {
  string connection_id = 1;  // ← 新增：WebSocket 连接 ID（等于 request_id）
  bool fin = 2;              // FIN 标志
  uint32 opcode = 3;         // Opcode (0x1=text, 0x2=binary, 0x8=close, 0x9=ping, 0xA=pong)
  bool masked = 4;           // 是否掩码
  bytes payload = 5;         // 帧数据
  Direction frame_direction = 6;  // ← 新增：帧的方向（CLIENT_TO_SERVER 或 SERVER_TO_CLIENT）
}
```

#### 2. WebSocket 连接生命周期

```
T1: WebSocket 握手请求
    → 生成 request_id = "ws-conn-1"
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketRequest }
    
T2: WebSocket 握手响应
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketResponse }
    → **保持 request_id = "ws-conn-1" 不变**
    
T3-T100: WebSocket 数据帧（持续）
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketFrame { connection_id: "ws-conn-1", 
                                                 opcode: TEXT, 
                                                 frame_direction: CLIENT_TO_SERVER } }
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketFrame { connection_id: "ws-conn-1", 
                                                 opcode: BINARY, 
                                                 frame_direction: SERVER_TO_CLIENT } }
    → ...（持续转发，request_id 始终是 "ws-conn-1"）
    
T101: WebSocket 关闭
    → TunnelMessage { request_id: "ws-conn-1", protocol_type: WEBSOCKET, 
                       payload: WebSocketFrame { opcode: CLOSE } }
    → 清理连接状态（完成）
```

#### 3. 多个 WebSocket 连接识别

```
同一个 Tunnel Stream 上的消息：

T1: TunnelMessage { request_id: "ws-conn-1", payload: WebSocketRequest }  ← 连接 1 握手
T2: TunnelMessage { request_id: "ws-conn-2", payload: WebSocketRequest }  ← 连接 2 握手
T3: TunnelMessage { request_id: "ws-conn-1", payload: WebSocketFrame }    ← 连接 1 数据帧
T4: TunnelMessage { request_id: "ws-conn-2", payload: WebSocketFrame }    ← 连接 2 数据帧
T5: TunnelMessage { request_id: "ws-conn-1", payload: WebSocketFrame }    ← 连接 1 数据帧
...
```

**识别机制**:
- ✅ 通过 `request_id` 区分不同的 WebSocket 连接
- ✅ 每个连接在整个生命周期内使用相同的 `request_id`
- ✅ Server 和 Client 端都维护 `request_id -> WebSocket connection` 映射

#### 4. WebSocket 帧识别实现

```rust
// Server 端：WebSocket Entry Handler
pub struct ServerWebSocketEntryTarget {
    pub websocket_connections: Arc<DashMap<String, WebSocketConnection>>,  // request_id -> connection
}

impl ServerWebSocketEntryTarget {
    pub async fn handle_websocket_upgrade(
        &self,
        req: HyperRequest<Body>,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        // 生成唯一的 request_id（WebSocket 连接 ID）
        let request_id = Uuid::new_v4().to_string();
        
        // 发送握手请求到 client
        let tunnel_msg = TunnelMessage {
            request_id: request_id.clone(),
            protocol_type: ProtocolType::Websocket as i32,
            payload: Some(Payload::WebSocketRequest(WebSocketRequest {
                url: req.uri().to_string(),
                host: extract_host(&req),
                headers: extract_headers(&req),
                subprotocol: String::new(),
            })),
            // ...
        };
        
        tunnel_tx.send(tunnel_msg).await?;
        
        // 等待握手响应
        let (resp_tx, resp_rx) = oneshot::channel();
        self.pending_requests.insert(request_id.clone(), PendingRequest::WebSocket(resp_tx));
        
        match resp_rx.await {
            Ok(WebSocketResponse { accepted: true, .. }) => {
                // 握手成功，升级到 WebSocket
                // 创建 WebSocket 连接状态
                self.websocket_connections.insert(request_id.clone(), WebSocketConnection {
                    request_id: request_id.clone(),
                    frontend_tx: websocket_tx,  // 前端 WebSocket sender
                    // ...
                });
                
                // 开始双向转发帧
                self.start_websocket_forwarding(request_id.clone(), tunnel_tx).await;
            }
            _ => {
                return Ok(resp_502());
            }
        }
    }
    
    /// WebSocket 帧转发（双向）
    async fn start_websocket_forwarding(
        &self,
        request_id: String,
        tunnel_tx: mpsc::Sender<TunnelMessage>,
    ) {
        // 从前端 WebSocket 接收帧，转发到 tunnel
        tokio::spawn(async move {
            while let Some(frame) = frontend_ws_rx.recv().await {
                let tunnel_msg = TunnelMessage {
                    request_id: request_id.clone(),  // ← 保持相同的 request_id
                    protocol_type: ProtocolType::Websocket as i32,
                    payload: Some(Payload::WebSocketFrame(WebSocketFrame {
                        connection_id: request_id.clone(),
                        fin: frame.fin,
                        opcode: frame.opcode,
                        masked: false,
                        payload: frame.payload,
                        frame_direction: Direction::ClientToServer as i32,
                    })),
                    // ...
                };
                
                tunnel_tx.send(tunnel_msg).await?;
            }
        });
        
        // 从 tunnel 接收帧，转发到前端 WebSocket
        tokio::spawn(async move {
            // 在统一的消息处理中，通过 request_id 找到连接并转发
        });
    }
}

// 统一消息处理中的 WebSocket 帧处理
async fn handle_websocket_frame(
    msg: TunnelMessage,
    websocket_connections: &Arc<DashMap<String, WebSocketConnection>>,
) {
    if let Some(Payload::WebSocketFrame(frame)) = msg.payload {
        // 通过 request_id（等于 connection_id）找到连接
        if let Some(conn) = websocket_connections.get(&msg.request_id) {
            match frame.frame_direction {
                Direction::ClientToServer => {
                    // 从前端来的帧，转发到后端
                    conn.backend_tx.send(frame).await?;
                }
                Direction::ServerToClient => {
                    // 从后端来的帧，转发到前端
                    conn.frontend_tx.send(frame).await?;
                }
            }
        }
    }
}
```

---

## 📋 gRPC Streaming 请求识别与拆分

### 问题分析

**gRPC Streaming 的特点**:
- 一个 Streaming RPC 可能有多个请求/响应
- 需要区分不同的 Streaming RPC
- 需要识别请求/响应的边界
- 需要保证请求的完整性

### 解决方案

#### 1. gRPC Streaming 标识

```protobuf
message GrpcRequest {
  string grpc_stream_id = 1;  // ← gRPC 后端 stream ID（用于标识后端 gRPC stream）
  string service = 2;
  string method = 3;
  bytes body = 4;
  bool is_first_chunk = 5;   // ← 是否是第一个 chunk
  bool is_last_chunk = 6;     // ← 是否是最后一个 chunk
  uint32 sequence_number = 7; // ← 新增：chunk 序号（可选，用于验证）
}

message GrpcResponse {
  string grpc_stream_id = 1;  // ← 关联 gRPC stream
  int32 status_code = 2;
  bytes body = 3;
  bool is_first_chunk = 4;
  bool is_last_chunk = 5;
  uint32 sequence_number = 6; // ← 新增：chunk 序号
}
```

#### 2. gRPC Streaming 请求拆分

**场景 1: Server Streaming（服务端流）**

```
Tunnel Stream 上的消息：

T1: TunnelMessage { request_id: "grpc-stream-1", 
                    payload: GrpcRequest { is_first_chunk: true, is_last_chunk: true } }
    → 这是完整的请求（Unary 或 Server Streaming 的第一个请求）
    
T2: TunnelMessage { request_id: "grpc-stream-1", 
                    payload: GrpcResponse { is_first_chunk: true, is_last_chunk: false } }
    → Server Streaming 的第一个响应 chunk
    
T3: TunnelMessage { request_id: "grpc-stream-1", 
                    payload: GrpcResponse { is_first_chunk: false, is_last_chunk: false } }
    → Server Streaming 的中间响应 chunk
    
T4: TunnelMessage { request_id: "grpc-stream-1", 
                    payload: GrpcResponse { is_first_chunk: false, is_last_chunk: true } }
    → Server Streaming 的最后一个响应 chunk（完成）
```

**场景 2: Client Streaming（客户端流）**

```
Tunnel Stream 上的消息：

T1: TunnelMessage { request_id: "grpc-stream-2", 
                    payload: GrpcRequest { is_first_chunk: true, is_last_chunk: false } }
    → Client Streaming 的第一个请求 chunk
    
T2: TunnelMessage { request_id: "grpc-stream-2", 
                    payload: GrpcRequest { is_first_chunk: false, is_last_chunk: false } }
    → Client Streaming 的中间请求 chunk
    
T3: TunnelMessage { request_id: "grpc-stream-2", 
                    payload: GrpcRequest { is_first_chunk: false, is_last_chunk: true } }
    → Client Streaming 的最后一个请求 chunk
    
T4: TunnelMessage { request_id: "grpc-stream-2", 
                    payload: GrpcResponse { is_first_chunk: true, is_last_chunk: true } }
    → 响应（完成）
```

**场景 3: Bidirectional Streaming（双向流）**

```
Tunnel Stream 上的消息：

T1: TunnelMessage { request_id: "grpc-stream-3", 
                    payload: GrpcRequest { is_first_chunk: true, is_last_chunk: false } }
    → 第一个请求 chunk
    
T2: TunnelMessage { request_id: "grpc-stream-3", 
                    payload: GrpcResponse { is_first_chunk: true, is_last_chunk: false } }
    → 第一个响应 chunk（可能先于请求完成）
    
T3: TunnelMessage { request_id: "grpc-stream-3", 
                    payload: GrpcRequest { is_first_chunk: false, is_last_chunk: false } }
    → 第二个请求 chunk
    
T4: TunnelMessage { request_id: "grpc-stream-3", 
                    payload: GrpcResponse { is_first_chunk: false, is_last_chunk: true } }
    → 最后一个响应 chunk（完成）
```

#### 3. gRPC Streaming 状态管理

```rust
use dashmap::DashMap;
use std::collections::VecDeque;

/// gRPC Streaming 状态
pub struct GrpcStreamState {
    pub request_id: String,
    pub grpc_stream_id: String,
    pub backend_stream: Option<tonic::Streaming<Bytes>>,  // 后端 gRPC stream
    pub request_chunks: VecDeque<Bytes>,  // 请求 chunks（Client Streaming）
    pub response_chunks: VecDeque<Bytes>, // 响应 chunks（Server Streaming）
    pub request_complete: bool,
    pub response_complete: bool,
}

pub struct GrpcStreamManager {
    // request_id -> GrpcStreamState
    pub streams: Arc<DashMap<String, Arc<Mutex<GrpcStreamState>>>>,
}

impl GrpcStreamManager {
    /// 处理 gRPC 请求 chunk
    pub async fn handle_request_chunk(
        &self,
        request_id: String,
        chunk: GrpcRequest,
    ) -> Result<(), String> {
        let state = self.streams
            .entry(request_id.clone())
            .or_insert_with(|| Arc::new(Mutex::new(GrpcStreamState {
                request_id: request_id.clone(),
                grpc_stream_id: chunk.grpc_stream_id.clone(),
                backend_stream: None,
                request_chunks: VecDeque::new(),
                response_chunks: VecDeque::new(),
                request_complete: false,
                response_complete: false,
            })))
            .clone();
        
        let mut s = state.lock().await;
        
        if chunk.is_first_chunk {
            // 第一个 chunk：建立后端 gRPC stream
            s.backend_stream = Some(self.create_backend_stream(&chunk).await?);
        }
        
        // 添加 chunk
        s.request_chunks.push_back(Bytes::from(chunk.body));
        
        if chunk.is_last_chunk {
            // 最后一个 chunk：标记请求完成
            s.request_complete = true;
            
            // 如果是 Client Streaming，发送所有 chunks 到后端
            if let Some(ref mut backend_stream) = s.backend_stream {
                while let Some(chunk) = s.request_chunks.pop_front() {
                    backend_stream.send(chunk).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 处理 gRPC 响应 chunk
    pub async fn handle_response_chunk(
        &self,
        request_id: String,
        chunk: GrpcResponse,
    ) -> Result<(), String> {
        if let Some(state) = self.streams.get(&request_id) {
            let mut s = state.lock().await;
            
            // 添加响应 chunk
            s.response_chunks.push_back(Bytes::from(chunk.body));
            
            if chunk.is_first_chunk {
                // 第一个响应 chunk：开始发送
                self.send_response_chunk(&request_id, chunk).await?;
            }
            
            if chunk.is_last_chunk {
                // 最后一个响应 chunk：完成并清理
                s.response_complete = true;
                self.streams.remove(&request_id);
            }
        }
        
        Ok(())
    }
}
```

#### 4. 多个 gRPC Streaming 请求识别

```
同一个 Tunnel Stream 上的消息：

T1: TunnelMessage { request_id: "grpc-1", payload: GrpcRequest { is_first_chunk: true } }
    → gRPC Streaming 1 开始
    
T2: TunnelMessage { request_id: "grpc-2", payload: GrpcRequest { is_first_chunk: true } }
    → gRPC Streaming 2 开始（并发）
    
T3: TunnelMessage { request_id: "grpc-1", payload: GrpcRequest { is_last_chunk: false } }
    → gRPC Streaming 1 的中间 chunk
    
T4: TunnelMessage { request_id: "grpc-2", payload: GrpcResponse { is_first_chunk: true } }
    → gRPC Streaming 2 的第一个响应（可能先完成）
    
T5: TunnelMessage { request_id: "grpc-1", payload: GrpcRequest { is_last_chunk: true } }
    → gRPC Streaming 1 的最后一个请求 chunk
    
T6: TunnelMessage { request_id: "grpc-1", payload: GrpcResponse { is_first_chunk: true } }
    → gRPC Streaming 1 的第一个响应 chunk
```

**识别机制**:
- ✅ 通过 `request_id` 区分不同的 gRPC Streaming RPC
- ✅ 通过 `is_first_chunk` 和 `is_last_chunk` 标识消息边界
- ✅ 通过 `grpc_stream_id` 关联后端 gRPC stream

---

## 🔧 完整实现方案

### 1. 统一消息处理（识别和路由）

```rust
async fn handle_tunnel_message(
    msg: TunnelMessage,
    pending_requests: &Arc<DashMap<String, PendingRequest>>,
    websocket_connections: &Arc<DashMap<String, WebSocketConnection>>,
    grpc_streams: &Arc<GrpcStreamManager>,
) {
    match msg.protocol_type {
        ProtocolType::Http => {
            // HTTP: 通过 request_id 匹配请求/响应
            handle_http_message(msg, pending_requests).await;
        }
        ProtocolType::Grpc => {
            // gRPC: 通过 request_id 和 chunk 标记识别
            handle_grpc_message(msg, pending_requests, grpc_streams).await;
        }
        ProtocolType::Websocket => {
            // WebSocket: 通过 request_id（连接 ID）识别连接和帧
            handle_websocket_message(msg, pending_requests, websocket_connections).await;
        }
        _ => {}
    }
}

async fn handle_grpc_message(
    msg: TunnelMessage,
    pending_requests: &Arc<DashMap<String, PendingRequest>>,
    grpc_streams: &Arc<GrpcStreamManager>,
) {
    match msg.payload {
        Some(Payload::GrpcRequest(req)) => {
            // gRPC 请求 chunk
            if req.is_first_chunk {
                // 第一个 chunk：创建 stream state
                grpc_streams.handle_request_chunk(msg.request_id.clone(), req).await?;
            } else {
                // 后续 chunk：添加到 stream state
                grpc_streams.handle_request_chunk(msg.request_id.clone(), req).await?;
            }
        }
        Some(Payload::GrpcResponse(resp)) => {
            // gRPC 响应 chunk
            if resp.is_last_chunk {
                // 最后一个 chunk：完成请求
                if let Some((_, pending)) = pending_requests.remove(&msg.request_id) {
                    if let PendingRequest::Grpc(sender) = pending {
                        let _ = sender.send(resp);
                    }
                }
            } else {
                // 中间 chunk：继续添加到 stream
                grpc_streams.handle_response_chunk(msg.request_id.clone(), resp).await?;
            }
        }
        _ => {}
    }
}

async fn handle_websocket_message(
    msg: TunnelMessage,
    pending_requests: &Arc<DashMap<String, PendingRequest>>,
    websocket_connections: &Arc<DashMap<String, WebSocketConnection>>,
) {
    match msg.payload {
        Some(Payload::WebSocketRequest(req)) => {
            // WebSocket 握手请求
            handle_websocket_upgrade(msg.request_id.clone(), req).await;
        }
        Some(Payload::WebSocketResponse(resp)) => {
            // WebSocket 握手响应
            if let Some((_, pending)) = pending_requests.get(&msg.request_id) {
                if let PendingRequest::WebSocket(sender) = pending {
                    // 握手成功，建立连接状态
                    websocket_connections.insert(msg.request_id.clone(), WebSocketConnection {
                        request_id: msg.request_id.clone(),
                        // ...
                    });
                }
            }
        }
        Some(Payload::WebSocketFrame(frame)) => {
            // WebSocket 数据帧：通过 request_id（等于 connection_id）找到连接
            if let Some(conn) = websocket_connections.get(&msg.request_id) {
                // 根据 frame_direction 转发到前端或后端
                match frame.frame_direction {
                    Direction::ClientToServer => {
                        // 从前端来的帧，转发到后端
                        conn.backend_tx.send(frame).await?;
                    }
                    Direction::ServerToClient => {
                        // 从后端来的帧，转发到前端
                        conn.frontend_tx.send(frame).await?;
                    }
                }
            }
        }
        _ => {}
    }
}
```

### 2. WebSocket 帧封装和识别

```rust
/// WebSocket 帧封装
pub struct WebSocketFrameEncoder;

impl WebSocketFrameEncoder {
    /// 将原始 WebSocket 帧封装为 TunnelMessage
    pub fn encode_frame(
        request_id: String,
        frame: tungstenite::Message,
        direction: Direction,
    ) -> TunnelMessage {
        let (opcode, payload, fin) = match frame {
            tungstenite::Message::Text(text) => (1, text.into_bytes(), true),
            tungstenite::Message::Binary(data) => (2, data, true),
            tungstenite::Message::Close(_) => (8, vec![], true),
            tungstenite::Message::Ping(data) => (9, data, true),
            tungstenite::Message::Pong(data) => (10, data, true),
            _ => (0, vec![], true),
        };
        
        TunnelMessage {
            request_id: request_id.clone(),
            protocol_type: ProtocolType::Websocket as i32,
            payload: Some(Payload::WebSocketFrame(WebSocketFrame {
                connection_id: request_id.clone(),
                fin,
                opcode,
                masked: false,
                payload,
                frame_direction: direction as i32,
            })),
            // ...
        }
    }
    
    /// 将 TunnelMessage 解码为 WebSocket 帧
    pub fn decode_frame(msg: TunnelMessage) -> Option<(String, tungstenite::Message)> {
        if let Some(Payload::WebSocketFrame(frame)) = msg.payload {
            let ws_msg = match frame.opcode {
                1 => tungstenite::Message::Text(String::from_utf8_lossy(&frame.payload).to_string()),
                2 => tungstenite::Message::Binary(frame.payload),
                8 => tungstenite::Message::Close(None),
                9 => tungstenite::Message::Ping(frame.payload),
                10 => tungstenite::Message::Pong(frame.payload),
                _ => return None,
            };
            
            Some((frame.connection_id, ws_msg))
        } else {
            None
        }
    }
}
```

### 3. gRPC Streaming 请求拆分

```rust
/// gRPC Streaming 请求拆分器
pub struct GrpcStreamSplitter;

impl GrpcStreamSplitter {
    /// 将 gRPC Streaming 请求拆分为多个 chunks
    pub async fn split_request_stream(
        request_id: String,
        mut stream: tonic::Streaming<Bytes>,
        tunnel_tx: mpsc::Sender<TunnelMessage>,
    ) -> Result<(), String> {
        let mut is_first = true;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let is_last = chunk.is_empty();  // 简化：空 chunk 表示结束
            
            let tunnel_msg = TunnelMessage {
                request_id: request_id.clone(),
                protocol_type: ProtocolType::Grpc as i32,
                payload: Some(Payload::GrpcRequest(GrpcRequest {
                    grpc_stream_id: request_id.clone(),
                    body: chunk.to_vec(),
                    is_first_chunk: is_first,
                    is_last_chunk: is_last,
                    // ...
                })),
                // ...
            };
            
            tunnel_tx.send(tunnel_msg).await?;
            is_first = false;
        }
        
        Ok(())
    }
    
    /// 将多个 chunks 组装为完整的 gRPC Streaming 响应
    pub async fn assemble_response_stream(
        request_id: String,
        mut chunks: VecDeque<Bytes>,
    ) -> Result<tonic::Streaming<Bytes>, String> {
        // 将 chunks 组装为 stream
        let stream = tokio_stream::iter(chunks.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }
}
```

---

## ✅ 识别机制总结

### WebSocket 消息识别

1. **连接识别**: 通过 `request_id`（等于 `connection_id`）识别不同的 WebSocket 连接
2. **帧识别**: 通过 `opcode` 识别帧类型（text/binary/close/ping/pong）
3. **方向识别**: 通过 `frame_direction` 识别帧的方向（CLIENT_TO_SERVER 或 SERVER_TO_CLIENT）

### gRPC Streaming 请求拆分

1. **Stream 识别**: 通过 `request_id` 识别不同的 gRPC Streaming RPC
2. **Chunk 识别**: 通过 `is_first_chunk` 和 `is_last_chunk` 标识 chunk 边界
3. **完整性保证**: 通过 `grpc_stream_id` 关联后端 stream，确保请求完整性

### 关键保证

- ✅ **唯一性**: 每个请求/连接有唯一的 `request_id`
- ✅ **边界标识**: 通过 `is_first_chunk` 和 `is_last_chunk` 标识消息边界
- ✅ **状态管理**: 通过 `request_id` 维护连接和 stream 状态
- ✅ **完全并发**: 所有消息并发处理，通过 `request_id` 匹配

---

## 🎯 总结

### WebSocket 识别

- ✅ 通过 `request_id`（连接 ID）识别不同的连接
- ✅ 通过 `opcode` 识别帧类型
- ✅ 通过 `frame_direction` 识别帧方向
- ✅ 整个连接生命周期内 `request_id` 不变

### gRPC Streaming 拆分

- ✅ 通过 `request_id` 识别不同的 Streaming RPC
- ✅ 通过 `is_first_chunk` 和 `is_last_chunk` 标识 chunk 边界
- ✅ 通过 `grpc_stream_id` 关联后端 stream
- ✅ 多个 Streaming RPC 可以并发处理

### 实现保证

- ✅ **准确识别**: 通过 `request_id` + `protocol_type` + chunk 标记
- ✅ **正确拆分**: 通过 chunk 标记和状态管理
- ✅ **完全并发**: 所有消息并发处理，互不阻塞

---

**结论**: 通过 `request_id` + chunk 标记 + 状态管理，可以准确识别 WebSocket 消息和拆分 gRPC Streaming 请求！

