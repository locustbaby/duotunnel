# DuoTunnel 已知问题记录

记录已分析但暂未修复的问题，供后续参考。

---

## Issue 1: HTTP/1.1 Keep-Alive 连接复用失败

### 问题描述

外部 HTTP/1.1 客户端在同一 TCP 连接上发送多个请求时，第二个及后续请求会失败（连接被 reset）。

### 根本原因：架构不对称

Server 侧把整个 TCP 连接作为字节流透传到一条 QUIC stream：

```
外部 TCP 连接 (生命周期 N 秒)
  └─► 1条 QUIC stream (server/proxy/base.rs:forward_with_initial_data)
        └─► tokio::io::copy 持续运行直到 TCP EOF
```

Client 侧每条 QUIC stream 只处理一个 HTTP 请求：

```
QUIC stream
  └─► HttpPeer::connect (tunnel-lib/src/proxy/http.rs)
        └─► Http1Driver::read_request()  ← 读一个请求
        └─► hyper::Client::request()
        └─► driver.write_response()
        └─► send.finish()               ← 关闭 QUIC stream，丢弃剩余字节
```

### 失败流程

1. 外部客户端发 Request #1，server 开一条 QUIC stream，client 处理完关闭 stream
2. 外部客户端复用 TCP 连接发 Request #2，bytes 进入已关闭的 QUIC stream RecvStream
3. QUIC 层发 `STOP_SENDING`，server 侧 `tcp_to_quic` copy 失败
4. `forward_with_initial_data` 返回，TCP 连接被 drop
5. 外部客户端收到 connection reset

### 额外问题

`Http1Driver::write_response` 始终写 `Transfer-Encoding: chunked` 但**不写 `Connection: close`**（[tunnel-lib/src/protocol/driver/h1.rs:219](../tunnel-lib/src/protocol/driver/h1.rs#L219)），导致客户端被误导为"连接可复用"，加剧了 reset 的发生。

### 实际影响

| 客户端行为 | 影响 |
|---|---|
| 浏览器 GET 请求 | 自动重试，用户感知到多一次 RTT |
| 浏览器 POST / API 调用 | 不重试，直接报网络错误 |
| curl / fetch（不自动重试） | 报 connection reset |
| `Connection: close` 或 HTTP/1.0 | 不受影响 |

### GT / frp 的对比

GT 和 frp 均使用**纯字节流透传**架构，不在代理层解析 HTTP 边界：
- GT：单 QUIC stream + taskID 帧协议，整个 TCP 连接 = 一个 taskID，keep-alive 透明
- frp：整条 TCP 连接透传给 workConn，本地服务自行处理 keep-alive

DuoTunnel 在 client 侧引入了 `HttpPeer` 做 HTTP 解析（可做 header 重写等高级功能），但因此需要显式处理 keep-alive 边界。

### 修复方案

**方案 A（最小改动）**：在 `Http1Driver::write_response` 写入 `Connection: close`，强制客户端每次新开 TCP 连接。语义上诚实，无 bug，但每请求一次握手（即当前实际行为，只是消除了误导）。

**方案 B（正确支持 keep-alive）**：Client 侧 `HttpPeer::connect` 改为循环处理多请求。难点在于 `Http1Driver::read_request` 将 `RecvStream` move 进 streaming body 闭包，消费后无法回收，需要重构为 `Arc<Mutex<RecvStream>>` 或 channel 方式，复杂度较高。

**方案 C（server 侧按请求 open_bi）**：Server 侧解析 HTTP/1.1 边界，每个请求单独 `open_bi()`，Client 侧不需要修改。需要 server 端理解 HTTP framing，改动范围较大。

### 状态

暂不修复。当前行为（每请求新 TCP 连接）与 HTTP/1.0 语义一致，优先级低于其他优化项。

---

## Issue 2: Buffer Pool 未实现

### 问题描述

热路径上无 buffer 复用机制，高并发下每个请求仍有堆分配。

### 现状

- `server/handlers/http.rs:52`：`let mut buf = [0u8; 16384]` — 已改为栈分配 ✅
- `server/handlers/tcp.rs:64`：`let mut buf = [0u8; 4096]` — 已改为栈分配 ✅
- `tunnel-lib/src/proxy/core.rs:47`：`let mut buf = vec![0u8; 4096]` — 仍是堆分配
- `tunnel-lib/src/protocol/driver/h1.rs:52`：`let mut first_buf = vec![0u8; 8192]` — 仍是堆分配
- `tunnel-lib/src/protocol/driver/h1.rs:173`：`let mut buf = vec![0u8; 8192]` — 仍是堆分配

### 修复方案

引入 `thread_local!` + `RefCell<Vec<u8>>` 或 crossbeam 对象池，在 `core.rs` 和 `Http1Driver` 中复用 read buffer。

### 状态

暂不修复。peek buffer 已优化为栈分配，剩余分配频率相对较低，收益不足以支撑引入复杂度。

---

## Issue 3: server/tunnel_handler.rs 中 `_protocol` 变量无用

### 问题描述

`server/tunnel_handler.rs:23` 计算了 `_protocol` 但从未使用，仅作为 suppress clippy warning 的临时解法。

```rust
let _protocol = match routing_info.protocol.as_str() {
    "websocket" => Protocol::WebSocket,
    ...
};
```

`engine.run_stream` 直接接收 `routing_info` 并在内部重新解析协议，`_protocol` 是死代码。

### 修复方案

直接删除这个 match 块，`Protocol` import 也可一并移除。

### 状态

低优先级，不影响正确性。
