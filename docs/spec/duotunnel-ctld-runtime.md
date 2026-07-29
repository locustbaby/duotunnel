# DuoTunnel CTLD Runtime Spec

## Scope

This spec defines the `duotunnel-ctld` crate runtime shape after the startup and boundary refactor.

## Public Entry

`duotunnel_ctld::run()` is the only external entry for the crate runtime.

Responsibilities at the outer boundary:

- parse CLI
- load config
- construct the app
- enter the runtime

The binary target should not orchestrate startup directly.

## Runtime Layers

The `duotunnel-ctld` crate is organized into four runtime layers.

### 1. CLI

`duotunnel-ctld/src/bootstrap/cli.rs`

Owns:

- command parsing
- config path input
- client admin subcommands

Must not own:

- database wiring
- watch server startup
- snapshot publish logic

### 2. Bootstrap

`duotunnel-ctld/src/bootstrap/config.rs`
`duotunnel-ctld/src/bootstrap/mod.rs`

Owns:

- loading ctld config
- environment overrides
- bootstrap state assembly

Must not own:

- runtime spawning
- control service loops
- watch protocol handling

Bootstrap is the composition root for runtime inputs.

### 3. App / Runtime

`duotunnel-ctld/src/runtime/app.rs`
`duotunnel-ctld/src/runtime/mod.rs`

Owns:

- startup flow
- logging init
- sqlite store startup
- YAML config layer loading and SQLite override initialization
- admin server socket startup
- healthz endpoint startup
- watch server handoff

Must not own:

- delta diff logic
- token cache query logic
- long-running control-plane internals

`CtldApp` is the application-level orchestrator.

### 4. Control Plane

`duotunnel-ctld/src/control/*`

Owns:

- snapshot and delta/operation shaping
- control service state
- config override layering and merge
- token cache polling
- watch server protocol loop

These modules are for process-lifetime control-plane work, not bootstrap concerns.

## Control Plane Flow

```
CtldApp::run
  → open SQLite (auth + routing) and initialize config tables
  → load and apply YamlConfigSource (if configured)
  → run admin socket listener (Unix socket)
  → ControlService::new (SQLite-consistent snapshot + latest-state watch signal)
  → WatchServer::run (TCP watch_addr)
       client connects → WatchRequest(last_applied_revision, last_applied_hash, token)
       → ConfigEvent::Snapshot (initial)
       → loop: ConfigEvent::Delta (or Snapshot if delta is too large or resync is requested)
       ← ACK(Applied / ResyncRequired)
```

The control connection sends an initial full snapshot. Ongoing configuration changes (either from YAML reload or SQLite admin API override) are computed as a list of `ConfigOperation`s. The connection sends a `ConfigEvent::Delta` if it is smaller than a full snapshot, otherwise it falls back to `ConfigEvent::Snapshot`.

The server consumes snapshots and deltas via `duotunnel-server/control/control_client.rs`, validates the target hash, builds a complete runtime generation, fences revoked sessions, reconciles listeners behind the application admission fence, and atomically publishes the generation pointer. If listener or session fence setup fails, it rolls back listener synchronization to keep the system in the last-known-good state.

## Invariants

- there is one public runtime entry
- startup assembly is centralized
- control-plane tasks stay under `control/`
- watch protocol handling stays out of bootstrap and main
- runtime internals are private by default
- watch events are either a full Snapshot or a relative Delta, validated by the target content hash
- one epoch/sequence maps to exactly one canonical hash
- ACK means applied and security-fenced, not merely received

## Non-Goals

This spec does not define:

- ctld wire protocol details
- sqlite schema
- routing semantics outside the service boundary
