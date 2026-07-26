# Server Runtime Spec

## Scope

This spec defines the `server` crate runtime shape after the startup and boundary refactor.

## Public Entry

`server::run()` is the only external entry for the crate runtime.

Responsibilities at the outer boundary:

- parse CLI
- construct the server app
- enter the runtime

The binary target should not orchestrate startup directly.

## Runtime Layers

The `server` crate is organized into four runtime layers.

### 1. CLI

`server/bootstrap/cli.rs`

Owns:

- command parsing
- config path input
- ctld mode input
- token subcommands

Must not own:

- runtime wiring
- background thread startup
- routing assembly

### 2. Bootstrap

`server/bootstrap/mod.rs`

Owns:

- loading server config
- resolving standalone vs managed mode
- building stores and config sources
- building routing snapshots
- building runtime state

Must not own:

- thread spawning
- long-running service loops
- ingress request handling

Bootstrap is the composition root for server runtime dependencies.

### 3. App / Runtime

`server/runtime/app.rs`
`server/runtime/mod.rs`

Owns:

- startup flow
- observability init
- prometheus recorder install
- shutdown token creation
- supervisor startup
- proxy main loop handoff

Must not own:

- low-level config parsing
- routing mutation details
- listener reconciliation internals

`ServerApp` is the application-level orchestrator.

## 4. Supervisor / Components

`server/runtime/supervisor.rs`

Owns:

- long-running component startup
- component shutdown coordination
- component state tracking
- background component hosting
- metrics component hosting
- named background tasks (e.g. `purge_loop`) with lifecycle ownership

Components are for process-lifetime tasks, not request handlers.

Background tasks spawned inside components must be named functions, not anonymous closures, to satisfy lifecycle ownership (Guideline §7).

## Request Path

Request handling belongs below the startup layers.

Main ingress/runtime modules:

- `server/ingress/handlers/*`
- `server/ingress/listener_mgr.rs`
- `server/control/control_client.rs`
- `server/control/hot_reload.rs`
- `server/ingress/plugins/*`

These modules consume runtime capabilities from `ServerState` and should not perform top-level runtime assembly.

## Call Relationships

See [architecture.md](./architecture.md) for full diagrams. Server-side summary:

| Direction | Entry | Core path |
| :--- | :--- | :--- |
| **Forward** (external → client) | `handlers/{http,tcp}.rs` accept | `IngressDispatcher` → plugin handler → `ClientRegistry` → `ConnectionHandle::open_stream` → `open_bi_guarded` → QUIC |
| **Client egress** (client → internet) | QUIC stream from client | `tunnel_handler.rs` → `ProxyEngine(ServerEgressMap)` → external upstream |
| **Tunnel control** | `handlers/quic.rs` | auth → register in `ClientRegistry` → `accept_bi` loop for server-initiated streams |

Routing reads always go through `routing_snapshot()` guard; mutations via `replace_routing()` only from control/hot-reload paths.

## Runtime State

`ServerState` is the runtime capability surface for request-time and background-time code.

Internal subdivisions:

- ingress runtime
- connection runtime
- control runtime

Rules:

- sub-runtime structs stay private
- callers should prefer capability methods over field access
- routing swaps happen through runtime methods
- listener management is reached through `ServerState`, not through raw shared maps
- the proxy runtime is itself a capability: `IngressRuntime.proxy_handle` (`server/bootstrap/mod.rs:141`) is captured with `Handle::current()` at state construction (`server/bootstrap/mod.rs:385`) and read through `ServerState::proxy_handle()` (`server/bootstrap/mod.rs:167`)

Examples of capability-style access:

- listener parameters
- ingress listener snapshot
- egress map
- auth store
- registry
- config source
- shutdown-related channels
- proxy runtime handle

## Routing Snapshot

`RoutingSnapshot` is an immutable runtime snapshot for ingress routing and egress routing state.

Rules:

- build as a whole
- swap atomically
- expose routing operations through methods
- avoid cross-module field reach-through

Expected uses:

- list desired listeners
- resolve client config for login
- resolve vhost routes
- get current egress map

## Listener Management

`server/ingress/listener_mgr.rs` is the owner of ingress listener lifecycle.

Rules:

- listener entry internals stay private
- external code uses reconcile-style functions
- listener generation and active map are implementation details
- listener startup and drain behavior stay inside the manager
- accept loops are spawned on the proxy runtime handle, never with bare `tokio::spawn`

