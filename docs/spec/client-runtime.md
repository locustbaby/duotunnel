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

## Shutdown

Both client and server use a 30-second resource drain window (`SHUTDOWN_DRAIN_TIMEOUT`) before exit.

On SIGINT/SIGTERM, `ClientApp` cancels the shutdown token, waits for `wait_for_resource_drain`, then logs `active_connections` / `pending_streams`.

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
| **Reverse** (local → internet) | `egress/listener.rs` | sniff → `EntryConnPool::open_stream` → `RoutingInfo` → server `tunnel_handler` → `ServerEgressMap` |
| **UDP reverse** | `egress/udp_listener.rs` | QUIC datagram ↔ local UDP socket |

Login path: `connect_to_server` → `open_bi` → `Login`/`LoginResp` → build `LocalProxyMap` from `resp.config`.

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
