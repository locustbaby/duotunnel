# DuoTunnel Core Spec

## Scope

`duotunnel-lib` is the shared library crate for DuoTunnel. It provides protocol primitives, transport adapters, plugin infrastructure, and domain types used by `duotunnel-server`, `duotunnel-client`, and `duotunnel-ctld`.

## Module Layout

```
duotunnel-lib/src/
  lib.rs               (crate root, re-exports all public surface)
  config/              (startup-time config types: QuicConfig, TcpConfig, ...)
  egress/              (HTTP egress client factories)
  engine/              (QUIC-to-TCP relay / bridge logic)
    copy.rs            (pooled BytesMut relay buffers, copy_buffered_*)
    relay.rs
    bridge.rs
  error.rs             (ProxyError, ErrorKind, RetryType)
  infra/               (infrastructure utilities)
    dns_cache.rs
    metrics.rs
    observability.rs
    peek_buf.rs
    pki.rs
    runtime.rs
    timeout.rs
  lb/                  (load-balancing and backpressure)
    inflight.rs        (InflightTable, InflightGuard, pick_least_inflight, pick_p2c_inflight)
    overload.rs        (OverloadLimits, maybe_slow_path, BackoffStrategy)
    shard.rs           (pick_from_preferred_shards, pick_p2c_inflight_owned, stable_shard_index)
  models/              (domain data types and wire messages)
    defs.rs            (ClientGroupDef, IngressListenerDef, TokenCacheEntryDef, ...)
    id.rs              (GroupId, ProxyName, ClientId newtypes)
    msg.rs             (Login, LoginResp, RoutingInfo, send_message, recv_message,
                        PROTOCOL_VERSION, negotiate_protocol, MAX_LOGIN_BYTES, ...)
  plugin/              (plugin trait system)
    ctx.rs             (ServerCtx, EgressCtx, PhaseResult, Timeouts, Route, ...)
    dispatcher.rs      (IngressDispatcher — 6-phase ingress pipeline)
    egress.rs          (LoadBalancer, UpstreamDialer, Resolver)
    ingress.rs         (IngressProtocolHandler, ProtocolHint, ProtocolKind)
    metrics.rs         (MetricsSink, NoopSink)
    module.rs          (ConnectionModule)
    registry.rs        (PluginRegistry)
    route.rs           (RouteResolver, NoRouteResolver)
    service.rs         (TunnelService)
  protocol/            (protocol parsing and wire codecs)
    ctld.rs            (ctld wire protocol: WatchRequest, ConfigSnapshot, ConfigDelta, ...)
    detect.rs          (detect_protocol_and_host, extract_tls_sni)
    driver/            (protocol driver internals)
      h1.rs            (Http1Driver — RFC 9112 request/response framing)
      h2.rs
    http_utils.rs      (sanitize_request_headers, sanitize_response_headers,
                        is_forwardable_trailer, parse_content_length, ...)
    rewrite.rs         (HTTP rewrite helpers)
    sniff.rs           (SniffRuntime, ProtocolDetector, SniffPolicy, SniffResult, ...)
  proxy/               (upstream proxy engine)
    base.rs
    buffer_params.rs
    core.rs            (ProxyEngine, UpstreamResolver, Protocol)
    h2.rs              (hardened_h2_server_builder, hardened_h1_server_builder,
                        H2_SERVER_* / H1_SERVER_* bounds, serve_h2_forward)
    h2_proxy.rs        (H2Sender, forward_h2_request)
    http.rs
    http_connector.rs
    peers.rs
    tcp.rs
    upstream.rs
    mod.rs             (UpstreamGroup, ProxyBufferParams)
  transport/           (connection-level transport primitives)
    accept.rs          (run_accept_worker, AcceptedConn)
    addr.rs            (address normalization)
    connection_handle.rs (ConnectionHandle, OpenStreamRequest — per-connection
                        stream and pending semaphores)
    listener.rs        (VhostRouter, RouteTarget, build_reuseport_listener, ...)
    open_bi.rs         (open_bi_guarded, PendingAdmission, OpenBiOutcome, OpenedStream)
    quic.rs            (TUNNEL_ALPN, QuicTransportParams, build_transport_config,
                        build_udp_socket)
    quinn_io.rs        (PrefixedReadWrite, QuinnStream)
    tcp_params.rs      (TcpParams)
```

