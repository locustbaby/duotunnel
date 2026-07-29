# DuoTunnel Design Document

> High-performance bidirectional tunnel proxy system based on QUIC (Quinn)
>
> Inspired by frp design philosophy, implementing transparent tunneling + configuration distribution + grouping + Rules-based routing

**Spec index:** [architecture.md](./architecture.md) (topology & call flows) · [parameters.md](./parameters.md) · per-crate `*-runtime.md`

---

## 1. Design Goals

### 1.1 Bidirectional Request Proxying

```
Forward Proxy (Ingress):  External Request → Server → Client → Local Service
Reverse Proxy (Egress):   Internal Request → Client → Server → External Service
```

### 1.2 Core Advantages (vs frp)

| Feature | frp (TCP + Yamux) | DuoTunnel (QUIC) |
|---------|-------------------|------------------|
| Data Channel Creation | frpc initiates TCP connection | Server directly calls `open_bi()` |
| Message Exchanges | 3 times | 1 time |
| Latency | At least 1.5 RTT | 0 RTT (Stream creation requires no handshake) |
| Connection Pool | Requires pre-creation | Not needed (on-demand creation) |
| Multiplexing | Yamux | Native QUIC Stream |
| 0-RTT | Not supported | Natively supported |
| Connection Migration | Not supported | Natively supported |

---

## 2. System Architecture

### 2.1 Overall Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        DuoTunnel Architecture                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   External                Server (Public)              Client (Private) │
│   ─────────              ═════════════               ═════════════      │
│                                                                         │
│   HTTP/HTTPS ────►  ┌─────────────────┐           ┌─────────────────┐  │
│   TCP/WS            │  Entry Listener │   QUIC    │ Control Handler │  │
│                     │  (HTTP/TCP)     │ ◄════════►│ (Maintain Conn) │  │
│                     │                 │           │                 │  │
│                     │  VHost Router   │   Bidir   │  Proxy Manager  │  │
│                     │  Client Registry│  Stream   │  LocalProxyMap  │  │
│                     │  Metrics        │           │                 │  │
│                     └─────────────────┘           └────────┬────────┘  │
│                                                            │           │
│                                                            ▼           │
│                                                   ┌─────────────────┐  │
│                                                   │  Local Services │  │
│                                                   │  (Private Svcs) │  │
│                                                   └─────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Module Structure

