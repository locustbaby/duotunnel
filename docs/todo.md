# Tunnel TODO

> Last synced against code: 2026-05-01.
>
> This file is the source of truth for unfinished work. Completed or stale items were moved to `docs/donelist.md`. Detailed design notes remain in the topical docs, especially `docs/pingora-tasks.md` and `docs/parameters.md`.

---

## 0. Build and Release Follow-Ups

### [TODO-54] Dial9 release follow-up
**Priority**: High | **Status**: TODO

After `dial9-tokio-telemetry` publishes a crates.io version that includes commit `64564b26`, remove the git `rev` patch and switch back to a released version.

**Steps**:
1. Remove `[patch.crates-io].dial9-tokio-telemetry`.
2. Keep client/server on the same released version.
3. Verify CI `stress-test` and `stress-trace-8k` still preserve dashboard metrics and phase visualization.

---

## 1. Active Mainline: Pingora-Inspired HTTP / Proxy Refactor

Recommended order: `66 -> 69 -> 72 -> 64 -> 67b -> 68 -> 71`.

Completed: `TODO-65` moved to `docs/donelist.md`.

### [TODO-66] Unified HttpConnector + H1/H2 fallback memory
**Priority**: High | **Status**: In Progress

**Current code state**:
- `HttpConnector` exists and wraps `HttpsClient` + `H2cClient`.
- H1 downstream requests go through `HttpConnector::request()`.
- Cleartext h2c empty-body requests can fall back to H1 once and mark `prefer_h1` with TTL.

**Remaining**:
1. `HttpConnector::connect()` still dispatches primarily by `spec.protocol`; finish decoupling downstream protocol from upstream protocol.
2. Keep-alive/session ownership still lives under `HttpPeer` / `H2Peer`; finish the Session split in TODO-67b.
3. Expand protocol preference beyond the current cleartext request path where it is valuable and testable.

### [TODO-69] h2c per-route sticky sender cache failover
**Priority**: Medium | **Status**: In Progress

**Current code state**:
- h2c `CachedSender` is now `{ selected: Arc<SelectedConnection>, sender }`.
- Stale sender invalidation is tied to `selected.conn.stable_id()`.
- Empty-body requests retry once after sender failure.

**Remaining**:
1. Replace ad-hoc `Mutex<HashMap<...>>` combinations with an explicit h2c connection state object, or document why the current structure is sufficient.
2. Finish replayability rules for non-empty request bodies.
3. Ensure failover behavior is covered by tests or CI stress cases.

### [TODO-72] Client-side connection pool and retry polish
**Priority**: Low | **Status**: In Progress

**Current code state**:
- `EntryConnPool` de-duplicates by `Connection::stable_id()`.
- Entry retry uses an exclude set and distinguishes stream capacity, transient connection loss, and fatal connection errors.
- Connection-level failures evict stale pool entries.

**Remaining**:
1. Decide whether future multi-server HA needs grouped pools rather than the current flat pool.

### [TODO-64] ClientId / GroupId / ProxyName / ReuseHash newtypes
**Priority**: Medium | **Status**: TODO

**Problem**:
Hot paths still use bare `String` / `Arc<str>` for IDs. There is no `tunnel-lib/src/ids.rs` and no `ReuseHash` type.

**Fix**:
1. Introduce `ClientId`, `GroupId`, `ProxyName`, and `ReuseHash` as cheap cloneable newtypes.
2. First apply them to in-memory hot paths: `server/registry.rs`, `client/conn_pool.rs`, `server/plugins/h2c/mod.rs`.
3. Keep wire/config/storage schemas as strings initially; convert at boundaries.

### [TODO-67b] Move keep-alive loop into Session layer
**Priority**: Medium | **Status**: TODO

**Problem**:
H1 keep-alive is still in `HttpPeer::connect_inner`, which mixes upstream description, connection reuse, and session loop behavior.

**Fix**:
Create `H1Session` / `H2Session` style ownership around request loops and reused connection metadata. This also gives TODO-65 a natural place to apply `RetryType::ReusedOnly`.

### [TODO-68] Ingress request lifecycle convergence
**Priority**: Medium | **Status**: TODO

**Problem**:
h2c still carries per-connection request lifecycle state inside the handler (`first_authority`, `route_cache`, `sender_cache`). Error classification, retry, and failover are not yet aligned with TLS/H1 paths.

**Fix**:
Keep per-request routing for h2c, but centralize request lifecycle state and response mapping. Do not introduce a single selected-client fast path that would break multi-authority h2c connections.

### [TODO-71] P2C pick algorithm
**Priority**: Low | **Status**: TODO

**Trigger condition**:
Only do this when a group has enough clients for linear least-inflight scans to show up in profiles. Until then, `pick_least_inflight` is simpler and adequate.

**Scope**:
Server `ClientGroup::select_healthy` and client `EntryConnPool::next_conn_excluding`.

### [TODO-62] Full per-peer protocol capability memory
**Priority**: Medium | **Status**: Partially covered by TODO-66

