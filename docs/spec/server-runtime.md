# Server Runtime Spec

## Scope

This spec defines the `duotunnel-server` crate runtime shape after the startup and boundary refactor.

## Public Entry

`duotunnel_server::run()` is the only external entry for the crate runtime.

Responsibilities at the outer boundary:

- parse CLI
- construct the server app
- enter the runtime

The binary target should not orchestrate startup directly.

## Runtime Layers

The `duotunnel-server` crate is organized into four runtime layers.

### 1. CLI

`duotunnel-server/bootstrap/cli.rs`

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

`duotunnel-server/bootstrap/mod.rs`

Owns:

- loading server config
- connecting to `duotunnel-ctld`
- building the local runtime from the watched configuration
- building routing snapshots
- building runtime state

Must not own:

- thread spawning
- long-running service loops
- ingress request handling

Bootstrap is the composition root for server runtime dependencies.

### 3. App / Runtime

`duotunnel-server/runtime/app.rs`
`duotunnel-server/runtime/mod.rs`

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

`duotunnel-server/runtime/supervisor.rs`

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

- `duotunnel-server/ingress/handlers/*`
- `duotunnel-server/ingress/listener_mgr.rs`
- `duotunnel-server/control/control_client.rs`
- `duotunnel-server/ingress/plugins/*`

These modules consume runtime capabilities from `ServerState` and should not perform top-level runtime assembly.

## Call Relationships

See [architecture.md](./architecture.md) for full diagrams. Server-side summary:

| Direction | Entry | Core path |
| :--- | :--- | :--- |
| **Forward** (external → client) | `handlers/{http,tcp}.rs` accept | `IngressDispatcher` → plugin handler → `ClientRegistry` → `ConnectionHandle::open_stream` → `open_bi_guarded` → QUIC |
| **Client egress** (client → internet) | QUIC stream from client | `tunnel_handler.rs` → `ProxyEngine(ServerEgressMap)` → external upstream |
| **Tunnel control** | `handlers/quic.rs` | auth → register in `ClientRegistry` → `accept_bi` loop for server-initiated streams |

Routing, token cache and client config reads pin one immutable `RuntimeGeneration`.
Control/hot-reload paths construct the complete replacement and publish it through one `ArcSwap`;
request paths never observe field-by-field mutation.

## Runtime State

`ServerState` is the runtime capability surface for request-time and background-time code.

Internal subdivisions:

- ingress runtime
- connection runtime
- control runtime

`IngressRuntime` also owns two process-stable capabilities:

- `ArcSwap<RuntimeGeneration>` for atomic configuration publication
- `Arc<UpstreamHealthRegistry>` for health state that must survive a generation swap

Rules:

- sub-runtime structs stay private
- callers should prefer capability methods over field access
- routing swaps happen through runtime methods
- listener management is reached through `ServerState`, not through raw shared maps
- the proxy runtime is itself a capability: `IngressRuntime.proxy_handle` (`duotunnel-server/bootstrap/mod.rs:141`) is captured with `Handle::current()` at state construction (`duotunnel-server/bootstrap/mod.rs:385`) and read through `ServerState::proxy_handle()` (`duotunnel-server/bootstrap/mod.rs:167`)

Examples of capability-style access:

- listener parameters
- ingress listener snapshot
- egress map
- auth store
- registry
- config source
- shutdown-related channels
- proxy runtime handle

## Runtime Generation and Routing Snapshot

`RuntimeGeneration` is the publication unit. It contains the immutable `RoutingSnapshot`, token
map and revision/hash. `RoutingSnapshot` contains ingress routing, client
configuration and egress routing objects.

Rules:

- build as a whole
- swap atomically
- reject rollback and equal-revision/different-hash input
- expose routing operations through methods
- avoid cross-module field reach-through
- pin one generation per request/stream work unit

Expected uses:

- list desired listeners
- resolve client config for login
- resolve vhost routes
- get current egress map

## Listener Management

`duotunnel-server/ingress/listener_mgr.rs` is the owner of ingress listener lifecycle.

Rules:

