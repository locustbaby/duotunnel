# Tunnel TODO

## Build & Dependency Plan

### [TODO-54] Dial9 release follow-up
**Priority**: High | **Status**: TODO

**Goal**:
After `dial9-tokio-telemetry` publishes a crate version that includes commit `64564b26`, remove the git `rev` patch and switch back to crates.io version pin.

**Implementation notes**:
1. Remove `[patch.crates-io].dial9-tokio-telemetry`.
2. Keep `client/server` using the same released crates.io version.
3. Verify CI `stress-test` + `stress-trace-8k` merged publish path keeps dashboard metrics and phase visualization correct.

## Auth & Config Source Plan

### [TODO-53] Delivery Plan (Incremental)
**Priority**: High

1. Milestone D: remove legacy static token map from server config (or keep read-only compatibility window).

## Code Quality Refactor

> 来源：CODE_REVIEW.md。按依赖顺序排列，必须先做 TODO-CR1，才能推进 TODO-CR2。TODO-CR3/4/5 完全独立可并行。

### [TODO-CR1] 强类型路由：RoutingInfo.protocol → enum
**Priority**: High | **Status**: TODO
**依赖**: 无（本项是后续 TODO-CR2 的前置）

**Problem**: `RoutingInfo.protocol` 是 `String`，整条链路有 4 处各自做字符串→枚举转换：
- `protocol/detect.rs` 返回 `"h2"` / `"websocket"` / `"h1"` / `"tcp"` 字面量
- `proxy/core.rs:84` `ri.protocol.as_str()` match 转回 `Protocol` 枚举
- `tunnel_handler.rs:16` `routing_info.protocol.as_str()` match 转 `Protocol`
- `client/proxy.rs:20` `routing_info.protocol == "h2"` 字符串比较

**Fix**:
1. `protocol/detect.rs::detect_protocol_and_host` 返回值从 `(&'static str, Option<String>)` 改为 `(Protocol, Option<String>)`
2. `RoutingInfo.protocol` 从 `String` 改为 `Protocol` 枚举（加 `#[repr(u8)]` + serde 支持保证 bincode 兼容）
3. 删除上述 4 处字符串 match，改为枚举 match
4. `VhostRouter` 返回值从匿名元组 `(Arc<str>, Arc<str>)` 改为 `RouteTarget { group_id, proxy_name }`

**Files**: `tunnel-lib/src/models/msg.rs`, `tunnel-lib/src/protocol/detect.rs`, `tunnel-lib/src/proxy/core.rs`, `server/tunnel_handler.rs`, `client/proxy.rs`, `server/handlers/http.rs`
**Risk**: 中（bincode 序列化格式变更，需同步升级 server+client）

---

### [TODO-CR2] Relay 逻辑归一化
**Priority**: Medium | **Status**: TODO
**依赖**: TODO-CR1（protocol 枚举化后，relay 分支判断更清晰）

**Problem**: 同一件事（join 两个方向的 copy_buf + shutdown/finish）散落在 3 个模块 6 个函数：
`bridge.rs`（relay / relay_unidirectional / relay_quic_to_tcp / relay_with_first_data）、`relay.rs`（relay_bidirectional / relay_with_initial）、`proxy/base.rs`（forward_to_client / forward_with_initial_data）。
任何新增（timeout、字节计数回调）都要改 6 处。

**Fix**: 统一为 `relay_bidir(r1, w1, r2, w2, initial: Option<&[u8]>)`，`into_split` vs `split` 在调用点处理。分两步 PR：先统一签名，再替换调用方。

**Files**: `tunnel-lib/src/engine/bridge.rs`, `tunnel-lib/src/engine/relay.rs`, `tunnel-lib/src/proxy/base.rs`, `tunnel-lib/src/proxy/tcp.rs`, `server/handlers/http.rs`, `server/handlers/tcp.rs`, `client/entry.rs`
**Risk**: 中（调用点多，建议分阶段替换）

---

### [TODO-CR3] URL 解析归一化
**Priority**: Low | **Status**: TODO
**依赖**: 无（独立）

**Problem**: `UpstreamScheme::from_address`（`proxy/tcp.rs`）和 `parse_upstream`（`transport/addr.rs`）各自实现了"port 443 → https"启发式规则，两份逻辑独立维护。`client/app.rs` 和 `server/egress.rs` 用前者，`egress/http.rs` 用后者。

**Fix**: 统一到 `transport/addr.rs::parse_upstream`，`UpstreamScheme::from_address` 改为调用它，消除重复。