**Remaining beyond TODO-66 Phase 1**:
1. Track upstream protocol capability per peer/reuse key, not only cleartext h2c fallback.
2. Decouple downstream H1/H2 from upstream H1/H2 across both server egress and client egress.
3. Decide how to observe TLS ALPN outcome if hyper does not expose enough connection-level detail.

---

## 2. Control Plane, Auth, and Config

### [TODO-53D] Remove legacy static token map
**Priority**: High | **Status**: TODO

Milestones A-C are done. Remaining work is Milestone D: remove the legacy static token map from server config, or keep it behind an explicit read-only compatibility window.

### [TODO-51] LocalTokenCache incremental updates
**Priority**: Low | **Status**: TODO

**Current code state**:
`LocalTokenCache::update()` rebuilds and atomically swaps the whole `HashMap<[u8; 32], CacheEntry>` on every snapshot/patch.

**Fix**:
1. Add a compatible `WatchEvent::TokenDelta { added, removed }`.
2. Add `LocalTokenCache::patch(added, removed)`.
3. Emit token deltas only for token-only changes; keep full routing snapshots for routing changes.

### [TODO-CR5] Config stream model
**Priority**: Low | **Status**: TODO

Move from pull-style `ConfigSource::load()` snapshots toward a stream model such as `Stream<Item = RoutingSnapshot>`, so file/db/ctld/dynamic config sources share one update contract.

### [TODO-PARAM-1] Unified parameter configuration schema
**Priority**: Medium | **Status**: TODO

Tracked in detail in `docs/parameters.md`.

**Steps**:
1. Add top-level schema version.
2. Clean dead config fields such as unused `max_connections` / `max_tcp_connections` comments.
3. Normalize timeout naming and units.
4. Split timeout semantics out of `reconnect.*`.
5. Add per-upstream overrides for TCP, HTTP pool, timeout, H2 stream limits, and H2 ping behavior.
6. Consider server-to-client delivery of recommended `overload.*` / `quic.*` values.

---

## 3. Code Quality and Maintainability

### [TODO-CR4] Decouple observability from business hot paths
**Priority**: Low | **Status**: TODO

**Current code state**:
The plugin `MetricsSink` exists, but many server handlers still call `metrics::xxx()` directly.

**Fix**:
Emit tracing events or a thin async telemetry event, then aggregate metrics out of band. If using a tracing subscriber, `on_event` must only enqueue through a non-blocking channel; do not update Prometheus counters under subscriber locks.

### [TODO-20] Bytes::copy_from_slice -> split_to().freeze()
**Priority**: Medium | **Status**: TODO

Remaining copy paths still exist in request/body handling, especially around H1 driver scratch buffers and initial bytes. Replace with ownership-preserving `BytesMut::split_to().freeze()` where lifetimes and buffer reuse make it safe.

### [TODO-22 / TODO-34] Remove relay split overhead where possible
**Priority**: Medium | **Status**: TODO / Partial

**Current code state**:
Some production paths use `into_split()`, but generic relay helpers still use `tokio::io::split()` where type constraints require it.

**Fix**:
1. Keep `tokio::io::split()` for stream types that do not support owned halves.
2. Replace remaining concrete TCP/QUIC relay paths with `into_split()` where safe.
3. Update or retire obsolete generic helpers that are only used in tests.

### [TODO-52] Route snapshot connection-level cache for H2
**Priority**: Low | **Status**: TODO

Cache `Arc<RoutingSnapshot>` at H2 connection scope when hot reload semantics allow old long-lived connections to keep using their initial routing snapshot.

### [TODO-36] Finish static dispatch cleanup
**Priority**: Low | **Status**: Partial

`PeerKind::Dyn` / `UpstreamPeer` are gone from the live path, but `PeerKind::{Http,H2}` still box peer structs as a transitional execution enum. After TODO-66/67b, either remove `PeerKind` entirely or make it a pure enum with no heap boxing.

### [TODO-ENTRY-POOL] Remove redundant EntryConnPool mutable Vec
**Priority**: Low | **Status**: TODO

`EntryConnPool` still stores a writer-side `PoolState { conns, ids }` plus an `ArcSwap` snapshot. Since N is small this is not urgent, but it can be simplified to `ArcSwap` plus a write mutex if it stays useful.

---

## 4. QUIC, Transport, and Major Features

### [TODO-26] Native UDP proxy over QUIC Datagram
**Priority**: High | **Status**: TODO

Enable QUIC Datagram and implement UDP session tracking, timeout, and resend behavior. This is a feature gap, not a micro-optimization.

### [TODO-27] QUIC certificate and 0-RTT persistence
**Priority**: Medium | **Status**: TODO

Persist server identity material and session ticket encryption keys so restarts do not break trust or disable 0-RTT. Include rotation strategy.

### [TODO-32] Root CA signing mode for generated certs
**Priority**: High | **Status**: TODO

Current generated/self-signed certificate behavior is expensive and not persistence-friendly. Move to persistent root CA + per-host signing/caching.

### [TODO-24] Multi-endpoint + thread-per-core architecture
**Priority**: Medium | **Status**: Research

Potential fix for `open_bi()` cross-thread wakeups and endpoint contention at very high QPS. This is an architectural change and should only start after profiling confirms the current runtime layout is the bottleneck.

