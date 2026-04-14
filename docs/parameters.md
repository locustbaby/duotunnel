# Parameter Reference: Timeouts, Limits, and Buffers

Full request path: `k6 → TCP (entry) → client → QUIC → server → TCP (upstream)`

---

## TCP Entry Listener (client side)

| Parameter | Location | Default | CI | Effect when hit |
|-----------|----------|---------|-----|-----------------|
| Listen backlog | `tunnel-lib/src/transport/listener.rs:28` | 4096 | — | Kernel drops new conns; client sees `ECONNREFUSED` |
| Accept workers | `tunnel-lib/src/transport/listener.rs:12` | 4 | — | Accept loop serializes; excess queue at kernel |
| EMFILE backoff | `client/entry.rs:16` (`EMFILE_BACKOFF_MS`) | 100ms | — | On errno 24, sleep 100ms before retry |
| TCP nodelay | `tunnel-lib/src/transport/tcp_params.rs:14` | true | — | Nagle disabled; low-latency small writes |
| TCP recv buf | `tunnel-lib/src/transport/tcp_params.rs:15` | 4 MiB | — | Kernel backpressure; sender blocks |
| TCP send buf | `tunnel-lib/src/transport/tcp_params.rs:16` | 4 MiB | — | Kernel backpressure; sender blocks |
| TCP keepalive | `tunnel-lib/src/transport/tcp_params.rs:17` | true | — | OS probes detect dead conns |
| TCP user timeout | `tunnel-lib/src/transport/tcp_params.rs:18` | 30s | — | Kernel force-closes after 30s unacked data |

---

## QUIC Transport (client ↔ server)

| Parameter | Location | Default | CI client | CI server | Effect when hit |
|-----------|----------|---------|-----------|-----------|-----------------|
| connections | `client/config.rs:28` | 1 | 1 | — | Total stream capacity = connections × max_concurrent_streams |
| max_concurrent_streams | `tunnel-lib/src/transport/quic.rs:19` | 1000 | 1000 | 1000 | `open_bi()` blocks waiting for slot; hits open_bi timeout if no slot freed in time |
| stream_window | `tunnel-lib/src/transport/quic.rs:20` | 4 MiB | — | — | Per-stream flow control; sender blocks when window exhausted |
| connection_window | `tunnel-lib/src/transport/quic.rs:21` | 32 MiB | — | — | Connection-level throttle; all streams slow down |
| send_window | `tunnel-lib/src/transport/quic.rs:22` | 8 MiB | — | — | Sender blocks when congestion window exhausted |
| keepalive_secs | `tunnel-lib/src/transport/quic.rs:23` | 20s | 10s | 10s | Keepalive pings prevent NAT/firewall timeout |
| idle_timeout_secs | `tunnel-lib/src/transport/quic.rs:24` | 60s | 30s | 30s | Connection closed if no packets received for N secs |
| congestion | `tunnel-lib/src/transport/quic.rs:25` | bbr | bbr | bbr | Controls throughput ramp-up and loss recovery |

---

## QUIC Stream Acquisition (hot path)

| Parameter | Location | Default | CI | Effect when hit |
|-----------|----------|---------|-----|-----------------|
| open_stream_timeout_ms (client) | `client/config.rs:103` | 5000ms | — (5000ms) | `open_bi()` gives up; tries next conn; if all fail, TCP conn to k6 is closed → k6 sees request timeout |
| open_stream_timeout_ms (server) | `tunnel-store/src/server_config.rs:101` | 5000ms | — (5000ms) | `open_bi()` gives up; ingress TCP conn closed with error |

**This is the known failure point for egress 8K QPS**: at high load, all stream slots fill up, `open_bi()` waits 5s and times out, k6 sees request timeout (~10s total with its own timeout).

---

## Server Overload Control

| Parameter | Location | Default | CI | Effect when hit |
|-----------|----------|---------|-----|-----------------|
| mode | `tunnel-store/src/server_config.rs:56` | `inflight_slowpath` | — | `burst`: no backpressure; `inflight_slowpath`: yield/sleep below |
| inflight_yield_threshold | `tunnel-store/src/server_config.rs:58` | 800 | — | ≥800 in-flight: `yield_now()`, defers current task |
| inflight_sleep_threshold | `tunnel-store/src/server_config.rs:59` | 950 | — | ≥950 in-flight: sleep `inflight_sleep_ms` |
| inflight_sleep_ms | `tunnel-store/src/server_config.rs:60` | 2ms | — | Duration of sleep at sleep_threshold |
| emfile_backoff_ms | `tunnel-store/src/server_config.rs:47` | 100ms | — | On EMFILE from accept(), sleep before retry |

