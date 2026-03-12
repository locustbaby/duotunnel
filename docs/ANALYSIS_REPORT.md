# DuoTunnel Project Analysis Report

**Generation Date**: 2026-02-22
**Branch**: fix/perf-comparison
**Reference Documents**: CODE_REVIEW.md · PERF_COMPARISON.md · KNOWN_ISSUES.md
**Comparison Targets**: frp (Go+TCP+yamux) · GT (Go+QUIC)

---

## 1. Project Architecture Overview

DuoTunnel is a QUIC-based bidirectional intranet penetration proxy. The Rust workspace consists of three crates:

- **tunnel-lib** — Shared library: QUIC transport, protocol detection, proxy engine, message encoding/decoding.
- **server** — Accepts external HTTP/TCP/TLS traffic and routes it to registered clients through QUIC tunnels.
- **client** — Maintains QUIC connections and forwards tunnel traffic to local services.

Core Data Flow:

```
Ingress: External Client → Server (entry_port) → VHost/Port Match → QUIC stream → Client → Local Service
Egress: Local Client → Client (http_entry_port) → QUIC stream → Server → External Backend
```

Key architectural differences compared to frp (single TCP connection + yamux multiplexing) and GT (single QUIC stream + custom frame protocol):

| Dimension | DuoTunnel | GT | frp |
|------|-----------|----|----|
| Transport Layer | QUIC (quinn 0.11) | QUIC / TCP+TLS | TCP+TLS / QUIC / KCP / WebSocket |
| Multiplexing | Native QUIC streams (one stream per proxy connection) | Single stream + taskID frame protocol | yamux (TCP) / QUIC stream |
| Flow Control | Native QUIC (independent per stream) | No stream-level flow control (custom protocol) | yamux flow control |
| HOL Blocking | ✅ No (QUIC stream isolation) | ❌ Yes (all tasks share single stream) | ❌ Yes (TCP path) |

DuoTunnel's architectural direction is correct—using native QUIC streams is superior to GT's single-stream frame protocol in terms of flow control and avoiding HOL blocking. Existing issues are mainly concentrated in **insufficient implementation-level optimizations**.

---

## 2. CODE_REVIEW.md Issue Fix Status Verification

### ✅ Fixed (10/30)

#### Issue #1 — Lack of H2 Flow Control (OOM Risk)
**Original Problem**: `reserve_capacity()` called without waiting, slow clients could cause infinite buffering leading to OOM.
**Current Status**: **Fixed**. Added `loop { poll_capacity() }` in `tunnel-lib/src/proxy/h2.rs` to block and wait until window capacity ≥ frame size before sending data.

#### Issue #2 — Incorrect Round-Robin Load Balancing
**Original Problem**: `DashMap::iter().nth(idx)` does not guarantee order, O(n) traversal, and not thread-safe for concurrency.
**Current Status**: **Fixed**. In `server/registry.rs`, `ClientGroup` was changed to `RwLock<Vec<(String, Connection)>>` + `AtomicUsize` counter, providing O(1) polling with stable ordering.

#### Issue #3 — Race Condition in Duplicate Client Registration
**Original Problem**: The three-step process of `get_client_connection` → `unregister` → `register` was not atomic.
**Current Status**: **Fixed**. Changed to `replace_or_register()` using DashMap's `entry()` API for atomic Check-and-Replace.

#### Issue #4 — Global Lack of TCP_NODELAY
**Original Problem**: Zero calls to `set_nodelay(true)` in the entire project, Nagle's algorithm causing 40ms+ latency.
**Current Status**: **Fixed**.
- `tunnel-lib/src/proxy/tcp.rs:157` (TcpPeer)
- `tunnel-lib/src/proxy/tcp.rs:182` (TlsTcpPeer)
- `server/handlers/http.rs:19` (Inbound HTTP)
Confirmed `set_nodelay(true)` calls in all three locations.

