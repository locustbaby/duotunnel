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
- YAML/SQLite source loading, merge, validation, and effective-state initialization
- admin Unix socket startup
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
  → open SQLite and complete schema migrations
  → initialize the SQLite override layer and record legacy migration markers
  → load configured sources and merge the effective configuration
  → validate and commit layers, materialized routing, state, and revision in one transaction
  → create ControlService from the committed effective snapshot
  → run the admin Unix socket listener
  → WatchServer::run (TCP watch_addr)
       client connects → WatchRequest(last_applied_revision, last_applied_hash, token)
       → ConfigEvent::Snapshot (initial)
       → loop: ConfigEvent::Delta (or Snapshot if delta is too large or resync is requested)
       ← ACK(Applied / ResyncRequired)
```

The control connection sends an initial full snapshot. Ongoing configuration changes (either from YAML reload or SQLite admin API override) are computed as a list of `ConfigOperation`s. The connection sends a `ConfigEvent::Delta` if it is smaller than a full snapshot, otherwise it falls back to `ConfigEvent::Snapshot`.

The server consumes snapshots and deltas via `duotunnel-server/control/control_client.rs`, validates the target hash, builds a complete runtime generation, fences revoked sessions, reconciles listeners behind the application admission fence, and atomically publishes the generation pointer. If listener or session fence setup fails, it rolls back listener synchronization to keep the system in the last-known-good state.

## ConfigSource contract

`control/layer.rs` defines a read-only source contract with an associated layer
type and a `watch::Receiver` subscription. The current coordinator registers at
most one YAML source and one SQLite source: `YamlConfigSource` watches the
configured file and emits validated `ConfigLayer` values, while
`SqliteConfigSource` only reads the committed `sqlite_override` row and polls
its source revision. The trait is the extension seam for future sources, but
adding Etcd or another backend also requires explicit coordinator registration
and merge/error-state policy; it is not an automatic plug-in mechanism.

`SqliteConfigSource` exposes no mutation method, so supported SQLite writes
remain inside the resident ctld admin API. A committed admin mutation is
observed by this source and applied through the same merge, validation, and
transaction path. Direct writes by another process are unsupported; detection
does not make them a supported write API.

When a legacy `server_config` field is present and no explicit YAML source is
configured, ctld treats its referenced routing file as the YAML base. Existing
normalized routing rows are retained as the high-priority SQLite override, so
an old database is never overwritten by the file. Startup records
`legacy-routing-to-sqlite-override-v1` and
`legacy-server-config-yaml-base-v1` in `schema_migrations`; rerunning startup
does not reseed or replace an existing override row.

## Admin API boundary

The admin entry point is a small HTTP-like request parser served on the Unix
socket `duotunnel-ctld.admin.sock`. It is part of the `duotunnel-ctld`
process, not a separate web service. CLI mutation commands are clients of this
socket and send `POST` requests to ctld; ctld validates the command, performs
the SQLite transaction, recomputes the effective configuration, and publishes
the new snapshot only after commit. No CLI or external process may mutate the
SQLite file directly.

Mutation requests require an `Idempotency-Key`. The key, mutation fingerprint,
operation, status and response metadata are persisted in the same write
transaction as the mutation. Configuration and revoke responses are replayable
from SQLite. Create/rotate responses contain bearer tokens, so their response
body is stored as `redacted-v1` rather than plaintext; a duplicate in the same
ctld process can be served from a bounded process-memory cache, while a replay
after restart returns `410` without repeating the mutation. This preserves
at-most-once token rotation without persisting raw bearer tokens in SQLite.

The socket is local-process administration only. The TCP watch endpoint is a
separate server-facing control channel and is not used for admin mutations.

## Startup and degraded behavior

Readiness is not enabled until migrations, source loading, merge validation,
effective-state commit, and successful binding of the admin, healthz (when
configured), and watch listeners have completed. A source parse or validation
failure keeps the last valid effective configuration in memory, marks that
source's degraded flag, and retries the same source revision with bounded
backoff. `config_state.degraded` is the OR of the YAML, SQLite, and coordinator
degraded flags, so recovery of one source cannot hide a failure in another.
SQLite source read failures retain the last valid override layer and mark only
the SQLite source degraded. SQLite unavailability or a failed migration
prevents startup because the high-priority override and effective-state
authority cannot be reconstructed safely.

## Invariants

- there is one public runtime entry
- startup assembly is centralized
- control-plane tasks stay under `control/`
- watch protocol handling stays out of bootstrap and main
- runtime internals are private by default
- watch events are either a full Snapshot or a relative Delta, validated by the target content hash
- one epoch/sequence maps to exactly one canonical hash
- ACK means applied and security-fenced, not merely received
- admin mutations are serialized through ctld and committed before publication
- all supported SQLite writes enter through the resident ctld admin API; source
  implementations are read-only observers
- legacy source migration is marker-backed and idempotent
- server has no routing YAML, SQLite, or admin-storage dependency

## Non-Goals

This spec does not define:

- ctld wire protocol details
- sqlite schema
- routing semantics outside the service boundary