- listener entry internals stay private
- external code uses reconcile-style functions
- listener generation and active map are implementation details
- listener startup and drain behavior stay inside the manager
- accept loops are spawned on the proxy runtime handle, never with bare `tokio::spawn`
- a reconcile computes every start/drain against one locked table view
- every required socket is pre-bound before any listener is cancelled or table state is committed
- any pre-bind failure drops prepared sockets and leaves all predecessor listeners unchanged
- rollback may restore a predecessor only while its workers are still live

### Runtime Ownership

Listeners belong to the multi-threaded proxy runtime, not to whichever runtime happened to apply the config that created them. `spawn_single_listener` spawns every accept worker through `state.proxy_handle()` — `duotunnel-server/ingress/listener_mgr.rs:130` for HTTP, `duotunnel-server/ingress/listener_mgr.rs:165` for TCP — and the re-spawn that follows a drained listener uses the same handle (`duotunnel-server/ingress/listener_mgr.rs:240`).

This is load-bearing in ctld mode, where `apply_snapshot` runs on the `BackgroundComponent` current_thread runtime. A bare `tokio::spawn` there binds listeners to that single thread, with two consequences:

- shutdown deadlock: the background runtime can be dropped before the accept workers observe cancellation, so the worker tail that decrements the counter and signals `drained` never runs, and `shutdown_all_listeners` waits on a notification nothing can send
- no ingress parallelism: `run_accept_worker` spawns per-connection work onto the current runtime (`duotunnel-lib/src/transport/accept.rs:39`), so accept, sniff, dispatch and relay all share that one thread while the proxy workers stay idle

See `docs/review-2026-07-26/02-scalability-and-cpu-affinity.md` §2.0 for the measured analysis.

Drain waits are bounded independently of that fix: `wait_listener_drained` gives each listener `LISTENER_DRAIN_TIMEOUT` (10s, `duotunnel-server/ingress/listener_mgr.rs:82`) and warns instead of hanging when a worker was dropped rather than cancelled.

## Tunnel Admission

`duotunnel-server/ingress/handlers/quic.rs` owns admission for client-facing tunnel connections. `run_quic_server` (`duotunnel-server/ingress/handlers/quic.rs:29`) checks in this order:

1. address validation first: an `Incoming` whose source address is not validated is answered with quinn `Incoming::retry()` and consumes no budget (`duotunnel-server/ingress/handlers/quic.rs:73`); an unvalidated address that cannot be retried is ignored (`duotunnel-server/ingress/handlers/quic.rs:81`). Without this, spoofed Initial packets would each hold budget for a full handshake window and could lock out real clients at the cost of one extra RTT for honest ones.
2. pre-auth budget: a validated `Incoming` must take a permit from a semaphore sized by `server.max_unauthenticated_connections` (`duotunnel-server/ingress/handlers/quic.rs:48`). Over budget the connection is refused rather than queued (`incoming.refuse()`, `duotunnel-server/ingress/handlers/quic.rs:94`), so a flood costs no task, stream, or crypto state.
3. refusals increment `duotunnel_unauthenticated_connections_refused_total` (`duotunnel-server/runtime/metrics.rs:39`) and are logged at most once per `REFUSAL_LOG_INTERVAL` (1s, `duotunnel-server/ingress/handlers/quic.rs:27`); the counter carries the exact rate.

Rules:

- the permit covers the whole pre-auth phase and is dropped explicitly once authentication concludes (`duotunnel-server/ingress/handlers/quic.rs:309`); every failure path releases it by returning
- the pre-auth phase shares a single deadline derived from `login_timeout` (`pre_auth_deadline`, `duotunnel-server/ingress/handlers/quic.rs:145`), not one timeout per step: handshake, login stream accept, message type, login body and the auth-store query all expire against it, so a peer that stalls each step cannot hold a permit for a multiple of the configured timeout
- authenticated connections release their permit and are governed by the registry instead
- rejection responses never echo internal error detail; `LoginResp.retryable` carries the only bit the client needs, and a backing-store fault must be marked retryable so a transient database blip does not permanently stop every client

### Protocol Negotiation

