# System Architecture

Cross-cutting architecture reference: crate boundaries, the unified deployment topology, data-plane call paths, control plane, and key design decisions.

For parameter defaults see [parameters.md](./parameters.md). For per-crate runtime layering see the `*-runtime.md` specs. For coding rules see [architecture-guidelines.md](./architecture-guidelines.md). For product goals and historical context see [DESIGN.md](./DESIGN.md).

---

## 1. Crate Topology

```
                    ┌─────────────────┐
                    │  duotunnel-ctld │  (ctld — control plane only)
                    └────────┬────────┘
                             │ watch TCP + SQLite
              ┌──────────────┬──────────────┐
              ▼              ▼
        ┌────────────────┐  ┌─────────────────┐
        │ duotunnel-server│  │ duotunnel-client│
        └────────────────┘  └─────────────────┘
              │              │
              └──────┬───────┘
                     ▼
              ┌──────────────┐
              │duotunnel-lib│  (shared protocol, transport, proxy, plugin core)
              └──────────────┘
```

| Crate | Role | Public entry |
| :--- | :--- | :--- |
| `duotunnel-lib` | Shared types, wire codec, QUIC/TCP transport, `ProxyEngine`, plugin traits, overload | library crate |
| `duotunnel-ctld` | Control service, watch protocol, admin API, internal SQLite storage | `duotunnel_ctld::run()` |
| `duotunnel-server` | QUIC tunnel listener, ingress listeners, client registry, server egress | `duotunnel_server::run()` |
| `duotunnel-client` | QUIC tunnel client, optional local entry, upstream proxy to private services | `duotunnel_client::run()` |
| `duotunnel-ctld` | ctld: routing DB, token admin, watch server | `duotunnel_ctld::run()` |

Dependency rule: server and client depend on `duotunnel-lib`; only ctld owns SQLite and storage adapters. `duotunnel-lib` does not depend on binaries.

---

## 2. Deployment Topology

All deployments use one topology: `duotunnel-ctld` publishes the effective
configuration, `duotunnel-server` watches it, and `duotunnel-client` connects
to the server over QUIC. The server and client do not read routing YAML or
SQLite directly.

### duotunnel-server

- Reads only server runtime tuning from its local `server.yaml`.
- Receives routing and token state through the ctld watch stream.
- Applies each Snapshot or Delta atomically to its runtime generation.

### duotunnel-client

- Always outbound-connects to `server_addr:server_port` over QUIC (ALPN `tunnel-quic/v1` — generation-scoped, so an incompatible peer fails at the QUIC handshake rather than at login).
- Upstream map and egress allowlist come from `LoginResp.config` at login — not from local YAML.
- Optional `entry.port` exposes a local forward-proxy entry; optional `udp_entries[]` for UDP.

### ctld (`duotunnel-ctld`)

- Owns canonical routing + token DB.
- Exposes watch TCP (`watch_addr`, default `127.0.0.1:7788`).
- On connect: `WatchRequest` → `Snapshot` → `Delta` events with ACK and resync handling.

---

## 3. Server Runtime State

`ServerState` (`duotunnel-server/bootstrap/mod.rs`) is the capability surface for all request-time code. Internal subdivisions stay private:

| Sub-runtime | Owns | Key types |
| :--- | :--- | :--- |
| **IngressRuntime** | listeners, routing snapshot, plugin registry, overload limits, peek pool | `ArcSwap<RoutingSnapshot>`, `PluginRegistry`, `ListenerManager` |
| **ConnectionRuntime** | registered QUIC clients | `ClientRegistry` (sharded actor) |
| **ControlRuntime** | ctld watch client, token cache, revision and revocation state | `ControlClientService`, `TokenMap` |

`RoutingSnapshot` is immutable per version: HTTP vhost routers per listener port, `TunnelManagement`, `ServerEgressMap`, egress vhost allowlist. Hot reload **replaces the whole snapshot** via `ArcSwap::store`.

Client selection: `ClientRegistry::select_healthy(preferred_shard)` rotates the shard start, selects the lowest-inflight candidate from every shard, and uses the rotated order to break ties (`duotunnel-server/ingress/registry.rs`). This preserves sharded mutation snapshots while preventing a fixed preferred shard from monopolizing a group.

---

## 4. Client Runtime State

| Component | Pattern | Role |
| :--- | :--- | :--- |
| `RuntimeEngine` | service orchestrator | runs `ClientService` tasks until shutdown |
| `EntryConnPool` | actor (mpsc) + `ArcSwap` snapshots | N QUIC connections, sharded; hot-path reads lock-free |
| `TunnelPoolService` | `ClientService` | spawns `run_supervisor` × `resolved_connections` |
| `EgressListenerService` | `ClientService` | optional TCP entry (`entry.port`) |
| `UdpEgressListenerService` | `ClientService` | optional UDP entry per `udp_entries[]` |

