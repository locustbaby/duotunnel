# Duotunnel Future Architectural & Research Directions

This document outlines the strategic, high-impact research areas and architectural evolutions for `duotunnel`. These directions go beyond simple code optimizations, focusing on advanced systems programming, protocol design, kernel-bypass technologies, and adaptive congestion/overload controls tailored for high-concurrency, low-latency, and high-security tunneling environments.

---

## 1. Multipath QUIC (MP-QUIC) & Multi-Homing Failover

### 1.1 Context & Motivation
Currently, `duotunnel` multiplexes streams over a single QUIC connection bound to a single local IP/port and a single remote destination ([transport/quic.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/transport/quic.rs)). If the network interface experiences packet loss or drops entirely (e.g., a mobile device switching from Wi-Fi to LTE, or a server with dual-ISP connections), the tunnel suffers from a temporary blackout during connection migration or reconnection.

Implementing **Multipath QUIC (MP-QUIC)** allows a single logical `duotunnel` connection to simultaneously utilize multiple physical paths (e.g., dual-homed servers utilizing both China Telecom and China Unicom paths, or client devices using Wi-Fi + 5G). 

### 1.2 Implementation Pathway in Duotunnel
1. **Upstream QUIC Engine Adaptation:** Quinn is tracking MP-QUIC draft specifications. To implement this today, `duotunnel` can spawn multiple independent UDP sockets bound to different network interfaces and supply them to a custom connection controller.
2. **Multipath Packet Scheduler:**
   - **MinRTT Scheduler:** Send interactive traffic (gRPC, control signals) over the path with the lowest RTT.
   - **Redundant Scheduler:** Replicate critical packets (handshakes, control frames) across all paths to guarantee delivery with minimal jitter.
   - **Throughput Aggregator:** Strip-stream large downloads across all paths based on their relative congestion window capacity.
3. **Dynamic Path Management:** Periodically probe secondary paths using QUIC `PATH_CHALLENGE` frames to measure RTT and loss before routing application streams.

### 1.3 Expected Impact
* **High Availability:** True zero-downtime failover. Sockets on one interface can die completely without terminating the tunnel connection.
* **Throughput:** Aggregation of bandwidth across multiple physical uplinks.

---

## 2. Cross-Protocol Adaptive Backpressure & Flow Control

### 2.1 Context & Motivation
In `duotunnel`, the data path crosses an L4 TCP boundary and a multiplexed QUIC stream boundary ([relay.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/relay.rs)). 
Currently, the proxy relies on fixed-size buffers (`DEFAULT_RELAY_BUF_SIZE` = 64KB) to bridge these streams. If a downstream client reads very slowly from its TCP socket but the upstream sends data at 10 Gbps over QUIC, the proxy has to buffer this data.
* If the user-space buffer is too small, throughput drops because we block the sender prematurely (rendezvous bottleneck).
* If the user-space buffer is too large, we accumulate megabytes of unread data in user-space memory, increasing memory usage and P99 latency (bufferbloat).

### 2.2 Implementation Pathway in Duotunnel
Develop a **Cross-Protocol Backpressure loop** that dynamically scales the QUIC Stream-level Flow Control window based on the actual TCP socket metrics.
1. **Dynamic BDP (Bandwidth-Delay Product) Buffer Auto-Scaling:**
   - Instead of allocating a static buffer, inspect the TCP socket status via `libc::getsockopt(..., IPPROTO_TCP, TCP_INFO, ...)` to retrieve the current congestion window (`snd_cwnd`) and Smoothed RTT (`srtt`).
   - Dynamically size the copying buffer: `buffer_size = clamp(cwnd * srtt, 16KB, 4MB)`.
2. **Proactive Stream Flow Control Feedback:**
   - Modify `quinn_io.rs` to adjust Quinn's stream receive window dynamically. If the local TCP socket's send buffer is full (`tcp_wmem` pressure), proactively reduce the QUIC stream flow control window to force the remote sender to throttle at the protocol layer, preventing user-space memory accumulation.

### 2.3 Expected Impact
* **Memory Efficiency:** Idle or slow tunnels consume `<16KB` of buffer space, while active high-speed pipes scale up to `4MB` dynamically.
* **Latency Control:** Eliminates bufferbloat-induced delay spikes inside the proxy memory queue.

---

## 3. Kernel-User Bypass: `io_uring` & Thread-Per-Core (TPC) Evolution

### 3.1 Context & Motivation
While Tokio's multi-threaded work-stealing runtime is highly general, it suffers from heavy syscall overhead (repeated `epoll_wait` + `read`/`write` calls) and CPU cacheline bouncing under massive concurrency.
To push performance to the physical hardware limit, we can transition the datapath from an event-driven `epoll` reactor to an **Asynchronous Completion Queue (`io_uring`)** operating in a **Thread-Per-Core (TPC) shared-nothing** architecture.

### 3.2 Implementation Pathway in Duotunnel
1. **Integrate Monoio / Tokio-Uring:** Write a specialized egress listener or relay runner using a thread-per-core runtime.
2. **Kernel Splitting via `IORING_OP_SPLICE`:**
   - For L4 TCP-to-TCP redirection, instead of reading bytes to user-space and writing them back, submit an `io_uring` splice command. This instructs the Linux kernel to pipe data directly between the two socket descriptors inside kernel space without a single user-space context switch.
