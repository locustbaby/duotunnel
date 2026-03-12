# Tunnel TODO

## Config Tuning (No Code Changes)

### [TODO-16] QUIC connections: 1 → 4

**Files**: `ci-helpers/client.yaml`
**Priority**: High

`quic.connections` is not configured in `client.yaml`, defaulting to 1. All traffic is squeezed into a single QUIC connection, creating a bottleneck for single UDP socket serial encryption/decryption and single-connection flow control.

Change to `connections: 4` to distribute the load across 4 QUIC connections.

### [TODO-17] max_concurrent_streams: 200 → 1000

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

Use `tokio-uring` or `monoio` as a replacement for tokio's epoll backend. Currently, no Rust QUIC library natively supports `io_uring`, pending progress on quinn issue #915.

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

### [TODO-35] Thread-Shared Global Connection Pooling
Based on Pingora's core design. Break the per-task connection pool limit and implement global `Upstream` connection sharing. Reduces TCP/TLS handshake frequency under high concurrency, significantly improving TTFB.

### [TODO-36] Static Dispatch Refactor: Eliminate Boxed Traits
**Files**: `tunnel-lib/src/proxy/peers.rs`
Extreme CPU optimization following Pingora practices. Refactor `PeerKind`'s `Dyn(Box<dyn UpstreamPeer>)` to `Enum`-based static dispatch where possible to reduce vtable lookup overhead and improve CPU branch prediction.

### [TODO-37] Seamless Graceful Handover
Implement a signal-based (e.g., SIGQUIT) graceful exit mechanism. Upon entering the shutdown sequence, reject new tunnel requests but allow existing long-lived connections to finish processing before exiting, ensuring zero business interruption during upgrades.

### [TODO-38] Vectorized IO Relaying (Vectorized IO / writev)
On the HTTP relaying path, combined with [TODO-33] zero-copy parsing, use `writev` to send Header offset slices and Body content together. Avoids the overhead of copying multiple memory blocks into a continuous buffer before sending.

---

## Bench Fixes

### [TODO-15] egress_http_post Overflow Phase Boundary

**Files**: `ci-helpers/k6/bench.js`, `bench/index.html`
**Priority**: Low

`egress_http_post` startTime=6s + stages 5s+20s = ends at 31s, but Phase "Basic" end=29. The chart annotation box does not completely cover this scenario, but it does not affect the data accuracy.