**Files**: `tunnel-lib/src/proxy/tcp.rs`, `tunnel-lib/src/transport/addr.rs`, `tunnel-lib/src/egress/http.rs`
**Risk**: 低

---

### [TODO-CR4] 解耦观测性逻辑
**Priority**: Low | **Status**: TODO
**依赖**: 无（独立）

**Problem**: 业务热路径里散布大量显式 `metrics::xxx()` 调用，与 tracing 体系并存，可读性差。`open_bi_wait_ms` 直方图桶硬编码，不同网络拓扑下分位数失真。

**Fix**: 业务层只抛 `tracing` 事件（`info!(event = "open_bi_complete", wait_ms = ...)`），由独立 subscriber 汇总成 Prometheus metrics，解耦观测与业务。

**Files**: `server/metrics.rs`, `server/handlers/http.rs`, `server/handlers/quic.rs`
**Risk**: 低

---

### [TODO-CR5] 配置流式化（ConfigStream）
**Priority**: Low | **Status**: TODO
**依赖**: 无（独立）

**Problem**: 当前是 Pull 模型，`ConfigSource::load()` 全量拉取快照，热更新依赖文件系统轮询或 ctld watch stream 触发。业务层需关注"配置从哪来"和"何时触发重载"。

**Fix**: 定义 `ConfigStream: Stream<Item = RoutingSnapshot>`，业务层只监听流的变化，配置源可自然扩展到 etcd / Nacos 等动态中心。

**Files**: `server/config.rs`, `server/control_client.rs`, `server/hot_reload.rs`
**Risk**: 中（涉及 ServerState 初始化流程）

---

## Code Optimization

## open_bi 优化专区

> `open_bi()` 是当前最主要的延迟来源，三条优化路线各自独立，按收益/代价排序如下。

### [TODO-56] H2 跨 ingress 连接 sender 复用 ★ 最高优先级

**Files**: `tunnel-lib/src/proxy/h2_proxy.rs`, `tunnel-lib/src/models/msg.rs`, `server/handlers/http.rs`, `server/registry.rs`
**Priority**: High
**预期收益**: `open_bi()` 调用次数从 O(ingress 并发数) 降到 O(proxy_name × client 数)，高并发短连接场景效果最显著。
**代价**: 引入 H2 framing 开销约 +0.15~0.3ms（已在 TODO-61 量化），可接受。

**Goal**: 把 H2Sender 从 per-ingress-TCP-connection 提升到 per-`(Connection, proxy_name)` 全局缓存，N 条 ingress TCP 连接共用同一条 QUIC H2 stream。

**当前状态**: sender 在同一 ingress TCP 连接内复用（已实现），跨连接未复用。

**必须解决的障碍（按依赖顺序）**:

1. **A. 请求语义分层设计（前置，最根本）**

   当前 `RoutingInfo` 是**连接级**元数据，在 `open_bi` 时一次性发送，绑定了来源 IP、协议、host。H2 复用后一条 QUIC stream 承载多个请求，这套设计不再成立。需要把语义分两层：

   - **Stream 级**（建连时发送，不变）：`proxy_name`、`protocol` — 决定 client 路由到哪个 upstream
   - **Request 级**（每个 H2 请求的 header 携带）：`src_addr`、`src_port`、`host` — 每请求独立溯源

   具体做法：`src_addr`/`src_port` 改写入 `X-Forwarded-For`/`X-Real-Port` header，client 侧从 header 读取，`RoutingInfo` 只保留 stream 级字段。**此项必须先完成，其余障碍依赖它。**

2. **B. 复用 key 设计** — 同 group 不同 vhost 有不同 `proxy_name`，不能共用同一 stream。
   → 复用 key 为 `(Connection stable_id, proxy_name)`。

3. **C. Connection 断开清理** — Connection 死亡时需清理 SenderMap 残留条目。
   → 在 `ClientRegistry.unregister()` 时同步清理。

4. **D. cache miss 初始化竞争** — 多个 ingress 连接并发 miss 时争锁建连。
   → `DashMap` 分片 + `tokio::sync::OnceCell` 保证单次 handshake。

**实现方案**:
```rust
type SenderKey = (quinn::Connection, Arc<str>);  // (conn, proxy_name)
type GlobalSenderMap = DashMap<SenderKey, H2Sender>;
// 存放在 ServerState 或 ClientRegistry
```

**场景兼容性**:
- 同 group 不同 vhost → 不同 SenderKey ✅
- 同 group 多 client 实例 → Connection 不同，SenderKey 自然隔离 ✅
- h2_single_authority=true → 兼容现有逻辑 ✅
- h2_single_authority=false → route_cache 的 OnceLock 需改为每请求 lookup ⚠️

