# DuoTunnel 项目分析报告

**生成时间**: 2026-02-22
**分支**: fix/perf-comparison
**对照文档**: CODE_REVIEW.md · PERF_COMPARISON.md · KNOWN_ISSUES.md
**对比对象**: frp (Go+TCP+yamux) · GT (Go+QUIC)

---

## 1. 项目架构概述

DuoTunnel 是一个基于 QUIC 的双向内网穿透代理，Rust workspace 由三个 crate 组成：

- **tunnel-lib** — 共享库：QUIC 传输、协议检测、代理引擎、消息编解码
- **server** — 接受外部 HTTP/TCP/TLS 流量，通过 QUIC 隧道路由至已注册的客户端
- **client** — 维持 QUIC 连接，将隧道流量转发至本地服务

核心数据流：

```
入站: 外部客户端 → Server (entry_port) → VHost/端口匹配 → QUIC stream → Client → 本地服务
出站: 本地客户端 → Client (http_entry_port) → QUIC stream → Server → 外部后端
```

与 frp（单 TCP 连接 + yamux 多路复用）和 GT（单 QUIC stream + 自定义帧协议）的关键架构差异：

| 维度 | DuoTunnel | GT | frp |
|------|-----------|----|----|
| 传输层 | QUIC (quinn 0.11) | QUIC / TCP+TLS | TCP+TLS / QUIC / KCP / WebSocket |
| 多路复用 | QUIC 原生 stream（每代理连接一个流） | 单 stream + taskID 帧协议 | yamux（TCP）/ QUIC stream |
| 流控 | QUIC 原生（每 stream 独立） | 无流级流控（自造协议） | yamux 流控 |
| HOL Blocking | ✅ 无（QUIC stream 隔离） | ❌ 有（单 stream 上所有 task 共享） | ❌ 有（TCP 路径） |

DuoTunnel 的架构方向是正确的——使用 QUIC 原生 stream 比 GT 的单 stream 帧协议在流控和 HOL blocking 方面都更优。现存问题主要集中在**实现层面的优化不足**。

---

## 2. CODE_REVIEW.md 问题修复状态全核查

### ✅ 已修复 (10/30)

#### Issue #1 — H2 流控缺失 (OOM 风险)
**原问题**: `reserve_capacity()` 调用后不等待，慢客户端无限缓冲导致 OOM。
**现状**: **已修复**。`tunnel-lib/src/proxy/h2.rs` 中增加了 `loop { poll_capacity() }` 阻塞等待，确认窗口容量 ≥ 帧大小才发送数据。

#### Issue #2 — Round-Robin 负载均衡错误
**原问题**: `DashMap::iter().nth(idx)` 不保证顺序，O(n) 遍历，并发不安全。
**现状**: **已修复**。`server/registry.rs` 中 `ClientGroup` 改为 `RwLock<Vec<(String, Connection)>>` + `AtomicUsize` 计数器，O(1) 轮询，顺序稳定。

#### Issue #3 — 重复客户端注册竞态条件
**原问题**: `get_client_connection` → `unregister` → `register` 三步不原子。
**现状**: **已修复**。改为 `replace_or_register()` 使用 DashMap `entry()` API 做原子 Check-and-Replace。

#### Issue #4 — 全局缺失 TCP_NODELAY
**原问题**: 全项目零处调用 `set_nodelay(true)`，Nagle 算法导致 40ms+ 延迟。
**现状**: **已修复**。
- `tunnel-lib/src/proxy/tcp.rs:157` (TcpPeer)
- `tunnel-lib/src/proxy/tcp.rs:182` (TlsTcpPeer)
- `server/handlers/http.rs:19` (入站 HTTP)
确认三处均有 `set_nodelay(true)` 调用。

#### Issue #6 — TLS 配置每次连接重建
**原问题**: 每个 `TlsTcpPeer::connect()` 克隆 ~150 个 WebPKI 根证书，数百次堆分配。
**现状**: **已修复**。`TlsTcpPeer` 内持有 `Arc<TlsConnector>`，根证书只在构造时加载一次，所有连接共享。

