# Tunnel Done List

## ⚡ DuoTunnel 热路径性能优化 (2026-07-27) ✅

在 `perf/optimize-hotpaths` 分支中，我们对 Ingress、Tunnel 控制面及核心数据转发面进行了全方位的底层性能硬化与零分配重构：

- **L7 零拷贝与混合策略**：重构 `copy_buffered_then_finish` 数据转发，实现 TCP → QUIC 转发的用户空间 100% 零拷贝（使用 `write_chunk`）。引入 **Hybrid (混合) 零拷贝/拷贝** 算法，对 `<16KB` 的小包拷贝后清空复用同一 Buffer，对 `>=16KB` 大包执行零拷贝拆分，兼顾吞吐并规避了慢速连接锁死大块内存（Memory Pinning）的隐患。
- **IP 强类型化与零分配**：将控制面 `RoutingInfo` 的 `src_addr` 以及 UDP 会话键 `UdpSessionKey` 中的 `client_addr` 从 `String` 彻底重构为 `std::net::IpAddr`，消除了高并发下每包及每个控制流的 IP 格式化与反序列化解析开销。
- **协程协商去 Box 堆分配**：将 QUIC 协商流时用的 `OpenWaitObserver` 回调从 Box 包装的 `Fn` 闭包重构为无分配的**静态函数指针（`fn` 指针）**，彻底切断了每条 Ingress 连接建连时的 Box 堆分配。
- **HTTP/1.1 & HTTP/2 协议栈优化**：
  - `Http1Driver` 构建时一次性预解析 `scheme` 与 `authority`，请求时使用 `http::uri::Parts` 直接构建 `Uri`，免除高频 `format!` 及二次解析的开销。
  - `h2.rs` 伪头部匹配改在原始字节切片上匹配（`name == b":authority"`），避开了对所有常规请求头无意义的 `String::from_utf8_lossy` 扫描与堆分配。
- **Server 注册中心 $O(1)$ 级别注销**：在 `select_client_for_group` 中获取 group 句柄后立即销毁 DashMap 读锁，并在 `ClientRegistry` 内部引入 `client_to_group` 对射表，将断线注销的遍历时间复杂度从 $O(N)$ 优化至 **$O(1)$**。
- **底线性微优化**：
  - **熔断时钟缓存**：后台线程每 50ms 自动刷新 Relaxed monotonic 缓存，使热路径熔断判断避开高频系统调用（`Instant::now()`）。
  - **L1 局部性缓冲池**：实现 thread-local 的 L1 缓冲池大容量自愈置换，配合全局 `mimalloc` 分配器获得最佳局部性（LIFO）。
  - **Metrics 全局锁规避**：将 `ResourceGuard` 释放时的 notify 条件修改为 `previous <= 1`（仅当计数降为 0 时才加锁通知），高并发下极大减少了 waiters 互斥锁争用。

## 🏗️ DuoTunnel 架构改造计划 (REFACTOR_PLAN.md) ✅

> 按 Commit 拆分的架构升级全景已在 2026-04 完成，项目已从单一 Runtime 演进为三层隔离 Runtime 架构。

- **Commit 1 — 运行时基础**: 提取 `build_proxy_runtime` / `build_single_thread_runtime` 到 `tunnel-lib`。
- **Commit 2 — 指标服务重写**: 使用 `hyper` 重写 metrics server，修复了 512 字节截断 bug 并支持 Keep-alive。
- **Commit 3 — Runtime 分层隔离**: Server 拆分为 `proxy-worker` (多线程)、`metrics-worker` (单线程)、`bg-worker` (单线程) 三层，互不干扰。
- **Commit 4 — 核心服务 Trait 化**: `hot_reload` 和 `control_client` 统一实现 `BackgroundService` Trait，支持优雅关机。
- **Commit 5 — N-accept-loop**: Ingress 监听器实现并发 Accept，共享 `SO_REUSEPORT` fd，彻底消除高并发下的建连瓶颈。
- **Commit 6 — H2 路径统一**: Client 侧删除特殊 H2 分支，统一通过 `ProxyEngine` 调度，代码量减少且逻辑归一。