---

### [TODO-24] Multi-Endpoint + Thread-per-Core 架构 ★ 中期架构优化

**Priority**: Medium
**Status**: 待调研设计
**预期收益**: 消除 `open_bi()` 的跨线程 wakeup 开销，Endpoint Mutex 竞争从全局降为 per-core，10 万+ QPS 必要架构。
**代价**: 改动量大，需重构 ServerState、handler、与 dial9-telemetry 集成。**先做 TODO-56，确认瓶颈仍在调度层再启动此项。**

**Root cause**: 当前单 Endpoint → 所有 Connection 共享 Endpoint Mutex → ConnectionDriver 被 work-stealing 跨线程迁移 → `open_bi()` 每次 2 次跨线程 wakeup。

**Design**:
```
N 核 → N 个 quinn Endpoint（SO_REUSEPORT，内核 hash 分发）
     → 每个 Endpoint 绑一个 tokio current_thread LocalSet（pin 到固定线程）
     → ConnectionDriver 与 handler 同线程，open_bi() wakeup 无跨线程
```

**Implementation challenges**:
1. `Endpoint` 需在创建它的 Runtime 上 poll，`spawn_local` 要求 `!Send` future
2. 现有 handler 都是 `Send` task，需适配
3. `ServerState` 需重构为可跨 thread-local Runtime 共享
4. 与 `dial9-telemetry` 的 `TracedRuntime` 集成需要适配

---

### [TODO-57] quinn stream-level lock 调研 ★ 长期/上游方向

**Priority**: Low
**Status**: 待调研
**预期收益**: 消除 per-Connection Mutex 对并发 stream 读写的串行化，在单 Connection 高并发场景（stream 数 > 100）才有显著收益。
**代价**: 需要 fork/patch quinn，维护成本高。**先确认 TODO-24 无法满足需求时再评估。**

**Background**: quinn 所有 stream 操作（write/read/finish/reset）都要 acquire `Mutex<State>`（per-Connection 粒度），多 stream 并发时互相串行。

**Research questions**:
1. `quinn-proto` 的 `StreamsState` 能否按 stream ID shard 成多个 Mutex？
2. connection-level flow control 和 congestion control 共享状态如何处理？
3. 是否有相关 quinn issue/PR 在推进？
4. 实际 8k-100k QPS 下 stream Mutex 竞争是否真实瓶颈（需 flamegraph 确认）？



### [TODO-14] Change discard buffer to stack allocation

**Files**: `server/handlers/http.rs`
**Priority**: Low

When draining the socket, `vec![0u8; len]` is still used (heap allocation). Since len ≤ 8192, this can be changed to a stack buffer.

### [TODO-20] Bytes::copy_from_slice → split_to().freeze() Zero-copy

**Files**: `tunnel-lib/src/protocol/driver/h1.rs:236`
**Priority**: Medium

Remaining copy path still uses `Bytes::copy_from_slice` from scratch buffer. Replace with a zero-copy path when ownership/lifetime safety allows.

### [TODO-22] relay() split → into_split

**Files**: `tunnel-lib/src/engine/bridge.rs:9-10`
**Priority**: Low

The generic `relay()` use `tokio::io::split()` (internal Arc+Mutex), whereas `relay_quic_to_tcp()` correctly uses `into_split()` (zero-cost owned halves).

### [TODO-51] LocalTokenCache 增量更新（替代全量重建 HashMap）

**Files**: `server/local_auth.rs`, `tunnel-service/src/proto.rs`, `server/control_client.rs`
**Priority**: Low（影响低）

**Background**:
当前每次 ctld 推送 Patch（哪怕只改 1 个 token），server 都重建整个 `HashMap<[u8;32], CacheEntry>`。10 万 token 时重建约几十毫秒 CPU。

**Impact**: 仅在 token 数量 10 万以上且频繁批量变更时可感知，普通场景可忽略。

**Implementation notes**:
1. 在 `WatchEvent` 协议中增加 `TokenDelta { added: Vec<TokenCacheEntry>, removed: Vec<String> }` 变体（需协议版本兼容）。
2. `LocalTokenCache` 增加 `patch(added, removed)` 方法，对现有 map 做增量 insert/remove（写时复制或 RCU 风格）。
3. 仅 token 变更时发 `TokenDelta`，路由变更时仍发全量 Patch。

### [TODO-52] ArcSwap 路由快照连接级缓存（H2 热路径优化）

