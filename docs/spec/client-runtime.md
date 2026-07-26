# Client Runtime Spec

## Scope

This spec defines the `client` crate runtime shape after the startup and boundary refactor.

## Public Entry

`client::run()` is the only external entry for the crate runtime.

Responsibilities at the outer boundary:

- parse CLI
- construct the client app
- enter the runtime

The binary target should not orchestrate startup directly.

## Runtime Layers

The `client` crate is organized into four runtime layers.

### 1. CLI

`client/bootstrap/cli.rs`

Owns:

- command parsing
- config path input

Must not own:

- runtime wiring
- reconnect orchestration
- local proxy handling

### 2. Bootstrap

`client/bootstrap/mod.rs`
`client/bootstrap/config.rs`

Owns:

- loading client config
- validating client config
- building bootstrap state

Must not own:

- task spawning
- long-running service loops
- QUIC connect/login execution

Bootstrap is the composition root for client runtime dependencies.

### 3. App / Runtime

`client/runtime/app.rs`
`client/runtime/mod.rs`
`client/runtime/engine.rs`

Owns:

- startup flow
- observability init
- shutdown token creation
- healthz service startup
- service engine startup
- runtime task spawning

Must not own:

- raw config parsing
- TLS verifier construction
- protocol-specific local service handling

`ClientApp` is the application-level orchestrator.

Startup resolves QUIC ownership topology before any service starts:

- `resolve_connection_count(quic.connections)` — `0` means auto from `effective_runtime_parallelism()`
- `resolve_shard_count(quic.shards, Some(connections))` — shard count capped by connection count on client
- `resolve_accept_workers(entry.accept_workers)` — entry accept parallelism

These values are logged once at startup (`client QUIC ownership topology resolved`).

### Service Engine

`client/runtime/engine.rs` defines `RuntimeEngine` and the `ClientService` trait.

`ClientApp` registers process-lifetime services and runs them concurrently until shutdown:

| Service | Module | When registered |
|---|---|---|
| `EgressListenerService` | `client/egress/listener.rs` | `entry.port` is set |
| `UdpEgressListenerService` | `client/egress/udp_listener.rs` | one per `udp_entries[]` item |
| `TunnelPoolService` | `client/tunnel/mod.rs` | always |

Rules:

- services implement `ClientService::start(shutdown)` — no top-level anonymous long-running spawns
- first service failure cancels the shared `CancellationToken` and drains siblings
- `metrics_port` starts an embedded `/healthz` + `/metrics` HTTP responder (not a separate crate)

### 4. Tunnel / Services

`client/tunnel/endpoint.rs`
`client/tunnel/client.rs`
`client/tunnel/pool.rs`
`client/tunnel/supervisor.rs`
`client/tunnel/conn_pool.rs`
`client/egress/listener.rs`
`client/egress/udp_listener.rs`

Owns:

- QUIC endpoint and TLS config construction (`endpoint.rs` — startup-time)
- server connect/login flow and session loop (`client.rs` — request-time)
- reconnect supervision (`supervisor.rs` — one task per resolved connection)
- sharded connection pool lifecycle (`conn_pool.rs` — actor via mpsc, read via snapshot)
- reverse TCP entry listener lifecycle (`egress/listener.rs`)
- reverse UDP entry listener lifecycle (`egress/udp_listener.rs`)

`EntryConnPool` is the client-side mirror of server `ClientRegistry`: pool mutations go through an actor task; hot-path reads use `Arc` snapshots per shard.

Boundary rule: `endpoint.rs` is startup-time only; `client.rs` is request-time only. These modules are for process-lifetime tunnel work, not CLI/bootstrap concerns.

### Session Establishment

`establish_session` (`client/tunnel/client.rs:129`) owns connect plus the whole login handshake and returns `(Connection, LoginResp, NegotiatedProtocol)`.

`run_client` races that phase against shutdown with `tokio::select!` (`client/tunnel/client.rs:38`). Nothing is in flight before the connection joins the pool, so dropping the future loses nothing that needs draining — and the race is necessary, because connect walks every resolved address and login spans several timeouts, which a sequential await would add to the stop latency. Everything after `entry_pool.push` (`client/tunnel/client.rs:58`) is awaited sequentially instead, so the drain cannot be interrupted.

Retry classification reads the wire flag, never the error text: `classify_login_failure(resp.retryable, resp.error.as_deref())` (`client/tunnel/supervisor.rs:165`, called at `client/tunnel/client.rs:170`) maps `retryable` to `ConnectError::transient` and everything else to `ConnectError::fatal`. Server error strings for unauthenticated peers are deliberately generic and must not be pattern-matched, or a transient backing-store fault would be read as a rejected token and stop the client for good.

Rules:

- `run_supervisor` does not race `run_client` against the cancel token; `run_client` observes it itself so it can drain in-flight streams and close gracefully (`client/tunnel/supervisor.rs:77`-`80`)
- only `FailureClass::Fatal` ends the supervisor loop; transient failures go through `JitterBackoff`