```
tunnel/
├── duotunnel-core/                    # Core library
│   └── src/
│       ├── config/                # Startup-time config types (QuicConfig, TcpConfig, ...)
│       ├── models/msg.rs          # Message protocol definitions
│       ├── transport/             # QUIC/TCP transport layer
│       │   ├── accept.rs          # Concurrency accept with SO_REUSEPORT
│       │   ├── listener.rs        # VhostRouter, build_reuseport_listener
│       │   ├── open_bi.rs         # open_bi_guarded, pending-queue overload gate
│       │   ├── quic.rs            # QuicTransportParams, UDP socket tuning
│       │   ├── quinn_io.rs        # QUIC stream adapter
│       │   └── addr.rs            # Address resolution
│       ├── protocol/              # Protocol handling
│       │   ├── detect.rs          # Protocol detection (httparse integration)
│       │   └── rewrite.rs         # Header rewriting
│       ├── proxy/                 # Proxy core
│       │   ├── core.rs            # UpstreamResolver trait
│       │   ├── peers.rs           # PeerSpec enum
│       │   ├── http_connector.rs  # H2c→H1 adaptive fallback
│       │   ├── tcp.rs             # TCP passthrough
│       │   ├── http.rs            # HTTP/1.1 proxy
│       │   └── h2.rs              # HTTP/2 proxy
│       ├── engine/                # Data engine
│       │   └── bridge.rs          # QUIC-TCP bridge
│       ├── egress/                # Outbound HTTP client factories
│       ├── lb/                    # Backpressure and load selection
│       │   ├── inflight.rs        # InflightTable, pick_p2c_inflight
│       │   └── overload.rs        # OverloadLimits, maybe_slow_path
│       ├── plugin/                # Ingress pipeline plugin core
│       │   ├── dispatcher.rs      # Pipeline dispatcher
│       │   ├── ingress.rs         # IngressProtocolHandler trait
│       │   └── egress.rs          # Egress plugin traits
│       ├── error.rs               # Structured proxy errors
│       └── infra/                 # Infrastructure
│           ├── runtime.rs         # effective_runtime_parallelism, cgroup CPU limit
│           ├── pki.rs             # Certificate generation (MITM)
│           ├── peek_buf.rs        # PeekBufPool thread-local cache
│           └── observability.rs   # Logging and tracing
│
├── crates/duotunnel-server/                        # Server
│   ├── main.rs                    # Thin binary entry
│   ├── lib.rs                     # Server runtime facade
│   ├── bootstrap/
│   │   ├── cli.rs                 # CLI parsing
│   │   ├── config.rs              # Config loading and sources
│   │   └── mod.rs                 # Mode resolution and runtime assembly
│   ├── runtime/
│   │   ├── app.rs                 # Startup orchestration
│   │   ├── metrics.rs             # Metrics helpers
│   │   ├── mod.rs                 # Runtime entry and observability
│   │   └── supervisor.rs          # Long-running component lifecycle
│   ├── control/
│   │   ├── control_client.rs      # Ctld watch client
│   │   ├── local_auth.rs          # In-memory token cache auth
│   │   └── service.rs             # Background service trait
│   ├── ingress/
│   │   ├── handlers/
│   │   │   ├── quic.rs            # QUIC connection & Login handling
│   │   │   ├── http.rs            # Ingress HTTP server handler
│   │   │   ├── tcp.rs             # Ingress TCP listener handler
│   │   │   └── metrics.rs         # Prometheus metrics service
│   │   ├── plugins/               # Ingress protocol plugins
│   │   │   ├── h1/                # HTTP/1.1 protocol plugin
│   │   │   ├── h2c/               # H2c protocol plugin (with CachedSender)
│   │   │   ├── tls/               # TLS termination / MITM plugin
│   │   │   ├── tcp_pass/          # TCP passthrough plugin
│   │   │   └── prometheus/        # Prometheus metrics plugin
│   │   ├── listener_mgr.rs        # Ingress listener lifecycle
│   │   ├── registry.rs            # ClientRegistry (sharded, actor via mpsc)
│   │   ├── tunnel_handler.rs      # Reverse stream handling
│   │   └── tunnel_service.rs      # Tunnel service implementation
│   └── egress/
│       └── mod.rs                 # Outbound routing (HttpConnector wrapper)
│
├── crates/duotunnel-ctld/                # Control service
│   └── src/
│       ├── main.rs                # Thin binary entry
│       ├── lib.rs                 # Ctld runtime facade
│       ├── bootstrap/
│       │   ├── cli.rs             # CLI parsing
│       │   ├── config.rs          # Config loading
│       │   └── mod.rs             # Bootstrap assembly
│       ├── runtime/
│       │   ├── app.rs             # Startup orchestration
│       │   └── mod.rs             # Tokio runtime entry
│       └── control/
│           ├── proto.rs           # Snapshot / patch conversion
│           ├── reactor.rs         # Publish debounce and DB poll tasks
│           ├── service.rs         # Control service state and mutations
│           ├── watch.rs           # Watch TCP server
│           └── token/
│               └── cache.rs       # Token cache provider
│
└── crates/duotunnel-client/                        # Client
    ├── main.rs                    # Thin binary entry
    ├── lib.rs                     # Client runtime facade
    ├── bootstrap/
    │   ├── cli.rs                 # CLI parsing
    │   ├── config.rs              # Configuration parsing
    │   └── mod.rs                 # Bootstrap assembly
    ├── runtime/
    │   ├── app.rs                 # Startup orchestration
    │   ├── engine.rs              # Runtime service engine
    │   └── mod.rs                 # Runtime entry and task spawning
    ├── tunnel/
    │   ├── client.rs              # QUIC connect/login/TLS path
    │   ├── conn_pool.rs           # Multi QUIC connection pool (RCU + Round-Robin)
    │   ├── pool.rs                # Pool startup helpers
    │   └── supervisor.rs          # Reconnect supervision
    ├── ingress/
    │   ├── app.rs                 # LocalProxyMap
    │   └── handler.rs             # Local service handler
    ├── egress/
    │   ├── listener.rs            # Reverse TCP entry (forward proxy from local apps)
    │   └── udp_listener.rs        # Reverse UDP entry
    └── plugins/                   # Client-side adapters (resolver, LB)
```

---

## 3. Message Protocol

> Canonical architecture and call flows: [architecture.md](./architecture.md). This section documents the on-wire format.