QUIC ownership topology (logged at startup): `connections` (auto via `effective_runtime_parallelism`), `shards` (capped by connections on client), `accept_workers` for entry listener.

---

## 5. Data-Plane Flows

### 5.1 Forward proxy (Ingress)

External client → private upstream through the tunnel.

```
External TCP
  → duotunnel-server/ingress/handlers/{http,tcp}.rs  (SO_REUSEPORT accept workers)
  → IngressDispatcher::dispatch            (duotunnel-lib/plugin/dispatcher.rs)
      Phase 1: sniff → ProtocolHint
      Phase 2: ConnectionModule::pre_admission
      Phase 3: TunnelService::admission
      Phase 4: RouteResolver (VhostPlugin) → Route{group_id, proxy_name}
      Phase 5: IngressProtocolHandler (h1/h2c/tls/tcp_pass)
      Phase 6: logging
  → ClientRegistry::select_healthy → ConnectionHandle
  → open_bi_guarded + send RoutingInfo     (duotunnel-lib/transport/open_bi.rs)
  → QUIC bidi stream
  → duotunnel-client: conn.accept_bi() loop (duotunnel-client/tunnel/client.rs)
  → handle_work_stream                     (duotunnel-client/ingress/handler.rs)
      recv_routing_info → ProxyEngine
  → IngressClientApp / LocalProxyMap       (duotunnel-client/ingress/app.rs)
      LB pick upstream → TCP/HTTP connect to local service
```

L7 handlers (H1, H2c, TLS-terminated H2) build `RoutingInfo` per request and call `forward_h2_request` or equivalent with `OpenStreamRequest` (overload limits + timeout). H2c skips Phase-4 route at connection level — resolves per-request `:authority`.

TCP passthrough sends `RoutingInfo` once then relays bytes.

### 5.2 Reverse proxy (Client entry / Egress)

Local app → internet via server egress.

```
Local TCP → duotunnel-client/egress/listener.rs
  → sniff (client detectors) + egress allowlist check (LoginResp.egress_rules)
  → EntryConnPool::next_conn_for_shard_excluding (P2C)
  → maybe_slow_path → ConnectionHandle::open_stream
  → send RoutingInfo { proxy_name: "entry", host, protocol, src_* }
  → QUIC → duotunnel-server/ingress/tunnel_handler.rs (on accept_bi from client-initiated stream)
  → recv_routing_info → ProxyEngine(ServerEgressMap)
  → vhost rule lookup → external upstream (HTTP connector or TCP relay)
```

Overload at entry: `QuicOpenRejectedOverloaded` → H1 returns `503` + `Retry-After: 1`; other protocols close cleanly.

### 5.3 Stream lifecycle on QUIC

Every forwarded flow uses a **bidi QUIC stream**:

1. Caller acquires a stream via `ConnectionHandle::open_stream` on both sides; it takes the per-connection concurrency permit and delegates to `open_bi_guarded`, which applies the per-connection pending-stream gate.
2. First frame on stream: `MessageType::RoutingInfo` (rkyv payload).
3. `ProxyEngine::run_stream` resolves `PeerSpec` and bridges QUIC ↔ upstream.
4. `InflightGuard` on stream drop decrements per-connection inflight counters.

Login uses a separate one-shot bidi stream: `Login` → `LoginResp`, then connection enters the accept loop.

---

## 6. Ingress Plugin Pipeline

Registered at bootstrap (`duotunnel-server/bootstrap/mod.rs`):

| Handler | `ProtocolKind` | Notes |
| :--- | :--- | :--- |
| `plugins::tls::TlsHandler` | `Tls` | MITM terminate → serve H2, per-request forward |
| `plugins::h2c::H2cHandler` | `H2c` | cleartext H2; per-request vhost resolve; `h2_single_authority` pin |
| `plugins::h1::H1Handler` | `Http` | HTTP/1.1 forward |
| `plugins::tcp_pass::TcpPassHandler` | `Tcp` | opaque TCP relay |
| `plugins::vhost::VhostPlugin` | — | `RouteResolver` over `RoutingSnapshot` |
| `plugins::prometheus::PrometheusSink` | — | `MetricsSink` |

`IngressDispatcher` is the single front door for HTTP/TCP ingress listeners. Handlers must not re-implement sniff or routing.

---

## 7. Control Plane

### Config distribution path

```
duotunnel-ctld ControlService (YAML + SQLite merge)
  → WatchServer: Snapshot / Delta
  → duotunnel-server ControlClient
  → build_routing_snapshot()
  → ServerState::replace_routing()
  → listener_mgr::sync_listeners() reconciles TCP listeners
```

