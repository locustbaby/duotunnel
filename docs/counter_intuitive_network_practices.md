# Counter-Intuitive Systems Programming Practices & Tunnel Optimization Guide

This document records the classic counter-intuitive systems programming patterns and details how to apply them to optimize the `duotunnel` architecture across both L4 (TCP/UDP) and L7 (HTTP/gRPC/WebSocket) layers.

---

## 1. Catalog of Counter-Intuitive Systems Patterns

### 1.1 Copying is Faster than Zero-Copy for Small Payloads
- **The Assumption:** Bypassing the CPU memory copy using zero-copy APIs (like Linux `sendfile`, `splice`, or `MSG_ZEROCOPY`) is always faster.
- **The Reality:** For small payloads (typically `< 8KB`), performing a simple CPU `memcpy` in user-space is faster and uses less CPU than zero-copy.
- **The Mechanics:** Zero-copy requires the kernel to pin user-space memory pages, update page tables, and deliver asynchronous completions (e.g., via the socket's error queue). These syscalls and page-table manipulations are far more expensive than copying a few kilobytes in CPU registers.
- **Duotunnel Application:** Keep L7 small gRPC/API request parsing on user-space memory buffers ([copy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/copy.rs#L7)), while enabling zero-copy (`read_chunk`) strictly for large body streams.

### 1.2 Thread-Per-Core (TPC) Outperforms Work-Stealing Event Loops
- **The Assumption:** Multi-threaded work-stealing schedulers (like Go's G-M-P or Rust's default multi-threaded Tokio runtime) are optimal because they automatically load-balance tasks across all CPU cores.
- **The Reality:** For high-throughput network proxies, work-stealing causes "cacheline bouncing." When a connection's task is stolen by another core, its socket state, file descriptors, and user-space cache lines must migrate across cores, causing L1/L2 cache misses. 
- **The Mechanics:** Thread-Per-Core (shared-nothing) architectures run isolated event loops per CPU core and pin connections to specific cores for their entire lifecycle, eliminating cross-core synchronization and lock contention.
- **Duotunnel Application:** Restructure hot tunnel worker threads to run on pinned, single-threaded runtimes (similar to the metrics-worker in [supervisor.rs:L152](file:///Users/sexy/Documents/GitHub/duotunnel/server/runtime/supervisor.rs#L152)), binding each QUIC tunnel socket to a dedicated core.

### 1.3 Nagle + Delayed ACK Causes Latency Deadlocks
- **The Assumption:** Enabling Nagle's algorithm (coalesces small send packets) and Delayed ACKs (delays packet confirmations to save bandwidth) is a good default combination.
- **The Reality:** They cause temporary latency deadlocks, injecting 40ms to 200ms of latency into small-packet interactions (like gRPC pings or API handshakes).
- **The Mechanics:** Nagle waits for an outstanding ACK before sending small packets, while the receiver's Delayed ACK waits for more data (up to 200ms) before sending the ACK. Both sides enter a silent wait state until the receiver's ACK timer expires.
- **Duotunnel Application:** Keep `nodelay: true` enforced on all client and server configurations ([client.yaml:L38](file:///Users/sexy/Documents/GitHub/duotunnel/config/client.yaml#L38)) to bypass Nagle entirely.

### 1.4 User-Space Buffer Assembly beats Vectored I/O (writev)
- **The Assumption:** Gathering multiple chunks into a vectored write array (`writev` / `poll_write_vectored`) is faster than doing multiple writes because it executes only a single syscall.
- **The Reality:** For small buffers, copying them into a single contiguous memory block in user-space and calling a standard `write` is faster.
- **The Mechanics:** `writev` forces the kernel to parse the `iovec` array, validate each user-space address range, and perform multiple address translations. The overhead of these validation steps exceeds the cost of a user-space `memcpy`.
- **Duotunnel Application:** In chunked HTTP response serialization, copy the headers and size prefix into a contiguous buffer before writing, rather than using vectored arrays.

### 1.5 Backoff with Jitter prevents Cascade Failures
- **The Assumption:** When a connection drops, clients should reconnect as fast as possible to minimize downtime.
- **The Reality:** Rapid reconnections create a "thundering herd" (self-induced DDoS) that repeatedly crashes the server upon recovery.
- **The Mechanics:** Randomizing client reconnection delay spreads the load across a wider window, allowing the server's connection queues to absorb the requests.
- **Duotunnel Application:** Keep and tune the `startup_jitter_ms: 300` parameter in client configs ([client.yaml:L63](file:///Users/sexy/Documents/GitHub/duotunnel/config/client.yaml#L63)).

---

## 2. Advanced Optimization Directions for Duotunnel

Below are the key architectural directions to consider for scaling `duotunnel` in high-throughput and low-latency environments:

```
                  ┌──────────────────────────────────────────────┐
                  │          duotunnel Optimization Core         │
                  └──────────────────────┬───────────────────────┘
          ┌──────────────────────────────┼──────────────────────────────┐
          ▼                              ▼                              ▼
 ┌─────────────────┐            ┌─────────────────┐            ┌─────────────────┐
 │   CPU Cache &   │            │   UDP Transport │            │  User-Space     │
 │ Thread Pinning  │            │   Buffer Tuning │            │  Spin-Polling   │
 └─────────────────┘            └─────────────────┘            └─────────────────┘
```

### 2.1 CPU Cacheline Alignment and False Sharing
Under high concurrency (e.g., 1000 streams), threads running on different cores will modify the connection registry and load-balancing states. If variables modified by different threads reside on the same 64-byte L1 cacheline, they cause **false sharing**—the CPU is forced to invalidate and bounce the cacheline between cores, killing performance.
- **Implementation:**
  - `InflightSlot` currently uses `CachePadded` in [inflight.rs:L12](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/lb/inflight.rs#L12) to align slots to 64-byte boundaries.
  - **Further action:** Ensure that global metrics, registry registration tables, and hot connection pool statistics also employ `crossbeam_utils::CachePadded` to align memory layout and avoid cache collisions on shared structures.

### 2.2 UDP Socket Buffer Tuning for QUIC
Because QUIC runs over UDP, the operating system's default UDP socket receive/send buffers (often only 200KB) are the primary bottleneck for high-bandwidth tunnels. If the socket buffer fills up, the OS drops incoming UDP datagrams, causing QUIC to trigger packet retransmissions and throttle the congestion window (BBR).
- **Implementation:**
  - Configure large UDP socket buffers (`udp_recv_buf_mb` / `udp_send_buf_mb`) on both client ([client.yaml:L32](file:///Users/sexy/Documents/GitHub/duotunnel/config/client.yaml#L32)) and server sides.
  - **Tuning rule:** Set UDP socket buffer size to at least the Bandwidth-Delay Product (BDP) of the link (e.g., for a 10Gbps link with 20ms RTT, BDP is 25MB).

### 2.3 User-Space Spin-Polling (Busy Polling)
For ultra-low latency requirements (like gRPC microservices), sleeping on `epoll` introduces 10us–20us of thread-wakeup latency.
- **Implementation:**
  - Introduce an optional busy-polling loop inside the client's QUIC-to-TCP copy loop. 
  - Instead of immediately yielding the task to Tokio when no data is ready, perform a brief spin loop (calling `.try_read()` or yielding the execution budget for up to 50 microseconds). If data arrives during the spin, it is processed instantly without thread sleep/wake cycles.

### 2.4 DNS Background Pre-Fetching
Proxies routing to external upstreams (e.g., `www.google.com:443`) suffer from latency spikes when DNS cache TTL expires, forcing the connection task to block on DNS resolution.
- **Implementation:**
  - Enhance the `resolver_cached` plugin. Instead of lazy-resolving on cache miss, run a background thread that proactively pre-fetches and refreshes cached DNS entries *before* they expire. This ensures that the hot path always resolves DNS in O(1) time without blocking.

---

## 3. Protocol-Specific Tuning Secrets (QUIC, TCP, UDP, HTTP)

Based on a deep review of `duotunnel`'s protocol logic, here are the core advanced tuning lessons for each protocol layer.

### 3.1 UDP & QUIC: The `SO_REUSEPORT` NAT Rebinding Trap
- **The Issue:**
  In [quic.rs:L70](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/transport/quic.rs#L70), `duotunnel` binds UDP sockets using `reuse_port(true)`. In Linux, `SO_REUSEPORT` on UDP distributes incoming packets across multiple sockets by hashing the 4-tuple (source IP/port, dest IP/port).
  However, QUIC is designed to support **Connection Migration (NAT Rebinding)**. If a client's IP or port changes (e.g., switching from Wi-Fi to 4G), the 4-tuple hashes to a *different* UDP socket (Core). Since that socket has no session state for the client's Connection ID (CID), the packet is dropped, terminating the connection.
- **The Solution:**
  Modern QUIC proxies (like Cloudflare, Nginx, or HAProxy) load an **eBPF program** using a `BPF_MAP_TYPE_REUSEPORT_SOCKARRAY` map. This program parses the QUIC header, extracts the **Connection ID (CID)**, and routes the packet to the exact socket owning that CID, ensuring seamless NAT rebinding across multiple ports/cores.

### 3.2 TCP: The Auto-Tuning Buffer Trap
- **The Issue:**
  In [tcp_params.rs:L28](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/transport/tcp_params.rs#L28), `duotunnel` explicitly sets `SO_RCVBUF` and `SO_SNDBUF` (defaults to 4MB):
  ```rust
  set_sock_opt_u32(fd, libc::SOL_SOCKET, libc::SO_RCVBUF, size)?;
  ```
  **By calling setsockopt to set buffer sizes manually, Linux's built-in TCP autotuning is permanently disabled for that socket!**
  Linux's autotuning (`tcp_rmem` and `tcp_wmem`) dynamically scales the receive/send windows up to 6MB or 16MB based on latency and RTT, and shrinks them for idle connections to conserve RAM. Once you set a static size, the OS locks the window, leading to:
  1. Window limit capping on high-bandwidth high-latency links.
  2. Severe memory waste (e.g., 10,000 idle connections statically locked at 8MB consumes 80GB of RAM).
- **The Solution:**
  Leave `recv_buf_size` and `send_buf_size` configured as `None` (default) to let the Linux kernel dynamically tune socket buffers.

### 3.3 TCP: The Power of `TCP_USER_TIMEOUT`
- **The Issue:**
  If a remote host goes down or drops packets without sending a FIN/RST, TCP will keep retrying with exponential backoff. Under default Linux settings, it takes **15 minutes** (`tcp_retries2 = 15`) before the socket reports an ETIMEDOUT error.
- **The Solution:**
  `duotunnel` implements `TCP_USER_TIMEOUT` ([tcp_params.rs:L37](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/transport/tcp_params.rs#L37)). Setting this to `30,000` ms (30s) ensures that if a tunnel connection drops, the client immediately detects the failure in 30 seconds instead of hanging for 15 minutes, allowing rapid reconnect triggers.

### 3.4 HTTP/2 (H2c): The Consumed Stream Retry Trap
- **The Issue:**
  In [http_connector.rs:L164](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/proxy/http_connector.rs#L164), if cleartext H2 (H2c) fails, `duotunnel` retries once with HTTP/1.1, but *only* if `request.body().is_end_stream()` is true.
  If a request has a body (e.g., a POST payload), the body stream is consumed by the first failed H2 attempt. It cannot be rewound or read again, so the request immediately fails.
- **The Solution:**
  For critical upstreams, implement protocol probing (sniffing / ALPN validation) on connection setup *before* the first request body is sent, preventing first-attempt body exhaustion.

