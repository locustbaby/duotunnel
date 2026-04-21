# Backpressure & Overload Control

## Current State (duotunnel)

### Layer 1 — TCP Accept (ingress entry)

| Location | Semaphore | Default | On limit |
|---|---|---|---|
| `server/handlers/http.rs` | `tcp_semaphore` | 10,000 | **wait** — `accept()` blocks, OS backlog absorbs |
| `server/handlers/tcp.rs` | `tcp_semaphore` | 10,000 | **wait** |
| `client/entry.rs` | per-listener semaphore | config | **wait** |

### Layer 2 — QUIC Connection (tunnel link)

| Location | Mechanism | Default | On limit |
|---|---|---|---|
| `server/handlers/quic.rs` | `quic_semaphore.acquire_owned()` | 10,000 | **wait** |
| `tunnel-lib/transport/quic.rs` | `max_concurrent_bidi_streams` | 1,000 | QUIC flow control blocks sender |
| `tunnel-lib/transport/quic.rs` | stream recv window | 4 MB | QUIC flow control |
| `tunnel-lib/transport/quic.rs` | conn recv window | 32 MB | QUIC flow control |

### Layer 3 — Work Stream (ingress: server→client)

| Location | Mechanism | Default | On limit |
|---|---|---|---|
| `client/main.rs` | `stream_semaphore.acquire_owned().await` | `max_concurrent_streams` (config, default 100) | **wait** — `accept_bi()` blocks, QUIC flow control backs up naturally |

### Layer 4 — Upstream HTTP Pool (egress)

| Location | Mechanism | Default | On limit |
|---|---|---|---|
| `tunnel-lib/egress/http.rs` | hyper connection pool | `pool_max_idle_per_host=10` | excess idle connections are closed; new requests open fresh connections |

### Layer 5 — open_bi() Timeout

Both directions have a hard timeout on `open_bi()`:
- Server side: `server.open_stream_timeout_ms` (default 5,000 ms)
- Client side: `reconnect.open_stream_timeout_ms` (default 5,000 ms)

On timeout → connection dropped with error. This is the only hard-drop path remaining.

### Layer 6 — TCP Socket Buffers

`SO_RCVBUF` / `SO_SNDBUF` both default 4 MB. TCP backlog hardcoded to 4,096 in `tunnel-lib/transport/listener.rs`.

---

## Pingora Reference Design

### What Pingora does differently

**Accept loop — FD exhaustion backoff**
When `accept()` returns `EMFILE` (too many open files), Pingora sleeps 1 s before retrying instead of spinning. This is the critical guard against fd-exhaustion CPU storms. (`pingora-core/src/services/listening.rs`)

**Upstream connection pool — LRU + hot queue**
Pool of reusable upstream connections with LRU eviction. A 16-slot lock-free hot queue per pool node reduces mutex contention on hot connections. Default pool size 128. (`pingora-pool/src/connection.rs`)

**HTTP/2 stream concurrency — atomic count, return None**
Each H2 connection tracks `current_streams` atomically. When at `max_streams`, `spawn_stream()` returns `Ok(None)` — caller must open a new connection or wait. No hard drop, no panic. (`pingora-core/src/connectors/http/v2.rs`)

**Subrequest queues — bounded channel size 4**
Internal subrequest pipelines use bounded async channels (size 4). Senders block when full — pure async backpressure, no extra state. (`pingora-core/src/protocols/http/subrequest/server.rs`)

**Load shedding hook — `early_request_filter`**
Executed before any upstream selection. Return `Err()` here to reject cheaply before touching any resource. This is where rate limiting, circuit breaking, or queue-depth shedding belongs. (`pingora-proxy/src/proxy_trait.rs`)

**Retry buffer — 64 KB cap with truncation flag**
Request bodies buffered for retry are capped at 64 KB. Above that, `truncated=true` and the request is non-retryable. Prevents OOM from large retried payloads.

**Health check — consecutive threshold**
Backend health requires N consecutive successes to flip healthy, N consecutive failures to flip unhealthy. Prevents flapping on transient errors.

---

## Gaps & TODOs

| Gap | Pingora has | duotunnel status |
|---|---|---|
| FD exhaustion backoff | 1 s sleep on EMFILE in accept | none — accept loop will error and exit |
| Early load shedding hook | `early_request_filter` | none |
| H2 pool stream limit | atomic count, `Ok(None)` path | `pool_max_idle_per_host` only; no per-conn stream count |
| Health check on upstreams | consecutive-threshold health flip | none |
| Retry body cap | 64 KB truncation flag | none |
| open_bi hard-drop | N/A | present — only remaining drop path |

The most impactful near-term gaps:
1. **FD exhaustion backoff** — add `tokio::time::sleep(1s)` when `accept()` returns `EMFILE` (same as Pingora). Prevents the accept loop from exiting under fd pressure.
2. **open_bi timeout** — currently drops the connection. Under overload this is correct; under normal load the 5 s timeout is fine. No change needed unless p99 shows spurious timeouts.
3. **upstream pool stream limit** — hyper's H2 client already respects server SETTINGS; no action needed.