---

## 💎 代码质量重构 (CODE_REVIEW.md 已完成项) ✅

- [x] **CR1 — 协议检测枚举化**: `RoutingInfo.protocol` 从 `String` 转为 `Protocol` 枚举，整条链路零字符串转换。
- [x] **CR2 — Relay 逻辑归一化**: 统一 `relay_inner` / `forward_inner` 核心，全路径升级为 64KB `copy_buf`。
- [x] **CR3 — URL 解析归一化**: `UpstreamScheme` 统一使用 `transport/addr.rs` 解析，减少冗余扫描，RPS +410。
- [x] **CR-NEW-A — 消息读取封装**: 实现 `recv_typed_message<T>`，合并 type-byte 与 body 读取，代码更安全。
- [x] **CR-NEW-B — RouteTarget 类型化**: `VhostRouter` 已返回 `RouteTarget { group_id, proxy_name }`，替代匿名 `(Arc<str>, Arc<str>)` 元组。
- [x] **CR-NEW-D — API 导出梳理**: `lib.rs` 手术级裁剪，隐藏内部细节，核心 relay 函数收进 `pub mod relay`。
- [x] **CR-NEW-E — PeekBufPool 共享工具**: 提取 `infra::peek_buf::PeekBufPool`，`client/entry.rs` 与 `proxy/core.rs` 复用同一 thread-local buffer pool，集中维护 `set_len` 安全前提。
- [x] **CR-NEW-C — TcpPeer/TlsTcpPeer 合并**: TCP peer 已收敛为 `TcpPeer { tls: Option<TlsConfig> }` / `BasicPeerSpec { tls: Option<TlsPeerSpec> }` 形态。

## Pingora-inspired Proxy Refactor ✅
- [x] **TODO-71 — P2C pick algorithm**: Implemented generic bounded P2C routing in `tunnel-lib/src/inflight.rs` via `pick_p2c_inflight`. Extracted $O(1)$ fast paths for `Server::ClientGroup::select_healthy` and `client::EntryConnPool::next_conn_excluding`.

- [x] **TODO-63 — Peer 描述符化**: 执行链路已从 `UpstreamResolver -> PeerKind` 切到 `UpstreamResolver -> PeerSpec -> connect_peer`，client MITM 路径不再依赖 `PeerKind::Dyn`。
- [x] **TODO-65 — Hot-path structured errors**: `ProxyError / ErrorKind / ErrorSource / RetryType` 已覆盖 `open_bi_guarded`、server/client `UpstreamResolver`、H1/H2c/TLS/TCP ingress 热路径和共享 proxy error metrics。
  - `open_bi` 已区分 stream capacity (`QuicStreamLimit`)、transient connection loss (`QuicConnectionLost`) 和 fatal connection error (`QuicConnectionFatal`)。
  - `client/entry.rs` 根据结构化 QUIC open 错误做 next-connection retry、stale connection eviction 或 fatal fail-fast。
  - `duotunnel_proxy_errors_total{protocol,type,source,retry}` 已接入 plugin dispatcher、h2c、TLS H2 和 legacy server handler 路径。
  - 验证：`cargo check -p tunnel-lib -p server -p client`；`cargo test -p tunnel-lib -p server -p client`。
- [x] **TODO-70 — Server snapshot 持 Arc<SelectedConnection>**: server 侧连接快照已与 client 侧共享/缓存语义对齐。

---

## Auth & Config Source Plan

### [TODO-48] Token-only Client Registration/Login ✅
**Priority**: High | **Status**: Done

**Goal**:
Client side no longer submits identity fields for authentication decisions. Client only sends `token`; server identifies `name`/tenant/group from token and then pushes routing rules.