### 3.1 Frame Format

```
┌──────────────┬──────────────┬────────────────────────────────────────┐
│  Type (1B)   │  Length (4B) │              Payload (variable)        │
└──────────────┴──────────────┴────────────────────────────────────────┘

Type:    MessageType (u8)
Length:  Payload length (u32, big-endian)
Payload: rkyv-serialized message body (not bincode)
```

### 3.2 Message Types

```rust
#[repr(u8)]
pub enum MessageType {
    Login       = 0x01,  // Client → Server: token auth
    LoginResp   = 0x02,  // Server → Client: success + ClientConfig
    Ping        = 0x04,
    Pong        = 0x05,
  ConfigPush  = 0x06,  // ctld watch: Snapshot / Delta
    RoutingInfo = 0x10,  // First message on each forwarded bidi stream
}
```

### 3.3 Core Message Structures

```rust
// Login request (token only — group derived server-side from auth store)
pub struct Login {
    pub token: String,
}

// Login response includes distributed client config
pub struct LoginResp {
    pub success: bool,
    pub error: Option<String>,
    pub config: ClientConfig,
    pub client_group: GroupId,
}

pub struct ClientConfig {
    pub config_version: String,
    pub upstreams: Vec<UpstreamConfig>,       // proxy_name → servers + lb_policy
    pub egress_rules: Vec<EgressVhostRuleDef>, // allowlist for client entry fast-fail
}

// Per-stream routing header (after open_bi on forward or reverse paths)
pub struct RoutingInfo {
    pub proxy_name: ProxyName,
    pub src_addr: String,
    pub src_port: u16,
    pub protocol: Protocol,  // H1 | H2 | Tcp | WebSocket | Unknown
    pub host: Option<String>,
}
```

Implementation: `crates/duotunnel-core/src/models/msg.rs`.

---

## 4. Connection Flow

### 4.1 Connection Establishment

```
┌────────────────┐                              ┌────────────────┐
│ Tunnel Client  │                              │ Tunnel Server  │
│                │                              │                │
│  Startup       │                              │  Listen QUIC   │
│    │           │  1. QUIC Connect             │    │           │
│    ├───────────┼─────────────────────────────►├────┤           │
│    │           │   (ALPN: "tunnel-quic/v1")   │    │           │
│    │           │                              │    │           │
│    │           │  2. TLS 1.3 Handshake        │    │           │
│    │           │◄────────────────────────────►│    │           │
│    │           │                              │    │           │
│    │           │  3. Login (open_bi)          │    │           │
│    ├───────────┼─────────────────────────────►├────┤           │
│    │           │     {client_id, group_id,    │    │ Auth      │
│    │           │      token}                  │    │ Verify    │
│    │           │                              │    │           │
│    │           │  4. LoginResp                │    │           │
│    │◄──────────┼─────────────────────────────├────┤           │
│    │           │     {success, config}        │    │           │
│    │           │                              │    │           │
│    ▼           │                              │    ▼           │
│  Ready         │                              │  Register to   │
│  Wait accept_bi│◄──────── Forward Proxy ─────│  ClientRegistry│
│  or open_bi    │──────── Reverse Proxy ─────►│               │
└────────────────┘                              └────────────────┘
```

### 4.2 Client Reconnection Mechanism

Implemented in `crates/duotunnel-client/tunnel/supervisor.rs`:

- `JitterBackoff` over `reconnect.initial_delay_ms` … `max_delay_ms` (doubling with jitter per step).
- `startup_jitter_ms` before first connect (thundering herd avoidance).
- `ConnectError::Fatal` vs `Transient` — fatal login errors stop the supervisor; transient errors retry.
- On session end: `reconnect.grace_ms` before reconnect loop resumes.

See [client-runtime.md](./client-runtime.md) and [architecture.md](./architecture.md) §5.

---

## 5. Forward Proxy (Ingress)

> Full call graph: [architecture.md](./architecture.md) §5.1.

### 5.1 Data Flow