**Files**: `server/handlers/http.rs` (`handle_plaintext_h2_connection`, `handle_tls_connection`)
**Priority**: Low（影响低）

**Background**:
`state.routing.load()` 在每个 HTTP 请求（包括 H2 多路复用的每个流）都执行一次 ArcSwap epoch-based 读。路由配置变更极低频，同一 H2 连接生命周期内路由快照不变。在 64 核 + 百万 rps 场景下，多核并发 load() 会争抢 cache line。

**Impact**: 路由不更新时 `load()` 约 3ns，几乎无开销。仅在 64 核以上高并发场景下可测量，普通场景可忽略。

**Implementation notes**:
1. 在 `handle_plaintext_h2_connection` 连接入口处 load 一次，将 `Arc<RoutingSnapshot>` 移入 `service_fn` 闭包捕获（不在每个请求 load）。
2. TLS H2 路径同理：`handle_tls_connection` 已在连接级别 `lookup_route` 一次，无需改动。
3. 注意热重载时连接级缓存会使用旧路由直到连接断开（可接受，符合长连接语义）。

---

## Architecture Level Optimization

### [TODO-25] io_uring instead of epoll

**Priority**: Low (Experimental)
**Status**: **Deferred** (Refactor only when native `tokio` support is mature to avoid breaking the current `Send` task model).


**Implementation Challenges**:
1. **Programming Model Shift**: Transition from Readiness-based (epoll) to Completion-based (io_uring). Requires passing buffer ownership (`Vec<u8>`) to the kernel rather than references.
2. **Runtime Constraints**: `tokio-uring` tasks are generally `!Send`. Requires a `Thread-per-core` architecture where each thread runs its own local executor, conflicting with the current global work-stealing scheduler.
3. **Library Compatibility**: `quinn` currently lacks native `io_uring` support. A shim layer might be needed, potentially negating performance gains for QUIC traffic.
4. **Buffer Management**: Necessity of a custom Buffer Pool to avoid frequent large allocations since `uring` requires owned buffers for every IO operation.

**Implementation Plan**:
1. Create a separate binary or crate (e.g., `client-uring`) to avoid polluting the main codebase with `!Send` constraints.
2. Focus strictly on the **TCP Relay path** (local TCP <-> Tunnel) where `io_uring` provides the most gain, while keeping the QUIC control plane on standard Tokio.
3. Replace `tokio::io::copy` with an `io_uring` optimized relay loop using owned buffers.


### [TODO-26] Native UDP Proxy Support (Based on QUIC Datagram)

**Priority**: High

Currently only TCP/HTTP is supported. Need to enable QUIC Datagram extension, implement UDP Session management (Session Tracking/Timeout) on the server, and implement UDP re-send logic on the client. Offers lower latency than simulating UDP over Streams, with no head-of-line blocking.

### [TODO-27] QUIC Certificate & 0-RTT State Persistence (Memory Persistence)

**Priority**: Medium

**Background**: The QUIC tunnel must terminate at the DuoTunnel process (LB can only do UDP passthrough), thus certificates and keys must be managed at the application layer. Current process restarts cause self-signed Key changes and STEK (Session Ticket Encryption Key) loss, causing 0-RTT failure.

**Implementation key points**:
1. **Identity Persistence**: Save the self-signed certificate or CA to disk (e.g., `pki/server.crt`) to avoid client distrust due to certificate fingerprint changes.
2. **STEK Persistence (Critical)**: Save the key used to encrypt Session Tickets to disk. Ensure the Server can still decrypt Tickets carried by Clients after a restart to achieve 0-RTT instant connection.
3. **Key Rotation**: Implement a scheduled STEK rotation mechanism (e.g., every 24 hours) to maintain 0-RTT while ensuring Forward Secrecy.
4. **LB Architecture Confirmation**: Maintain LB as UDP Layer 4 Passthrough mode to ensure QUIC features (connection migration, etc.) are effective end-to-end.

## QUIC Feature Audit & Comparison

| Feature | Currently Used | Recommendation / Necessity |
| :--- | :--- | :--- |
| **Multi-streaming** | ✅ Deeply integrated | Core to the project (TCP/HTTP), prevents HOL blocking. |
| **BBR Congestion Control** | ✅ Enabled | Enabled in `quic.rs:47`. Critical for cross-border/weak network environments. |
| **Connection Migration** | ⚠️ Partially active | Enabled by default in `quinn`. Vital for maintaining tunnels during Wi-Fi/5G switching on mobile. |
| **0-RTT (Fast)** | ❌ Disabled | Medium priority. Requires STEK persistence to be effective across restarts. |
| **Datagram (Datagrams)** | ❌ Disabled | **High priority (for UDP proxy)**. Avoids stream retransmission delay, key to high-performance UDP. |

