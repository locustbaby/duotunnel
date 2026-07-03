# Tunnel-Lib Spec

## Scope

`tunnel-lib` is the shared library crate for DuoTunnel. It provides protocol primitives, transport adapters, plugin infrastructure, and domain types used by `server`, `client`, and `tunnel-service`.

## Module Layout

```
tunnel-lib/src/
  lib.rs               (crate root, re-exports all public surface)
  config/              (startup-time config types: QuicConfig, TcpConfig, ...)
  egress/              (HTTP egress client factories)
  engine/              (QUIC-to-TCP relay / bridge logic)
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
  models/              (domain data types and wire messages)
    defs.rs            (ClientGroupDef, IngressListenerDef, TokenCacheEntryDef, ...)
    msg.rs             (Login, LoginResp, RoutingInfo, send_message, recv_message, ...)
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
    ctld.rs            (ctld wire protocol: WatchRequest, ConfigSnapshot, ConfigPatch, ...)
    detect.rs          (detect_protocol_and_host, extract_tls_sni)
    driver/            (protocol driver internals)
    http_utils.rs      (HTTP utility functions)
    rewrite.rs         (HTTP rewrite helpers)
    sniff.rs           (SniffRuntime, ProtocolDetector, SniffPolicy, SniffResult, ...)
  proxy/               (upstream proxy engine)
    base.rs
    buffer_params.rs
    core.rs            (ProxyEngine, UpstreamResolver, Protocol)
    h2.rs
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
    listener.rs        (VhostRouter, RouteTarget, build_reuseport_listener, ...)
    open_bi.rs         (open_bi_guarded, OpenBiOutcome, OpenedStream)
    quic.rs            (QuicTransportParams, build_transport_config, build_udp_socket)
    quinn_io.rs        (PrefixedReadWrite, QuinnStream)
    tcp_params.rs      (TcpParams)
```

## Compatibility Aliases

`lib.rs` exposes two backward-compat module aliases:

- `pub mod ctld_proto` → re-exports `protocol::ctld::*`
- `pub mod shared` → re-exports `models::defs::*`

These exist for callers that predate the reorganization. New code should use canonical paths.

## Invariants

- All public types are re-exported from `lib.rs` at stable paths
- Module boundaries follow runtime responsibility (guideline §4)
- `lb/` owns all backpressure, overload limits, and load-selection primitives
- `transport/open_bi.rs` owns guarded `open_bi()` with pending-queue rejection
- `protocol/` owns all wire format and protocol detection code
- `transport/` owns all connection-level I/O primitives
- `models/` owns all shared domain data types and message framing
- `proxy/http_connector.rs` implements cleartext H2c→H1 adaptive fallback (300s pin TTL)
- `plugin/dispatcher.rs` implements the server ingress 6-phase pipeline (sniff → pre_admission → admission → route → handle → log); see [architecture.md](./architecture.md) §6
- Internal `crate::` paths use canonical submodule paths, not the compat aliases