---

## HTTP Upstream Pool (server egress / client proxy)

| Parameter | Location | Default | CI server | Effect when hit |
|-----------|----------|---------|-----------|-----------------|
| max_idle_per_host | `tunnel-lib/src/config/http_pool.rs:14` | 128 | 128 | Excess idle conns closed; new requests create fresh conn |
| idle_timeout_secs | `tunnel-lib/src/config/http_pool.rs:13` | None | None | None = never evict idle; `Some(n)` = evict after n secs |
| tcp_keepalive_secs | `tunnel-lib/src/config/http_pool.rs:15` | 15s | — (15s) | OS keepalive probes; hyper `is_open()` detects stale conns |

Note: `pool_timer` is set on all clients so `idle_timeout_secs` actually fires when configured.

---

## Proxy Buffers

| Parameter | Location | Default | CI | Effect when hit |
|-----------|----------|---------|-----|-----------------|
| peek_buf_size | `tunnel-lib/src/config/proxy_buffers.rs:17` | 16 KiB | — | Protocol detection window; insufficient → detection failure |
| http_header_buf | `tunnel-lib/src/config/proxy_buffers.rs:18` | 8 KiB | — | HTTP header read buffer; hard cap is 64 KiB (`egress/http.rs`) |
| http_body_chunk | `tunnel-lib/src/config/proxy_buffers.rs:19` | 8 KiB | — | Read chunk size for HTTP body streaming |
| relay_buf_size | `tunnel-lib/src/config/proxy_buffers.rs:20` | 64 KiB | — | BufReader capacity on QUIC stream relay path; min 4 KiB |

---

## Client Reconnect / Connection Setup

| Parameter | Location | Default | CI | Effect when hit |
|-----------|----------|---------|-----|-----------------|
| connect_timeout_ms | `client/config.rs:99` | 10000ms | 3000ms | QUIC handshake timeout; transient error → retry with backoff |
| resolve_timeout_ms | `client/config.rs:100` | 5000ms | 2000ms | DNS timeout; transient error → retry |
| login_timeout_ms | `client/config.rs:101` | 5000ms | 3000ms | Per-step timeout for login handshake; transient error → reconnect |
| login_timeout (server) | `tunnel-store/src/server_config.rs:98` | 10s | — | Server closes stream if client doesn't send login in time |
| initial_delay_ms | `client/config.rs:96` | 1000ms | 500ms | First reconnect wait |
| max_delay_ms | `client/config.rs:97` | 60000ms | 5000ms | Exponential backoff cap |
| grace_ms | `client/config.rs:98` | 100ms | — | Wait between cancel and reconnect |
| startup_jitter_ms | `client/config.rs:102` | 300ms | 50ms | Random delay on startup to prevent thundering herd |

---

## Known Issues and TODOs

### TODO: Error Code Design and Propagation

Currently when a request fails mid-path (e.g. `open_bi` timeout, upstream EOF, upstream 502), the error is:
- Logged locally as a `debug!` or `warn!` with no structured code
- Not propagated back to the caller in a machine-readable form
- Indistinguishable at the k6/client level (all appear as connection close / request timeout)

Need to design:
- Structured error codes for each failure point in the path (entry accept, open_bi timeout, QUIC stream error, upstream connect fail, upstream EOF, upstream timeout)
- Error propagation: carry error context from server back to client entry, and from client entry back to the TCP caller (e.g. return HTTP 502/504 with error code header instead of bare close)
- Unified logging: each error emits a log with `error_code`, `phase` (entry/quic/upstream), `direction` (ingress/egress), and `upstream` fields
- Metrics: per-error-code counters exposed on metrics port

### TODO: Unified Parameter Configuration Design

The current parameter space is fragmented across multiple config structs (`ReconnectConfig`, `QuicConfig`, `HttpPoolConfig`, `ProxyBuffers`, `OverloadConfig`, `ServerBasicConfig`) with inconsistent naming and no single place to reason about the full path.

Need to design:
- A unified per-path timeout budget: total allowed latency broken down per segment (TCP accept → QUIC open → upstream connect → upstream response)
- Consistent naming convention across client and server configs
- Parameter inheritance: server pushes recommended parameters to client via control channel instead of requiring manual sync of both yamls
- Validation that the sum of per-segment timeouts is less than the client-facing SLO