3. **Zero-Copy UDP Transmission (`IORING_OP_SEND_ZC`):**
   - QUIC's heavy UDP datagram processing can be offloaded by submitting ring requests with the `MSG_ZEROCOPY` flag, bypassing physical packet copies from user-space memory to the kernel socket buffers.

### 3.3 Expected Impact
* **Syscall Reduction:** Reduces CPU usage on system calls by up to 70% during high-concurrency streaming.
* **P99 Latency:** Thread-per-core eliminates task stealing and cross-thread lock contention, stabilizing tail latency.

---

## 4. Self-Adaptive Queueing Delay Control (CoDel) for Overload Protection

### 4.1 Context & Motivation
The current `overload.rs` ([lb/overload.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/lb/overload.rs)) uses static concurrent stream thresholds to trigger slowpath yields or sleep backoffs.
However, concurrent stream counts do not accurately reflect actual queueing delay or resource saturation. 100 fast, lightweight requests might cause less latency than 5 slow, heavy queries. Relying on absolute thresholds causes the proxy to either under-utilize resources or experience latency spikes.

### 4.2 Implementation Pathway in Duotunnel
Implement a **Controlled Delay (CoDel)** queueing algorithm for incoming requests inside the proxy dispatcher.
1. **Enqueue Timestamping:**
   - Every request entering the proxy is tagged with an enqueue timestamp `t_enq`.
2. **Moving Minimum Queueing Delay Tracking:**
   - The registry tracks the minimum delay $D_{min}$ experienced by requests over a sliding window (e.g., 100ms).
   - $D_{min} = t_{dequeue} - t_{enq}$.
3. **Adaptive Slow-path Drop / Backoff Trigger:**
   - If $D_{min}$ exceeds a target threshold (e.g., 5ms) consistently for the duration of the window, the overload controller enters the **Drop/Backoff State**.
   - Instead of yielding, the controller drops or rejects new streams early (Load Shedding) to protect existing streams, or aggressively throttles the backoff timers in `maybe_slow_path`.
   - Once $D_{min}$ falls below 5ms, the controller automatically exits the drop state.

### 4.3 Expected Impact
* **Jitter Prevention:** Avoids the "cliff effect" where the proxy suddenly becomes unresponsive due to thread starvation.
* **Fairness:** Ensures that slow backend services do not cause queue starvation for fast services sharing the same QUIC tunnel.

---

## 5. DPI (Deep Packet Inspection) Defeating & Traffic Obfuscation

### 5.1 Context & Motivation
Because `duotunnel` acts as a tunnel, it is susceptible to detection by advanced Deep Packet Inspection (DPI) firewalls. Original QUIC handshakes, ALPN negotiations, TLS Cipher Suites, and traffic volume signatures can be analyzed to identify, classify, or block `duotunnel` traffic.

### 5.2 Implementation Pathway in Duotunnel
1. **uTLS-style Client Hello Camouflage:**
   - Standard Rustls handshakes produce a specific fingerprint (JA3/JA4). Research is needed to customize the ClientHello packet generation in Quinn's TLS config ([pki.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/infra/pki.rs)) to mimic common browsers (Chrome, Firefox) or standard mobile application traffic.
2. **Adaptive Packet Padding & Morphing:**
   - Implement randomized frame padding. Instead of sending raw payloads, inject random padding bytes into the QUIC packets to alter the packet-size histogram.
   - Inject dummy `PING` or `PADDING` frames during active transfer to confuse statistical classifiers that look for specific upload/download packet sequences.
3. **Active Multiplexing Obfuscation:**
   - Introduce low-bandwidth background "heartbeat" traffic that mimics standard HTTPS browsing patterns (e.g., periodic small bursts) when the tunnel is otherwise idle, preventing traffic profiling from detecting silent tunnels.

### 5.3 Expected Impact
* **Censorship Resistance:** High survivability in hostile network environments.
* **Anonymity:** Obfuscates the proxy's operational patterns, making it look like standard web traffic.

---

## 6. Zero-Downtime Hot Upgrades (FD Passing)

### 6.1 Context & Motivation
Currently, upgrading `duotunnel` or changing static configurations requires restarting the service. While hot-reload is supported for routing rules ([hot_reload.rs](file:///Users/sexy/Documents/GitHub/duotunnel/server/control/hot_reload.rs)), upgrading the binary itself requires dropping the listening TCP/UDP sockets. This disconnects all active user connections and breaks ongoing QUIC tunnels.

### 6.2 Implementation Pathway in Duotunnel
Implement **File Descriptor Passing (FD Passing)** via Unix Domain Sockets to allow seamless binary hot-upgrades.

1. **State Transfer Protocol:**
   - When the new binary starts with a specific upgrade flag (e.g., `--upgrade`), it connects to the old process via a local Unix Domain Socket.
2. **FD Passing via `scm_rights`:**
   - The old process passes the file descriptors of all active listening sockets (UDP for QUIC, TCP for ingress listeners) to the new process using `ancillary data` (`sendmsg` with `SCM_RIGHTS`).
3. **Handover and Drain:**
   - The new process immediately takes over the sockets and starts accepting new connections.
   - The old process stops accepting new tunnels, but remains alive in a "draining" state until all of its current QUIC streams and active TCP connections finish their transfers, then exits cleanly.

### 6.3 Expected Impact
* **Zero-Downtime Upgrades:** Network sockets are never closed. Clients experience zero connection drops or timeouts during proxy upgrades.
* **Operational Agility:** Enables continuous deployment pipelines to push compiler updates or runtime changes directly to production tunnels at any time of day.