```
External Client        Server                           Client              Local Server
      │                  │                                │                    │
      │  1. HTTP Request │                                │                    │
      │─────────────────►│                                │                    │
      │                  │  2. VHost Routing              │                    │
      │                  │  host → group_id               │                    │
      │                  │                                │                    │
      │                  │  3. open_bi() Create Stream    │                    │
      │                  │  4. Send RoutingInfo           │                    │
      │                  │───────────────────────────────►│                    │
      │                  │   {proxy_name, protocol, host} │                    │
      │                  │                                │                    │
      │                  │                                │  5. Find LocalProxy│
      │                  │                                │  proxy_name → addr │
      │                  │                                │                    │
      │                  │                                │  6. TCP Connect    │
      │                  │                                │───────────────────►│
      │                  │                                │                    │
      │                  │  7. Bidirectional Passthrough  │                    │
      │                  │◄═══════════════════════════════│◄══════════════════►│
      │  8. HTTP Response│                                │                    │
      │◄─────────────────│                                │                    │
```

### 5.2 Protocol Support

| Protocol | Server Handling | Routing Method | Data Processing |
|----------|----------------|----------------|-----------------|
| HTTP/1.1 | `handle_plaintext_h1_connection` | Host Header | Byte passthrough |
| HTTP/2 | `handle_plaintext_h2_connection` | :authority / Host | L7 proxy |
| HTTPS | `handle_tls_connection` | TLS SNI | TLS termination + H2 proxy |
| TCP | `run_tcp_listener` | Port | Pure byte passthrough |
| WebSocket | Passthrough (HTTP Upgrade) | Host Header | Byte passthrough |

### 5.3 TLS Termination and MITM

```rust
// Server-side TLS termination flow
async fn handle_tls_connection(stream: TcpStream, host: String) -> Result<()> {
    // 1. Dynamic certificate generation (cached)
    let (certs, key) = generate_self_signed_cert_for_host(&host)?;

    // 2. TLS handshake
    let tls_stream = acceptor.accept(stream).await?;

    // 3. H2 service (rewrite authority)
    H2Builder::new()
        .serve_connection(io, service)
        .await?;
}
```

---

## 6. Reverse Proxy (Egress)

### 6.1 Data Flow

```
Internal App           Client                           Server              External API
      │                  │                                │                    │
      │  1. HTTP Request │                                │                    │
      │─────────────────►│                                │                    │
      │                  │  2. Match Routing Rules        │                    │
      │                  │  host → upstream               │                    │
      │                  │                                │                    │
      │                  │  3. open_bi() Create Stream    │                    │
      │                  │  4. Send RoutingInfo           │                    │
      │                  │───────────────────────────────►│                    │
      │                  │   {protocol: "egress", host}   │                    │
      │                  │                                │                    │
      │                  │                                │  5. Connect to     │
      │                  │                                │  External Service  │
      │                  │                                │───────────────────►│
      │                  │                                │                    │
      │                  │  6. Bidirectional Passthrough  │                    │
      │                  │◄═══════════════════════════════│◄══════════════════►│
      │  7. HTTP Response│                                │                    │
      │◄─────────────────│                                │                    │
```

---

## 7. Core Components

### 7.1 ClientRegistry (Lock-free Concurrency)

```rust
pub struct ClientRegistry {
    // client_id → Connection
    clients: DashMap<String, quinn::Connection>,
    // group_id → Vec<client_id>
    groups: DashMap<String, Vec<String>>,
}

impl ClientRegistry {
    // Round-robin selection of healthy client
    pub fn select_client_for_group(&self, group_id: &str) -> Option<quinn::Connection> {
        let client_ids = self.groups.get(group_id)?;
        // Skip disconnected connections
        for id in client_ids.iter() {
            if let Some(conn) = self.clients.get(id) {
                if conn.close_reason().is_none() {
                    return Some(conn.clone());
                }
            }
        }
        None
    }
}
```

### 7.2 VhostRouter (Domain Routing)

```rust
pub struct VhostRouter<T: Clone + Send + Sync> {
    exact: DashMap<String, T>,           // Exact match
    wildcard: RwLock<Vec<(String, T)>>,  // Wildcard match
}

impl<T> VhostRouter<T> {
    pub fn get(&self, host: &str) -> Option<T> {
        // 1. Exact match (O(1))
        if let Some(v) = self.exact.get(host) {
            return Some(v.clone());
        }
        // 2. Wildcard match (*.example.com)
        let wildcards = self.wildcard.read().unwrap();
        for (pattern, value) in wildcards.iter() {
            if host.ends_with(&pattern[1..]) {
                return Some(value.clone());
            }
        }
        None
    }
}
```

### 7.3 UpstreamResolver and PeerSpec (Protocol Abstraction)