## Compatibility Aliases

`lib.rs` exposes two backward-compat module aliases:

- `pub mod ctld_proto` → re-exports `protocol::ctld::*`
- `pub mod shared` → re-exports `models::defs::*`

These exist for callers that predate the reorganization. New code should use canonical paths.

## Wire Protocol and Negotiation

`models/msg.rs` owns the client↔server wire protocol; `transport/quic.rs` owns the
ALPN generation that gates it.

| Item | Location | Value / role |
| :--- | :--- | :--- |
| `TUNNEL_ALPN` | `transport/quic.rs:17` | `b"tunnel-quic/v1"`. Single constant referenced by both ends. A breaking layout change bumps the generation suffix so incompatible peers fail at the QUIC/TLS handshake instead of on rkyv validation after connect |
| `PROTOCOL_VERSION` | `models/msg.rs:34` | `1` — highest wire version this build speaks |
| `MIN_SUPPORTED_VERSION` | `models/msg.rs:36` | `1` — oldest client version still accepted at login |
| `CAP_NONE` / `SUPPORTED_CAPABILITIES` | `models/msg.rs:40`, `:43` | Plain `u64` capability masks (bitflags semantics without the dependency). No bits are defined yet |
| `NegotiatedProtocol` | `models/msg.rs:48` | `{ version, capabilities }`, stored in per-connection session state so future features can be gated on what the peer actually negotiated |
| `negotiate_protocol` | `models/msg.rs:58` | Server-side negotiation. A client reporting a *newer* version is not an error — it still speaks everything up to ours, so the result is `min(ours, theirs)` (forward compatibility for client-first upgrades). Only versions below `MIN_SUPPORTED_VERSION` return `None`, and the caller must say so in the `LoginResp` failure |
| `recv_message_bounded` | `models/msg.rs:247` | Length-bounded message read. The caller's `max_bytes` is additionally clamped to `MAX_MESSAGE_BYTES` (`msg.rs:254`). `recv_message` is the unbounded-caller convenience wrapper that passes `MAX_MESSAGE_BYTES` |
| `MAX_LOGIN_BYTES` | `models/msg.rs:31` | 64 KiB ceiling for the `Login` frame specifically, which is read *before* the peer is authenticated — far below the general `MAX_MESSAGE_BYTES` (10 MiB, `msg.rs:25`) so an unauthenticated peer cannot dictate a large allocation |

**Evolution discipline** (`models/msg.rs:1-10`) — rkyv binary layout is bound to the
field definitions, so:

- Only append fields at the end of a struct; never reorder, remove, or change the
  type of an existing field.
- Every appended field must be gated by a capability bit negotiated at login;
  peers that did not negotiate the bit must not depend on it.
- Breaking layout changes require a new ALPN generation, not a version bump alone.

## Stream Admission

`transport/open_bi.rs` guards `open_bi()`; `transport/connection_handle.rs` owns the
per-connection semaphores it draws on.

- `ConnectionHandle::spawn` builds two semaphores: `stream_semaphore` sized to
  `max_concurrent_streams`, and `pending_semaphore` sized to
  `max_pending_streams.max(1)` (`connection_handle.rs:39-42`). The `.max(1)` floor
  keeps a misconfigured zero from wedging every stream.
- `open_bi_guarded` takes a `PendingAdmission<'a>` (`open_bi.rs:52`) that bundles the
  semaphore with its configured capacity, so the rejection can name the limit it hit
  (`open_bi.rs:94-97`) without `open_bi_guarded` reaching back into config. Previously
  the semaphore was passed bare and the limit was unavailable at the rejection site.
