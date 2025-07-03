# 隧道代理项目分步实现方案

本文档基于 `new.md` 中的 v3 设计方案，提供一个从零开始、分步实现、逐步测试的开发路线图。

## 阶段 0: 项目结构与基础设置

**目标**: 搭建项目骨架，定义核心数据结构。

1.  **创建 Workspace**: 按照 `new.md` 第 8 节的目录结构，使用 `cargo new --lib` 和 `cargo new` 创建所有的 `crates` (`client`, `server`, `tunnel-lib`, `transports/quic-transport`, `proxy-handlers`)。
2.  **配置 `Cargo.toml`**: 在根目录的 `Cargo.toml` 中定义好 `[workspace]`，并设置好各 `crate` 之间的依赖关系。
3.  **协议定义**: 在 `transports/quic-transport` (或一个独立的 `proto` crate) 中创建 `tunnel.proto` 文件，并使用 `tonic-build` 或 `prost-build` 将其编译为 Rust 代码。这将生成 `TunnelPacket`, `ControlMessage` 等核心结构体。
4.  **定义 Trait**: 在 `transports` 的 `lib.rs` (或一个 `traits` crate) 中定义 `Transport` Trait。在 `proxy-handlers/src/lib.rs` 中定义 `ProxyHandler` Trait。

**产出**: 一个可以 `cargo build` 通过的项目骨架，所有核心数据结构和 Trait 已定义但未实现。

---

## 阶段 1: 核心传输层 (QUIC)

**目标**: 实现 Client 和 Server 之间最基础的、基于 QUIC 的通信能力。

1.  **实现 `QuicTransport`**: 在 `transports/quic-transport` 中，使用 `quinn` 库来实现 `Transport` Trait。
    - `connect()`: 实现客户端连接到服务端的逻辑。
    - `accept()`: 实现服务端接受一个新客户端的逻辑。
    - `write_packet()`: 将 `TunnelPacket` 序列化 (e.g., using `bincode` or `prost`) 并通过 QUIC 的 `send_datagram` 或 `open_bi` stream 发送。
    - `read_packet()`: 从 QUIC 连接读取数据并反序列化成 `TunnelPacket`。
2.  **编写 `client` 和 `server` 的 `main.rs`**: 
    - **Server**: 启动并监听一个 QUIC 端口。
    - **Client**: 连接到 Server 的 QUIC 端口。
3.  **测试**: 编写一个简单的测试。Client 连接成功后，通过 `write_packet` 发送一个硬编码的 `TunnelPacket`。Server 接收到后，通过 `read_packet` 读取并打印出来。验证双向通信。

**产出**: 一个可以建立稳定 QUIC 连接并在 Client 和 Server 之间交换 `TunnelPacket` 的基本程序。

---

## 阶段 2: `StreamManager` 与 `LogicalStream`

**目标**: 在 `tunnel-lib` 中构建隧道的核心逻辑，管理多个逻辑流。

1.  **实现 `LogicalStream`**: 在 `tunnel-lib` 中创建 `stream.rs`。实现 `LogicalStream` 结构体，包含 `stream_id` 和用于收发 `bytes` 的 `mpsc` channels。
2.  **实现 IO Traits**: 为 `LogicalStream` 实现 `tokio::io::AsyncRead` 和 `tokio::io::AsyncWrite`。这是后续代理 gRPC 的关键。
3.  **实现 `StreamManager`**: 在 `tunnel-lib` 中创建 `manager.rs`。
    - 实现 `new()`: 初始化 `HashMap` 和用于与 `Transport` 通信的 `mpsc` channel。
    - 实现 `run()`: 这是核心的事件循环。使用 `tokio::select!` 同时监听来自 `Transport` 的入站包和来自所有 `LogicalStream` 的出站包，并进行相应的转发。
    - 实现 `new_stream()`: 处理 `StreamOpenRequest`，与远端进行信令交互，创建并返回一个新的 `LogicalStream`。
4.  **集成**: 修改 `client` 和 `server` 的 `main.rs`。在 QUIC 连接建立后，实例化并启动 `StreamManager`。

**产出**: 一个功能完整的 `tunnel-lib`。虽然还没有代理任何应用，但隧道本身已经能够处理多个并发的、虚拟的流。