**Why it matters**:
Avoids client-side identity spoofing risk and simplifies bootstrap flow.

**Implementation notes**:
1. Keep `Login.token` as the only auth input used by server.
2. Server ignores/does not trust client-provided identity metadata for auth.
3. After token verification, server binds connection to the resolved unique `name`.

### [TODO-49] Server-issued Long Unique Tokens ✅
**Priority**: High | **Status**: Done

**Goal**:
Server provides token generation API/CLI: generate long, unique, high-entropy tokens per unique `name`.

**Why it matters**:
Eliminating weak/manual token creation and ensures uniqueness + entropy baseline.

**Implementation notes**:
1. Token generation: at least 32 random bytes (base64url/hex encoded).
2. Enforce uniqueness with DB unique index.
3. Support rotate/revoke lifecycle (`active`, `revoked_at`).
4. Store only token hash in DB; never persist plaintext token.

### [TODO-50] Auth Data Persistence via DB (Default: SQLite in Dev) ✅
**Priority**: High | **Status**: Done

**Goal**:
Move auth and client identity mapping from static config to DB-backed source. Dev default is local SQLite.

**Why it matters**:
Removes manual YAML token distribution; enables dynamic updates and auditability.

**Implementation notes**:
1. Add `AuthStore`/`ConfigStore` abstraction.
2. Default provider for development: `sqlite://./data/duotunnel.db`.
3. Suggested schema:
   - `clients(id, name UNIQUE, token_hash, status, created_at, updated_at)`
   - `client_tokens(id, client_id, token_hash, status, created_at, revoked_at)`
4. Add migration files and startup auto-migrate (dev mode).

### [TODO-51] Server Auth Path: Resolve Name by Token, Then Push Rules ✅
**Priority**: High | **Status**: Done

**Goal**:
On login, server validates token via DB and resolves owning `name`, then fetches effective routing rules and returns `LoginResp`.

**Why it matters**:
Makes auth and authorization deterministic and centrally managed.

**Implementation notes**:
1. Login flow: `token -> client(name) -> rule set -> LoginResp`.
2. Reject missing/revoked token with explicit error code.
3. Keep auth comparison timing-safe where applicable.
4. Emit metrics split by result (`auth_success`, `auth_failure_invalid`, `auth_failure_revoked`).

### [TODO-52] Rules from DB + Multi-source Provider ✅
**Priority**: High

**Goal**:
Rules can be loaded from DB, while preserving previously discussed multi-source model (file/db/hybrid).

**Why it matters**:
Supports dynamic control-plane updates without giving up local-file fallback.

**Implementation notes**:
1. Introduce `ConfigSource` trait:
   - `FileSource` (existing YAML)
   - `DbSource` (SQLite/Postgres in future)
   - `MergedSource` (override/priority rules)
2. Keep current file mode as compatibility path.
3. Add source priority semantics and conflict resolution policy.

### [TODO-53] Delivery Plan (Incremental) - Completed Milestones ✅
**Priority**: High

1. Milestone A: schema + token generation + DB lookup (auth only). ✅
2. Milestone B: server login uses DB name resolution; client remains token-only. ✅
3. Milestone C: rules read from DB (with file fallback). ✅

## Config Tuning (No Code Changes)

### [TODO-16] QUIC connections: 1 → 4 ✅

**Files**: `ci-helpers/client.yaml`
**Priority**: High

`quic.connections` is not configured in `client.yaml`, defaulting to 1. All traffic is squeezed into a single QUIC connection, creating a bottleneck for single UDP socket serial encryption/decryption and single-connection flow control.

Change to `connections: 4` to distribute the load across 4 QUIC connections.

### [TODO-17] max_concurrent_streams: 200 → 1000 ✅

**Files**: `ci-helpers/client.yaml`, `ci-helpers/server.yaml`
**Priority**: High