- Admission is **per connection**. The `stream_pending_queue_depth` gauge inside
  `PendingSlot` is a process-wide *observation* only and no longer gates anything
  (`open_bi.rs:88-89`).
- A pending permit is only taken on the slow path: if `open_bi()` is immediately
  ready via `now_or_never()`, the call returns without touching the semaphore
  (`open_bi.rs:70-85`). Dropping `PendingSlot` on any outcome — success, failure,
  timeout, cancellation — releases both the permit and the gauge.

## Relay Buffer Pool

`engine/copy.rs` pools relay buffers as `BytesMut`, not `Vec<u8>`:

- Two tiers: a thread-local free list (≤8 buffers, `copy.rs:8-10`, `:41-51`) backed by
  a global `ArrayQueue` of 1024 (`copy.rs:12-15`).
- Buffers are handed out **empty** (`len == 0`) with at least the requested capacity
  (`copy.rs:20-33`). `copy_buffered` clears and then calls `reader.read_buf(&mut *guard)`
  (`copy.rs:109-110`), which fills the uninitialized capacity region directly.
- Consequences: no `unsafe set_len` over uninitialized memory (the previous form was
  UB), and no `memset` cost for zeroing a buffer that is about to be overwritten.
- `PooledBufGuard` (`copy.rs:61`) returns the buffer on drop; undersized buffers are
  dropped rather than pooled (`copy.rs:35-39`).

## HTTP/1.1 Framing (`Http1Driver`)

`protocol/driver/h1.rs` implements RFC 9112 framing in both directions. Framing rules
are a correctness *and* security boundary here — this driver sits between two hops, so
a framing disagreement is a request-smuggling primitive.

**Request side** — `validate_framing` (`h1.rs:140`) runs before anything else is parsed:

| Condition | Response | Why |
| :--- | :--- | :--- |
| `Content-Length` **and** `Transfer-Encoding` both present | 400 | RFC 9112 §6.3 CL.TE conflict |
| `Transfer-Encoding` alone | 411 Length Required | This driver frames request bodies by content-length only; a transfer-encoded body would be parsed as the *next* request on the stream. 411 tells the client to resend with a length |
| `Content-Length` not `1*DIGIT`, or conflicting duplicates | 400 | `usize::from_str` would accept `+5`; a length we accept but the upstream rejects gets forwarded verbatim next to hyper's chunked framing — a CL.TE request of our own making. Repeated *identical* values are accepted (RFC 9110 §8.6 legacy field merging), including comma-separated lists |
| `Content-Length` exceeds `usize` | 400 | — |
| Header section ≥ 8 KiB without completing | 431 | `h1.rs:241-245` |
| Malformed request line / unparsable target / unknown method | 400 | `h1.rs:207-219` |

Rejections are answered and then closed (`reject`, `h1.rs:88-92`): a bare connection
reset leaves the client unable to tell a protocol error from a network fault.

`Expect` handling (`h1.rs:283-305`): `100-continue` is answered **locally** — the body
is streamed upstream unconditionally, so waiting for the upstream's own 100 would only
stall the client until its own timeout. An HTTP/1.0 sender's expectation is ignored
(RFC 9110 §10.1.1). Any other expectation is refused with 417; silently dropping it
would deadlock a client that waits for an interim response before sending a body.

**Response side** — `write_response` (`h1.rs:390`) picks exactly one of three framings
(RFC 9112 §6.1):

1. **No body** — 1xx / 204 / 304, or a response to `HEAD`: headers only, terminated by
   the blank line (`h1.rs:458-479`). A `HEAD` or 304 may still *echo* the upstream
   `Content-Length` (RFC 9110 §9.3.2); a 204 never claims a length.
2. **Exact length known** — `Content-Length` (`h1.rs:498-531`). The written byte count
   is checked against the declared length; over- or under-run sets `should_close` and
   errors, so a peer cannot misread the next response as this one's remainder.
   Trailers are dropped on this path.