---

## Advanced Performance & Architecture

### [TODO-28] Kernel-level Zero-copy (Splice/Sendfile)
**Priority**: Medium (Linux Only)
Currently `bridge.rs` uses user-mode relaying. Should try using `tokio-splice` to move data directly within kernel buffers, targeting a 50% reduction in CPU soft-interrupt overhead.

### [TODO-29] Dynamic Buffer Tuning (Dynamic Windows)
**Priority**: Medium
For high-latency links, upgrade `bridge.rs`'s 8KB buffer to a delay-aware Dynamic Buffer (64KB ~ 4MB) to maximize Long Fat Network (LFN) bandwidth utilization.

### [TODO-30] Upstream Pre-warming
**Priority**: Low
Maintain a "warm" connection pool on the client side. Concurrently establish upstream connections while protocol detection is in progress to eliminate TCP handshake-induced Time-to-First-Byte (TTFB) delay.

### [TODO-31] Routing Algorithm Upgrade: Linear Scan -> Trie
**Files**: `tunnel-lib/src/transport/listener.rs`
**Priority**: Medium
Currently `VhostRouter` wildcard matching is O(N). Should switch to a Radix Tree or Trie to make complex query time complexity constant.

### [TODO-32] Certificate Generation: Root CA Signing Mode
**Files**: `tunnel-lib/src/infra/pki.rs`
**Priority**: High
Currently, self-signed certificates generate a new key pair every time (CPU intensive). Should optimize to maintain a persistent Root CA and only perform signing for site-specific certificates, improving efficiency by 10-100x.

### ~~[TODO-33] Zero-copy HTTP Header Parsing (httparse Deep Integration)~~ ✓
**Files**: `tunnel-lib/src/transport/listener.rs`
**Priority**: Medium
~~Refactor `extract_host_from_http`. Stop using string line scanning and directly reuse `httparse` offset indices to achieve completely allocation-free Host extraction.~~
**Done**: `detect_protocol_and_host` 主路径已改用 httparse，`extract_host_from_http` 降为非热路径 fallback。

### [TODO-34] Eliminate Synchronization Overhead in Relaying (Eliminate Mutex)
**Files**: `tunnel-lib/src/engine/relay.rs`
**Priority**: Medium
Switch the generic relaying function to use type-specific `into_split()`, avoiding `Arc<Mutex>` lock contention within `tokio::io::split` and improving vertical scalability under high concurrency.

---

## Inspiration from Pingora (Industrial Grade Architecture)

### [TODO-35] Thread-Shared Global Connection Pooling (Two-Tier Architecture)
**Priority**: High
Based on Pingora's `pingora-pool` core design. Instead of a single highly-contended `RwLock<HashMap>` or completely unshared thread-local pools, implement a generic Two-Tier Pool for local egress TCP connections. Use a small, fixed-capacity, cache-friendly lock-free queue (like `crossbeam_queue::ArrayQueue`) as an L1 hot cache for immediate connection retrieval without locking, falling back to a global `Mutex<HashMap>` (L2 cache) only during peak spikes. Additionally, implement active idle monitoring to proactively drop connections closed by peers.

### [TODO-36] Static Dispatch Refactor: Eliminate Boxed Traits
**Files**: `tunnel-lib/src/proxy/peers.rs`
**Priority**: Medium
Extreme CPU optimization following Pingora practices. Refactor `PeerKind`'s `Dyn(Box<dyn UpstreamPeer>)` to `Enum`-based static dispatch where possible to reduce vtable lookup overhead and improve CPU branch prediction.

### [TODO-37] Seamless Graceful Handover (Hot Upgrades)
**Priority**: Medium
Implement a signal-based (e.g., SIGQUIT) graceful exit mechanism and file-descriptor passing via SCM_RIGHTS. Upon entering the shutdown sequence, transfer the listening socket fd to the newly spawned DuoTunnel process while allowing existing long-lived connections to finish processing on the old process, ensuring true zero-downtime binary upgrades.

### [TODO-38] Vectorized IO Relaying (Vectorized IO / writev)
**Priority**: High
On the HTTP relaying path, combined with [TODO-33] zero-copy parsing, use `writev` (via `tokio::io::AsyncWrite::write_vectored`) to send Header offset slices and Body content together. Avoids the overhead of continuously copying multiple memory blocks into a linear buffer before transmitting them to the NIC.