### Protocol Negotiation

The client advertises `PROTOCOL_VERSION` and `SUPPORTED_CAPABILITIES` in `Login` (`client/tunnel/client.rs:140`) and validates the server's answer:

- a `negotiated_version` outside `MIN_SUPPORTED_VERSION..=PROTOCOL_VERSION` is fatal, not retryable: the server negotiates `min(its max, ours)`, so anything else means a broken or hostile peer (`client/tunnel/client.rs:178`)
- capabilities are re-masked with `SUPPORTED_CAPABILITIES` so a capability this build never advertised cannot be enabled by an echo (`client/tunnel/client.rs:192`)

The resulting `NegotiatedProtocol` is stored on the pool entry (`PooledConnection`, `client/tunnel/conn_pool.rs:13`, set via `EntryConnPool::push`, `client/tunnel/conn_pool.rs:181`) for future capability gating at the connection-selection site.

ALPN pins the wire generation: `TUNNEL_ALPN` is `tunnel-quic/v1` (`tunnel-lib/src/transport/quic.rs:17`), so an incompatible peer fails in the TLS handshake rather than at login.

### Stream Admission

Reverse streams are opened through `ConnectionHandle::open_stream` (`tunnel-lib/src/transport/connection_handle.rs:75`). Both the concurrency limit and the pending limit are per-connection semaphores on the handle (`tunnel-lib/src/transport/connection_handle.rs:39`-`42`), sized from `quic.max_concurrent_streams` and the resolved `OverloadLimits::max_pending_streams` (`tunnel-lib/src/lb/overload.rs:25`) when the pool is built (`client/runtime/app.rs:77`-`82`).

Waiting streams are admitted against that per-connection `pending_semaphore` (`tunnel-lib/src/transport/open_bi.rs:90`) rather than a global check-then-act counter; the process-wide `stream_pending_queue_depth` gauge is a pure metric that gates nothing (`tunnel-lib/src/transport/open_bi.rs:88`).

## Shutdown

On SIGINT/SIGTERM, `ClientApp` cancels the shared token (`client/runtime/app.rs:41`-`45`). The sequence mirrors the server's — stop accepting, drain, then close:

1. entry listeners stop accepting: every `ClientService` gets the same token, so the TCP entry accept loop and the UDP listeners return on cancellation
2. `run_client` breaks out of the stream-accept loop, removes the connection from `EntryConnPool` (`client/tunnel/client.rs:108`) so no new stream can be routed onto it, and clears `ready`
3. in-flight local relays drain for up to the connection-level `SHUTDOWN_DRAIN_TIMEOUT` (15s, `client/tunnel/client.rs:24`), and only then `conn.close` (`client/tunnel/client.rs:114`-`116`) — `CONNECTION_CLOSE` aborts every open stream, so the close must follow the drain
4. `run_supervisor` returns instead of reconnecting once the token is cancelled (`client/tunnel/supervisor.rs:90`)
5. `ClientApp` backstops with `wait_for_resource_drain` for the app-level `SHUTDOWN_DRAIN_TIMEOUT` (30s, `client/runtime/app.rs:16`), then logs `active_connections` / `pending_streams`

The two constants share a name but not a scope: the connection-level window is deliberately shorter than the app-level backstop so the outer wait still fires after connections have closed. The server nests its windows the same way.

## Request Path

Request handling belongs below the startup layers.

Main ingress/runtime modules:

- `client/ingress/app.rs`
- `client/ingress/handler.rs`
- `client/egress/listener.rs`
- `client/plugins/*`

These modules consume runtime capabilities from config and tunnel services and should not perform top-level runtime assembly.

## Call Relationships

See [architecture.md](./architecture.md) for full diagrams. Client-side summary:

| Direction | Entry | Core path |
| :--- | :--- | :--- |
| **Forward** (server → local) | `tunnel/client.rs` `accept_bi` | `ingress/handler.rs` → `ProxyEngine` → `LocalProxyMap` → upstream TCP/HTTP |
| **Reverse** (local → internet) | `egress/listener.rs` | sniff → `EntryConnPool` pick → `ConnectionHandle::open_stream` → `RoutingInfo` → server `tunnel_handler` → `ServerEgressMap` |
| **UDP reverse** | `egress/udp_listener.rs` | QUIC datagram ↔ local UDP socket |

Login path: `establish_session` → `connect_to_server` → `open_bi` → `Login`/`LoginResp` → `NegotiatedProtocol` + `LocalProxyMap` built from `resp.config`, then `EntryConnPool::push`.

## Invariants

- there is one public runtime entry
- startup assembly is centralized
- reconnect and pool lifecycle stay in `client/tunnel`
- listener accept loops stay in `client/egress`
- local service handling stays in `client/ingress`
- long-running tasks are owned by `ClientService` implementations, not free-floating spawns
- runtime internals are private by default

## Non-Goals

This spec does not define:

- wire protocol details
- metrics schema
- plugin contracts inside `tunnel-lib`