### Runtime Ownership

Listeners belong to the multi-threaded proxy runtime, not to whichever runtime happened to apply the config that created them. `spawn_single_listener` spawns every accept worker through `state.proxy_handle()` — `server/ingress/listener_mgr.rs:130` for HTTP, `server/ingress/listener_mgr.rs:165` for TCP — and the re-spawn that follows a drained listener uses the same handle (`server/ingress/listener_mgr.rs:240`).

This is load-bearing in ctld mode, where `apply_snapshot` runs on the `BackgroundComponent` current_thread runtime. A bare `tokio::spawn` there binds listeners to that single thread, with two consequences:

- shutdown deadlock: the background runtime can be dropped before the accept workers observe cancellation, so the worker tail that decrements the counter and signals `drained` never runs, and `shutdown_all_listeners` waits on a notification nothing can send
- no ingress parallelism: `run_accept_worker` spawns per-connection work onto the current runtime (`tunnel-lib/src/transport/accept.rs:39`), so accept, sniff, dispatch and relay all share that one thread while the proxy workers stay idle

See `docs/review-2026-07-26/02-scalability-and-cpu-affinity.md` §2.0 for the measured analysis.

Drain waits are bounded independently of that fix: `wait_listener_drained` gives each listener `LISTENER_DRAIN_TIMEOUT` (10s, `server/ingress/listener_mgr.rs:82`) and warns instead of hanging when a worker was dropped rather than cancelled.

## Tunnel Admission

`server/ingress/handlers/quic.rs` owns admission for client-facing tunnel connections. `run_quic_server` (`server/ingress/handlers/quic.rs:29`) checks in this order:

1. address validation first: an `Incoming` whose source address is not validated is answered with quinn `Incoming::retry()` and consumes no budget (`server/ingress/handlers/quic.rs:73`); an unvalidated address that cannot be retried is ignored (`server/ingress/handlers/quic.rs:81`). Without this, spoofed Initial packets would each hold budget for a full handshake window and could lock out real clients at the cost of one extra RTT for honest ones.
2. pre-auth budget: a validated `Incoming` must take a permit from a semaphore sized by `server.max_unauthenticated_connections` (`server/ingress/handlers/quic.rs:48`). Over budget the connection is refused rather than queued (`incoming.refuse()`, `server/ingress/handlers/quic.rs:94`), so a flood costs no task, stream, or crypto state.
3. refusals increment `duotunnel_unauthenticated_connections_refused_total` (`server/runtime/metrics.rs:39`) and are logged at most once per `REFUSAL_LOG_INTERVAL` (1s, `server/ingress/handlers/quic.rs:27`); the counter carries the exact rate.

Rules:

- the permit covers the whole pre-auth phase and is dropped explicitly once authentication concludes (`server/ingress/handlers/quic.rs:309`); every failure path releases it by returning
- the pre-auth phase shares a single deadline derived from `login_timeout` (`pre_auth_deadline`, `server/ingress/handlers/quic.rs:145`), not one timeout per step: handshake, login stream accept, message type, login body and the auth-store query all expire against it, so a peer that stalls each step cannot hold a permit for a multiple of the configured timeout
- authenticated connections release their permit and are governed by the registry instead
- rejection responses never echo internal error detail; `LoginResp.retryable` carries the only bit the client needs, and a backing-store fault must be marked retryable so a transient database blip does not permanently stop every client

### Protocol Negotiation

`Login` carries `protocol_version` and `capabilities`. `negotiate_protocol` (`tunnel-lib/src/models/msg.rs:58`) returns `min(server max, client version)` plus the capability intersection, or `None` below `MIN_SUPPORTED_VERSION`; the server rejects a mismatch without echoing its supported range back to an unauthenticated peer (`server/ingress/handlers/quic.rs:228`).

The resulting `NegotiatedProtocol` is passed to `ClientRegistry::register` (`server/ingress/registry.rs:291`) and stored on the connection entry (`RegisteredConn`, `server/ingress/registry.rs:20`; surfaced as `SelectedConnection::negotiated`, `server/ingress/registry.rs:32`) so future features can gate on capabilities at the connection-selection site without re-plumbing. `LoginResp::success` echoes the negotiated version and capabilities to the client.

Wire generation is pinned by ALPN: `TUNNEL_ALPN` is `tunnel-quic/v1` (`tunnel-lib/src/transport/quic.rs:17`). Breaking layout changes require a new generation so incompatible peers fail in the TLS handshake instead of at login.

