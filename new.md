# 高性能多协议隧道代理详细设计方案 (v3 - 综合版)

## 1. 核心设计哲学

本方案的核心是 **“L4-over-L7”** 架构，旨在实现极致的性能、扩展性和可维护性。

- **L7 传输层**: 我们利用 gRPC 或 QUIC 作为 Client 和 Server 之间的底层传输协议。这使我们能享受其带来的连接复用、内置加密 (TLS) 和流量控制等现代化高级特性。
- **L4 隧道内容**: 在这条 L7 的“高速公路”上，我们传输的是逻辑上的 L4 原始字节流 (`bytes`)。隧道本身不关心、不解析流经它的应用层协议，从而避免了昂贵的“解析-序列化-反序列化-重组”循环。
- **L7 端点处理**: 所有应用层协议的解析、修改和处理，都只在隧道的入口（`client` 端）进行。`client` 扮演“智能 L7 代理”的角色，`server` 扮演“哑 L4 转发器”的角色。

## 2. 整体架构

系统分为四个主要部分：

1.  **`client` / `server` (应用层)**: 负责启动、配置和监听。根据配置加载不同的 `ProxyHandler`。
2.  **`proxy-handlers` (代理逻辑层)**: 包含每种应用层协议的具体代理逻辑，如 `HttpProxyHandler` 和 `GrpcProxyHandler`。
3.  **`tunnel-lib` (核心库)**: 协议无关的核心逻辑库，被 `client` 和 `server` 共享。包含 `StreamManager` 和 `LogicalStream`。
4.  **`transports` (传输层)**: 包含 `grpc-transport` 和 `quic-transport`，实现统一的 `Transport` Trait。

## 3. 协议定义 (`tunnel.proto`)

这是 Client 和 Server 之间通信的唯一契约，设计得尽可能简单和通用。

```protobuf
syntax = "proto3";
package tunnel;

service TunnelService {
  rpc BidirectionalStream(stream TunnelPacket) returns (stream TunnelPacket);
}

message TunnelPacket {
  string stream_id = 1;
  oneof payload {
    ControlMessage control = 2;
    bytes data = 3;
  }
}

message ControlMessage {
  oneof command {
    StreamOpenRequest open_request = 1;
    StreamOpenResponse open_response = 2;
    bool close = 3;
  }
}

message StreamOpenRequest {
  string upstream_address = 1;
  bool use_tls = 2;
}

message StreamOpenResponse {
  bool success = 1;
  string error_message = 2;
}
```

## 4. 核心抽象与 Trait 设计

### 4.1. `Transport` Trait

此 Trait 抽象了底层传输（gRPC/QUIC），使其可以被 `StreamManager` 无差别地使用。

```rust
use async_trait::async_trait;
use crate::tunnel_packet::TunnelPacket;

#[async_trait]
pub trait Transport: Send + Sync {
    // ... (connect, accept, etc.)

    /// 从底层连接读取一个数据包
    async fn read_packet(&mut self) -> Result<Option<TunnelPacket>, Error>;

    /// 向底层连接写入一个数据包
    async fn write_packet(&mut self, packet: TunnelPacket) -> Result<(), Error>;
}
```
- **功能**: 定义了建立、接受、读、写和关闭一个底层物理连接的标准接口。
- **实现**: 将会有 `GrpcTransport` 和 `QuicTransport` 两个结构体分别实现此 Trait。

### 4.2. `LogicalStream`: 虚拟网络连接

`LogicalStream` 是隧道中**单个代理会话**的具象化。它通过实现 `tokio::io::AsyncRead` 和 `tokio::io::AsyncWrite`，使其对于上层代码（无论是 `copy_bidirectional` 还是 `tonic`）来说，行为与一个标准的 `TcpStream` 完全一致。

- **功能**: 为单个代理会话提供一个虚拟的、双向的、异步的 I/O 通道。
- **核心实现**: 其内部的 `read` 和 `write` 方法分别与 `StreamManager` 的 `mpsc` channel 交互，将 I/O 操作转化为 `TunnelPacket` 的收发。这是实现 gRPC 代理的核心技术关键。

### 4.3. `ProxyHandler` Trait

这是对 `client` 端协议处理逻辑的关键抽象。

```rust
use async_trait::async_trait;
use tokio::net::TcpStream;
use crate::manager::StreamManager;

#[async_trait]
pub trait ProxyHandler: Send + Sync {
    /// 处理一个新接受的 TCP 连接
    async fn handle(&self, inbound_stream: TcpStream, stream_manager: &mut StreamManager<impl Transport>);
}
```
- **功能**: 定义了处理一种特定应用层协议（HTTP, gRPC, etc.）的统一接口。

## 5. 关键数据结构与处理器实现

### 5.1. `StreamManager`

这是 `tunnel-lib` 的大脑，负责管理所有逻辑流，并与单个 `Transport` 实例交互。