#### Issue #7 — QUIC 窗口大小未调优
**原问题**: 默认 stream 窗口 ~48KB，连接窗口 ~1.5MB，大文件传输瓶颈。
**现状**: **已修复**。`tunnel-lib/src/transport/quic.rs` 中新增 `QuicTransportParams`：
- `stream_receive_window`: 1 MB（默认）
- `connection_receive_window`: 8 MB（默认）
- `send_window`: 8 MB（默认）
- `max_concurrent_bidi_streams`: 1000（默认）
- 支持 BBR 拥塞控制（可选配置）

#### Issue #9 — 每次重连创建新 QUIC Endpoint（fd 泄漏）
**原问题**: 重连循环每次绑定新 UDP socket。
**现状**: **已修复**。`client/main.rs` 中 `build_quic_endpoint()` 在重连循环外创建，循环内只调用 `endpoint.connect()`。

#### Issue #10 — ServerConfig 每个 egress 流深拷贝
**原问题**: `Arc::new(state.config.clone())` 每个流深拷贝整个 Config。
**现状**: **已修复**。Config 在启动时用 `Arc` 包装，流处理时只 `clone()` Arc 引用。

#### Issue #28 — Token 验证非常量时间
**原问题**: `expected == token` 短路比较存在时序侧信道。
**现状**: **已修复**。`server/config.rs` 使用 `subtle::ConstantTimeEq`：`expected.as_bytes().ct_eq(token.as_bytes())`。

#### Issue #25 — bincode 反序列化无大小限制
**原问题**: 攻击者可伪造超大 `len` 导致巨额分配。
**现状**: **已修复**。`tunnel-lib/src/models/msg.rs` 中读取 payload 前检查 `len > 10 * 1024 * 1024`，超过则返回错误，不做分配。

---

### ⚠️ 部分修复 (3/30)

#### Issue #12 — `tokio::io::split` 带 Mutex 开销
**原问题**: 泛型 `tokio::io::split` 用 `Arc<Mutex>` 包装，每次 read/write 都经过锁。
**现状**: **部分修复**。
- `bridge.rs` 中 TcpStream 已改用 `into_split()`（零开销）
- `base.rs` 中 TcpStream 也已改用 `into_split()`
- **但 `engine/relay.rs` 中的 `relay_bidirectional` 泛型函数仍使用 `tokio::io::split()`**，因为它接受任意 `AsyncRead + AsyncWrite` 类型（如 TLS 流），无法使用 `into_split()`。这不是 bug，而是 API 限制——对于非 TcpStream 类型（如 TLS），必须用 `split()`。

**结论**: 对于 TcpStream 路径已优化；TLS 等包装类型的路径由于 API 约束，Arc<Mutex> 无法完全消除，不算未修复。

#### Issue #29 — 证书缓存无限增长
**原问题**: 过期条目永不驱逐，缓存命中时完整克隆证书链+私钥。
**现状**: **部分修复**。
- ✅ 已加驱逐：`pki.rs` 在每次 insert 时 `retain` 删除过期条目（TTL=1h）
- ❌ 仍有克隆：`cached.certs.clone()` 在缓存命中时仍克隆完整证书链。应改为 `Arc<Vec<CertificateDer>>` 实现零拷贝共享

#### Issue #17 — Client entry listener 无背压
**原问题**: 无 semaphore 限流，允许无限连接。
**现状**: **已修复**（与 ANALYSIS_REPORT.md 前版本的记录不同）。`client/entry.rs` 中有 `Arc<Semaphore>` 配合 `try_acquire_owned()`，超限则拒绝新连接。

---

### ❌ 未修复 (17/30)

#### Issue #5 — HTTP/1.1 不支持 Keep-Alive（同 KNOWN_ISSUES Issue #1）
**原问题**: `HttpPeer::connect` 只处理一个请求，Keep-Alive 后续请求被 reset。
**现状**: **未修复**。
- `tunnel-lib/src/proxy/http.rs` 仍然单请求模式
- `tunnel-lib/src/protocol/driver/h1.rs:write_response` 写 `Transfer-Encoding: chunked` 但**不写 `Connection: close`**，客户端被误导为连接可复用
- 详见下文 KNOWN_ISSUES 分析

#### Issue #8 — H2 每请求完整握手
**原问题**: 每个 HTTP/2 请求都执行 H2 握手（SETTINGS + WINDOW_UPDATE + ACK）。
**现状**: **未修复**。`tunnel-lib/src/proxy/h2_proxy.rs:35` 仍然 `handshake(TokioExecutor::new(), io).await?` per-request，未实现会话复用池。