---

## 阶段 3: 实现 HTTP/WebSocket 代理

**目标**: 实现第一个有实际用途的代理功能。

1.  **实现 `HttpProxyHandler`**: 在 `proxy-handlers` crate 中创建 `http_handler.rs`。
    - 实现 `ProxyHandler` Trait 的 `handle` 方法。
    - 逻辑：调用 `stream_manager.new_stream()` 创建 `LogicalStream`，然后使用 `tokio::io::copy_bidirectional` 在传入的 `TcpStream` 和 `LogicalStream` 之间进行双向拷贝。
2.  **实现 `client` 监听逻辑**: 
    - 在 `client/src/main.rs` 中，根据配置文件，启动一个 `TcpListener` 监听本地端口 (e.g., `8080`)。
    - 在一个循环中 `accept()` 新的 TCP 连接。
    - 对于每个新连接，`spawn` 一个新任务，并调用 `HttpProxyHandler` 的 `handle` 方法来处理它。
3.  **实现 `server` 上游连接逻辑**: 
    - 在 `server` 的 `StreamManager` 中，当收到 `StreamOpenRequest` 时，根据请求中的 `upstream_address` 和 `use_tls`，使用 `tokio::net::TcpStream::connect` 连接到真正的上游服务。
4.  **端到端测试**: 
    - 启动 `server`。
    - 启动 `client`，配置其监听 `localhost:8080` 并代理到某个公共网站 (e.g., `http://example.com:80`)。
    - 在本地执行 `curl --proxy http://localhost:8080 http://example.com`。
    - 验证是否能正确获取网页内容。
    - 测试 WebSocket 代理。

**产出**: 一个功能完善的 HTTP 和 WebSocket 隧道代理。

---

## 阶段 4: 实现 gRPC 代理

**目标**: 攻克最复杂的 gRPC 代理。

1.  **实现 `GrpcProxyHandler`**: 在 `proxy-handlers` crate 中创建 `grpc_handler.rs`。
    - 实现 `ProxyHandler` Trait 的 `handle` 方法。
    - **终结逻辑**: 使用 `tonic` 和 `hyper` 将传入的 `TcpStream` 作为一个 gRPC 服务端连接。
    - **重新发起逻辑**: 定义一个 `tonic` 服务。在服务的方法被调用时，执行以下操作：
        a. 创建 `LogicalStream`。
        b. 创建一个连接到此 `LogicalStream` 的 `tonic` 客户端 `Channel`。
        c. “缝合” Unary 或 Streaming 调用。
2.  **更新 `client` 监听逻辑**: 修改 `client/src/main.rs`，使其能够根据配置加载并使用 `GrpcProxyHandler`。
3.  **端到端测试**: 
    - 准备一个简单的 gRPC 测试服务 (e.g., helloworld)。
    - 启动 `server`。
    - 启动 `client`，配置其监听 `localhost:50051` 并代理到测试服务。
    - 编写一个 gRPC 测试客户端，让它连接到 `localhost:50051` 并发起 Unary 和 Streaming 调用。
    - 验证调用是否成功。

**产出**: 一个支持 gRPC 代理的隧道，至此核心功能已全部完成。

---

## 阶段 5: 完善与收尾

**目标**: 增加健壮性、可观测性和易用性。

1.  **gRPC Transport**: 如果需要，可以按照阶段 1 的方式，实现 `GrpcTransport`，并允许在配置中切换底层传输协议。
2.  **错误处理**: 完善所有 `.unwrap()` 和 `panic!` 的地方，定义统一的 `Error` 类型，并实现优雅的错误处理和日志记录。
3.  **可观测性**: 引入 `tracing` 库进行结构化日志和分布式追踪。在关键位置（如 `new_stream`, `data_transfer`）添加 `metrics` 指标（如当前流数量、总吞吐量）。
4.  **配置与文档**: 完善 `client` 和 `server` 的配置文件，使其更灵活。编写 `README.md` 和用户文档。
5.  **打包与发布**: 编写 `Dockerfile` 或其他打包脚本，方便部署。

**产出**: 一个生产可用的、高性能、多协议的隧道代理项目。