#### Issue #6 — TLS Config Rebuilt per Connection
**Original Problem**: Each `TlsTcpPeer::connect()` clones ~150 WebPKI root certificates, causing hundreds of heap allocations.
**Current Status**: **Fixed**. `TlsTcpPeer` now holds an `Arc<TlsConnector>`, so root certificates are loaded only once at construction and shared among all connections.

#### Issue #7 — QUIC Window Sizes Not Tuned
**Original Problem**: Default stream window ~48KB, connection window ~1.5MB, causing bottlenecks in large file transfers.
**Current Status**: **Fixed**. Added `QuicTransportParams` in `tunnel-lib/src/transport/quic.rs`:
- `stream_receive_window`: 1 MB (default)
- `connection_receive_window`: 8 MB (default)
- `send_window`: 8 MB (default)
- `max_concurrent_bidi_streams`: 1000 (default)
- Supports BBR congestion control (optional config)

#### Issue #9 — New QUIC Endpoint for Each Reconnection (fd Leak)
**Original Problem**: Reconnection loop bound a new UDP socket every time.
**Current Status**: **Fixed**. `build_quic_endpoint()` in `client/main.rs` is now created outside the reconnection loop, and only `endpoint.connect()` is called inside the loop.

#### Issue #10 — ServerConfig Deep Copy for Each Egress Stream
**Original Problem**: `Arc::new(state.config.clone())` performed a deep copy of the entire Config for every stream.
**Current Status**: **Fixed**. The Config is wrapped in an `Arc` at startup, and only the Arc reference is `clone()`d during stream processing.

#### Issue #28 — Non-Constant Time Token Verification
**Original Problem**: `expected == token` short-circuit comparison had timing side-channels.
**Current Status**: **Fixed**. `server/config.rs` now uses `subtle::ConstantTimeEq`: `expected.as_bytes().ct_eq(token.as_bytes())`.

#### Issue #25 — No Size Limit on bincode Deserialization
**Original Problem**: Attackers could forge an excessively large `len`, leading to massive memory allocation.
**Current Status**: **Fixed**. In `tunnel-lib/src/models/msg.rs`, added a check for `len > 10 * 1024 * 1024` before reading the payload; returns an error if exceeded without performing allocation.

---

## 3. PERF_COMPARISON.md Performance Issue Fix Status

### Buffer Recycling (relay path)

**PERF_COMPARISON point**: DuoTunnel performs a new `vec![0u8; 4096]` allocation per connection, and the three relay implementations are inconsistent.

