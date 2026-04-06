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
Eliminates weak/manual token creation and ensures uniqueness + entropy baseline.

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

### [TODO-53] Delivery Plan (Incremental)
**Priority**: High

1. Milestone A: schema + token generation + DB lookup (auth only). ✅
2. Milestone B: server login uses DB name resolution; client remains token-only. ✅
3. Milestone C: rules read from DB (with file fallback). ✅
4. Milestone D: remove legacy static token map from server config (or keep read-only compatibility window).

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

---

## Code Optimization

### [TODO-14] Change discard buffer to stack allocation

**Files**: `server/handlers/http.rs`
**Priority**: Low

When draining the socket, `vec![0u8; len]` is still used (heap allocation). Since len ≤ 8192, this can be changed to a stack buffer.

### [TODO-18] H1 body read: BytesMut::zeroed → unsafe set_len

**Files**: `tunnel-lib/src/protocol/driver/h1.rs:276`
**Priority**: Medium

Saves one `memset` (8KB) for every body chunk read. Large body request path CPU reduced by ~30%.

### [TODO-19] H1 double header parse → single parse

**Files**: `tunnel-lib/src/protocol/driver/h1.rs:104-140`
**Priority**: Medium

Current flow: Loop parse until `Complete` (discard result), then parse again to extract fields.
Change: Record `header_end` when `Complete` is reached within the loop, and use the results from that same parse.

### [TODO-20] Bytes::copy_from_slice → split_to().freeze() Zero-copy

**Files**: `tunnel-lib/src/protocol/driver/h1.rs:201,283,287`
**Priority**: Medium

Found 5 instances where `Bytes::copy_from_slice` copies from `BytesMut` to a new `Bytes`. These can be replaced with `split_to().freeze()` to achieve zero-copy (sharing underlying memory).

### [TODO-21] tokio::io::copy 8KB → 64KB buffer

**Files**: `tunnel-lib/src/engine/bridge.rs`
**Priority**: Medium

`tokio::io::copy()` defaults to an 8KB internal buffer. Given the QUIC stream window is 4MB and the TCP socket buffer is 4MB, 8KB leads to a syscall overhead for every 8KB.

Wrap the reader with `BufReader::with_capacity(65536)` before passing it to `tokio::io::copy_buf()`.

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

### [TODO-23] server entry listener SO_REUSEPORT

**Files**: `server/handlers/http.rs:13`, `server/handlers/tcp.rs:17`
**Priority**: Medium

The server entry uses `TcpListener::bind()` without `SO_REUSEPORT`. `tunnel-lib/src/transport/listener.rs` already has `build_reuse_listener()`, but the server handler is not using it.

### [TODO-24] Multiple QUIC Endpoints (Multiple UDP sockets)

**Priority**: Low

A quinn `Endpoint` binds to a single UDP socket, serializing `recvmsg/sendmsg`. Even if multiple QUIC connections are opened, they all go through the same socket. Consider having the client create multiple Endpoints (bound to different ports), coupled with `SO_REUSEPORT` to distribute UDP processing.

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

### [TODO-33] Zero-copy HTTP Header Parsing (httparse Deep Integration)
**Files**: `tunnel-lib/src/transport/listener.rs`
**Priority**: Medium
Refactor `extract_host_from_http`. Stop using string line scanning and directly reuse `httparse` offset indices to achieve completely allocation-free Host extraction.

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

### [TODO-41] Thread-per-Core & Anti-Work-Stealing (pingora-runtime)
**Priority**: Low (High Complexity)
Transition from Tokio's default work-stealing scheduler to a Strict `Share-Nothing` Thread-per-Core architecture. Similar to Pingora's `NoStealRuntime`, manually spawn multiple single-threaded Tokio executors (`Builder::new_current_thread`) mapped to specific NUMA Nodes. Work-stealing for heavy network IO causes severe CPU L2/L3 Cache misses when tasks cross cores. By locking processing to specific cores and removing global atomic structs like `DashMap`, we can achieve linearly scalable performance and **Deterministic Latency** without switching out of the Tokio ecosystem entirely.

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

## Non-Applicable / Reference Architectures 

### [Reference] Case-insensitive Trie for Header Matching (pingora-http)
Pingora uses a highly compact static dictionary/Trie (e.g., matching `HeaderName::Host` without String allocation) to sniff HTTP headers. *Why it's not applicable*: DuoTunnel's HTTP parsing primarily extracts the `Host` or `Upgrade` header for initial protocol detection and then switches to hyper/h2 or raw TCP relay. We don't act as a full Layer 7 reverse proxy that modifies complex headers natively, so a full Trie is overkill.

### [Reference] TinyLFU Memory Caching (pingora-lru / tinyufo)
Pingora implements an advanced LFU + Bloom Filter (TinyUFO) with sharded locks for memory caching to prevent cache pollution from single-hit cold traffic. *Why it's not applicable*: DuoTunnel is a transparent Tunnel/Proxy without static content caching or heavy LRU caching semantics. Our only "cache" is the connection pool.

### [Reference] Memory-Contiguous Ketama Hash Ring (pingora-ketama)
For load balancing, Pingora flattens the Hash Ring into a contiguous 1D array to achieve 99%+ CPU L1 Cache hits during binary search for upstream routing. *Why it's not applicable*: DuoTunnel routes traffic to specific user machines based on exact subdomain mapping (`ClientRegistry`), it does not perform typical weight-based or predictable-hash load balancing across a replica set.

---

## Bench Fixes

### [TODO-15] egress_http_post Overflow Phase Boundary

**Files**: `ci-helpers/k6/bench.js`, `bench/index.html`
**Priority**: Low

`egress_http_post` startTime=6s + stages 5s+20s = ends at 31s, but Phase "Basic" end=29. The chart annotation box does not completely cover this scenario, but it does not affect the data accuracy.
