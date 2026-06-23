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