### Stream Admission

Server-initiated streams go through `ConnectionHandle::open_stream` (`tunnel-lib/src/transport/connection_handle.rs:75`), which owns two per-connection semaphores: `stream_semaphore` for concurrent streams and `pending_semaphore` for streams waiting on QUIC flow-control credit (`tunnel-lib/src/transport/connection_handle.rs:39`-`42`), sized from `max_concurrent_streams` and the resolved `OverloadLimits::max_pending_streams` (`tunnel-lib/src/lb/overload.rs:25`) that the registry carries into `ConnectionHandle::spawn` (`server/ingress/registry.rs:159`).

The pending limit is per-connection, not a global check-then-act: `open_bi_guarded` admits through the connection's semaphore (`tunnel-lib/src/transport/open_bi.rs:90`), and the process-wide `stream_pending_queue_depth` gauge is a pure metric that gates nothing (`tunnel-lib/src/transport/open_bi.rs:88`). Rejections surface as `quic_open_rejected_overloaded` carrying the per-connection limit.

## Background Modes

The server supports two background modes.

- standalone: file watch and db-backed config source
- managed: ctld watch stream and local token cache

Mode selection belongs to bootstrap and supervisor wiring, not handlers.

## Shutdown

`ServerApp` installs SIGINT/SIGTERM handlers and cancels the shared shutdown token (`server/runtime/app.rs:121`). The sequence is ordered so drain counters can reach zero before anything is force-closed:

1. public accepts stop first: `proxy_main` calls `shutdown_all_listeners` (`server/runtime/app.rs:183`), which cancels every listener and waits for each to report drained, bounded by `LISTENER_DRAIN_TIMEOUT` (10s, `server/ingress/listener_mgr.rs:82`)
2. each tunnel connection handler drains in-flight streams for up to `CONN_SHUTDOWN_DRAIN_TIMEOUT` (15s, `server/ingress/handlers/quic.rs:22`) and only then issues `conn.close` (`server/ingress/handlers/quic.rs:355`-`357`) — QUIC `CONNECTION_CLOSE` aborts every open stream, so the close must follow the drain, not precede it
3. the connection's `UdpSessionManager::shutdown` cancels and joins its session tasks with a 2s cap (`server/ingress/handlers/udp_datagram.rs:106`), then the connection unregisters from the registry
4. `run_quic_server` waits on the connection `TaskTracker` for `CONN_TASK_WAIT_TIMEOUT` (20s, `server/ingress/handlers/quic.rs:24`) before `endpoint.close`; closing the endpoint earlier would abort the handlers mid-drain
5. `ServerApp` backstops the whole sequence with `wait_for_resource_drain` for the app-level `SHUTDOWN_DRAIN_TIMEOUT` (30s, `server/runtime/app.rs:18`) and then forces exit

The constants are deliberately nested (10s < 15s < 20s < 30s) so every inner wait can complete before the next layer gives up.

During drain, `tunnel_lib::METRICS` reports `active_connections` and `pending_streams`.

## Plugin Ingress Pipeline

Bootstrap builds `PluginRegistry` at startup and wires server ingress plugins:

- `plugins::tls::TlsHandler`
- `plugins::h2c::H2cHandler` (respects `h2_single_authority`)
- `plugins::h1::H1Handler`
- `plugins::tcp_pass::TcpPassHandler`
- `plugins::vhost::VhostPlugin` as `RouteResolver`
- `plugins::prometheus::PrometheusSink` as `MetricsSink`

Request-time ingress uses `IngressDispatcher` with this registry; handlers do not construct plugins ad hoc.

Shard topology is resolved at bootstrap: `resolve_shard_count(server.quic.shards, None)` feeds `new_shared_registry(shard_count, max_streams, max_pending_streams)` (`server/bootstrap/mod.rs:333`, `server/bootstrap/mod.rs:345`) — the pending bound is per-connection and therefore travels with the registry into each `ConnectionHandle`.

## Invariants

- there is one public runtime entry
- startup assembly is centralized
- long-running components are started by the supervisor
- request handlers depend on runtime capabilities, not bootstrap objects
- runtime internals are private by default
- configuration mode decisions happen before runtime start

## Non-Goals

This spec does not define:

- wire protocol details
- metrics schema
- database schema
- plugin protocol contracts inside `tunnel-lib`
