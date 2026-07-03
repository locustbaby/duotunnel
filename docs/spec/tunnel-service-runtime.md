# Tunnel Service Runtime Spec

## Scope

This spec defines the `tunnel-service` crate runtime shape after the startup and boundary refactor.

## Public Entry

`tunnel_service::run()` is the only external entry for the crate runtime.

Responsibilities at the outer boundary:

- parse CLI
- load config
- construct the app
- enter the runtime

The binary target should not orchestrate startup directly.

## Runtime Layers

The `tunnel-service` crate is organized into four runtime layers.

### 1. CLI

`tunnel-service/src/bootstrap/cli.rs`

Owns:

- command parsing
- config path input
- client admin subcommands

Must not own:

- database wiring
- watch server startup
- snapshot publish logic

### 2. Bootstrap

`tunnel-service/src/bootstrap/config.rs`
`tunnel-service/src/bootstrap/mod.rs`

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

`tunnel-service/src/runtime/app.rs`
`tunnel-service/src/runtime/mod.rs`

Owns:

- startup flow
- logging init
- sqlite store startup
- routing seed on first boot
- healthz endpoint startup
- watch server handoff

Must not own:

- patch diff logic
- token cache query logic
- long-running control-plane internals

`CtldApp` is the application-level orchestrator.

### 4. Control Plane

`tunnel-service/src/control/*`

Owns:

- snapshot and patch shaping
- control service state
- publish debounce
- token cache polling
- watch server protocol loop

These modules are for process-lifetime control-plane work, not bootstrap concerns.

## Control Plane Flow

```
CtldApp::run
  → open SQLite (auth + routing)
  → seed routing from server_config YAML on first boot (if configured)
  → ControlService::new (in-memory snapshot + watch channel)
  → WatchServer::run (TCP watch_addr)
       client connects → WatchRequest
       → Snapshot (full ConfigSnapshot)
       → loop: Patch on DB mutation (debounced via reactor)
```

Managed server consumes patches via `server/control/control_client.rs` → `replace_routing` + `sync_listeners`.

Standalone server uses `hot_reload.rs` (file notify + `ConfigSource::load`) instead of watch TCP.

Wire types: `tunnel-lib/protocol/ctld.rs`, events wrapped in `MessageType::ConfigPush`.

## Invariants

- there is one public runtime entry
- startup assembly is centralized
- control-plane tasks stay under `control/`
- watch protocol handling stays out of bootstrap and main
- runtime internals are private by default

## Non-Goals

This spec does not define:

- ctld wire protocol details
- sqlite schema
- routing semantics outside the service boundary