`Login` carries `protocol_version` and `capabilities`. `negotiate_protocol` (`duotunnel-lib/src/models/msg.rs:58`) returns `min(server max, client version)` plus the capability intersection, or `None` below `MIN_SUPPORTED_VERSION`; the server rejects a mismatch without echoing its supported range back to an unauthenticated peer (`duotunnel-server/ingress/handlers/quic.rs:228`).

The resulting `NegotiatedProtocol` is passed to `ClientRegistry::register` (`duotunnel-server/ingress/registry.rs:291`) and stored on the connection entry (`RegisteredConn`, `duotunnel-server/ingress/registry.rs:20`; surfaced as `SelectedConnection::negotiated`, `duotunnel-server/ingress/registry.rs:32`) so future features can gate on capabilities at the connection-selection site without re-plumbing. `LoginResp::success` echoes the negotiated version and capabilities to the client.

Wire generation is pinned by ALPN: `TUNNEL_ALPN` is `tunnel-quic/v1` (`duotunnel-lib/src/transport/quic.rs:17`). Breaking layout changes require a new generation so incompatible peers fail in the TLS handshake instead of at login.

### Stream Admission

Server-initiated streams go through `ConnectionHandle::open_stream` (`duotunnel-lib/src/transport/connection_handle.rs:75`), which owns two per-connection semaphores: `stream_semaphore` for concurrent streams and `pending_semaphore` for streams waiting on QUIC flow-control credit (`duotunnel-lib/src/transport/connection_handle.rs:39`-`42`), sized from `max_concurrent_streams` and the resolved `OverloadLimits::max_pending_streams` (`duotunnel-lib/src/lb/overload.rs:25`) that the registry carries into `ConnectionHandle::spawn` (`duotunnel-server/ingress/registry.rs:159`).

The pending limit is per-connection, not a global check-then-act: `open_bi_guarded` admits through the connection's semaphore (`duotunnel-lib/src/transport/open_bi.rs:90`), and the process-wide `stream_pending_queue_depth` gauge is a pure metric that gates nothing (`duotunnel-lib/src/transport/open_bi.rs:88`). Rejections surface as `quic_open_rejected_overloaded` carrying the per-connection limit.

## Control-plane Mode

The server runs exclusively with `duotunnel-ctld`, receiving its routing and token configuration from the watch stream.

Mode selection and connection parameters are initialized during bootstrap.

## Control-plane Apply

The control client receives and processes both `ConfigEvent::Snapshot` and `ConfigEvent::Delta` events. The apply flow is as follows:

1. Validates the incoming target revision and target hash. If a Delta is received, the operations are applied to a candidate copy of the last applied snapshot and validated against the target hash.
2. Raises the config-apply admission fence.
3. Pre-binds and commits the listener set on the proxy runtime. If any listener synchronization fails, the apply is aborted.
4. Takes the security publication write gate and fences sessions authenticated by tokens revoked in the new generation. If session fencing fails, the listeners are rolled back to the previous listener configuration.
5. Publishes the generation, lowers the admission fence, and returns an Applied ACK.

Authentication holds the matching read gate through auth and registry registration, then releases
it before writing the login response. This prevents a login from escaping between revoke fencing
and generation publication without allowing a slow peer write to stall security updates.

On a successful apply with a healthy listener lifecycle, listener commit and generation publication
form an application-level quiesced cutover for new business work: the apply fence rejects new
public connections/requests, logins, client-initiated reverse streams and new UDP sessions
throughout the transition. They are not one OS-level atomic socket operation:
pre-bound listeners may become kernel-visible before the generation pointer is published, but
cannot admit business work until the fence is lowered. New accept workers start before predecessor
drain and are held behind a start gate until lifecycle handles are registered. Existing work
remains pinned to its prior generation.

Public admission double-checks readiness around generation load. HTTP route resolution carries the
pinned sequence and fails closed if a concurrent publish made it stale. QUIC reverse streams move
the admitted generation's egress map into the task. Every new UDP session loads an admitted
generation, resolves through that egress map and then retains the fixed connected target until
eviction; new sessions are rejected while apply/stale policy fences admission.