#### Issue #11 — TLS handler 客户端选择绑定到连接
**原问题**: TLS 连接上所有 H2 请求路由到同一客户端，客户端断开导致整条连接失败。
**现状**: **未修复**。`server/handlers/http.rs:88-95` 在 TLS accept 阶段选择客户端，整个连接生命周期共用该客户端。明文 H2 handler 正确地做了 per-request 路由，但 TLS 路径没有。

#### Issue #13 — 三套重复的 relay 实现
**原问题**: `engine/relay.rs`、`engine/bridge.rs`、`proxy/base.rs` 三套实现，base.rs 多加 BufReader/BufWriter。
**现状**: **未修复**。三套实现依然存在。`proxy/base.rs` 的 BufReader/BufWriter 在 QUIC 流上是反效果（QUIC 传输层已有帧缓冲，多一层 8KB 用户态 buffer 只增加内存和拷贝）。

#### Issue #14 — Relay 错误被静默吞掉
**现状**: **部分改善**。`relay.rs` 已改为传播错误，但 `bridge.rs:28` 的 `unwrap_or(0)` 模式仍存在。

#### Issue #15 — 初始字节读取仅 4KB
**文件**: `tunnel-lib/src/proxy/core.rs:47`
**现状**: **未修复**。`let mut buf = vec![0u8; 4096]`，单次 `recv.read()` 不保证读到完整 HTTP 头，大 Cookie/Auth 头超过 4KB 可能导致协议检测失败。

#### Issue #16 — 热路径中 String 分配
**现状**: **部分修复**。
- ✅ `detect.rs` 返回类型已改为 `(&'static str, Option<String>)`，协议标识零分配
- ❌ `listener.rs` 的 `VhostRouter.get()` 仍做 `to_lowercase()` 堆分配
- ❌ `extract_host_from_http` 仍对每行 HTTP header 做 `to_lowercase()` 堆分配

#### Issue #18 — 模糊的回退路由
**现状**: **未修复**。`client/app.rs:107` 仍然 `name.contains(proxy_name)` 子串匹配，且 fallback 为 `group.first()`（不是 round-robin）。行为不确定性仍然存在。

#### Issue #19 — DNS 无缓存
**现状**: **未修复**。`client/app.rs` 中每个上游连接仍触发 DNS 查询，无 TTL 缓存。

#### Issue #20 — 死代码 connection_pool.rs
**现状**: **已清理**（替换为新的 `client/pool.rs`，实现了真正的多 QUIC 连接池）。

#### Issue #21 — HTTP egress: `parse_result.unwrap()` panic
**现状**: **已修复**。`egress/http.rs` 改为 `match parse_result` 带错误处理，不再 panic。

#### Issue #22 — HTTP egress: Chunked Transfer-Encoding 被忽略
**现状**: **未修复**。`egress/http.rs` 仅处理 `Content-Length` body，chunked 编码请求 body 被静默丢弃。

#### Issue #23 — HTTP egress: 未知 HTTP method 静默变为 GET
**现状**: **需验证**（受 agent 探索限制，未确认是否使用 `Method::from_bytes()`）。

#### Issue #24 — HTTP egress: host_rewrite 无效
**现状**: **需验证**。

#### Issue #26 — 地址解析: `:443` 子串匹配误判 HTTPS
**现状**: **已修复**。`tunnel-lib/src/transport/addr.rs` 的逻辑已改为按端口号精确匹配（`port == 443`），`example.com:4430` 不再被误判为 HTTPS。

#### Issue #27 — 地址解析: IPv6 地址解析错误
**现状**: **已修复**。`addr.rs` 正确处理 `[::1]:8080` 格式，提取 host 和 port 逻辑均有 IPv6 分支。

#### Issue #30 — HTTP 响应 header 含非 ASCII 值时中断
**现状**: **需验证**。

---

## 3. PERF_COMPARISON.md 性能问题修复状态

### Buffer 回收（relay 路径）

**PERF_COMPARISON 指出**: DuoTunnel 每连接 `vec![0u8; 4096]` 新分配，三套 relay 实现不统一。

**现状**:
- `server/handlers/http.rs` 和 `server/handlers/tcp.rs` 的 peek buffer 已改为栈分配 `[0u8; N]`（见 KNOWN_ISSUES Issue #2 ✅）
- `engine/relay.rs` 和 `bridge.rs` 使用 `tokio::io::copy`，其内部仍动态分配 8KB buffer per-copy
- **Buffer pool 未实现**（见 KNOWN_ISSUES Issue #2 "暂不修复"状态）

