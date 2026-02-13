# DuoTunnel Design Document

> High-performance bidirectional tunnel proxy system based on QUIC (Quinn)
>
> Inspired by frp design philosophy, implementing transparent tunneling + configuration distribution + grouping + Rules-based routing

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
├── tunnel-lib/                    # Core library (~2,600 LOC)
│   └── src/
│       ├── models/msg.rs          # Message protocol definitions
│       ├── transport/             # QUIC/TCP transport layer
│       │   ├── listener.rs        # VhostRouter, PortRouter
│       │   ├── quinn_io.rs        # QUIC stream adapter
│       │   └── addr.rs            # Address resolution
│       ├── protocol/              # Protocol handling
│       │   ├── detect.rs          # Protocol detection (TLS/HTTP/H2)
│       │   └── rewrite.rs         # Header rewriting
│       ├── proxy/                 # Proxy core
│       │   ├── core.rs            # ProxyApp trait
│       │   ├── peers.rs           # UpstreamPeer trait
│       │   ├── tcp.rs             # TCP passthrough
│       │   ├── http.rs            # HTTP/1.1 proxy
│       │   └── h2.rs              # HTTP/2 proxy
│       ├── engine/                # Data engine
│       │   ├── relay.rs           # Bidirectional data relay
│       │   └── bridge.rs          # QUIC-TCP bridge
│       ├── egress/                # Outbound proxy
│       │   └── http.rs            # HTTP client
│       └── infra/                 # Infrastructure
│           ├── pki.rs             # Certificate generation (MITM)
│           └── observability.rs   # Logging and tracing
│
├── server/                        # Server (~1,400 LOC)
│   ├── main.rs                    # Entry point
│   ├── config.rs                  # Configuration parsing
│   ├── registry.rs                # ClientRegistry (DashMap)
│   ├── egress.rs                  # Outbound routing
│   └── handlers/
│       ├── quic.rs                # QUIC connection handling
│       ├── http.rs                # HTTP entry (H1/H2/TLS)
│       ├── tcp.rs                 # TCP entry
│       └── metrics.rs             # Prometheus metrics
│
└── client/                        # Client (~1,100 LOC)
    ├── main.rs                    # Entry + reconnection logic
    ├── config.rs                  # Configuration parsing
    ├── app.rs                     # LocalProxyMap
    ├── proxy.rs                   # Workflow handling
    └── entry.rs                   # Reverse proxy entry
```

---

## 3. Message Protocol

### 3.1 Frame Format

```
┌──────────────┬──────────────┬────────────────────────────────────────┐
│  Type (1B)   │  Length (4B) │              Payload (variable)        │
└──────────────┴──────────────┴────────────────────────────────────────┘

Type:    Message type (u8)
Length:  Payload length (u32, big-endian)
Payload: bincode serialized message body
```

### 3.2 Message Types

```rust
#[repr(u8)]
pub enum MessageType {
    Login      = 0x01,  // Client → Server: Login authentication
    LoginResp  = 0x02,  // Server → Client: Login response + config distribution
    RoutingInfo= 0x10,  // Workflow routing information
    Ping       = 0x04,  // Heartbeat
    Pong       = 0x05,  // Heartbeat response
    ConfigPush = 0x06,  // Config push (reserved)
}
```

### 3.3 Core Message Structures

```rust
// Login request
pub struct Login {
    pub client_id: String,
    pub group_id: Option<String>,
    pub token: String,
}

// Login response (includes config distribution)
pub struct LoginResp {
    pub success: bool,
    pub error: Option<String>,
    pub config: ClientConfig,
}

// Client configuration
pub struct ClientConfig {
    pub config_version: String,
    pub proxies: Vec<ProxyConfig>,      // Proxy definitions
    pub upstreams: Vec<UpstreamConfig>, // Upstream services
    pub rules: Vec<RuleConfig>,         // Routing rules
}

