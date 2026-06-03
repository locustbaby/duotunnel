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

Components are for process-lifetime tasks, not request handlers.

## Request Path

Request handling belongs below the startup layers.

Main ingress/runtime modules:

- `server/ingress/handlers/*`
- `server/ingress/listener_mgr.rs`
- `server/control/control_client.rs`
- `server/control/hot_reload.rs`
- `server/ingress/plugins/*`

These modules consume runtime capabilities from `ServerState` and should not perform top-level runtime assembly.

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

Examples of capability-style access:

- listener parameters
- ingress listener snapshot
- egress map
- auth store
- registry
- config source
- shutdown-related channels

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

## Background Modes

The server supports two background modes.

- standalone: file watch and db-backed config source
- managed: ctld watch stream and local token cache

Mode selection belongs to bootstrap and supervisor wiring, not handlers.

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