**结论**: 热路径分配减少但未消除。高并发下仍有 allocator 压力，但 Rust 的 `jemalloc`/`mimalloc` 替换方案也未引入（`Cargo.toml` 使用系统 allocator）。

### QUIC 传输层配置

**PERF_COMPARISON 指出**: 窗口太小（~48KB stream / ~1.5MB connection），并发流数 100 硬编码，无 BBR。

**现状**: **已修复**（见 Issue #7 分析）。`QuicTransportParams` 支持完整配置，默认值已大幅调优，BBR 可选。

但注意：当前 `profile.release` 中 `lto = "thin"`（非 `"fat"`），跨 crate 优化不完全。PERF_COMPARISON 第 10.6 节建议 `lto = "fat"` + `panic = "abort"`，这些仍是可选优化空间。

### 连接池

**PERF_COMPARISON 指出**: 单 QUIC 连接承载所有流量，单 UDP socket 有 CPU 瓶颈。

**现状**: **已修复**。`client/pool.rs` 实现多 QUIC 连接池，配置 `quic_connections: N` 后每个 slot 维护独立连接（suffixed client_id: `client-0`, `client-1`...），连接间完全独立，互不干扰。

### TLS 客户端选择（TLS H2 连接的 per-request 路由）

**PERF_COMPARISON 指出**: TLS 连接上所有 H2 请求路由到同一客户端。

**现状**: **未修复**。明文 H2 路径已做 per-request 路由（`handle_plaintext_h2_connection`），但 `handle_tls_connection` 在连接级选择客户端，整条连接所有请求共用该客户端。

---

## 4. KNOWN_ISSUES.md 专项核查

### Issue 1: HTTP/1.1 Keep-Alive 连接复用失败

**现状**: **完全存在，未修复**。

根因分析（代码核实）：

**Server 侧**（`proxy/base.rs`）: 将整个 TCP 连接作为字节流透传到一条 QUIC stream（`tokio::io::copy` 持续运行直到 TCP EOF）。

**Client 侧**（`proxy/http.rs` + `protocol/driver/h1.rs`）:
1. `HttpPeer::connect` 调用 `Http1Driver::read_request()` 读单个请求
2. 转发请求，接收响应
3. `driver.write_response()` 写入响应后调用 `send.finish()`（关闭 QUIC stream）
4. **关键 bug**: `write_response` 写 `Transfer-Encoding: chunked` 但无 `Connection: close`
   ```
   // h1.rs:219
   self.send.write_all(b"Transfer-Encoding: chunked\r\n").await?;
   // 没有 Connection: close
   ```
5. 客户端（浏览器/curl）误以为连接可复用，发第二个请求到已关闭的 QUIC stream
6. QUIC 层发 `STOP_SENDING`，server 侧 copy 失败，TCP 连接被 drop
7. 客户端收到 connection reset

**与 frp 的对比**:
- frp 采用**纯 L4 透传**：server 和 client 之间的 work connection 是原始字节流，Keep-Alive 由"外部客户端 ↔ frp-server ↔ frp-client ↔ 本地服务"整条 TCP 链路天然维持
- DuoTunnel 在 client 侧引入了 L7 解析（hyper Client），打破了 TCP 透明性，因此需要显式处理 Keep-Alive 边界

**可选修复方案**:

方案 A（最小改动）: `h1.rs:write_response` 中写入 `Connection: close`，强制每请求新 TCP 连接，消除客户端误导，行为变为"HTTP/1.0 语义"，无 bug，但每请求一次握手。

方案 B（支持 Keep-Alive）: `HttpPeer::connect` 改为循环处理多请求。难点：`Http1Driver::read_request` 将 `RecvStream` move 进 streaming body 闭包，消费后无法回收，需重构为 `Arc<Mutex<RecvStream>>` 或 channel 方式。

方案 C（Server 侧按请求 open_bi）: Server 端解析 HTTP/1.1 边界，每个请求单独 `open_bi()`，Client 侧无需改动。改动范围较大。

**推荐**: 方案 A 作为短期修复（5 行代码），方案 B 作为长期正确实现。

---

### Issue 2: Buffer Pool 未实现