3. **Unknown length** — `Transfer-Encoding: chunked` (`h1.rs:532+`), the only framing
   that may carry trailers.

Toward an **HTTP/1.0** downstream, chunked is forbidden (such a client reads chunk
metadata as body bytes and then waits for a close that never comes), so an unknown
length degrades to close-delimited with an explicit `connection: close`
(`h1.rs:434-441`, `:480-497`).

The upstream length is captured *before* sanitizing removes the field, and
`Transfer-Encoding` takes precedence over any `Content-Length` beside it
(`h1.rs:402-417`) — matching hyper's own decoder precedence. `size_hint` alone is not
usable: proxied bodies are wrapped in `MapFrame`, which does not forward it.

Header hygiene lives in `protocol/http_utils.rs`:

- `sanitize_request_headers` (`http_utils.rs:104`) strips hop-by-hop fields and `Host`,
  and keeps `TE` only when its value is exactly `trailers`.
- `sanitize_response_headers` (`http_utils.rs:118`) strips hop-by-hop fields, `TE`, and
  `Content-Length` — the caller re-derives framing from the length it already captured,
  so the upstream's own framing fields must not survive.
- `is_forwardable_trailer` (`http_utils.rs:133`) allow-lists trailer fields. RFC 9112
  §7.1.2 forbids framing, routing, and control fields in a trailer section; relaying
  them unfiltered would let an upstream inject `Content-Length` or `Transfer-Encoding`
  *after* the chunked terminator, which a shared downstream reads as the start of a
  second response. Dropped fields are logged at debug (`h1.rs:563-567`).

## Downstream H2 / H1 Hardening

`proxy/h2.rs:18-73` holds one hardening policy for every connection accepted from a
downstream peer, exposed as two shared builders:

- `hardened_h2_server_builder` (`h2.rs:45`) — used by all three server-side H2 entries:
  cleartext h2c, TLS with ALPN `h2`, and `serve_h2_forward` (`h2.rs:140`).
- `hardened_h1_server_builder` (`h2.rs:62`) — the TLS listener's `http/1.1` fallback for
  peers that negotiated `http/1.1` over ALPN or sent no ALPN at all.

Values are set **explicitly rather than inherited** from hyper: defaults drift across
versions and the safety property must not depend on them. See
[parameters.md](./parameters.md) §3.9 for the value table. `H2_SERVER_MAX_CONCURRENT_STREAMS`
pins hyper 1.x's current server default and must not be raised above it — every call
site previously inherited that default, so a larger number would *widen* the
stream-flood budget instead of bounding it. Both builders install a `TokioTimer`,
without which hyper panics at runtime on the keep-alive / header-read-timeout paths.

## Invariants

- All public types are re-exported from `lib.rs` at stable paths
- Module boundaries follow runtime responsibility (guideline §4)
- `lb/` owns all backpressure, overload limits, and load-selection primitives
- `transport/open_bi.rs` owns guarded `open_bi()`; pending admission is per connection,
  supplied by the caller as `PendingAdmission`
- `protocol/` owns all wire format and protocol detection code
- `transport/` owns all connection-level I/O primitives
- `models/` owns all shared domain data types and message framing; `msg.rs` layout
  changes follow the append-only rules in its module docs
- `engine/copy.rs` is the only place relay buffers are allocated or pooled; no relay
  path may write into uninitialized capacity by hand
- `proxy/h2.rs` is the only place downstream H2/H1 server bounds are set; new
  downstream-facing entry points must go through the two `hardened_*_server_builder`
  constructors rather than a bare `Builder::new`
- `proxy/http_connector.rs` implements cleartext H2c→H1 adaptive fallback
  (`PREFER_H1_TTL` = 300s, `http_connector.rs:18`)
- `plugin/dispatcher.rs` implements the server ingress 6-phase pipeline (sniff → pre_admission → admission → route → handle → log); see [architecture.md](./architecture.md) §6
- Internal `crate::` paths use canonical submodule paths, not the compat aliases
