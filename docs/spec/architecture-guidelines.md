# Architecture Guidelines

## Purpose

These guidelines define how code should be organized in DuoTunnel when adding or refactoring modules.

## 1. One Composition Root Per Process

Each executable should have a single composition root near the outer boundary.

The composition root is responsible for:

- parsing inputs
- loading config
- choosing mode
- building dependencies
- starting the runtime

Business handlers should not construct process-wide dependencies.

## 2. Thin Entry, Thick Interior

Entry files should stay minimal.

Good entry responsibilities:

- delegate to app/runtime entrypoints
- return process result

Bad entry responsibilities:

- mixed config parsing and runtime spawning
- direct listener reconciliation
- shared-state field assembly spread across many functions

## 3. Private by Default

Types, fields, and modules should start private and only be widened when there is a real caller need.

Prefer:

- private fields
- `pub(crate)` for crate-internal boundaries
- narrow methods that expose capability

Avoid:

- state bags with public fields
- public maps or locks
- exposing lifecycle internals just for convenience

## 4. Organize by Runtime Responsibility

Prefer modules that reflect runtime responsibility:

- cli
- bootstrap
- app/runtime
- supervisor/components
- request handlers
- control/background services

When a layer has more than a couple of files, prefer directory grouping over flat root-level files.

Example:

- `bootstrap/`
- `runtime/`
- `control/`
- `ingress/`
- `egress/`

Avoid modules that mix:

- config loading
- request processing
- thread orchestration
- state mutation

## 5. Separate Startup-Time and Request-Time Concerns

Startup-time code decides what the runtime is.

Request-time code uses the runtime that already exists.

Startup-time examples:

- mode resolution
- plugin registry construction
- store construction
- runtime state assembly

Request-time examples:

- auth check
- route lookup
- connection forwarding
- listener accept loops

## 6. Prefer Capability Objects Over Field Reach-Through

Shared runtime state should be consumed through methods that express intent.

Prefer:

- `state.egress_map()`
- `state.open_stream_timeout()`
- `state.client_config_for_group(...)`

Avoid:

- reaching through multiple nested structs
- exposing internal maps to unrelated modules
- making callers understand storage layout

## 7. Long-Running Tasks Need Lifecycle Ownership

Background tasks should not be free-floating `spawn` calls scattered across the codebase.

Long-running tasks should have:

- a named owner
- startup point
- shutdown path
- failure handling policy

When multiple process-lifetime tasks exist, prefer a supervisor.

## 8. Immutable Snapshots for Shared Routing State

Routing and similar shared read-heavy state should prefer immutable snapshots with atomic replacement.

Benefits:

- simpler reader code
- fewer partial-update hazards
- easier hot reload and control-plane apply paths

Rules:

- build a full snapshot first
- swap once
- keep snapshot internals encapsulated

## 9. Keep Adapter Code at the Edge

Protocol adapters, config adapters, database adapters, and control-plane adapters should sit near the edges of the system.

Core runtime coordination should not depend on edge-specific details leaking everywhere.

Examples:

- file watch belongs to hot-reload edge logic
- ctld watch belongs to control-plane edge logic
- request forwarding belongs to handler/plugin path

## 10. Refactor Toward Fewer Reasons to Change

A module should ideally change for one dominant reason.

Signals that a module is overloaded:

- it parses config and starts threads
- it knows both business routing and socket lifecycle
- it owns both persistence wiring and request forwarding

When refactoring, split along runtime responsibility rather than arbitrary utility categories.

## 11. Avoid Empty Abstractions

Do not add a manager, coordinator, or context type unless it owns a real boundary.

A good abstraction usually owns one of:

- lifecycle
- dependency assembly
- state transition
- protocol boundary
- external integration

If a wrapper only renames existing calls without reducing coupling, it should not exist.

## 12. Keep Specs Aligned With Reality

When major module boundaries change:

- update the relevant spec in `docs/spec`
- keep file structure examples current
- prefer concise invariants over prose-heavy explanation