### [TODO-44] Zero-Cost Lazy Timers (O(1) Timeout Resolution)
**Priority**: High
Following `pingora-timeout`, replacing `tokio::time::timeout` for network stream idle detection. Implement a custom wrapper that only registers the timeout with Tokio's reactor wheel if the underlying `Future` (like an IO read) returns `Poll::Pending` on its first poll. Since 99% of hot-path reads complete instantly, this entirely eliminates the overhead of creating and destroying thousands of timer timers per second. Additionally, round timer deadlines to 10ms boundaries to share expiration slots.

### [TODO-39] TCP Fast Open (TFO) for Egress Connections
**Priority**: Medium
When the server/client forwards traffic to local/remote upstreams, setting `TCP_FASTOPEN` on the socket allows the 0-RTT transmission of the initial payload during the TCP SYN packet. This will significantly reduce the TTFB for establishing new egress connections.

### [TODO-40] Buffer Slab Allocator / Arena
**Priority**: Medium
While `BytesMut` with `Jemalloc` handles memory decently, allocating read/write buffers per connection still hits the heap. Implement a connection-independent, thread-local Slab Allocator or an Arena pool to recycle fixed-size byte arrays instantly, completely bypassing OS heap mechanisms.

### [TODO-42] Kernel Bypass (AF_XDP / eBPF) for QUIC
**Priority**: Low (Experimental)
Since QUIC relies entirely on UDP packets, the Linux kernel networking stack (sk_buff allocations, iptables, netfilter) introduces major latency. Using `AF_XDP` sockets allows reading the UDP datagrams directly from the NIC driver rings into user space, circumventing the kernel entirely.

### [TODO-43] Memory HugePages Support
**Priority**: Low
Enable Transparent HugePages (THP) or explicit 2MB HugePages, and configure `Jemalloc` to use them. For a networking tunnel doing heavy buffer copying, this drastically reduces TLB (Translation Lookaside Buffer) misses in the CPU.

### [TODO-45] Zero-Copy HTTP Header Serialization (pingora-header-serde & KVRef)
**Priority**: Medium
Instead of deeply parsing HTTP headers into String maps, use a **KVRef (Key-Value Reference)** approach. Record only the **Offset and Length** of headers within the original receive buffer. For proxying/relaying, untouched headers should be forwarded exactly as they appeared in the raw `[u8]` buffer, combining these slices at the end using `writev` to bypass allocation and concatenation costs entirely, significantly reducing heap fragmentation and GC pressure.

### [TODO-46] Dynamic TCP Congestion Control & Buffer Tuning
**Priority**: Low
Expose advanced sys_socket options for egress connections. Similar to Pingora, dynamically set `TCP_NODELAY`, `TCP_KEEPINTVL`, or even tweak `Initcwnd` (Initial Congestion Window) and SNDBUF/RCVBUF on a per-connection basis to better handle Long Fat Networks (LFNs) without relying solely on OS defaults.

### [TODO-47] Memory-efficient Load Balancing Ring (Ring V2)
**Priority**: Low
If DuoTunnel evolves to support multi-replica upstream selection for high-availability subdomains, implement a contiguous 1D array-based Hash Ring (Ketama Ring V2). Use binary search over memory-contiguous points to ensure 99%+ CPU L1 cache hits during selection, rather than traversing complex tree or map structures.

---

## Upstream Research

---

## Upstream Bug / Patch

### [TODO-55] quinn ConnectionDriver debug_span! per-poll overhead

**Files**: `quinn/quinn/src/connection.rs:252`
**Priority**: ~~High~~ → **Deprioritized（不确定是真实问题）**
**Status**: 暂缓，需要在实际 flamegraph 中确认是否真实存在后再决定是否 patch。

`ConnectionDriver::poll` creates a `debug_span!("drive", id=...)` on **every single poll call** with no `#[cfg]` guard. Since quinn does not set `release_max_level_warn`, the span is constructed and subscriber-queried at runtime on every poll even when tracing is disabled. This shows up as ~36% CPU in flamegraphs under 8k QPS load.

**Root cause**: Code leak — `debug_span!` at line 252 is unconditional, no feature flag or cfg gate.

**Fix**: Patch quinn locally (pinned to tag `quinn-0.11.9`) via `[patch.crates-io]`. Wrap the span creation with `if tracing::enabled!(tracing::Level::DEBUG)` or gate with `#[cfg(feature = "tracing")]`. Also consider upstreaming a PR to quinn-rs.

---

## Bench Fixes

### [TODO-58] ingress_multihost 全部 100% err