If listener synchronization or session fencing fails during apply, the server rolls back the listeners to their previous state and retains the existing generation, ensuring availability remains intact.

Control freshness is based on the last confirmation from the authority, including an idempotent
confirmation of the current revision/hash; the last successful apply is tracked separately for
durability and LKG age. A different control epoch fails closed. The current deployment contract is
single-leader; multi-leader failover requires an explicit authority-reset workflow.

The last-known-good cache stores a bounded, validated envelope with format/protocol version,
payload length, hash, revision and timestamp. Rotation durably writes `previous` before replacing
`primary`; load validates both candidates and selects the highest valid revision.

## Upstream Health

`UpstreamHealthRegistry` is owned by `ServerState`, not by a routing generation. Keys include the
upstream group and backend address, so reload preserves state for a stable backend while identical
addresses in different groups remain isolated. HTTP connect failures, TCP/DNS failures and TLS
handshake failures feed passive ejection; success clears the matching entry. The registry and
h2c-to-H1 preference cache have hard entry caps and TTL cleanup. Active probes have a process-wide
128-task budget and deterministic jitter; a probe owner token prevents stale-task ABA cleanup.

Client-initiated reverse streams have both per-connection (1000) and process-wide (4096) admission
budgets. Routing metadata is capped at 8 KiB and must arrive within 5 seconds; over-budget streams
are reset before a task is spawned.

## Shutdown

`ServerApp` installs SIGINT/SIGTERM handlers and cancels the shared shutdown token (`duotunnel-server/runtime/app.rs:121`). The sequence is ordered so drain counters can reach zero before anything is force-closed:

1. public accepts stop first: `proxy_main` calls `shutdown_all_listeners` (`duotunnel-server/runtime/app.rs:183`), which cancels every listener and waits for each to report drained, bounded by `LISTENER_DRAIN_TIMEOUT` (10s, `duotunnel-server/ingress/listener_mgr.rs:82`)
2. each tunnel handler retires/unregisters its connection before drain, so no selector can open new
   work during shutdown
3. UDP workers and reverse-stream tasks drain with deadlines; timeout aborts their owned handles
4. each tunnel connection handler drains in-flight streams for up to
   `CONN_SHUTDOWN_DRAIN_TIMEOUT` (15s) and only then issues `conn.close` — QUIC
   `CONNECTION_CLOSE` aborts every open stream, so close follows drain
5. `run_quic_server` waits on the connection `TaskTracker` for `CONN_TASK_WAIT_TIMEOUT` (20s)
   before `endpoint.close`
6. `ServerApp` backstops the sequence with the app-level 30s drain deadline

H2c, TLS-H2 and TLS-H1 first request Hyper graceful shutdown and then wait at most 15s. Timeout
drops the connection future, force-closes the socket and returns a typed downstream drain error.

The constants are deliberately nested (10s < 15s < 20s < 30s) so every inner wait can complete before the next layer gives up.

During drain, `duotunnel_lib::METRICS` reports `active_connections` and `pending_streams`.

## Plugin Ingress Pipeline

Bootstrap builds `PluginRegistry` at startup and wires server ingress plugins:

- `plugins::tls::TlsHandler`
- `plugins::h2c::H2cHandler` (respects `h2_single_authority`)
- `plugins::h1::H1Handler`
- `plugins::tcp_pass::TcpPassHandler`
- `plugins::vhost::VhostPlugin` as `RouteResolver`
- `plugins::prometheus::PrometheusSink` as `MetricsSink`

Request-time ingress uses `IngressDispatcher` with this registry; handlers do not construct plugins ad hoc.

Shard topology is resolved at bootstrap: `resolve_shard_count(server.quic.shards, None)` and `server.connection_registry_capacity` feed `new_shared_registry(shard_count, max_streams, max_pending_streams, connection_registry_capacity)` (`duotunnel-server/bootstrap/mod.rs`) — the pending bound is per-connection and therefore travels with the registry into each `ConnectionHandle`; registry capacity is a separate checked admission budget and does not evict live connections.

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
- plugin protocol contracts inside `duotunnel-lib`