Client config is **not** pushed over watch; it is embedded in `LoginResp.config` (`ClientConfig { upstreams, egress_rules, config_version }`) built from `tunnel_management.client_configs.groups` + server egress allowlist.

### Auth

- Unified control-plane deployment: `LocalTokenCache` synced from ctld validates
  `Login.token` at the QUIC handshake; `revocation_tx` broadcasts forced
  disconnects. The server has no standalone/local SQLite authority mode.

---

## 8. Shared Primitives (`duotunnel-lib`)

| Area | Module | Responsibility |
| :--- | :--- | :--- |
| Wire codec | `models/msg.rs` | `MessageType`, rkyv frame, `send_message` / `recv_*` |
| Transport | `transport/` | QUIC params, `open_bi_guarded`, `build_reuseport_listener`, `TcpParams` |
| Proxy core | `proxy/core.rs` | `ProxyEngine`, `UpstreamResolver`, `Protocol` |
| Overload | `lb/overload.rs`, `lb/inflight.rs` | `maybe_slow_path`, `InflightTable`, pending queue gate |
| Plugins | `plugin/` | `IngressDispatcher`, traits, `PluginRegistry` |
| Runtime | `infra/runtime.rs` | `effective_runtime_parallelism`, cgroup CPU cap |

`ProxyEngine` is reused on **both** sides: client `IngressClientApp` resolves local upstreams; server `ServerEgressMap` resolves external upstreams.

---

## 9. Module Maps

### `duotunnel-server/`

```
bootstrap/     composition root, ServerState, RoutingSnapshot, ConfigSource
runtime/       ServerApp, supervisor, metrics, shutdown drain
ingress/
  handlers/    quic (tunnel), http, tcp accept loops
  plugins/     protocol handlers (h1, h2c, tls, tcp_pass, vhost, prometheus)
  registry.rs  ClientRegistry actor
  listener_mgr.rs  ingress listener reconcile/drain
  tunnel_handler.rs  client-initiated egress streams
egress/        ServerEgressMap (UpstreamResolver for external targets)
control/       control_client, local_auth, null_stores
```

### `duotunnel-client/`

```
bootstrap/     ClientConfigFile load/validate
runtime/       ClientApp, RuntimeEngine, observability
tunnel/        endpoint, client (login loop), conn_pool, supervisor, pool
ingress/       LocalProxyMap, handle_work_stream (forward from server)
egress/        entry TCP listener, UDP listener
plugins/       CachedResolver, RoundRobinLb
```

### `duotunnel-ctld/`

```
bootstrap/     ctld config
runtime/       CtldApp startup
control/       ControlService, WatchServer, proto (snapshot/delta), reactor (debounce)
```

---

## 10. Key Design Decisions

| Decision | Rationale |
| :--- | :--- |
| Server calls `open_bi` on registered client connections | Avoids extra control round-trips; stream creation is 0-RTT relative to login |
| `RoutingSnapshot` + `ArcSwap` | Lock-free readers on hot path; atomic routing updates |
| Registry / conn pool as actors | Serialize mutation; snapshots for reads; prevents DashMap churn on forward path |
| `open_bi_guarded` pending cap | Fail fast under per-connection queue pressure instead of timing out every caller (a process-wide budget is still open — TODO-142) |
| Ingress 6-phase dispatcher | Separates sniff, admission, routing, protocol handling; plugins swap without touching accept loop |
| `LoginResp` carries `ClientConfig` | Client upstream map authoritative from server; YAML only has connection tuning |
| Egress allowlist on client | Fast local reject (`502`/`EOF`) before opening QUIC stream for disallowed hosts |
| Layered config sources | YAML supplies defaults; SQLite overrides are merged into the effective snapshot |
| H2c per-request routing | One cleartext H2 connection may carry multiple authorities when `h2_single_authority: false` |
| HttpConnector H2c→H1 pin (300s) | Cleartext H2 probing without per-upstream YAML; degrades gracefully |

---

## 11. Spec Index

| Document | Covers |
| :--- | :--- |
| [architecture.md](./architecture.md) | This file — topology, flows, modules, design |
| [parameters.md](./parameters.md) | Tunables, defaults, overload thresholds |
| [architecture-guidelines.md](./architecture-guidelines.md) | How to structure/refactor code |
| [client-runtime.md](./client-runtime.md) | Client startup layers and services |
| [server-runtime.md](./server-runtime.md) | Server startup layers, supervisor, shutdown |
| [duotunnel-ctld-runtime.md](./duotunnel-ctld-runtime.md) | ctld startup and control modules |
| [duotunnel-lib.md](./duotunnel-lib.md) | Shared library module layout |
| [duotunnel-ctld-storage.md](./duotunnel-ctld-storage.md) | Persistence layer and feature flags |
| [DESIGN.md](./DESIGN.md) | Product goals, wire format detail, config examples |
