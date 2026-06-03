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

### 4. Tunnel / Services

`client/tunnel/endpoint.rs`
`client/tunnel/client.rs`
`client/tunnel/*`
`client/egress/listener.rs`

Owns:

- QUIC endpoint and TLS config construction (`endpoint.rs` — startup-time)
- server connect/login flow and session loop (`client.rs` — request-time)
- reconnect supervision
- connection pool lifecycle
- reverse entry listener lifecycle

Boundary rule: `endpoint.rs` is startup-time only; `client.rs` is request-time only. These modules are for process-lifetime tunnel work, not CLI/bootstrap concerns.

## Request Path

Request handling belongs below the startup layers.

Main ingress/runtime modules:

- `client/ingress/app.rs`
- `client/ingress/handler.rs`
- `client/egress/listener.rs`
- `client/plugins/*`

These modules consume runtime capabilities from config and tunnel services and should not perform top-level runtime assembly.

## Invariants

- there is one public runtime entry
- startup assembly is centralized
- reconnect and pool lifecycle stay in `client/tunnel`
- listener accept loops stay in `client/egress`
- local service handling stays in `client/ingress`
- runtime internals are private by default

## Non-Goals

This spec does not define:

- wire protocol details
- metrics schema
- plugin contracts inside `tunnel-lib`