**Current Status**:
- Peak buffers in `server/handlers/http.rs` and `server/handlers/tcp.rs` have been changed to stack allocation `[0u8; N]` (see KNOWN_ISSUES Issue #2 ✅).
- `engine/relay.rs` and `bridge.rs` use `tokio::io::copy`, which still dynamically allocates an 8KB buffer internally per-copy.
- **Buffer pool not implemented** (see KNOWN_ISSUES Issue #2 "Not fixing for now" status).

**Conclusion**: Allocations on the hot path have been reduced but not eliminated. High concurrency still exerts pressure on the allocator, and alternatives like Rust's `jemalloc` or `mimalloc` have not been introduced yet (`Cargo.toml` uses the system allocator).

### QUIC Transport Layer Configuration

**PERF_COMPARISON point**: Windows were too small (~48KB stream / ~1.5MB connection), concurrent streams hardcoded to 100, and no BBR.

**Current Status**: **Fixed** (see Issue #7 analysis). `QuicTransportParams` supports full configuration, defaults have been significantly tuned, and BBR is available.

Note: Current `profile.release` uses `lto = "thin"` (not `"fat"`), so cross-crate optimization is incomplete. PERF_COMPARISON section 10.6 suggests `lto = "fat"` + `panic = "abort"`, which remain optional optimization opportunities.

### Connection Pooling

**PERF_COMPARISON point**: A single QUIC connection carries all traffic, causing a CPU bottleneck on a single UDP socket.

**Current Status**: **Fixed**. `client/pool.rs` implements a multi-QUIC connection pool; after configuring `quic_connections: N`, each slot maintains an independent connection (with suffixed client_ids: `client-0`, `client-1`...), making the connections completely independent.

### TLS Client Selection (per-request routing for TLS H2 connections)

**PERF_COMPARISON point**: All H2 requests on a TLS connection are routed to the same client.

**Current Status**: **Unfixed**. The plaintext H2 path already performs per-request routing (`handle_plaintext_h2_connection`), but `handle_tls_connection` selects a client at the connection level, sharing one client for all requests on that connection.

---

## 4. KNOWN_ISSUES.md Special Verification

### Issue 1: HTTP/1.1 Keep-Alive Connection Reuse Failure

**Current Status**: **Fully present, unfixed**.

Root cause analysis (code verification):

**Server side** (`proxy/base.rs`): Passes the entire TCP connection as a byte stream through to a single QUIC stream (`tokio::io::copy` runs continuously until TCP EOF).

**Client side** (`proxy/http.rs` + `protocol/driver/h1.rs`):
1. `HttpPeer::connect` calls `Http1Driver::read_request()` to read a single request.
2. Forwards the request and receives the response.
3. After `driver.write_response()` writes the response, it calls `send.finish()` (closing the QUIC stream).
4. **Critical bug**: `write_response` writes `Transfer-Encoding: chunked` but **missing `Connection: close`**.
   ```
   // h1.rs:219
   self.send.write_all(b"Transfer-Encoding: chunked\r\n").await?;
   // No Connection: close
   ```
5. The client (browser/curl) assumes the connection is reusable and sends a second request to the already closed QUIC stream.
6. The QUIC layer sends `STOP_SENDING`, the server-side copy fails, and the TCP connection is dropped.
7. The client receives a connection reset.

**Comparison with frp**:
- frp uses **pure L4 passthrough**: the work connection between server and client is a raw byte stream. Keep-Alive is naturally maintained by the entire TCP chain "External Client ↔ frp-server ↔ frp-client ↔ Local Service".
- DuoTunnel introduces L7 parsing (hyper Client) on the client side, breaking TCP transparency. Therefore, it must explicitly handle Keep-Alive boundaries.

**Optional repair plans**:

Plan A (Minimal change): Write `Connection: close` in `h1.rs:write_response`, forcing a new TCP connection per request. This eliminates client confusion, behaving with "HTTP/1.0 semantics," avoiding the bug at the cost of one handshake per request.

Plan B (Support Keep-Alive): Modify `HttpPeer::connect` to loop-process multiple requests. Difficulty: `Http1Driver::read_request` moves `RecvStream` into a streaming body closure; once consumed, it cannot be reclaimed, necessitating a refactor to `Arc<Mutex<RecvStream>>` or a channel-based approach.

Plan C (Server-side per-request open_bi): The server side parses HTTP/1.1 boundaries and `open_bi()` per request. No changes needed on the Client side. This involves a larger scope of changes.

**Recommendation**: Plan A as a short-term fix (5 lines of code), Plan B as a long-term correct implementation.

---

## 5. Detailed Source Code Comparison with frp

### 5.1 HTTP Keep-Alive Architectural Difference

frp's "pure passthrough" architecture (where the server and client exchange raw TCP byte streams) naturally supports Keep-Alive without L7 awareness. To support features like Header rewriting (`protocol/rewrite.rs`), DuoTunnel introduced hyper HTTP parsing on the client side at the cost of needing to handle connection boundaries explicitly.

frp's vhost HTTP mode (`pkg/vhost/http.go`) uses `net/http.ReverseProxy` + `Transport` connection pool to perform L7 proxying and maintain backend connection pools on the **server side**—while the client side remains a pure byte stream. This is another design trade-off.

### 5.2 H2 Handshake Optimization

frp uses yamux to open lightweight virtual connections on a single TCP connection, avoiding L7 handshake overhead. DuoTunnel's `h2_proxy.rs` performs `h2::client::handshake` on every QUIC stream, which adds a full H2 protocol handshake (SETTINGS + WINDOW_UPDATE + ACK) on top of the QUIC stream's own creation overhead, essentially a double handshake.

**Insight from frp**: Maintain a `DashMap<GroupId, h2::client::SendRequest<B>>` H2 session pool in memory. New requests should prioritize picking an existing H2 sender from the pool and concurrently `send_request`; only create a new session if one doesn't exist in the pool. Since Quinn already provides stream-level isolation, H2 multiplexing over QUIC is redundant—directly mapping each H2 request to an independent QUIC stream rather than a shared H2 session might be a more appropriate long-term solution.

### 5.3 Round-Robin and Connection Management

frp uses `sync.Mutex` + an ordered list to guarantee stable round-robin; GT uses `sync.RWMutex` + a stable slice + an atomic index. DuoTunnel's fix (`RwLock<Vec>` + `AtomicUsize`) is entirely consistent with GT's approach and is correct.

---

## 6. Priority Repair Recommendations (Updated)

Based on code verification, prioritizing:

### P0 — Behavioral Bugs (Affecting Correctness)

| # | Issue | Location | Recommended Fix | Cost |
|---|------|------|---------|------|
| 1 | HTTP/1.1 Keep-Alive Connection Reset | `h1.rs:write_response` | Write `Connection: close` (Plan A) | 5 lines |
| 2 | TLS Connection Client Binding | `server/handlers/http.rs:88` | Perform per-request routing within H2 service_fn | Medium |
| 3 | `_protocol` Dead Code | `server/tunnel_handler.rs` | Remove match block | 10 lines |

### P1 — Performance Bottlenecks (Significant Impact)

| # | Issue | Location | Recommended Fix | Cost |
|---|------|------|---------|------|
| 4 | H2 Handshake per Request | `h2_proxy.rs:35` | Implement H2 session pool | High |
| 5 | Cloning Certificate Cache on Hit | `pki.rs:33` | Change to `Arc<Vec<CertificateDer>>` | Low |
| 6 | Fragmented relay implementations + BufReader/Writer | `proxy/base.rs` | Unify implementation, remove redundant buffer | Medium |
| 7 | Heap allocation for peek buffer in core.rs | `proxy/core.rs:47` | Change to stack allocation or fixed size | Low |

---

## 7. Conclusion

### Key Completed Repairs (Commendable)

- H2 Flow Control (OOM Protection) ✅
- Correct Round-Robin ✅
- Comprehensive TCP_NODELAY Coverage ✅
- QUIC Window Tuning + Configurable Parameters ✅
- Global TLS Config Sharing ✅
- Multi-QUIC Connection Pool ✅
- QUIC Endpoint Reuse ✅
- Constant-Time Token Verification ✅
- bincode Message Size Limits ✅
- IPv6 + `:443` Address Parsing Fixes ✅

### Core Issues Still Requiring Attention

1. **HTTP/1.1 Keep-Alive bug**: `write_response` lacks `Connection: close`, causing clients to be misled into reusing connections, leading to resets. High-priority P0 bug, fixable in 5 lines.

2. **TLS H2 per-connection routing**: All H2 requests on a TLS connection are bound to the same client; if the client disconnects, the entire connection fails, inconsistent with the per-request routing of plaintext H2.

3. **H2 Double Handshake**: Every H2 request performs a full H2 handshake on a QUIC stream, weakening the advantage of QUIC multiplexing.

4. **Relay Implementation Fragmentation**: Three different relay implementations, and the BufReader/BufWriter in `base.rs` is counterproductive; should be unified.

5. **`_protocol` Dead Code**: Can be cleaned up in 10 lines; should not be kept long-term.