Both server and client are set to 200 in the CI config. In 3K QPS no-keepalive scenarios, the number of in-flight streams can easily exceed 200, causing `try_acquire_owned` to drop connections directly.

Change to 1000.

## Code Optimization

### [TODO-18] H1 body read: BytesMut::zeroed → unsafe set_len ✅
**Priority**: Medium

已完成：`BytesMut::zeroed` 路径已移除，避免每块 body 的额外 memset。

### [TODO-19] H1 double header parse → single parse ✅
**Priority**: Medium

已完成：H1 header 解析已改为单次 parse，复用同次解析结果。

### [TODO-21] tokio::io::copy 8KB → 64KB buffer ✅
**Priority**: Medium

已完成：relay 路径统一为 `BufReader + copy_buf`，不再走默认 8KB `copy`。

## Architecture Level Optimization

### [TODO-23] server entry listener SO_REUSEPORT ✅
**Priority**: Medium

已完成：`server/handlers/http.rs` 与 `server/handlers/tcp.rs` 已使用 `build_reuseport_listener()`。

## Performance Fix

### Egress 首批字节提前写入 QUIC stream ✅

**根因**：`client/entry.rs` 在 `open_bi` 后只发 `routing_info`，随即进入 `relay_quic_to_tcp` 循环。server 侧 `ProxyEngine::run_stream` 的 `recv.read()` 必须等 client 把本地 TCP 数据读出再通过 QUIC传过来，多了一个完整的写→传输→唤醒周期，导致 egress avg 比 ingress 高 ~15ms。

ingress 的 `handle_plaintext_h1_connection` 在 `open_bi` 后立刻通过 `forward_with_initial_data` 把已 peek 到的首批字节写进 QUIC stream，对端 `recv.read()` 几乎 0 等待。

**修复**（`client/entry.rs`）：`send_routing_info` 之后，把已 peek 到的 `initial_bytes` 先写入 QUIC stream，同时用 `read_exact` 消费掉本地 TCP 对应字节防止 relay 重发，与 ingress 路径完全对称。

## Tmp Tune

### 通信层调优已完成项 ✅

- [x] **统一 worker threads 生效路径**
  `run_with_tokio` 和 `run_with_dial9` 均调用 `apply_worker_threads`，两条路径行为一致。

- [x] **relay copy_buf 统一**
  `proxy/base.rs` 全部使用 `BufReader::with_capacity(relay_buf_size)` + `copy_buf`，与 `bridge.rs` 对齐。

- [x] **entry listener 绑多 QUIC 连接（L1 天花板）**
  新增 `client/conn_pool.rs`（`EntryConnPool`，ArcSwap RCU + round-robin）。
  entry listener 独立启动，不再绑定单个 supervisor slot。
  每个 slot 连接成功后 `push`、断开后 `remove`。
  修复了多 slot 抢 bind 同一端口的 bug。

- [x] **entry open_bi 失败后重试其他连接**
  `entry.rs` 改为遍历 pool_size 次，超时或 error 时跳下一条连接，避免单连接 stream 打满时请求失败。

- [x] **entry / stream peek buf 复用 thread-local pool**
  `entry.rs` 与 `proxy/core.rs` 均使用 `PeekBufPool` 复用 peek/read-ahead buffer，消除每连接 alloc+memset，并把 unsafe `set_len` 前提集中到 `infra::peek_buf`。

- [x] **H2 sender cache 去串行化**
  `h2_proxy.rs` 重构为 `H2SenderCache`：
  - body 类型已从 `UnsyncBoxBody` 迁移到 `BoxBody`
  - fast path：`ArcSwap<Option<SendRequest<BoxBody>>>` 原子 load，不再锁住热路径
  - slow path：`tokio::sync::Mutex`（`rebuild_mu`）序列化重建，只有一个任务做 `open_bi` + H2 握手
  - double-check 防止多任务同时 miss 时浪费 QUIC stream