**现状**: **核心位置已优化，但 pool 未实现**。

- ✅ `server/handlers/http.rs:52`: 已改为栈分配 `[0u8; 16384]`
- ✅ `server/handlers/tcp.rs:64`: 已改为栈分配 `[0u8; 4096]`
- ❌ `tunnel-lib/src/proxy/core.rs:47`: `let mut buf = vec![0u8; 4096]` 仍是堆分配
- ❌ `tunnel-lib/src/protocol/driver/h1.rs:52`: `let mut first_buf = vec![0u8; 8192]` 仍是堆分配
- ❌ `tunnel-lib/src/protocol/driver/h1.rs:173`: `let mut buf = vec![0u8; 8192]` 仍是堆分配
- ❌ `tokio::io::copy` 内部动态分配 8KB buffer（relay 主路径）

**与 frp 的对比**:
frp 使用分层 `sync.Pool`（5 个 tier：1K/2K/4K/16K/32K），在 `io.CopyBuffer` 路径复用 buffer，GC 压力极低。GT 使用 `sync.Pool` + 预填帧头，单次 write 覆盖帧头+payload。

**当前状态**（KNOWN_ISSUES 注释"暂不修复"）: 合理，堆分配频率相对较低，引入 `thread_local!` pool 或 crossbeam 的复杂度收益比不足，可在高并发压测确认瓶颈后再优化。

---

### Issue 3: `_protocol` 变量无用（server/tunnel_handler.rs）

**现状**: **仍然存在**。

`server/tunnel_handler.rs` 中计算 `_protocol` 仅为压制 clippy 警告，实际从未使用，`engine.run_stream` 内部重新解析协议。这是死代码，不影响正确性，但是代码整洁性问题。

**修复**: 直接删除该 match 块，`Protocol` import 一并移除（约 10 行代码，风险极低）。

---

## 5. 与 frp 源码深度对比分析

### 5.1 HTTP Keep-Alive 架构差异

frp 的"纯透传"架构（server 和 client 之间是原始 TCP 字节流）使其天然支持 Keep-Alive，无需 L7 感知。DuoTunnel 为了支持 Header 重写等高级功能（`protocol/rewrite.rs`），在 client 侧引入了 hyper HTTP 解析，代价是需要显式处理连接边界。

frp vhost HTTP 模式（`pkg/vhost/http.go`）使用 `net/http.ReverseProxy` + `Transport` 连接池，在 **server 侧**做 L7 代理并维护后端连接池——client 侧仍是纯字节流。这是另一种权衡。

### 5.2 H2 握手优化

frp 使用 yamux 在单 TCP 连接上打开轻量虚连接，没有七层握手开销。DuoTunnel 的 `h2_proxy.rs` 在每个 QUIC stream 上都做 `h2::client::handshake`，相当于 QUIC stream 本身的握手开销之外又叠加了完整 H2 协议握手（SETTINGS + WINDOW_UPDATE + ACK），是双重握手。

**frp 启示**: 在内存中维护 `DashMap<GroupId, h2::client::SendRequest<B>>` 的 H2 会话池。新请求优先从池中取已有的 H2 sender 直接并行 `send_request`，池中不存在才创建新会话。Quinn 已提供流级隔离，H2 多路复用在 QUIC 之上是冗余的——直接将每个 H2 请求映射到独立 QUIC stream 而非共享 H2 会话可能是更合适的长期方案。

### 5.3 Round-Robin 与连接管理

frp 使用 `sync.Mutex` + 有序 list 保证 round-robin 稳定性；GT 使用 `sync.RWMutex` + stable slice + atomic index。DuoTunnel 的修复方向（`RwLock<Vec>` + `AtomicUsize`）与 GT 完全一致，正确。

### 5.4 流量安全与消息协议

frp 的消息协议（`pkg/msg/msg.go`）对字符串字段有隐式长度限制（Go JSON 反序列化会拒绝超出 `maxBufSize` 的帧）。DuoTunnel 的 bincode 限制（10MB）是明确的 size guard，实现合理，但攻击者仍可在限制内发送大量小消息进行 DoS，这需要连接级的消息速率限制（当前无）。

---

## 6. 优先修复建议（更新版）

基于代码核实，重新排列优先级：

### P0 — 行为 Bug（影响正确性）