// Routing information (workflow)
pub struct RoutingInfo {
    pub proxy_name: String,
    pub src_addr: String,
    pub src_port: u16,
    pub protocol: String,  // "http", "h2", "tcp", "ws"
    pub host: Option<String>,
}
```

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
│    │           │     (ALPN: "tunnel-quic")    │    │           │
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

```rust
// Exponential backoff reconnection
let mut retry_delay = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);

loop {
    match run_client(&config).await {
        Ok(_) => {
            retry_delay = Duration::from_secs(1);  // Reset
        }
        Err(e) => {
            tokio::time::sleep(retry_delay).await;
            retry_delay = min(retry_delay * 2, MAX_RETRY_DELAY);  // Exponential backoff
        }
    }
}
```

---

## 5. Forward Proxy (Ingress)

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

### 7.3 UpstreamPeer (Protocol Abstraction)

```rust
#[async_trait]
pub trait UpstreamPeer: Send {
    async fn connect_and_proxy(
        self: Box<Self>,
        quic_send: quinn::SendStream,
        quic_recv: quinn::RecvStream,
        initial_data: Option<&[u8]>,
    ) -> Result<()>;
}

// Implementations
pub struct TcpPeer { addr: String }
pub struct TlsTcpPeer { addr: String, host: String }
pub struct HttpPeer { addr: String, client: Client }
pub struct H2Peer { addr: String }
```

---

## 8. Concurrency Control

### 8.1 Connection Limits (Semaphore)

```rust
pub struct ServerState {
    // QUIC connection limit
    pub quic_semaphore: Arc<Semaphore>,  // default: 10000
    // TCP connection limit
    pub tcp_semaphore: Arc<Semaphore>,   // default: 10000
}

// Usage
let permit = state.tcp_semaphore.clone().try_acquire_owned()?;
tokio::spawn(async move {
    let _permit = permit;  // Hold until completion
    handle_connection(stream).await;
});
```

### 8.2 Flow Control

- Native QUIC flow control (stream/connection level)
- `max_concurrent_bidi_streams` configuration
- Client-side `stream_semaphore` limiting

---

## 9. Configuration Format

### 9.1 Server Configuration (YAML)

```yaml
server:
  tunnel_port: 4433        # QUIC tunnel port
  entry_port: 8443         # HTTP entry port
  max_connections: 10000   # Max QUIC connections
  max_tcp_connections: 10000
  metrics_port: 9090       # Prometheus metrics
  auth_tokens:
    group-a: "secret-token-a"
    group-b: "secret-token-b"

# Server egress proxy configuration
server_egress_upstream:
  upstreams:
    external-api:
      servers:
        - address: "api.external.com:443"
  rules:
    http:
      - match_host: "*.external.com"
        action_upstream: "external-api"

# Tunnel management configuration
tunnel_management:
  # Inbound routing (external → private network)
  server_ingress_routing:
    rules:
      vhost:
        - match_host: "app.example.com"
          action_client_group: "group-a"
        - match_host: "*.internal.com"
          action_client_group: "group-b"
      tcp:
        - match_port: 2222
          action_client_group: "group-a"
          action_proxy_name: "ssh"

  # Client configuration distribution
  client_configs:
    client_egress_routings:
      group-a:
        config_version: "v1"
        upstreams:
          local-web:
            servers:
              - address: "127.0.0.1:8080"
        rules:
          vhost:
            - match_host: "app.example.com"
              action_upstream: "local-web"
```

### 9.2 Client Configuration (YAML)

```yaml
server_host: "tunnel.example.com"
server_port: 4433
client_id: "client-001"
client_group_id: "group-a"
auth_token: "secret-token-a"

# TLS configuration
tls_skip_verify: false       # Should be false in production
tls_ca_cert: "/path/to/ca.pem"  # Optional: custom CA

# Performance configuration
max_concurrent_streams: 100

# Reverse proxy entry (optional)
http_entry_port: 8080

# Local proxy mapping (can be overridden by server distribution)
local_proxies:
  app.example.com: "127.0.0.1:3000"
  ssh: "127.0.0.1:22"
```

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
- **ALPN**: `tunnel-quic` protocol identifier
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