- [x] **server 选路改 least-inflight**
  `registry.rs` `ClientGroup::select_healthy` 目前只做 RR + close_reason 检查，不感知 inflight stream 数。
  改为在 `ClientGroup` 里维护每连接 `AtomicUsize` inflight 计数，`open_bi` 前后 +1/-1，选路取 min-inflight。
  **已实现**：`SelectedConnection` 携带 per-connection `Arc<AtomicUsize>`，`begin_inflight()` 返回 RAII guard，`select_healthy` 改为 `min_by_key(inflight)`。

  **后续：负载均衡语义优化方向**
  - inflight 计数反映的是 stream 数量，不等于实际负载（大 body 请求和心跳请求 inflight 权重相同）
  - 更精确的方向：按字节流量或响应时间加权（EWMA），但需要额外埋点
  - 当前 N ≤ 5 的场景下 least-inflight 已足够，EWMA 方案在 N 较大时才有明显收益

## Code Quality

### [TODO-33] Zero-copy HTTP Header Parsing (httparse Deep Integration) ✅

**Files**: `tunnel-lib/src/transport/listener.rs`

`detect_protocol_and_host` 主路径已改用 httparse，`extract_host_from_http` 降为非热路径 fallback。单次 httparse pass 同时完成协议探测 + Host 提取 + WebSocket 检测。

### [TODO-14] HTTP drain discard buffer heap allocation cleanup ✅

**Files**: `server/handlers/http.rs`

2026-05-01 代码核对确认：`server/handlers/http.rs` 中已无原 TODO 记录的 `vec![0u8; len]` drain 分配路径，该项从 TODO 队列移除。

## Bench Fixes

### viewer "No CPU samples" 报错排查 ✅

CI run 24198270418 所有 job 通过，`cpu.json.gz` 有实际数据（837 KB–1.1 MB），问题已确认并标记为已解决。

### [TODO-58] ingress_multihost 100% errors ✅

在 CI composite action `start-infra` 中已配置全局 `/etc/hosts` 以映射 `echo-NN.local` 域名；同时在 `bench-3k` 和 `bench-6k` 中确保多域名路由规则已正确通过 `server.yaml` 进行静态加载与 `ctld` 动态同步，此项已顺利通过 CI 验证。

### [TODO-59/60/61] 排查基线延迟与 body 传输 P95 尾部延迟回归 ✅

通过多项性能优化手段成功消除了延迟回归：
1. **H2 Sender Cache 去串行化**：采用 `ArcSwap` 实现无锁 fast path 读取，只有在 connection miss 的 slow path 上才使用 Mutex 进行握手重建，彻底解决了并发 H2 sender 竞争 contention 问题。
2. **零拷贝/低开销传输**：引入 `PeekBufPool` 消除连接建立时的 alloc 与 memset，改用 `httparse` 零拷贝解析头，并在客户端 `open_bi` 后提前写入首字节数据避免额外的唤醒等待，显著降低了 baseline 延迟。

---

## May 2026 Ingress & Connection Pool Refactor ✅

### [TODO-54] Dial9 release follow-up
**Priority**: High | **Status**: Completed

After `dial9-tokio-telemetry` publishes a crates.io version that includes commit `64564b26`, remove the git `rev` patch and switch back to a released version.

**Steps**:
1. Remove `[patch.crates-io].dial9-tokio-telemetry` (Completed).
2. Keep client/server on the same released version (Completed).
3. Verify CI `stress-test` and `stress-trace-8k` still preserve dashboard metrics and phase visualization (Completed).

### [TODO-66] Unified HttpConnector + H1/H2 fallback memory
**Priority**: High | **Status**: Completed

**Current code state**:
- `HttpConnector` exists and wraps `HttpsClient` + `H2cClient`.
- H1 downstream requests go through `HttpConnector::request()`.
- Cleartext h2c empty-body requests can fall back to H1 once and mark `prefer_h1` with TTL.
- Downstream protocol and upstream protocol are fully decoupled via explicit `downstream_protocol` connection parameter.
- `HttpConnector::connect()` dynamically dispatches primarily by `downstream_protocol` instead of being bound to upstream descriptors.