**Files**: `.github/workflows/ci.yml`, `ci-helpers/server.yaml`
**Priority**: High
**Status**: TODO

**Symptom**: `ingress_multihost` / `ingress_multihost_8000qps` 在 CI 中全部 100% err，rps 只有 433/740（远低于目标 3K/8K）。`egress_multihost` 正常（走 8082 + Host header），说明 client 侧 H2SenderMap 逻辑没问题，问题在 server 侧 ingress 路由。

**Baseline vs current** (`1bb79bf` → `96d0918`): 新增 case，之前不存在。

**排查方向**:
1. CI runner 的 `/etc/hosts` 是否真正写入了 50 条 `echo-NN.local` 条目
2. `server.yaml` `tunnel_management.server_ingress_routing.listeners[port=8080].vhost` 是否包含全部 50 个 `echo-NN.local` 条目
3. k6 发 `http://echo-01.local:8080/` 时 server 是否能 resolve 域名并路由到 `ci-group`

---

### [TODO-59] ingress_http_get / bidir_mixed p95 长尾恶化

**Files**: `tunnel-lib/src/proxy/h2_proxy.rs`, `server/handlers/http.rs`
**Priority**: Medium
**Status**: TODO

**Baseline vs current** (`1bb79bf` → `96d0918`):
- `ingress_http_get` p95: 1.05ms → 15.98ms (**+14.93ms**)
- `bidir_mixed` p95: 0.58ms → 25.06ms (**+24.48ms**)
- p50 正常（0.52→0.72ms），说明是偶发慢请求，不是整体变慢

**Root cause**: H2Sender 复用后，所有请求共享同一条 QUIC H2 stream。`basic` phase 和 `body_size` phase 时间窗口有重叠（startTime 差 35s，basic 持续 25s），大 body 请求（100K）占满 H2 connection window，后续小请求排队等 WINDOW_UPDATE → 长尾。

**排查/修法方向**:
1. 确认 basic/body_size phase 时间窗口是否真正重叠
2. 考虑给 `bidir_mixed` 独立 H2Sender（不与 ingress_http_get 共用）
3. 或者在调度上拉开 basic 和 body_size 的时间间隔，避免并发

---

### [TODO-60] ingress_post_100k p95 大幅上涨

**Files**: `tunnel-lib/src/proxy/h2_proxy.rs`
**Priority**: Medium
**Status**: TODO

**Baseline vs current** (`1bb79bf` → `96d0918`):
- `ingress_post_100k` p95: **3.96ms → 34.30ms (+30.34ms)**
- p50: 2.71ms → 2.83ms（基本正常）

H2 window 已扩大（4MB stream / 16MB conn / 1MB frame），但 p95 未改善。

**排查方向**:
1. 100K body 走 `k6 → server:8080(H1 recv) → forward_h2_request(H2 over QUIC) → client entry(H1) → echo:9999`，server 侧是 H1 streaming 转 H2，body 是 chunk by chunk 转发
2. 可能是 BBR 在 loopback（RTT≈0）上行为异常，导致发送速率被人为限制
3. 可能是 `max_frame_size=1MB` 过大，hyper H2 实现在大帧时有额外延迟
4. 可能是 `ingress_post_100k` 与 `ingress_3000qps`（startTime=65s）在时间上不重叠，但与 `ingress_post_1k/10k` 并发，共享 H2 connection window 被 10k body 部分占用

---

### [TODO-61] 全局基线延迟轻微上涨（relay → H2Sender 协议栈开销）

**Priority**: Medium
**Status**: TODO

**Baseline vs current** (`1bb79bf` → `96d0918`，普遍轻微上涨):
- `ingress_http_get` p50: 0.52 → 0.72ms (+0.20ms)
- `egress_http_get` p50: 0.48 → 0.65ms (+0.17ms)
- `ingress_post_1k` p50: 0.59 → 0.90ms (+0.31ms)
- `ingress_post_10k` p50: 1.03 → 1.22ms (+0.19ms)
- `egress_post_10k` p50: 0.93 → 1.08ms (+0.15ms)

**Root cause**: `fb776a8` 把 ingress/egress 从 per-request `open_bi()` raw relay 改为复用 H2Sender（H2 over QUIC），引入了额外的 H2 framing/deframing 协议栈开销（约 +0.15~0.3ms per hop）。这是复用换来的固定代价，不可完全消除，但可以优化。

