# DuoTunnel Overview

> High-performance bidirectional tunnel proxy system based on QUIC (Quinn)
>
> Inspired by frp design philosophy, implementing transparent tunneling + configuration distribution + grouping + Rules-based routing

**Spec index:** [architecture.md](./architecture.md) (topology & call flows, index center) · [duotunnel-lib.md](./duotunnel-lib.md) (shared library, wire protocol) · [parameters.md](./parameters.md) (tunables, defaults) · per-crate `*-runtime.md`

This document keeps only the content that is unique to a product/design overview:
design goals, the frp comparison, the Prometheus metrics surface, a security
summary, design principles, and a pointer to open optimization work. Everything
else previously covered in the old `DESIGN.md` (module tree, wire format,
connection/data flows, core-component code samples, concurrency control,
configuration format) now lives in the specs linked above — see the "详见"
pointers in each section below.

---

## 1. Design Goals

### 1.1 Bidirectional Request Proxying

```
Forward Proxy (Ingress):  External Request → Server → Client → Local Service
Reverse Proxy (Egress):   Internal Request → Client → Server → External Service
```

### 1.2 Core Advantages (vs frp)

| Feature | frp (TCP + Yamux) | DuoTunnel (QUIC) |
|---------|-------------------|------------------|
| Data Channel Creation | frpc initiates TCP connection | Server directly calls `open_bi()` |
| Message Exchanges | 3 times | 1 time |
| Latency | At least 1.5 RTT | 0 RTT (Stream creation requires no handshake) |
| Connection Pool | Requires pre-creation | Not needed (on-demand creation) |
| Multiplexing | Yamux | Native QUIC Stream |
| 0-RTT | Not supported | Natively supported |
| Connection Migration | Not supported | Natively supported |

---

## 2. System Architecture, Module Tree

详见 [architecture.md](./architecture.md) §1–§9 (crate topology, deployment topology,
server/client runtime state, data-plane flows, ingress plugin pipeline, control
plane, module maps) and [duotunnel-lib.md](./duotunnel-lib.md) (shared-library
module layout under `duotunnel-lib/src/`).

---

## 3. Message Protocol / Wire Format

详见 [duotunnel-lib.md](./duotunnel-lib.md) §"Wire Protocol and Negotiation" for
`TUNNEL_ALPN`, `PROTOCOL_VERSION`, capability negotiation, and the evolution
discipline for `models/msg.rs`. Frame layout and message type definitions live
in `duotunnel-lib/src/models/msg.rs` — read that file directly for the current
struct fields rather than a doc snapshot, since it is append-only and changes
over time.

---

## 4–6. Connection Flow, Forward Proxy (Ingress), Reverse Proxy (Egress)

详见 [architecture.md](./architecture.md) §5 "Data-Plane Flows" (forward proxy /
ingress call chain, reverse proxy / egress call chain, stream lifecycle on
QUIC) and §4 "Client Runtime State" for the client reconnection mechanism
(`JitterBackoff`, `ConnectError::Fatal` vs `Transient`, `reconnect.grace_ms`).
See also [client-runtime.md](./client-runtime.md).

---

## 7. Core Components

详见 [architecture.md](./architecture.md) §3 "Server Runtime State" and §8
"Shared Primitives" for `ClientRegistry`, `VhostRouter`, `UpstreamResolver`,
and `PeerSpec`. Read the current source directly for exact fields and methods
(they have changed shape since the last doc snapshot — for example
`ClientRegistry` is now a sharded actor behind an `mpsc` channel backed by
`InflightTable`, not a plain `DashMap`; `VhostRouter` fields are `exact`,
`wildcards`, `has_wildcards`):

- `duotunnel-server/ingress/registry.rs` — `ClientRegistry`
- `duotunnel-lib/src/transport/listener.rs` — `VhostRouter`
- `duotunnel-lib/src/proxy/core.rs` — `UpstreamResolver`, `ProxyEngine`
- `duotunnel-lib/src/proxy/peers.rs` — `PeerSpec`

---

## 8. Concurrency Control

详见 [parameters.md](./parameters.md) §2.5 "过载保护 (Overload Protection)" and
[architecture.md](./architecture.md) §8 "Shared Primitives" (`lb/overload.rs`,
`lb/inflight.rs`) for the current three-layer overload model (inflight
slow-path, per-connection pending-queue cap, QUIC transport limits).

---

## 9. Configuration Format

详见 the root [`README.md`](../../README.md) "Configuration" section for live
`ctld.yaml` / `server.yaml` / `routing.yaml` / `client.yaml` examples, and
[parameters.md](./parameters.md) for the full tunable/default table. Routing
base-layer semantics (YAML + SQLite merge, egress vhost allowlist matching)
are documented in [architecture.md](./architecture.md) §7 "Control Plane".

---

## 10. Monitoring Metrics

