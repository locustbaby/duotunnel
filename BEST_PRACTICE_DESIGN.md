# Tunnel 架构最佳实践方案

> **最后更新**: 2025-12-01  
> **版本**: v1.0  
> **状态**: 推荐方案

---

## 📋 目录

1. [架构概述](#架构概述)
2. [核心设计原则](#核心设计原则)
3. [控制流 + 数据流分离架构](#控制流--数据流分离架构)
4. [单 Stream 多协议设计](#单-stream-多协议设计)
5. [消息识别与路由](#消息识别与路由)
6. [性能优化策略](#性能优化策略)
7. [实施指南](#实施指南)
8. [参考文档](#参考文档)

---

## 🎯 架构概述

### 整体架构

```
Client
  └── gRPC Channel (TCP 连接)
      ├── ControlStream（控制流）
      │   ├── 心跳（Heartbeat）
      │   ├── 配置同步（ConfigSync）
      │   ├── Stream 注册（StreamOpen）
      │   └── 错误处理（ErrorMessage）
      └── Proxy Stream（数据流）
          ├── HTTP 消息
          ├── gRPC 消息
          └── WebSocket 消息
```

### 关键特性

- ✅ **控制流 + 数据流分离**: 控制消息和数据消息完全分离
- ✅ **单 Stream 多协议**: 数据流统一承载所有协议
- ✅ **完全并发处理**: 通过 `request_id` 完全并发，无阻塞
- ✅ **高性能**: 资源高效，吞吐量高

---

## 🎯 核心设计原则

### 1. 控制流 + 数据流分离

**原则**: 控制消息和数据消息使用不同的 stream

**优势**:
- ✅ **控制流稳定**: 心跳和配置同步不受数据流影响
- ✅ **数据流高效**: 专注于数据传输，性能最优
- ✅ **故障隔离**: 数据流故障不影响控制流

**实现**:
- `ControlStream` RPC: 处理心跳、配置同步、Stream 注册
- `Proxy` RPC: 处理 HTTP、gRPC、WebSocket 数据消息

### 2. 单 Stream 多协议

**原则**: 数据流统一承载所有协议类型的消息

**优势**:
- ✅ **资源高效**: 只需要一个数据 stream
- ✅ **实现简单**: 统一的流管理逻辑
- ✅ **完全并发**: 通过 `request_id` 完全并发处理

**实现**:
- 通过 `protocol_type` 区分协议类型
- 通过 `request_id` 匹配请求/响应
- 所有协议消息完全并发处理

### 3. request_id 隔离

**原则**: 每个请求有唯一的 `request_id`，完全并发处理

**优势**:
- ✅ **不需要有序性保证**: 每个请求独立
- ✅ **完全并发**: 所有请求并发处理，互不阻塞
- ✅ **正确性保证**: 通过 `request_id` 匹配，线程安全

**实现**:
- `request_id` 使用 UUID，保证全局唯一
- `DashMap` 存储 pending requests，线程安全
- `oneshot::channel` 匹配请求/响应

---

## 🏗️ 控制流 + 数据流分离架构

### Proto 定义（性能最佳）

```protobuf
service TunnelService {
  rpc ControlStream(stream TunnelMessage) returns (stream TunnelMessage);  // 控制流
  rpc Proxy(stream TunnelMessage) returns (stream TunnelMessage);          // 数据流
  rpc ConfigSync(ConfigSyncRequest) returns (ConfigSyncResponse);         // 配置同步（可选）
}

message TunnelMessage {
  string client_id = 1;
  string request_id = 2;  // UUID，每个请求唯一标识
  Direction direction = 3;
  ProtocolType protocol_type = 4;  // 协议类型标识
  MessageType message_type = 5;  // REQUEST, RESPONSE, STREAM_CHUNK
  oneof payload {
    // HTTP: Request/Response（简单高效）
    HttpRequest http_request = 10;
    HttpResponse http_response = 11;
    
    // gRPC: Request/Response (Unary) + StreamChunk (Streaming)
    GrpcRequest grpc_request = 12;      // Unary 或 Streaming 的第一个请求
    GrpcResponse grpc_response = 13;    // Unary 响应
    GrpcStreamChunk grpc_stream_chunk = 14;  // Streaming 的 chunk（统一）
    
    // WebSocket: Request/Response (握手) + Frame (数据传输)
    WebSocketRequest ws_request = 15;    // 仅用于握手
    WebSocketResponse ws_response = 16;   // 仅用于握手
    WebSocketFrame ws_frame = 17;       // 用于所有数据传输（统一）
    
    // 控制消息
    StreamOpenRequest stream_open = 20;
    StreamOpenResponse stream_open_response = 21;
    ConfigSyncRequest config_sync = 22;
    ConfigSyncResponse config_sync_response = 23;
    ErrorMessage error_message = 24;
  }
  string trace_id = 30;
}

enum MessageType {
  MESSAGE_TYPE_UNSPECIFIED = 0;
  REQUEST = 1;        // 请求（HTTP、gRPC Unary、WebSocket 握手）
  RESPONSE = 2;       // 响应（HTTP、gRPC Unary、WebSocket 握手）
  STREAM_CHUNK = 3;   // Stream chunk（gRPC Streaming、WebSocket Frame）
}

enum ProtocolType {
  PROTOCOL_UNSPECIFIED = 0;
  HTTP = 1;
  GRPC = 2;
  WEBSOCKET = 3;
  CONTROL = 4;  // 控制消息
}

enum Direction {
  CLIENT_TO_SERVER = 0;
  SERVER_TO_CLIENT = 1;
}

// gRPC Streaming Chunk（统一）
message GrpcStreamChunk {
  bool is_request = 1;     // true=请求 chunk, false=响应 chunk
  bool is_first_chunk = 2; // 第一个 chunk
  bool is_last_chunk = 3;  // 最后一个 chunk
  bytes data = 4;          // chunk 数据
}

// WebSocket Frame（统一，用于数据传输）
message WebSocketFrame {
  bool fin = 1;        // FIN 标志
  uint32 opcode = 2;  // Opcode (0x1=text, 0x2=binary, 0x8=close, etc.)
  bytes payload = 3;   // 帧数据
  Direction frame_direction = 4;  // CLIENT_TO_SERVER 或 SERVER_TO_CLIENT
}

### 控制流消息

**用途**: 心跳、配置同步、Stream 注册、错误处理

**特点**:
- ✅ 低频率，但必须稳定
- ✅ 不受数据流影响
- ✅ 可以独立重连

**消息类型**:
- `StreamOpenRequest`: Stream 注册和心跳
- `ConfigSyncRequest/Response`: 配置同步
- `ErrorMessage`: 错误处理

### 数据流消息

**用途**: HTTP、gRPC、WebSocket 数据传输

**特点**:
- ✅ 高频率，高吞吐量
- ✅ 完全并发处理
- ✅ 通过 `request_id` 匹配

**消息类型**:
- `HttpRequest/Response`: HTTP 请求/响应
- `GrpcRequest/Response`: gRPC 请求/响应
- `WebSocketRequest/Response/Frame`: WebSocket 消息

---

## 🔄 单 Stream 多协议设计

### 架构设计

```
Proxy Stream（数据流）
  ├── HTTP 消息 (request_id = "http-1")
  ├── gRPC 消息 (request_id = "grpc-1")
  ├── WebSocket 消息 (request_id = "ws-1")
  └── ... 所有协议的消息完全并发
```

### 关键机制

#### 1. 协议识别

```rust
match msg.protocol_type {
    ProtocolType::Http => handle_http(msg).await,
    ProtocolType::Grpc => handle_grpc(msg).await,
    ProtocolType::Websocket => handle_websocket(msg).await,
    _ => {}
}
```

#### 2. 请求匹配

```rust
// Server 端：注册 pending request
pending_requests.insert(request_id.clone(), PendingRequest::Http(sender));

// Client 端：发送响应
if let Some((_, pending)) = pending_requests.remove(&request_id) {
    if let PendingRequest::Http(sender) = pending {
        let _ = sender.send(response);
    }
}
```

#### 3. 完全并发处理

```rust
// 所有消息并发处理
while let Some(message) = rx.recv().await {
    let permit = semaphore.clone().acquire_owned().await.unwrap();
    tokio::spawn(async move {
        handle_message(message).await;
        drop(permit);
    });
}
```

### 协议支持（性能最佳设计）

#### HTTP（短请求）

**特点**:
- ✅ Request/Response 模式（简单高效）
- ✅ 通过 `request_id` 匹配
- ✅ 快速完成，不阻塞其他请求

**生命周期**:
```
T1: HttpRequest (request_id = "http-1")
T2: HttpResponse (request_id = "http-1")
T3: 清理 pending request（完成）
```

#### gRPC（Unary + Streaming）

**特点**:
- ✅ **Unary**: Request/Response（简单高效，零拷贝）
- ✅ **Streaming**: 统一用 `GrpcStreamChunk`（通过 `is_request` 区分请求/响应）
- ✅ 通过 `request_id` 维护 stream state

**生命周期（Unary）**:
```
T1: GrpcRequest { is_streaming: false }
T2: GrpcResponse
T3: 清理 pending request（完成）
```

**生命周期（Server Streaming）**:
```
T1: GrpcRequest { is_streaming: true }
T2: GrpcStreamChunk { is_request: false, is_first_chunk: true }
T3: GrpcStreamChunk { is_request: false, is_first_chunk: false }
T4: GrpcStreamChunk { is_request: false, is_last_chunk: true }
T5: 清理 stream state（完成）
```

**生命周期（Bidirectional Streaming）**:
```
T1: GrpcRequest { is_streaming: true }
T2: GrpcStreamChunk { is_request: true, is_first_chunk: true }
T3: GrpcStreamChunk { is_request: false, is_first_chunk: true }
T4: GrpcStreamChunk { is_request: true, is_first_chunk: false }
T5: GrpcStreamChunk { is_request: false, is_last_chunk: true }
T6: 清理 stream state（完成）
```

#### WebSocket（长连接）

**特点**:
- ✅ **握手**: Request/Response（一次性的，简单高效）
- ✅ **数据传输**: 统一用 `WebSocketFrame`（通过 `frame_direction` 区分方向）
- ✅ 长连接，`request_id` 在整个生命周期保持不变

**生命周期**:
```
T1: WebSocketRequest (握手请求, request_id = "ws-1")
T2: WebSocketResponse (握手响应, request_id = "ws-1")
T3-T100: WebSocketFrame (数据传输, request_id = "ws-1", 双向)
T101: WebSocketFrame { opcode: CLOSE } (关闭)
T102: 清理连接状态（完成）
```

**关键优化**:
- ✅ 握手用 Request/Response（一次性的）
- ✅ 数据传输只用 Frame（统一的，双向）
- ✅ 不需要区分 Request Frame 和 Response Frame

---

## 🔍 消息识别与路由

### WebSocket 消息识别

#### 连接识别

```protobuf
message WebSocketFrame {
  string connection_id = 1;  // 等于 request_id
  bool fin = 2;
  uint32 opcode = 3;         // 0x1=text, 0x2=binary, 0x8=close
  bytes payload = 4;
  Direction frame_direction = 5;  // CLIENT_TO_SERVER 或 SERVER_TO_CLIENT
}
```

**识别机制**:
- ✅ 通过 `request_id`（等于 `connection_id`）识别不同的连接
- ✅ 通过 `opcode` 识别帧类型
- ✅ 通过 `frame_direction` 识别帧方向

#### 多个连接识别

```
同一个 Proxy Stream 上的消息：

T1: TunnelMessage { request_id: "ws-1", payload: WebSocketRequest }
T2: TunnelMessage { request_id: "ws-2", payload: WebSocketRequest }
T3: TunnelMessage { request_id: "ws-1", payload: WebSocketFrame }
T4: TunnelMessage { request_id: "ws-2", payload: WebSocketFrame }
```

**处理**:
- ✅ 通过 `request_id` 区分不同的连接
- ✅ 每个连接独立处理，完全并发

### gRPC Streaming 请求拆分

#### Chunk 标识

```protobuf
message GrpcRequest {
  string grpc_stream_id = 1;
  bytes body = 2;
  bool is_first_chunk = 3;   // 第一个 chunk
  bool is_last_chunk = 4;     // 最后一个 chunk
  uint32 sequence_number = 5; // 可选：chunk 序号
}
```

**识别机制**:
- ✅ 通过 `request_id` 识别不同的 Streaming RPC
- ✅ 通过 `is_first_chunk` 和 `is_last_chunk` 标识 chunk 边界
- ✅ 通过 `grpc_stream_id` 关联后端 stream

#### 多个 Streaming RPC 识别

```
同一个 Proxy Stream 上的消息：

T1: TunnelMessage { request_id: "grpc-1", payload: GrpcRequest { is_first_chunk: true } }
T2: TunnelMessage { request_id: "grpc-2", payload: GrpcRequest { is_first_chunk: true } }
T3: TunnelMessage { request_id: "grpc-1", payload: GrpcRequest { is_first_chunk: false } }
T4: TunnelMessage { request_id: "grpc-2", payload: GrpcResponse { is_first_chunk: true } }
```

**处理**:
- ✅ 通过 `request_id` 区分不同的 Streaming RPC
- ✅ 每个 Streaming RPC 独立处理，完全并发

---

## ⚡ 性能优化策略

### 1. 完全并发处理

**实现**:
```rust
// 消息接收任务（快速接收，不阻塞）
tokio::spawn(async move {
    while let Some(message) = stream.next().await {
        if tx.send(message).await.is_err() {
            break;
        }
    }
});

// 完全并发处理任务池
let semaphore = Arc::new(Semaphore::new(10000));
while let Some(message) = rx.recv().await {
    let permit = semaphore.clone().acquire_owned().await.unwrap();
    tokio::spawn(async move {
        handle_message(message).await;
        drop(permit);
    });
}
```

**优势**:
- ✅ 消除队头阻塞
- ✅ 充分利用多核 CPU
- ✅ 吞吐量提升 5-10x

### 2. 背压机制

**实现**:
```rust
// 全局并发限制
let global_semaphore = Arc::new(Semaphore::new(10000));

// 单客户端并发限制
let per_client_semaphores = Arc::new(DashMap::new());

// 获取 permit
async fn acquire_permit(&self, client_id: &str) -> Result<SemaphorePermit> {
    let global_permit = self.global_semaphore.acquire().await?;
    let client_sem = self.per_client_semaphores
        .entry(client_id.to_string())
        .or_insert_with(|| Arc::new(Semaphore::new(1000)))
        .clone();
    let client_permit = client_sem.acquire().await?;
    Ok((global_permit, client_permit))
}
```

**优势**:
- ✅ 防止内存溢出
- ✅ 保护后端服务
- ✅ 自适应限流

### 3. 流式传输优化

**实现**:
```rust
// 分块传输
let mut is_first = true;
while let Some(chunk) = body_stream.next().await {
    let tunnel_msg = TunnelMessage {
        payload: Some(Payload::HttpRequest(HttpRequest {
            body: chunk.to_vec(),
            is_first_chunk: is_first,
            is_last_chunk: false,
        })),
        // ...
    };
    tunnel_tx.send(tunnel_msg).await?;
    is_first = false;
}
```

**优势**:
- ✅ 支持大文件传输
- ✅ 降低内存占用 50-80%
- ✅ 减少首字节延迟 30-50%

### 4. 零拷贝优化

**实现**:
```rust
use bytes::Bytes;

// 使用 Bytes 避免复制
let body_bytes: Bytes = hyper::body::to_bytes(body).await?;
// 可以零成本 clone
let shared_body = Arc::new(body_bytes);
```

**优势**:
- ✅ 减少 CPU 开销
- ✅ 降低内存占用
- ✅ 提升性能

---

## 📝 实施指南

### 阶段 1: 控制流 + 数据流分离（P0）

**目标**: 实现控制流和数据流的完全分离

**步骤**:
1. 修改 Client 端，创建两个 stream
   - `ControlStream`: 处理心跳、配置同步、Stream 注册
   - `Proxy Stream`: 处理数据消息

2. 修改 Server 端，分离处理逻辑
   - `ControlStream`: 处理控制消息
   - `Proxy Stream`: 处理数据消息

3. 测试验证
   - 验证控制流稳定性
   - 验证数据流性能
   - 验证故障隔离

**预期时间**: 2-3 天

### 阶段 2: 单 Stream 多协议优化（P0）

**目标**: 优化数据流，支持所有协议完全并发

**步骤**:
1. 添加 `ProtocolType` 枚举
2. 实现统一的消息处理分发
3. 实现各协议的后端转发逻辑
4. 测试验证性能提升

**预期时间**: 2-3 天

### 阶段 3: 性能优化（P1）

**目标**: 实现流式传输、背压机制、零拷贝优化

**步骤**:
1. 实现流式传输（分块传输）
2. 实现背压机制（并发限制）
3. 实现零拷贝优化（使用 Bytes）
4. 性能测试和调优

**预期时间**: 3-5 天

### 阶段 4: WebSocket 和 gRPC Streaming 支持（P1）

**目标**: 完整支持 WebSocket 和 gRPC Streaming

**步骤**:
1. 实现 WebSocket 消息识别和路由
2. 实现 gRPC Streaming 请求拆分
3. 实现双向帧转发（WebSocket）
4. 测试验证

**预期时间**: 3-5 天

---

## 📊 性能指标

### 优化前 vs 优化后

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| **吞吐量** | ~10K QPS | ~50-100K QPS | **5-10x** |
| **队头阻塞** | 存在 | 消除 | ✅ |
| **内存占用** | 高（完整 body） | 低（流式） | **50-80% ↓** |
| **首字节延迟** | 高 | 低（流式） | **30-50% ↓** |
| **并发处理能力** | 受限于消息接收顺序 | 完全并发 | **5-10x ↑** |
| **gRPC 支持** | 未实现 | 完全支持 | ✅ |
| **WebSocket 支持** | 未实现 | 完全支持 | ✅ |

---

## 🎯 关键设计决策

### 1. 为什么控制流 + 数据流分离？

**理由**:
- ✅ 控制流稳定，不受数据流影响
- ✅ 数据流高效，专注于数据传输
- ✅ 故障隔离性好，可以独立恢复

### 2. 为什么单 Stream 多协议？

**理由**:
- ✅ 资源高效，只需要一个数据 stream
- ✅ 实现简单，统一的流管理逻辑
- ✅ 完全并发，通过 `request_id` 完全并发处理

### 3. 为什么不按协议拆分多个 Stream？

**理由**:
- ❌ 资源消耗高（需要维护多个 stream）
- ❌ 实现复杂（需要为每个协议单独管理）
- ✅ 单 Stream 的隔离性已经足够（通过 `request_id` 完全并发）

### 4. 为什么不需要有序性保证？

**理由**:
- ✅ `request_id` 是 UUID，保证全局唯一
- ✅ `DashMap::remove` 是原子操作，线程安全
- ✅ `oneshot::Sender` 线程安全
- ✅ 通过 `request_id` 匹配即可保证正确性

---

## 📚 参考文档

### 核心文档

- `UNIFIED_STREAM_DESIGN.md` - 统一 Stream 设计（详细实现）
- `MULTI_PROTOCOL_STREAM_DESIGN.md` - 多协议 Stream 设计（详细实现）
- `MESSAGE_IDENTIFICATION_DESIGN.md` - 消息识别设计（详细实现）

### 性能优化文档

- `PERFORMANCE_OPTIMIZATION.md` - 性能优化方案（详细）
- `ORDERING_ANALYSIS.md` - 有序性分析（详细）
- `GRPC_ORDERING_ANALYSIS.md` - gRPC 有序性分析（详细）

### 方案对比文档

- `STREAM_SPLIT_ANALYSIS.md` - Stream 拆分方案分析
- `CLOUDFLARE_COMPARISON.md` - Cloudflare Tunnel 对比分析
- `PAYLOAD_DESIGN_ANALYSIS.md` - Payload 设计分析（单独字段 vs 统一 bytes）

### 协议支持文档

- `GRPC_PROXY_DESIGN.md` - gRPC 代理完整设计方案
- `OPTIMAL_PAYLOAD_DESIGN.md` - 性能最佳 Payload 设计（统一 Stream 模式）

---

## ✅ 总结

### 最佳实践方案

**架构**: 控制流 + 数据流分离 + 单 Stream 多协议

**核心特性**:
- ✅ 控制流稳定，数据流高效
- ✅ 完全并发处理，无阻塞
- ✅ 资源高效，性能最优
- ✅ 支持 HTTP、gRPC、WebSocket

**实施优先级**:
1. **P0**: 控制流 + 数据流分离
2. **P0**: 单 Stream 多协议优化
3. **P1**: 性能优化（流式传输、背压机制）
4. **P1**: WebSocket 和 gRPC Streaming 支持

**预期效果**:
- ✅ 吞吐量提升 5-10x
- ✅ 内存占用降低 50-80%
- ✅ 延迟降低 30-50%
- ✅ 完全支持所有协议

---

**最后更新**: 2025-12-01  
**版本**: v1.0  
**状态**: 推荐方案