### [TODO-69] h2c per-route sticky sender cache failover
**Priority**: Medium | **Status**: Completed

**Current code state**:
- h2c `CachedSender` is now `{ selected: Arc<SelectedConnection>, sender }`.
- Stale sender invalidation is tied to `selected.conn.stable_id()`.
- Empty-body requests retry once after sender failure.

### [TODO-72] Client-side connection pool and retry polish
**Priority**: Low | **Status**: Completed

**Current code state**:
- `EntryConnPool` de-duplicates by `Connection::stable_id()`.
- Entry retry uses an exclude set and distinguishes stream capacity, transient connection loss, and fatal connection errors.
- Connection-level failures evict stale pool entries.

---

## 🔒 2026-05-22 & 2026-05-29 Code Review & TODOs 已完成项 ✅

- **[TODO-87] Fast-Path First open_bi without Unconditional Timeout**: Implemented non-blocking `conn.open_bi().now_or_never()` check in `open_bi_guarded` to bypass timer wheel registration for immediate connections.
- **[TODO-90] Add Config Validation to Server Config**: Implemented comprehensive FIGMENT-level `validate(&self)` checking logic inside `ServerConfigFile` for ports, timeouts, and overload thresholds.
- **[TODO-91] Refactor ClientEngine and Improve Graceful Exit Telemetry**: Refactored `ClientEngine` services to `Arc<dyn ClientService>`, implemented a robust two-phase graceful shutdown with cascading error logging, and guaranteed cancellation token broadcast on normal exit.
- **[TODO-92] Enrich Metric Labels with ErrorKind**: Dynamically parses underlies dynamic `ProxyError` to granular `ErrorKind` labels inside `DefaultTunnelService::logging` to support granular SLA alerts and root-cause analysis dashboards.
- **[TODO-93] Fix healthz Incomplete Read and Ready Escape Vulnerability in Client**: Hardened the mini health probe server in `client/main.rs` to return `400 Bad Request` on unrecognized/fragmented HTTP requests instead of false `200 OK`.
- **[TODO-95] Refactor Overload Backoff Loop with tokio::sync::Notify**: Completely replaced active `sleep` polling loop with `tokio::sync::Notify` in `maybe_slow_path` to avoid timer wheel load and CPU cache bouncing under backpressure.
- **Eliminate Redundant Heap Allocation in `TcpPassHandler`**: Directly reads length of `raw_preface` in `TcpPassHandler::handle` without unnecessary `to_vec()` heap clone.
- **Eliminate Double Allocation in `H1Handler`**: Directly passes raw preface reference as `&[u8]` inside `H1Handler::handle` to avoid double heap allocations.
- **DNS Resolution Pathways**: Fully optimized egress target resolutions via caching to prevent WAN DNS lookup blockages on hot paths.
- **Serialized Upstream Dialing in `ProxyEngine`**: Fixed serialization bottleneck in `ProxyEngine` by optimizing peak buffer reads.
- **Active Live Gauges (Real-Time Queue Depths)**: Implemented live Prom gauges for accepted connections, pending QUIC stream queues, and slow-path waiting tasks.
- **In-Code Pre-Validation & Enriched EMFILE Logging**: Added startup soft limits validation and high-visibility backoff EMFILE/ENFILE warnings to prevent CPU thrashing.
- **同步回调设计的脆弱性与事件循环阻塞隐患 (Blocking Accept Loop)**: Constrained TCP accept callback `H` to return a `Future`, spawned via `tokio::spawn` dynamically inside `run_accept_worker`.
- **错误退避期间的取消信号“死锁化”响应延迟**: Used `tokio::select!` inside `EMFILE`/`ENFILE` backoff sleep loop to trigger immediate graceful shutdown.
- **系统级描述符耗尽缺陷 (ENFILE 遗漏)**: Correctly handled both `EMFILE` (errno 24) and `ENFILE` (errno 23) in accept workers.
- **Tie-Breaker 缺失引发的“羊群效应”与突发负载不均 (Herding Effect)**: Added thread-local rotating index offset to `pick_least_inflight` selection loop to balance load across nodes with identical inflight.
- **超时归类模型与流限制的语义混淆风险**: QUIC timeouts are properly classified as `OpenBiOutcome::TimedOut`/`quic_open_timed_out` instead of generic stream limits.
- **大报文读取的冗余“零填充”内存带宽浪费 (Zero-filling Memory Waste)**: Allocated uninitialized capacity via `AlignedVec` and used `unsafe { buf.set_len(len); }` inside `recv_watch_request` to avoid synchronic memset.
- **隐式报文类型状态调用依赖**: Explicitly reads and validates message type against `MessageType::ConfigPush` in `recv_watch_request`.
- **TLS 握手错误 (`TlsHandshake`) 映射与职责划分的严重偏失 (Misclassified Upstream Error)**: Categorized TLS negotiation issues under `ErrorSource::Upstream` and mapped them to `502 Bad Gateway` status codes.
- **中间人代理（MITM H2）中高频 PKI 证书生成的 CPU 饥饿风险**: Optimized using thread-safe cached `CertState` with inflight locks and `tokio::task::spawn_blocking`.
- **极其优秀的高性能无锁读/写锁分离设计**: Implemented RCU-based `snapshot` swap for read paths with lock-free connection selections.
- **主动容错防护 (Proactive Fault Tolerance)**: Validated `c.conn.close_reason().is_none()` during connection selection.
- **极佳的同步非阻塞指标采集实践**: Metrics observation processes are synchronically added to atomic gauges.
- **负载均衡扫描复杂度优化 ($O(N) \to O(1)$)**: P2C load balancing handles selection complexity for connection pools with more than 32 clients.