```rust
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct StreamManager<T: Transport> {
    transport: T,
    streams: HashMap<String, mpsc::Sender<bytes::Bytes>>,
    // ... other fields
}

impl<T: Transport> StreamManager<T> {
    // 启动 StreamManager，在 transport 和所有 streams 之间双向转发消息
    pub async fn run(mut self) { /* ... */ }

    // 为上层应用（ProxyHandler）创建一个新的逻辑流
    pub async fn new_stream(&mut self, req: StreamOpenRequest) -> Result<LogicalStream, Error> { /* ... */ }
}
```

### 5.2. `HttpProxyHandler`

- **功能**: 处理 HTTP/1.1 和 WebSocket 代理请求。
- **实现 `handle`**: 
    1. 调用 `stream_manager.new_stream()` 创建一个 `LogicalStream`。
    2. **核心逻辑**: 调用 `tokio::io::copy_bidirectional(&mut inbound_stream, &mut logical_stream)`。
    3. `copy_bidirectional` 会同时使用两个流的 `AsyncRead`/`AsyncWrite` 实现，高效地双向拷贝原始字节流。

### 5.3. `GrpcProxyHandler`

- **功能**: 处理 gRPC 代理请求（包括 Unary 和 Streaming）。
- **实现 `handle`**: 
    1. **协议终结**: 使用 `tonic` 将 `inbound_stream` 作为一个 gRPC **服务端**连接来处理。
    2. **定义服务**: 定义一个 `tonic` 服务，当一个 gRPC 调用被触发时：
        a. 调用 `stream_manager.new_stream()` 创建 `LogicalStream`。
        b. **协议重新发起**: 创建一个 `tonic` **客户端** `Channel`，通过 `connect_with_connector` 强制它使用 `LogicalStream` 作为底层 I/O。
        c. **缝合流**: 根据是 Unary 还是 Streaming 调用，进行请求/响应的转发或双向流的“缝合”。

## 6. 工作流程与核心问题详解

### 6.1. `LogicalStream` 和 `TunnelPacket` 的普适性

- **所有代理都需要 `LogicalStream`**: 无论是 HTTP, WebSocket, 还是 gRPC，每个代理会话都必须创建一个 `LogicalStream`。它是 `StreamManager` 用来区分和路由流量的**唯一凭证**。
- **所有数据都封装在 `TunnelPacket`**: 所有在底层传输的内容，无论是信令还是数据，都必须包裹在 `TunnelPacket` 结构中，这是 Client 和 Server 之间唯一的通信契约。

### 6.2. gRPC 代理：为何是特例？

gRPC (基于 HTTP/2) 本身是一个复杂的、自带复用和流控机制的 L7 协议。直接转发其字节流会导致“隧道中的隧道”问题，破坏两个独立 HTTP/2 会话的内部状态。因此，必须采用“终结并重新发起”的模式，而 `LogicalStream` 实现的 `AsyncRead`/`AsyncWrite` Trait 正是实现这一模式的核心适配器。

### 6.3. gRPC Unary vs. Streaming Call 的代理

- **Unary Call (短链接)**: 代理会话的生命周期与 `LogicalStream` 的生命周期一样短暂。请求处理完毕，`LogicalStream` 被 `drop`，`close` 信令被发送。
- **Streaming Call (长链接)**: `GrpcProxyHandler` 通过“流缝合”的异步任务，维持一个有状态的代理会话。只要下游和上游的流还活跃，`LogicalStream` 和其 `stream_id` 就会一直存在。流结束时，`LogicalStream` 才被 `drop`。

## 7. Client/Server 监听与分发

`client` 和 `server` 的主程序是一个分发器，它根据配置文件，为每个服务（如一个 HTTP 代理，一个 gRPC 代理）绑定一个 TCP 监听器，并实例化一个对应的 `ProxyHandler`。当有新连接进来时，它会 `spawn` 一个新任务，让 `handler` 去处理这个连接。

```rust
// client/src/main.rs (概念代码)
async fn main() {
    let config = load_config();
    let stream_manager = connect_to_server(&config).await;

    for service_config in config.services {
        let listener = TcpListener::bind(service_config.local_address).await.unwrap();
        let handler: Arc<dyn ProxyHandler> = match service_config.protocol {
            "http" => Arc::new(HttpProxyHandler::new(service_config.upstream)),
            "grpc" => Arc::new(GrpcProxyHandler::new(service_config.upstream)),
            _ => panic!("Unsupported protocol"),
        };

        tokio::spawn(async move {
            loop {
                let (inbound_stream, _) = listener.accept().await.unwrap();
                // ... spawn a task to handle the stream with the selected handler
            }
        });
    }
    // ...
}
```

## 8. 目录结构建议

```
.
├── Cargo.toml
├── client/
├── server/
├── tunnel-lib/
├── transports/
└── proxy-handlers/         # 新增: 代理逻辑实现
    ├── Cargo.toml
    └── src/
        ├── lib.rs          # 定义 ProxyHandler trait
        ├── http_handler.rs # HttpProxyHandler impl
        └── grpc_handler.rs # GrpcProxyHandler impl
```