| # | 问题 | 位置 | 修复方案 | 代价 |
|---|------|------|---------|------|
| 1 | HTTP/1.1 Keep-Alive 连接 reset | `h1.rs:write_response` | 写 `Connection: close`（方案 A） | 5 行 |
| 2 | TLS 连接客户端绑定 | `server/handlers/http.rs:88` | 在 H2 service_fn 内做 per-request 路由 | 中 |
| 3 | `_protocol` 死代码 | `server/tunnel_handler.rs` | 删除 match 块 | 10 行 |

### P1 — 性能瓶颈（显著影响）

| # | 问题 | 位置 | 修复方案 | 代价 |
|---|------|------|---------|------|
| 4 | H2 每请求完整握手 | `h2_proxy.rs:35` | 实现 H2 会话池 | 高 |
| 5 | 证书缓存命中时克隆 | `pki.rs:33` | 改为 `Arc<Vec<CertificateDer>>` | 低 |
| 6 | relay 多套实现 + BufReader/Writer | `proxy/base.rs` | 统一实现，去掉多余 buffer | 中 |
| 7 | core.rs 初始读 vec 堆分配 | `proxy/core.rs:47` | 改为栈分配或固定大小 | 低 |

### P2 — 功能完整性

| # | 问题 | 位置 | 修复方案 | 代价 |
|---|------|------|---------|------|
| 8 | HTTP Keep-Alive 完整支持 | `proxy/http.rs` + `h1.rs` | 方案 B 循环处理 | 高 |
| 9 | Chunked 编码 egress 支持 | `egress/http.rs` | 使用 hyper Body | 中 |
| 10 | VhostRouter 路由无分配 | `listener.rs` | `eq_ignore_ascii_case` 替代 `to_lowercase()` | 低 |
| 11 | app.rs 回退路由不确定 | `client/app.rs:107` | 精确匹配 + 确定性回退 | 低 |

### P3 — 编译与可观测性优化

| # | 问题 | 修复 | 代价 |
|---|------|------|------|
| 12 | `lto = "thin"` → `"fat"` | `Cargo.toml` | 编译时间增加 |
| 13 | `panic = "abort"` | `Cargo.toml` | 去掉 unwind 表 |
| 14 | jemalloc/mimalloc 替换系统分配器 | 新依赖 | 低 |
| 15 | DNS 缓存 | `client/app.rs` | 引入 `hickory-resolver` | 中 |

---

## 7. 总结

### 已完成的关键修复（值得肯定）

- H2 流控（OOM 防护）✅
- Round-Robin 正确性 ✅
- TCP_NODELAY 全面覆盖 ✅
- QUIC 窗口调优 + 参数可配置 ✅
- TLS 配置全局共享 ✅
- 多 QUIC 连接池 ✅
- QUIC Endpoint 复用 ✅
- Token 常量时间验证 ✅
- bincode 消息大小限制 ✅
- IPv6 + `:443` 地址解析修复 ✅

### 仍需关注的核心问题

1. **HTTP/1.1 Keep-Alive bug**：`write_response` 缺 `Connection: close`，客户端被误导复用连接导致 reset。这是高优 P0 bug，5 行可修。

2. **TLS H2 per-connection 路由**：TLS 连接上所有 H2 请求绑定同一客户端，客户端断开导致整条连接失败，与明文 H2 的 per-request 路由不一致。

3. **H2 双重握手**：每个 H2 请求在 QUIC stream 上再做完整 H2 握手，QUIC 多路复用的优势被削弱。

4. **relay 实现碎片化**：三套 relay 实现，`base.rs` 上的 BufReader/BufWriter 是反效果，应统一为一套。

5. **`_protocol` 死代码**：10 行即可清理，不应长期留存。

### DuoTunnel vs frp/GT 差距评估

经过本轮修复，DuoTunnel 在**正确性**层面已显著提升（竞态修复、OOM 防护、QUIC 调优）。剩余性能差距主要来自：

- H2 握手开销（约 2-3 RTT per H2 request，frp/GT 无此开销）
- HTTP/1.1 Keep-Alive 不支持（每请求重建流，消耗 QUIC 流创建 RTT）
- relay 路径的额外 buffer 层（base.rs 的 BufReader/BufWriter）

解决上述三点后，DuoTunnel 的底层 QUIC 原生 stream 架构（无 HOL blocking、独立流控）将是相比 frp（TCP+yamux）和 GT（单 stream 帧协议）的真正优势。
