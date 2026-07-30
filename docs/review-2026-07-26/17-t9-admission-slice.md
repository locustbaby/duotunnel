# T9 / TODO-142: admission controller slice

## Delivered

`duotunnel-lib/src/lb/overload.rs` now provides `AdmissionController` with:

- an optional process-wide limit;
- optional limits for named groups;
- immediate `try_acquire` decisions with no waiter queue;
- an RAII `AdmissionPermit` that releases global and group reservations exactly once;
- global and configured-group `admitted`, `rejected`, and `active` counters;
- CAS-based reservation and rollback, so concurrent callers cannot oversubscribe either limit.
- explicit `try_acquire_global` / `try_acquire_group` APIs with `Global` vs `Group`
  rejection scope; the old `Option<&str>` API remains only as a compatibility wrapper.

The public types are re-exported from `duotunnel-lib/src/lib.rs`. A configured group limit is an isolation boundary, not a fairness scheduler. The abstraction intentionally does not queue callers or promise FIFO ordering; a caller that needs queue fairness must provide it above this primitive.

## Production integration: UDP session domain

The first production integration is deliberately limited to one resource
domain: a UDP session owned by one QUIC client connection. This is the only
current path whose budget unit and owner are explicit enough to wire without
changing protocol semantics.

`UdpSessionManager` now uses one `AdmissionController` with a global limit of
`MAX_UDP_SESSIONS_PER_CONNECTION`. The controller is local to that manager,
so it counts only UDP sessions for that client connection. The existing
process-wide UDP semaphore remains a separate process resource budget; it is
not folded into the controller.

`SessionEntry` owns the `AdmissionPermit` for the entire session lifecycle:

```text
try_acquire_global
  → map insertion / creation queue
  → resolving, socket bind/connect, pending-packet drain
  → connected reply pump
  → idle eviction, establishment failure, QUIC close, or shutdown
  → SessionEntry drop releases the permit
```

The existing cancellation and timeout boundaries remain authoritative:

- session establishment resolves, binds, connects, and sends with the
  existing three-second operation timeout;
- connecting entries are evicted after the existing bounded establishment
  window;
- idle connected sessions are evicted after the existing idle timeout;
- manager shutdown cancels the root token, removes entries, and waits for
  tracked tasks before the bounded abort fallback;
- queue-full, capacity rejection, cancellation, timeout, and upstream errors
  all remove or drop the owning `SessionEntry`, so no permit is leaked.

The admission controller is not used for each datagram. Queued datagrams
retain the existing queue semaphore and are a different resource domain from
the long-lived UDP session.

## Integration contract

The permit must be acquired after the caller knows the resource identity and must remain in the owning task/struct until the resource's full lifecycle ends. It must not be dropped after only the accept, sniff, or `open_bi` phase.

The remaining integration points are deliberately unchanged in this slice:

1. `duotunnel-lib/src/plugin/dispatcher.rs` is the global connection-admission boundary, but its early admission phase does not always know the route group. A global permit can be acquired there; per-group admission belongs after route resolution and must be carried through the selected ingress handler.
2. `duotunnel-lib/src/transport/connection_handle.rs::ConnectionHandle::open_stream` is the QUIC stream admission boundary. The existing per-connection semaphores should remain the local guard; the controller should supply the process/group budget and its permit should be retained by the returned stream lifecycle.
3. `duotunnel-server/ingress/handlers/udp_datagram.rs::UdpDatagramDispatcher` keeps its queue budget separate from the production UDP session admission described above. The controller is intentionally not applied per datagram.
4. `duotunnel-lib/src/engine/bridge.rs::{relay, relay_unidirectional, relay_with_first_data}` are the relay lifetime endpoints. They were not changed because T6 owns those files; their callers must retain the permit around the relay task rather than adding a second admission policy inside the bridge.

These callsites require separate wiring because they have different budget units and group-identity availability. No runtime behavior is changed until that policy is selected explicitly. The controller is still not a production admission registry: each resource domain needs its own controller and must retain the lease through the complete owner lifecycle.

## Why other production wiring is intentionally deferred

The concrete ingress chain is `duotunnel-server/ingress/handlers/http.rs::run_http_accept_loop` → `IngressDispatcher::dispatch` → `IngressProtocolHandler::handle`. The dispatcher lifetime would be safe for a connection-wide permit, but it is not the correct budget unit for multiplexed TLS/H2: `duotunnel-server/ingress/plugins/tls/mod.rs::TlsHandler::handle` creates a nested `service_fn`, and each request future can outlive the initial protocol dispatch while sharing one connection.

The correct request-level insertion point is inside that `service_fn`, after `route_target` has been resolved and around the complete `forward_h2_request` retry loop. Implementing it now would require a controller in `ServerState`/`TlsHandler` and an authoritative configured global/per-group limit. The current `OverloadLimits::max_pending_streams` cannot be reused: it specifically bounds pending QUIC `open_bi` waits in `ConnectionHandle::open_stream`, and treating it as an active request budget would silently change overload behavior. H1, H2C, TCP passthrough, and UDP also have different lifecycle units.

Therefore this slice does not add a shared hidden limit across protocols. UDP
session admission is production-wired with the existing, already enforced
per-connection bound. HTTP request, raw relay, reverse stream, and UDP queue
budgets remain separate and are not counted by this controller.

The HTTP/raw/reverse integrations remain deferred until each domain has an
authoritative limit, identity source, and rejection contract. In particular:

| Domain | Required owner | Current decision |
| --- | --- | --- |
| HTTP request | H1 request future or H2 stream future through response-body completion | Deferred; must not use the accept connection lifetime. |
| Raw relay | bidirectional relay task until both directions finish | Deferred; must not use `open_bi` permit lifetime. |
| Reverse stream | reverse stream task including drain/cancel | Deferred; must not share the UDP or HTTP budget. |
| UDP session | `SessionEntry` until removal/drop | Production integrated in this slice. |
| UDP queue | queued envelope until worker consumes/drops it | Existing semaphore remains separate. |

Acceptance criteria for the deferred domains are: a named resource-domain
counter, an owner-held RAII guard, explicit cancellation and timeout release,
an overload response/close policy, metrics for active/admitted/rejected and
hold duration, and tests covering success, reject, body/relay completion,
task cancellation, timeout, shutdown, and retry. Until those criteria are
met, T9 must remain partial rather than claiming full active-stream admission.

## Tests

The focused tests in `duotunnel-lib/src/lb/overload.rs` cover invalid group configuration, RAII counter release, explicit release/reuse, group isolation under shared global capacity, cancellation-by-drop, and concurrent attempts at the global limit. UDP session tests cover stale replacement and idle recheck; the production field placement keeps the permit owned by `SessionEntry`.

They are run with:

```text
cargo test -p duotunnel-lib lb::overload::admission_tests -- --test-threads=1
```