```rust
pub trait UpstreamResolver: Send + Sync {
    async fn upstream_peer(&self, ctx: &mut Context) -> Result<PeerSpec, ProxyError>;
    async fn connect_peer(&self, peer: &PeerSpec, ctx: &mut Context) -> Result<BiStream, ProxyError>;
}

// Peer specification (pure descriptor object, doesn't hold connection pools)
pub enum PeerSpec {
    Tcp(BasicPeerSpec),
    Http(HttpPeerSpec),
    MitmH2(MitmPeerSpec),
}

pub struct BasicPeerSpec {
    pub addr: SocketAddr,
    pub tls: Option<TlsConfig>,
}

pub struct HttpPeerSpec {
    pub addr: SocketAddr,
    pub host: String,
    pub scheme: Scheme,
}

pub struct MitmPeerSpec {
    pub addr: SocketAddr,
    pub host: String,
}
```

---

## 8. Concurrency Control

### 8.1 Overload Protection (current)

Overload is enforced at three layers:

1. **Inflight slow-path** (`lb/overload.rs` `maybe_slow_path`) — yield/sleep before `open_bi()` when per-connection inflight streams approach `max_concurrent_streams`
2. **Pending queue cap** (`transport/open_bi.rs` `open_bi_guarded`) — **per-connection** semaphore on `ConnectionHandle`, sized from `overload.max_pending_streams` (default: `max_concurrent_streams / 4`); acquired with `try_acquire_owned` so the cap is a hard limit rather than advisory, and rejects with `quic_open_rejected_overloaded`. The global `pending_streams` metric is reported but no longer gates admission; a process-wide budget is still missing (TODO-142)
3. **QUIC transport limits** — `max_concurrent_bidi_streams`, stream/connection windows

Client entry overload responses:

- HTTP/1.x: `503 Service Unavailable` + `Retry-After: 1`
- TCP/TLS: clean connection close

Historical note: global `quic_semaphore` / `tcp_semaphore` connection caps were removed; per-stream backpressure replaced process-wide semaphores.

### 8.2 Flow Control

- Native QUIC flow control (stream/connection level)
- `max_concurrent_bidi_streams` configuration
- Client-side `stream_semaphore` limiting

---

## 9. Configuration Format

### 9.1 Server Configuration (YAML)

```yaml
server:
  tunnel_port: 4433
  metrics_port: 9090
  login_timeout_secs: 10
  open_stream_timeout_ms: 5000
  h2_single_authority: true
  quic:
    max_concurrent_streams: 1000
    shards: 4                    # optional; default = effective_runtime_parallelism
  overload:
    inflight_yield_pct: 0.80
    inflight_sleep_pct: 0.95
    max_pending_streams: 250     # optional; default = max_concurrent_streams / 4
```

### 9.2 Routing Base Layer (YAML)

`routing.yaml` is loaded by `duotunnel-ctld` as the low-priority base layer. SQLite overrides are merged by resource key and the effective result is published to every server.

```yaml
server_egress_upstream:
  upstreams:
    external-api:
      servers:
        - address: "api.external.com:443"
  rules:
    vhost:
      - match_host: "*.external.com"
        action_upstream: "external-api"

tunnel_management:
  server_ingress_routing:
    listeners:
      - port: 8443
        mode:
          http:
            vhost:
              - match_host: "app.example.com"
                client_group: "group-a"
                proxy_name: "local-web"
  client_configs:
    groups:
      group-a:
        config_version: "v1"
        upstreams:
          local-web:
            servers:
              - address: "127.0.0.1:8080"
            lb_policy: "round_robin"
```

Egress vhost matching semantics:

- Egress vhost rules are an allowlist: a matched host forwards to the named upstream group, and an unmatched host is rejected by default.
- `action_upstream` is always an upstream group name. `reject` has no special meaning unless a normal upstream group is actually named `reject`.
- Egress host matching strips an optional `:port` suffix before comparison and lowercases domain names. For example, `Example.COM:443` and `example.com:8443` resolve to the same rule key.
- Duplicate canonical egress hosts are configuration errors, including case-only duplicates and host-with-port duplicates.
- Wildcards follow `VhostRouter` semantics: exact routes win over wildcard routes, and `*.example.com` matches `api.example.com` but not `example.com`.
- Client-side local rejection is an early fast-fail mirror of the same allowlist. If a sniffed host has no matching egress vhost rule, HTTP/1.x receives `502 Bad Gateway` with `X-DuoTunnel-Reject: no-egress-route`; TLS/Other connections receive a clean EOF. Connections with no sniffed host continue to the server for final routing.