**优化方向**:
1. **减少 H2 framing 层数**：当前 ingress 路径是 H1→H2→QUIC→H2→H1，中间有两次 H2 framing。考虑 server 侧直接透传 H2（client 发 H2 到 server，server 不解包直接转发到 QUIC H2 stream）
2. **H2 frame size 调优**：当前 `max_frame_size=1MB` 可能对小请求反而有负面影响（padding/alignment），小请求考虑用默认 16KB
3. **减少锁竞争**：`H2Sender` 的 `Arc<Mutex<Option<SendRequest>>>` 每次请求都 lock，高并发下有竞争，考虑用 `tokio::sync::RwLock` 或无锁结构

---

## Tmp Tune Pending (Migrated from `docs/tmp-tune/todo.md`)

### P1

- [ ] **把 `UnsyncBoxBody` 全链路换成 `BoxBody`（Send 版本）**
  涉及文件：`tunnel-lib/src/proxy/h2_proxy.rs`、`tunnel-lib/src/egress/http.rs`、`tunnel-lib/src/proxy/h2.rs`、`tunnel-lib/src/proxy/http.rs`、`tunnel-lib/src/protocol/driver/mod.rs`、`tunnel-lib/src/protocol/driver/h1.rs`、`server/egress.rs`、`server/handlers/http.rs`。
  操作：`.boxed_unsync()` → `.boxed()`，`UnsyncBoxBody<B,E>` → `BoxBody<B,E>`。
  **验证**：链路上所有 body 类型（`Incoming`、`MapErr`、`MapFrame`、`Full`）均已确认是 `Sync`，改动安全。
  **收益**：`SendRequest<BoxBody>` 变成 `Send + Sync`，`h2_proxy.rs` fast path 可换 `ArcSwap` 彻底去锁。

- [ ] **H2 sender fast path 换 ArcSwap（依赖上一项）**
  上一项完成后，`H2SenderCache.sender` 从 `std::sync::Mutex<Option<SendRequest>>` 换成 `ArcSwap<Option<SendRequest>>`，fast path 降为单个原子 load，并发 H2 请求不再有任何锁竞争。

### P2

- [x] **server 选路改 least-inflight**
  `registry.rs` `ClientGroup::select_healthy` 目前只做 RR + close_reason 检查，不感知 inflight stream 数。
  改为在 `ClientGroup` 里维护每连接 `AtomicUsize` inflight 计数，`open_bi` 前后 +1/-1，选路取 min-inflight。
  **已实现**：`SelectedConnection` 携带 per-connection `Arc<AtomicUsize>`，`begin_inflight()` 返回 RAII guard，`select_healthy` 改为 `min_by_key(inflight)`。

  **后续：负载均衡语义优化方向**
  当前选路逻辑（`host → group_id → ClientGroup::select_healthy`）性能已足够，真正的优化空间在**语义**层面：
  - inflight 计数反映的是 stream 数量，不等于实际负载（大 body 请求和心跳请求 inflight 权重相同）
  - 更精确的方向：按字节流量或响应时间加权（EWMA），但需要额外埋点
  - 当前 N ≤ 5 的场景下 least-inflight 已足够，EWMA 方案在 N 较大时才有明显收益

- [ ] **CI 加连接矩阵**
  核心基准覆盖 `connections=1/2/4`，验证多连接是否真正突破单连接吞吐天花板。

### P3

- [x] **viewer "No CPU samples" 报错排查**
  CI run 24198270418 所有 job 通过，`cpu.json.gz` 有实际数据（837 KB–1.1 MB），但 viewer 页面操作时仍报 "No CPU samples"。
  待查：meta.json 的 `cpuSampleCount` 字段值；viewer lazy loader（~line 855）是否因 CORS 或解压失败静默跳过；触发条件 `trace.cpuSamples.length === 0 && !trace._cpuUrl`（~line 2314）是否因 `_cpuUrl` 未设置而提前报错。
  参考 run：https://github.com/locustbaby/duotunnel/actions/runs/24198270418

- [ ] **`EntryConnPool` 去掉冗余的 `mu` Vec**
  当前 `mu: Mutex<Vec<Connection>>` 和 `snapshot: ArcSwap<Vec<Connection>>` 存两份数据，写时 O(n) clone。
  可改为只用 `ArcSwap` + `Mutex<()>` 序列化写操作，消除冗余存储。
  连接数 N ≤ 4，当前方案功能正确，属于可选清理。

---

### [TODO-15] egress_http_post Overflow Phase Boundary

**Files**: `ci-helpers/k6/bench.js`, `bench/index.html`
**Priority**: Low

`egress_http_post` startTime=6s + stages 5s+20s = ends at 31s, but Phase "Basic" end=29. The chart annotation box does not completely cover this scenario, but it does not affect the data accuracy.
