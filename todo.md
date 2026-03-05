# Tunnel TODO

## Config 调优（不改代码）

### [TODO-16] QUIC connections: 1 → 4

**文件**: `ci-helpers/client.yaml`
**优先级**: 高

client.yaml 里 `quic.connections` 没配，默认 1。所有流量挤在单条 QUIC 连接上，
是单 UDP socket 串行加解密 + 单连接流控的瓶颈。

改为 `connections: 4`，让 4 条 QUIC 连接分摊负载。

### [TODO-17] max_concurrent_streams: 200 → 1000

**文件**: `ci-helpers/client.yaml`, `ci-helpers/server.yaml`
**优先级**: 高

CI 配置里 server 和 client 都设了 200。3K QPS no-keepalive 场景下，
同时在飞的 stream 容易超 200，超过就 `try_acquire_owned` 直接丢弃连接。

改为 1000。

---

## 代码优化

### [TODO-14] discard buffer 改栈分配

**文件**: `server/handlers/http.rs`
**优先级**: 低

drain socket 时仍用堆分配 `vec![0u8; len]`，len ≤ 8192，可改为栈 buffer。

### [TODO-18] H1 body 读取 BytesMut::zeroed → unsafe set_len

**文件**: `tunnel-lib/src/protocol/driver/h1.rs:276`
**优先级**: 中

每次 body chunk 读取省一次 memset（8KB）。大 body 请求路径 CPU -30%。

### [TODO-19] H1 double header parse → 单次

**文件**: `tunnel-lib/src/protocol/driver/h1.rs:104-140`
**优先级**: 中

当前流程：循环 parse 直到 Complete（丢弃结果），然后再 parse 一次提取字段。
改为：loop 内 Complete 时直接记录 header_end，用同一次 parse 的结果。

### [TODO-20] Bytes::copy_from_slice → split_to().freeze() 零拷贝

**文件**: `tunnel-lib/src/protocol/driver/h1.rs:201,283,287`
**优先级**: 中

5 处 `Bytes::copy_from_slice` 都是从 BytesMut 拷贝到新 Bytes，
可以用 `split_to().freeze()` 实现零拷贝（共享底层内存）。

### [TODO-21] tokio::io::copy 8KB → 64KB buffer

**文件**: `tunnel-lib/src/engine/bridge.rs`
**优先级**: 中

`tokio::io::copy()` 内部默认 8KB buffer。QUIC stream window 4MB、TCP socket buffer 4MB，
8KB 导致每 8KB 一次 syscall 开销。

用 `BufReader::with_capacity(65536)` 包装 reader 后传给 `tokio::io::copy_buf()`。

### [TODO-22] relay() split → into_split

**文件**: `tunnel-lib/src/engine/bridge.rs:9-10`
**优先级**: 低

泛型 `relay()` 用 `tokio::io::split()`（内部 Arc+Mutex），而 `relay_quic_to_tcp()`
正确用了 `into_split()`（零成本 owned halves）。

---

## 架构级优化

### [TODO-23] server entry listener SO_REUSEPORT

**文件**: `server/handlers/http.rs:13`, `server/handlers/tcp.rs:17`
**优先级**: 中

server 入口用 `TcpListener::bind()`，没有 `SO_REUSEPORT`。
`tunnel-lib/src/transport/listener.rs` 已有 `build_reuse_listener()`，但 server handler 没用它。

### [TODO-24] 多 QUIC Endpoint（多 UDP socket）

**优先级**: 低

quinn Endpoint 绑定单个 UDP socket，`recvmsg/sendmsg` 串行化。
即使开多条 QUIC connection 也走同一个 socket。
考虑 client 端创建多个 Endpoint（各绑不同端口），配合 SO_REUSEPORT 分散 UDP 处理。

### [TODO-25] io_uring 替代 epoll

**优先级**: 低（实验性）

用 tokio-uring 或 monoio 替代 tokio 的 epoll 后端。
目前无 Rust QUIC 库原生支持 io_uring，需等 quinn issue #915 推进。

---

## Bench 修复

### [TODO-15] egress_http_post 溢出 Phase 边界

**文件**: `ci-helpers/k6/bench.js`, `bench/index.html`
**优先级**: 低

`egress_http_post` startTime=6s + stages 5s+20s = 结束于 31s，但 Phase "Basic" end=29。
仅图表注释框不能完全覆盖该场景，不影响数据。