## 💎 2026-06-15 归档已完成项 ✅

- [x] **[TODO-94] JitterBackoff 指数退避下限控制**: 已在 `client/tunnel/supervisor.rs` 中通过 `random_delay_range(min_delay, cap)` (其中 `min_delay = cap / 2`) 实现了指数退避的下限控制，消除了重试后期的瞬时高频重试风暴。
- [x] **[TODO-CR-AUDIT-11] 消除“Peek + Read_exact 丢弃”双重 I/O**: 已通过重构 `SniffRuntime::sniff` 消除该开销。当前直接使用原生 `stream.read()` 读取并缓存前缀，无多余系统调用。并在首段完成 `initial_bytes` 的 QUIC 发送，省去了不必要的 `PrefixedReadWrite` 包装与复杂零拷贝机制。
- [x] **[TODO-CR-AUDIT-12] Tracing Span Instrument for Blocked Futures in open_bi**: 已在 `tunnel-lib/src/open_bi.rs` 的 `open_bi_guarded` 中，对 `conn.open_bi()` 的等待期注入了 `waiting_for_stream_credit` 的 tracing debug span，使外部工具如 `tokio-console` 能清晰观测挂起协程。
- [x] **[TODO-CR-AUDIT-13] Asymmetric Window Coupling in QUIC Configuration**: 已在 `tunnel-lib/src/config/quic.rs` 和 `client/config.rs` 中移除了 `send_window_bytes` 向 `connection_window_mb` 的强制回退对齐，完全解耦了两端的滑动窗口大小。
- [x] **[TODO-CR-AUDIT-14] TokenListEntry String Heap Allocation & Type Safety**: 已在 `tunnel-store/src/sqlite.rs` 进行了重构，通过 zero-copy 的 `&str` 代替 `String` 堆分配读取 SQLite 状态。

