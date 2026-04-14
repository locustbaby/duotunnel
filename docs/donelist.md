# Tunnel Done List

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
- [x] **CR-NEW-D — API 导出梳理**: `lib.rs` 手术级裁剪，隐藏内部细节，核心 relay 函数收进 `pub mod relay`。

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

- [x] **entry peek buf 复用 thread-local**
  `entry.rs` 用模块级 `thread_local!` + `set_len` 替代每连接 `BytesMut::zeroed`，消除 alloc+memset。

- [x] **H2 sender cache 去串行化**
  `h2_proxy.rs` 重构为 `H2SenderCache`：
  - fast path：`std::sync::Mutex` 只包住一次 `clone()`，临界区 ns 级，不跨 await
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

## Bench Fixes

### viewer "No CPU samples" 报错排查 ✅

CI run 24198270418 所有 job 通过，`cpu.json.gz` 有实际数据（837 KB–1.1 MB），问题已确认并标记为已解决。