The Prometheus metric names below were verified against
`duotunnel-server/runtime/metrics.rs` (and related `metrics.rs` files) on
2026-09-23. The previous doc snapshot listed several metric names that no
longer exist in code — those are dropped or renamed here; see the note at the
end of this section.

```
# Connection metrics
duotunnel_total_quic_connections            # QUIC connections opened (counter)
duotunnel_active_quic_connections           # QUIC connections currently open (gauge)
duotunnel_total_tcp_connections             # TCP connections opened (counter)
duotunnel_active_tcp_connections            # TCP connections currently open (gauge)
duotunnel_unauthenticated_connections_refused_total  # Pre-auth budget exhausted
duotunnel_connection_rejected_not_ready_total{protocol}  # Rejected: server not ready
duotunnel_reverse_stream_rejected_total{reason}          # Rejected reverse (egress) streams

# Client registry metrics
duotunnel_clients_active                    # Currently registered clients (gauge)
duotunnel_registry_connections_capacity     # Registry slot table capacity
duotunnel_registry_connections_active       # Registry slots in use
duotunnel_registry_connections_available    # Registry slots free
duotunnel_registry_connections_high_water   # High-water mark of registry usage
duotunnel_registry_connections_exhausted    # 1 if registry is at capacity
duotunnel_registry_capacity_exhaustions_total  # Times registration was rejected at capacity

# Authentication metrics
duotunnel_auth_success_total                # Authentication successes
duotunnel_auth_failure_total                # Authentication failures

# Request metrics
duotunnel_requests_total{protocol,status}   # Total requests (protocol: tcp/http/...; status: success/error/...)

# QUIC stream admission metrics
duotunnel_open_bi_total                     # open_bi() attempts
duotunnel_open_bi_inflight                  # open_bi() calls currently in flight
duotunnel_open_bi_wait_ms                   # open_bi() pending-queue wait time (histogram)
duotunnel_open_bi_timed_out_total           # open_bi() timeouts
duotunnel_open_bi_rejected_overloaded_total # open_bi() rejected: pending-queue cap hit

# Resource gauges
duotunnel_reverse_streams_active            # Active reverse tunnel streams
duotunnel_http_requests_active              # Active HTTP requests
duotunnel_udp_tasks_active                  # Active UDP dispatch/reply tasks
duotunnel_udp_datagram_dropped_total{reason}  # Dropped UDP datagrams
duotunnel_slowpath_waiting_tasks            # Tasks waiting in overload slow-path backoff

# Control-plane (ctld watch client) metrics
duotunnel_control_lkg_persist_failures_total  # LKG (last-known-good) config persist failures
duotunnel_control_lkg_durability_degraded     # 1 if LKG durability is degraded

# Proxy error / ingress timing metrics
duotunnel_proxy_errors_total                # Structured ProxyError observations
duotunnel_ingress_total_ms                  # End-to-end ingress request time
```

**Dropped or renamed since the previous doc snapshot** (no longer accurate — do
not use these names):

| Old name | Status |
|---|---|
| `duotunnel_quic_connections_total` | Renamed to `duotunnel_total_quic_connections` |
| `duotunnel_tcp_connections_total` | Renamed to `duotunnel_total_tcp_connections` |
| `duotunnel_connections_rejected_total` | Dropped; replaced by the more specific `duotunnel_connection_rejected_not_ready_total{protocol}`, `duotunnel_unauthenticated_connections_refused_total`, and `duotunnel_reverse_stream_rejected_total{reason}` |
| `duotunnel_clients_registered_total` / `duotunnel_clients_unregistered_total` | Dropped; collapsed into the single gauge `duotunnel_clients_active` |
| `duotunnel_duplicate_clients_total` | Dropped; no such counter exists in code |
| `duotunnel_auth_success_total{group}` / `duotunnel_auth_failure_total{group}` | The `group` label does not exist — `auth_success`/`auth_failure` take a `group_id` parameter but do not attach it as a metric label |
| `duotunnel_requests_total{type,status}` | Label is `protocol`, not `type` |

This is not an exhaustive metrics catalog (e.g. `metrics_exporter_prometheus`
also emits process-level metrics, and `duotunnel-client/metrics.rs` has its own
surface). For the authoritative, current list, read `duotunnel-server/runtime/metrics.rs`,
`duotunnel-server/ingress/handlers/metrics.rs`, `duotunnel-lib/src/plugin/metrics.rs`,
`duotunnel-lib/src/infra/metrics.rs`, and `duotunnel-client/metrics.rs` directly.

---

## 11. Security Summary

- **QUIC TLS 1.3**: all tunnel traffic is encrypted end to end.
- **ALPN**: `TUNNEL_ALPN` = `b"tunnel-quic/v1"` (`duotunnel-lib/src/transport/quic.rs:17`),
  generation-scoped — a breaking wire-layout change bumps the ALPN generation
  suffix so incompatible peers fail at the QUIC/TLS handshake rather than
  after connecting. Handshake additionally negotiates `protocol_version` and
  capability bits. Full detail: [duotunnel-lib.md](./duotunnel-lib.md) §"Wire
  Protocol and Negotiation".