### [TODO-57] quinn stream-level lock research
**Priority**: Low | **Status**: Research

Research whether `quinn-proto` stream state can be sharded by stream ID without breaking connection-level flow control and congestion control.

### [TODO-25] io_uring instead of epoll
**Priority**: Low | **Status**: Deferred

Deferred until native Tokio/io_uring support is mature enough to avoid disrupting the current `Send` task model.

### [TODO-55] quinn ConnectionDriver debug_span per-poll overhead
**Priority**: Low | **Status**: Deferred pending evidence

Only patch or upstream this if a current flamegraph confirms the span construction is still a real hotspot.

---

## 5. Performance Ideas and Future Architecture

These are not on the immediate implementation path. Pull one forward only with a benchmark/profile that justifies it.

### [TODO-28] Kernel-level zero-copy relay
**Priority**: Medium | **Status**: TODO

Evaluate splice/sendfile-style relay on Linux for TCP-heavy paths.

### [TODO-29] Dynamic buffer/window tuning
**Priority**: Medium | **Status**: TODO

Tune relay buffers and QUIC/TCP windows for high-BDP links.

### [TODO-30] Upstream pre-warming
**Priority**: Low | **Status**: TODO

Open upstream connections while protocol detection is in progress when the route is predictable enough to avoid wasted dials.

### [TODO-31] VhostRouter wildcard trie/radix tree
**Priority**: Medium | **Status**: TODO

Move wildcard matching from linear scan to a trie/radix structure if wildcard count becomes large enough to show up in profiles.

### [TODO-35] Two-tier upstream connection pool
**Priority**: High | **Status**: TODO

Use a small lock-free hot queue plus global pool for local egress TCP connections if upstream connection churn becomes a bottleneck.

### [TODO-37] Seamless graceful handover / hot upgrades
**Priority**: Medium | **Status**: TODO

Explore listener fd transfer and graceful process replacement for long-lived deployments.

### [TODO-38] Vectorized IO relaying / writev
**Priority**: High | **Status**: TODO

Combine header/body slices with vectored writes where the parser can preserve source-buffer references safely.

### [TODO-44] Generalized lazy timers
**Priority**: Medium | **Status**: Partial

H1 keep-alive already has lazy timeout behavior. Generalize only where profiles show timer registration overhead remains meaningful.

### [TODO-39] TCP Fast Open for egress connections
**Priority**: Medium | **Status**: TODO

Evaluate TFO on local/remote upstream dials.

### [TODO-40] Buffer slab allocator / arena
**Priority**: Medium | **Status**: TODO

Consider fixed-size thread-local buffer pools beyond current peek buffer reuse if allocation remains hot.

### [TODO-42] Kernel bypass for QUIC
**Priority**: Low | **Status**: Research

AF_XDP/eBPF experiments only; high complexity and not on the current path.

### [TODO-43] HugePages support
**Priority**: Low | **Status**: TODO

Evaluate only after memory profiles show TLB/cache pressure from buffers.

### [TODO-45] Zero-copy HTTP header serialization
**Priority**: Medium | **Status**: TODO

Represent header fields as offsets into the original buffer and pair with vectored writes. Depends on a careful ownership model.

### [TODO-46] Dynamic TCP congestion control and socket tuning
**Priority**: Low | **Status**: TODO

Expose per-connection/per-upstream socket tuning only if real deployments need it.

### [TODO-47] Memory-efficient load balancing ring
**Priority**: Low | **Status**: TODO

Only relevant if DuoTunnel grows multi-replica upstream selection where consistent hashing is useful.

---

## 6. Bench, CI, and Observability Follow-Ups

### [TODO-58] ingress_multihost 100% errors
**Priority**: High | **Status**: TODO

CI case shows all errors for ingress multihost while egress multihost works. Verify `/etc/hosts`, generated server ingress vhost rules, and route resolution for `echo-NN.local`.

### [TODO-59] ingress_http_get / bidir_mixed p95 tail regression
**Priority**: Medium | **Status**: TODO

Likely caused by shared H2 sender/window contention between overlapping benchmark phases. Confirm phase overlap and isolate sender/cache scope if needed.

### [TODO-60] ingress_post_100k p95 regression
**Priority**: Medium | **Status**: TODO

Investigate H2 window/frame tuning, BBR on loopback-like RTT, and overlap with other body-size phases.

### [TODO-61] Global baseline latency increase after H2Sender path
**Priority**: Medium | **Status**: TODO

H2-over-QUIC reuse adds a fixed framing cost. Optimize only where the reuse tradeoff is not worth it or can be reduced.

### [TODO-CI-1] CI connection matrix
**Priority**: Low | **Status**: TODO

Run key benchmark cases with `connections=1/2/4` to validate whether multiple QUIC connections actually improve throughput and tail latency.

### [TODO-15] egress_http_post phase boundary annotation
**Priority**: Low | **Status**: TODO

Fix benchmark chart annotation: `egress_http_post` extends beyond the "Basic" phase box. This affects chart readability, not raw data accuracy.
