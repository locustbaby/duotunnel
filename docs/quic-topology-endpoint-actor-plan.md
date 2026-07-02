# QUIC Topology Performance Plan

## Summary

The near-term topology work is not an endpoint-actor rewrite. The current priority is to remove self-inflicted hot-path cross-core costs while keeping the existing Tokio multi-thread runtime and Quinn endpoint model.

Current target:

- client entry TCP/UDP selection reads the connection pool through lock-free snapshots
- server client registration uses stable hashing so registration shard and request preferred shard align
- TLS-terminated H2 reuses the selected client and H2 sender per route target
- local egress reject checks are O(1) snapshot reads
- endpoint actors, thread-per-core, and multi-endpoint fanout stay out of the current implementation

## Why Not Endpoint Actors Now

`quinn::Endpoint` is already cheap to clone and supports concurrent `connect` / `accept` use. Moving endpoint access behind an actor mailbox would add another wakeup to connection setup while leaving the more likely bottlenecks untouched:

- Quinn internal per-connection synchronization
- endpoint UDP driver capacity
- request-side selection and local rule lookup
- H2 sender reuse after a route has already selected a client

Endpoint actors only become interesting if profiling shows a single endpoint UDP driver core saturated while other cores are idle after the current hot-path fixes land.

## Current Work

### Client Entry Pool

`EntryConnPool` keeps its actor for cold-path mutation only. Push and remove messages acknowledge after the actor updates write-side indexes, publishes shard snapshots, and updates total size. TCP and UDP entry paths read synchronously from `ArcSwap` shard snapshots and use shard-local P2C without a mailbox round trip.

Client connection registration uses deterministic spreading by `Connection::stable_id()` rather than a shared round-robin counter. Request preferred shards remain request-key based, so fallback across shards is still required and expected.

### Server Registry

`ClientRegistry::register` maps each group to `stable_shard_index(group_id, shard_count)`. This puts a group's registered clients in the same shard that `select_client_for_group` checks first, reducing fallback scans while preserving existing actor-owned mutation and snapshot publication.

### Data Plane

Local egress rules are compiled into an allowed-host snapshot. Reads are lock-free snapshot loads and answer whether the sniffed host has any matching allowlist rule. Matching strips optional ports, lowercases domains, and supports the same exact/wildcard semantics as server egress routing; `action_upstream` is only an upstream group name.

TLS-terminated H2 caches `SelectedConnection + H2Sender` by `RouteTarget { group_id, proxy_name }`, matching the h2c invalidation model. Forward failures invalidate the route-target cache entry and unregister lost connections before retrying.

`SniffPrefix::Pooled` converts to `Bytes` using an owner wrapper, so cloning the initial prefix is a reference-counted clone instead of copying the peek buffer.

## Deferred Work

UDP datagram wire-format migration is intentionally separate from the pool and registry changes. It should move session keys to native `SocketAddr`, replace rkyv envelope encoding with a manual header, return borrowed/`Bytes` decode views, and replace per-packet wall-clock calls with coarse aging.

Thread-per-core is not a current goal. It would give up Tokio work stealing, force broad Quinn/Hyper integration rewrites, and complicate operations for tunnel traffic that naturally aggregates N clients to M upstreams rather than staying shared-nothing.

Multi-endpoint plus `SO_REUSEPORT` remains a research item. It should only be designed after benchmarks show endpoint UDP driver saturation that cannot be explained by per-connection locks, route selection, H2 sender churn, local rule scans, or datagram copies.

## Validation

Required checks:

- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `ci-helpers/local-test/test.sh`

Performance comparisons:

- entry TCP open-stream P99 before and after lock-free pool reads
- UDP packets per second before and after lock-free pool reads
- TLS/H2 throughput and CPU before and after route-target sender caching
- `connections = 1` versus `connections = 4` to separate per-connection bottlenecks from endpoint-driver bottlenecks