- **Authentication**: unified control-plane deployment only — `LocalTokenCache`
  synced from `duotunnel-ctld` validates `Login.token` at the QUIC handshake;
  `revocation_tx` broadcasts forced disconnects. There is no standalone/local
  static-token authority mode: the static YAML `auth_tokens: {group: hash}`
  map shown in older doc snapshots no longer exists in code (verified: no
  `auth_tokens` field in any `*.rs` file). Tokens are created/rotated/revoked
  through `duotunnel-ctld client ...` — see the root `README.md` "Token
  management" section. Detail: [architecture.md](./architecture.md) §7 "Auth".
- **Duplicate ClientID handling**: `duotunnel-server/ingress/registry.rs`
  registration replaces the existing entry for a `client_id` and retires the
  superseded connection handle (see `RegistryMsg::Register` handling around
  `registry.rs:339-365`); it is not a bare `DashMap` swap as older snapshots
  showed — registration is serialized through the registry's actor (`mpsc`)
  loop.
- **Certificate verification**: supports custom CA or system trust store
  (client `tls_skip_verify` / CA config); server-side TLS termination (MITM)
  uses dynamically generated, cached certificates — see
  `duotunnel-lib/src/infra/pki.rs`.

---

## 12. Design Principles

### 12.1 Principles Followed

| Principle | Implementation |
|-----------|----------------|
| **Single Responsibility** | Clear module separation: transport/protocol/proxy/engine |
| **Open/Closed** | Trait extension: `UpstreamResolver`, `IngressProtocolHandler`, plugin traits |
| **Dependency Inversion** | Depend on abstractions, not concrete implementations |
| **Zero-Copy** | Pooled `BytesMut` relay buffers (`engine/copy.rs`), `read_buf`-based fills |
| **Lock-Free Concurrency** | `DashMap` / `ArcSwap` instead of `RwLock<HashMap>` on hot paths |

### 12.2 Performance Characteristics

- **On-Demand Stream Creation**: No connection pool overhead
- **Protocol Detection**: `peek()` avoids data copying
- **Connection Reuse**: Single QUIC connection multiplexing (client pools multiple connections; server multiplexes per client)
- **Lazy Certificate Loading**: MITM certificates generated on demand and cached

---

## 13. Future Optimizations

详见 [../todo.md](../todo.md) for the current, actively-maintained backlog
(roadmap phases, `TODO-*` IDs, and the S1–S9 execution order). Do not treat
this document as a backlog — it is not kept in sync with `todo.md`.

Of the items previously listed here as "Future Optimizations", verification
against the current codebase (2026-09-23) shows all but one are already done
or superseded, and are intentionally **not** carried forward as open items:

- Extract MITM implementation to separate module — done (`duotunnel-server/ingress/plugins/tls/`)
- Unify protocol detection logic — done (`duotunnel-lib/src/protocol/detect.rs`)
- Abstract LoadBalancer trait — done (`duotunnel-lib/src/plugin/egress.rs::LoadBalancer`)
- Add health check mechanism — done (`duotunnel-lib/src/proxy/upstream.rs::UpstreamHealthRegistry`)
- Custom error types — done (`duotunnel-lib/src/error.rs::ProxyError`/`ErrorKind`/`RetryType`)
- Remove unused `ProtocolDriver` trait — stale: the trait is in active use (`Http1Driver` implements it, consumed by `duotunnel-lib/src/proxy/http.rs`)
- Add performance benchmarks — done/ongoing (see the benchmark-gated `TODO-1xx` items in `todo.md` and `ci-helpers/`)
- Hot configuration reload — done (`duotunnel-ctld` watch → `ServerState::replace_routing()`, see [architecture.md](./architecture.md) §7)

**One gap remains untracked**: a per-backend **circuit breaker** was discussed
in [design/02-lb-quality.md](../design/02-lb-quality.md) and
[reviews/2026-07-26/09-lb-grade-capability-gap.md](../reviews/2026-07-26/09-lb-grade-capability-gap.md)
(as `OutlierDetector`) but has no corresponding entry in `todo.md` — it is
folded into D2's larger "统一 LB + 健康/outlier + 重试预算" design, which is
listed in `todo.md`'s roadmap only at the design-doc level, not as its own
`TODO-*` ID. Anyone picking this up should check `todo.md` first in case it
has since been assigned one.

---

## References

- [frp GitHub](https://github.com/fatedier/frp)
- [Quinn QUIC Library](https://github.com/quinn-rs/quinn)
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [TLS SNI RFC 6066](https://datatracker.ietf.org/doc/html/rfc6066#section-3)
