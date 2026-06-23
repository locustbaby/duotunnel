# QUIC Topology And Endpoint Actor Plan

## Summary

This note captures the next topology refactor after the existing connection-actor work.

Current state:

- request-side shard selection is already stable-hash plus shard-local P2C
- connection ownership is already handled by per-connection actors
- shard registration is still round-robin
- endpoint lifecycle is still driven by bare `quinn::Endpoint` values

Target state:

- shard registration uses an explicit topology policy
- client and server both introduce real endpoint actors
- connection actors remain the owner of per-connection QUIC control operations
- the runtime shape becomes `endpoint actor -> shard directory -> connection actor`

## Topology Policy

The default topology policy should become stable-hash based, not round-robin.

Required capabilities:

- choose the preferred shard for a request key
- choose the target shard for a new connection identity
- keep the policy swappable so future alternatives can be added without changing callers

Default policy:

- `StableHashTopologyPolicy`
- client connection registration key: server address plus logical endpoint identity
- server connection registration key: client group plus client connection id
- request-side keys remain request-local:
  - client TCP and UDP use host or proxy-derived keys
  - server uses `group_id`

Round-robin should stop being the default registration behavior. It may remain as a future optional policy, but not the baseline implementation.

Shard-count changes:

- first implementation uses stable hash plus modulo shard count
- changing shard count intentionally remaps some connection identities and request keys
- this is acceptable for static runtime configuration and restart-based topology changes
- online scale-up/down needs a later consistent-hash or rendezvous-hash policy before it is safe to treat shard-count changes as low churn
- the policy trait should expose shard count as an input so the implementation can be replaced without touching callers

## Endpoint Actor Responsibilities

Endpoint actors are needed on both client and server.

They should own:

- the `quinn::Endpoint`
- endpoint-level connect or accept loops
- shard topology policy
- connection actor creation
- registration into shard-aware directories
- endpoint shutdown and cleanup

They should not own:

- TCP listener accept loops
- protocol sniffing or route resolution
- upstream TCP, HTTP, or H2 drivers
- relay of already-opened QUIC streams

Connection actors remain necessary because endpoint actors do not remove connection-level serialization needs such as `open_bi`, initial control writes, and datagram sends.

## Client Shape

Client runtime should move from:

- bare endpoint passed into supervisor and connect loop
- pool deciding shard assignment internally

To:

- endpoint actor owns the client endpoint and reconnect loop
- endpoint actor computes shard assignment before registration
- entry pool becomes a shard-aware directory, not a shard allocator
- per-connection session tasks still handle:
  - login handshake
  - reverse `accept_bi`
  - incoming datagrams
  - connection closed handling

The client endpoint actor should expose capability-style handles, not raw endpoint access.

## Server Shape

Server runtime should move from:

- bare endpoint in `run_quic_server`
- registry deciding shard assignment internally

To:

- endpoint actor owns server `accept()` lifecycle
- login-success path computes target shard before registry insertion
- registry becomes a shard-aware directory and query surface
- per-connection tasks still handle:
  - login handshake body after accept
  - reverse streams from client
  - incoming datagrams
  - token revocation shutdown

The registry should stop owning shard assignment policy state.

## Interfaces

Expected shared additions:

- `ShardTopologyPolicy`
- `StableHashTopologyPolicy`
- `ConnectionIdentity`
- `RequestShardKey`

Expected runtime boundary changes:

- client pool registration should accept an explicit `shard_id`
- server registry registration should accept an explicit `shard_id`
- endpoint actor handles should replace raw endpoint ownership in long-running runtime code

Existing connection-handle APIs should remain the data-plane boundary:

- `open_stream(...)`
- `send_datagram(...)`

## Migration Plan

The migration should land in small compatibility-preserving steps.

1. Add `ShardTopologyPolicy`, key types, and deterministic unit tests while keeping current round-robin registration behavior.
2. Change client pool and server registry registration APIs to accept explicit `shard_id`; keep temporary call sites passing the old round-robin value.
3. Move client registration shard choice into a client endpoint actor, leaving session tasks and connection actors unchanged.
4. Move server registration shard choice into a server endpoint actor after login success, leaving auth, token revocation, and datagram handling in existing per-connection tasks.
5. Remove registry/pool-owned round-robin state once both endpoint actors own shard assignment.
6. Run the local integration matrix, then delete compatibility shims in a follow-up commit.

Rollback strategy:

- keep endpoint actor handles behind existing runtime constructors until both client and server paths pass integration tests
- avoid changing wire protocol or connection-handle APIs in this refactor
- if a phase regresses, revert that phase without touching already-landed policy tests or data-plane connection actors

## Testing

The refactor should be accepted only if these behaviors hold:

- stable request keys map to stable preferred shards
- stable connection identities map to stable target shards
- single-shard mode still resolves to shard `0`
- client registration no longer uses round-robin
- server registration no longer uses round-robin
- preferred-shard request routing still falls back across shards when needed
- connection cleanup removes entries from the correct shard
- existing TCP, HTTP, H2, WebSocket, gRPC, and datagram paths do not regress

Validation baseline:

- `cargo clippy --workspace -- -D warnings`
- local integration test stack in `ci-helpers/local-test/test.sh`

## Assumptions

- stable-hash is the default topology policy
- policy abstraction should support later replacement, but only stable-hash is required now
- endpoint actors are introduced on both client and server
- connection actors remain in place
- multi-endpoint or endpoint-per-core is not required yet, but this refactor must keep that path open