### 9.3 Client Configuration (YAML)

```yaml
server_addr: "tunnel.example.com"
server_port: 4433
auth_token: "dt_replace_with_token"

entry:
  port: 8080                   # optional local HTTP/TCP entry (forward proxy)
  # accept_workers: 4          # optional; default = effective_runtime_parallelism

# udp_entries:                 # optional local UDP entries
#   - port: 5353
#     proxy_name: "dns-proxy"

quic:
  connections: 0               # 0 = auto (effective_runtime_parallelism)
  max_concurrent_streams: 1000 # code default when omitted: 100
  # shards: 4

tls_skip_verify: false
reconnect:
  login_timeout_ms: 10000
  open_stream_timeout_ms: 5000
```

Upstream routing for reverse proxy is distributed from server `tunnel_management.client_configs` at login time; the client does not hardcode local proxy maps in YAML.

---

## 10. Monitoring Metrics

### Prometheus Metrics

```
# Connection metrics
duotunnel_quic_connections_total      # Total QUIC connections
duotunnel_tcp_connections_total       # Total TCP connections
duotunnel_connections_rejected_total  # Rejected connections

# Client metrics
duotunnel_clients_registered_total    # Registered clients
duotunnel_clients_unregistered_total  # Unregistered clients
duotunnel_duplicate_clients_total     # Duplicate ClientID handling

# Authentication metrics
duotunnel_auth_success_total{group}   # Authentication successes
duotunnel_auth_failure_total{group}   # Authentication failures

# Request metrics
duotunnel_requests_total{type,status} # Total requests (tcp/http, success/error)
```

---

## 11. Security Features

### 11.1 Transport Security

- **QUIC TLS 1.3**: All tunnel traffic encrypted
- **ALPN**: `tunnel-quic/v1` protocol identifier (generation-scoped; a breaking wire change takes a new generation so incompatible peers fail at the QUIC handshake). Handshake additionally negotiates `protocol_version` + capability bits — see `crates/duotunnel-core/src/models/msg.rs`.
- **Certificate Verification**: Supports custom CA or system certificates

### 11.2 Authentication Mechanism

```yaml
# Server-side configuration
auth_tokens:
  group-a: "sha256-hashed-token"

# Client-side configuration
auth_token: "sha256-hashed-token"
```

### 11.3 Duplicate ClientID Handling

```rust
// When new connection detects duplicate ClientID
if let Some(existing) = registry.get_client(&client_id) {
    existing.close(0u32.into(), b"duplicate client");  // Close old connection
    registry.unregister(&client_id);
}
registry.register(client_id, group_id, new_conn);
```

---

## 12. Design Principles

### 12.1 Principles Followed

| Principle | Implementation |
|-----------|----------------|
| **Single Responsibility** | Clear module separation: transport/protocol/proxy/engine |
| **Open/Closed** | Trait extension: `UpstreamPeer`, `ProxyApp` |
| **Dependency Inversion** | Depend on abstractions, not concrete implementations |
| **Zero-Copy** | `tokio::io::copy` for direct relay |
| **Lock-Free Concurrency** | DashMap instead of RwLock<HashMap> |

### 12.2 Performance Characteristics

- **On-Demand Stream Creation**: No connection pool overhead
- **Protocol Detection**: `peek()` avoids data copying
- **Connection Reuse**: Single QUIC connection multiplexing
- **Lazy Certificate Loading**: MITM certificates generated on demand and cached

---

## 13. Future Optimizations

### High Priority
- [ ] Extract MITM implementation to separate module
- [ ] Unify protocol detection logic
- [ ] Add Circuit Breaker

### Medium Priority
- [ ] Abstract LoadBalancer trait
- [ ] Add health check mechanism
- [ ] Custom error types

### Low Priority
- [ ] Remove unused ProtocolDriver trait
- [ ] Add performance benchmarks
- [ ] Hot configuration reload

---

## References

- [frp GitHub](https://github.com/fatedier/frp)
- [Quinn QUIC Library](https://github.com/quinn-rs/quinn)
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [TLS SNI RFC 6066](https://datatracker.ietf.org/doc/html/rfc6066#section-3)
