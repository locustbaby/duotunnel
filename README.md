# DuoTunnel

A high-performance bidirectional tunnel proxy system built on QUIC protocol in Rust.

Inspired by [frp](https://github.com/fatedier/frp), but leveraging QUIC's native multiplexing for significantly lower latency and simpler architecture.

## Why QUIC?

| Feature | frp (TCP + Yamux) | DuoTunnel (QUIC) |
|---------|-------------------|------------------|
| Stream Creation | Client initiates TCP connection | Server calls `open_bi()` directly |
| Handshake Roundtrips | 3 messages | 1 message |
| New Stream Latency | >= 1.5 RTT | 0 RTT |
| Connection Pool | Required (pre-created) | Not needed (on-demand) |
| Multiplexing | Yamux over TCP | Native QUIC streams |
| 0-RTT Resumption | Not supported | Natively supported |
| Connection Migration | Not supported | Natively supported |

## Features

- **Bidirectional Forwarding** - Both Server → Client (ingress) and Client → Server (egress)
- **Multi-Protocol** - HTTP/1.1, HTTP/2, HTTPS (TLS/SNI), WebSocket, raw TCP
- **Unified VHost Routing** - O(1) exact match + wildcard pattern support
- **TLS Termination** - Dynamic self-signed certificate generation with caching
- **Group-Based Management** - Multiple client groups with round-robin load balancing
- **Lock-Free Concurrency** - DashMap-based registry, zero-copy relay
- **Auto Reconnection** - Exponential backoff (1s → 60s)
- **Config Distribution** - Server pushes routing config to clients on login
- **Prometheus Metrics** - Connection, authentication, and request metrics

## Supported Protocols

| Protocol | Direction | Routing Method |
|----------|-----------|----------------|
| HTTP/1.1 | Bidirectional | Host header |
| HTTP/2 | Bidirectional | `:authority` / Host |
| HTTPS | Ingress | TLS SNI |
| WebSocket | Ingress | Host header (HTTP Upgrade) |
| TCP | Ingress | Port-based |

## Architecture

```
                    External Clients
                     │            │
                     │ HTTP/S/WS  │ HTTP
                     ▼            ▼
┌────────────────────────┐   ┌────────────────────────┐
│     Tunnel Server      │   │     Tunnel Client      │
│                        │   │                        │
│  Entry Listener        │   │  Entry Listener        │
│  (HTTP/TCP/TLS)        │   │  (HTTP, optional)      │
│         │              │   │         │              │
│         ▼              │   │         ▼              │
│  VHost Router     ◄════════════►  Forwarder         │
│  Client Registry  QUIC │   │  LocalProxyMap         │
│         │       Tunnel │   │         │              │
│         ▼              │   │         ▼              │
│  Server Egress         │   │  Upstream Pool         │
└─────────┬──────────────┘   └─────────┬──────────────┘
          │                            │
          ▼                            ▼
   External Backend              Local Service
```

**Ingress** (reverse proxy):
```
External Client → Server → VHost Match → QUIC Stream → Client → Local Service
```

**Egress** (forward proxy):
```
Local App → Client Entry → QUIC Stream → Server → External Service
```

## Quick Start

### Simple Ingress Tunnel Example

Expose a local web service (e.g., `localhost:8080`) to the internet through a public server:

**Server (public IP: `example.com`):**
```yaml
# config/server.yaml
server:
  tunnel_port: 10086
  entry_port: 8001

tunnel_management:
  server_ingress_routing:
    rules:
      vhost:
        - match_host: "app.example.com"
          action_client_group: "group-a"

  client_configs:
    client_egress_routings:
      group-a:
        upstreams:
          local_web:
            servers:
              - address: "127.0.0.1:8080"
        rules:
          vhost:
            - match_host: "app.example.com"
              action_upstream: "local_web"
```

**Client (private network):**
```yaml
# config/client.yaml
server_addr: "example.com"
server_port: 10086
client_group_id: "group-a"
```

**Run:**
```bash
# On server
./server --config config/server.yaml

# On client
./client --config config/client.yaml

# Access your local service at http://app.example.com:8001
```

Traffic flow: `Internet → example.com:8001 → QUIC tunnel → Client → localhost:8080`

---

### Build from Source

**Prerequisites:** Rust 1.70+

```bash
cargo build --release
```

### Server Configuration (`config/server.yaml`)

```yaml
server:
  config_version: "v1.0.0"
  tunnel_port: 10086        # QUIC tunnel port
  entry_port: 8001          # HTTP/TCP entry port
  log_level: "info"

server_egress_upstream:
  upstreams:
    backend:
      servers:
        - address: "https://api.example.com"
          resolve: true
      lb_policy: "round_robin"
  rules:
    http:
      - match_host: "api.example.com"
        action_upstream: "backend"

tunnel_management:
  server_ingress_routing:
    rules:
      vhost:
        - match_host: "app.example.com"
          action_client_group: "group-a"
      tcp:
        - match_port: 50051
          action_client_group: "group-a"
          action_proxy_name: "grpc_service"

  client_configs:
    client_egress_routings:
      group-a:
        config_version: "v1.0.0"
        upstreams:
          local_web:
            servers:
              - address: "127.0.0.1:8080"
                resolve: false
            lb_policy: "round_robin"
        rules:
          vhost:
            - match_host: "app.example.com"
              action_upstream: "local_web"
```

### Client Configuration (`config/client.yaml`)

```yaml
log_level: "info"
tls_skip_verify: true       # Set false in production
server_addr: "127.0.0.1"
server_port: 10086
tls_server_name: "example.com" # Optional SNI override when server_addr is IP
client_group_id: "group-a"
auth_token: "your-token-here"
http_entry_port: 8003       # Optional: egress entry
reconnect:
  initial_delay_ms: 1000
  max_delay_ms: 60000
  connect_timeout_ms: 10000
  resolve_timeout_ms: 5000
  login_timeout_ms: 5000
```

### Run

```bash
# Start server
./target/release/server --config config/server.yaml

# Start client
./target/release/client --config config/client.yaml

# Verify
sh verify_proxy.sh
```

## Project Structure

```
tunnel/
├── tunnel-lib/          # Core library: QUIC transport, protocol detection, proxy engine
│   └── src/
│       ├── models/      # Message protocol (bincode serialization)
│       ├── transport/   # QUIC config, VhostRouter, address resolution
│       ├── protocol/    # Protocol detection (TLS/H2/H1/WS), header rewriting
│       ├── proxy/       # ProxyEngine, UpstreamPeer trait, HTTP/H2/TCP peers
│       ├── engine/      # Bidirectional relay, QUIC-TCP bridge
│       ├── egress/      # Outbound HTTP client
│       └── infra/       # PKI (cert generation), observability (tracing)
├── server/              # Server binary
│   ├── handlers/        # QUIC, HTTP, TCP entry handlers
│   ├── registry.rs      # Lock-free client registry (DashMap)
│   └── config.rs        # YAML config parsing
├── client/              # Client binary
│   ├── app.rs           # LocalProxyMap (routing)
│   ├── proxy.rs         # Stream handler
│   └── entry.rs         # Local egress entry listener
└── config/              # Example YAML configurations
```

## Tech Stack

- [quinn](https://github.com/quinn-rs/quinn) - QUIC protocol (ALPN: `tunnel-quic`)
- [rustls](https://github.com/rustls/rustls) - TLS 1.3 (ring provider)
- [hyper](https://github.com/hyperium/hyper) - HTTP/1.1 & HTTP/2
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [dashmap](https://github.com/xacrimon/dashmap) - Lock-free concurrent maps
- [bincode](https://github.com/bincode-org/bincode) - Binary message serialization
- [rcgen](https://github.com/est31/rcgen) - Dynamic certificate generation

## Use Cases

**Intranet Penetration** - Expose private services through a public server
```
Internet → Server (Public) → QUIC Tunnel → Client (Private) → Local Service
```

**Forward Proxy** - Route internal traffic through the tunnel to external services
```
Local App → Client → QUIC Tunnel → Server → External API
```

**WebSocket Proxy** - Forward WSS connections through the tunnel
```
WSS Client → Server → QUIC Tunnel → Client → Local WS Service
```

## Acknowledgments

DuoTunnel is inspired by [frp](https://github.com/fatedier/frp) (Fast Reverse Proxy), a popular and mature tunnel solution. We learned from frp's architecture and design principles, particularly its client-server model and proxy routing mechanisms.

### Key Innovations Beyond frp

While building upon frp's foundation, DuoTunnel introduces several architectural improvements:

1. **Server-to-Client Config Distribution**
   - **frp**: Clients use local configuration files only. Dynamic updates require manual file edits and `frpc reload`.
   - **DuoTunnel**: Server pushes routing configurations to clients via `LoginResp` messages. Clients receive and apply routing rules automatically without local config files.
   - **Benefit**: Centralized configuration management, instant updates across client groups.

2. **Bidirectional Tunnel (Ingress + Egress)**
   - **frp**: Primarily a reverse proxy (ingress only). Exposes internal services to the internet.
   - **DuoTunnel**: Supports both directions simultaneously:
     - **Ingress**: External → Server → Client → Backend (like frp)
     - **Egress**: Local → Client → Server → External (forward proxy)
   - **Benefit**: Single tunnel for both inbound and outbound traffic, unified architecture.

3. **QUIC-Native Multiplexing**
   - **frp**: Uses TCP + Yamux for stream multiplexing, requires connection pools and pre-created connections.
   - **DuoTunnel**: Leverages QUIC's native multiplexing with 0-RTT stream creation (see comparison table above).
   - **Benefit**: Lower latency, simpler code, automatic connection migration, built-in 0-RTT resumption.

We're grateful to the frp team for pioneering this space and providing a solid reference implementation.

## Performance Tuning

### Optimised Release Build

```bash
cargo build --release
# Binaries are built with:
#   -C target-cpu=native   (AES-NI, AVX2, SHA-NI auto-enabled via .cargo/config.toml)
#   lto = "fat"            (whole-program inlining across all crates)
#   codegen-units = 1
#   panic = "abort"
```

### Profile-Guided Optimisation (PGO) — +10–20% throughput

PGO feeds real traffic profiles to LLVM for branch-prediction and inlining decisions.

```bash
bash scripts/pgo-build.sh
# Interactive: builds instrumented binaries → prompts you to run traffic → rebuilds with profiles
```

### OS / Kernel Tuning — +20–40% at high concurrency

Apply sysctl, IRQ affinity, and CPU-governor tuning with a single script:

```bash
sudo bash scripts/tune-os.sh          # auto-detects NIC
sudo bash scripts/tune-os.sh eth0     # explicit NIC
```

What it sets:
| Setting | Value | Effect |
|---------|-------|--------|
| `net.core.rmem_max` | 16 MB | Matches 4 MB socket buffers (kernel doubles) |
| `net.core.somaxconn` | 65535 | Large accept queue (matches SO_REUSEPORT backlog) |
| `net.ipv4.tcp_tw_reuse` | 1 | Fast TIME_WAIT reuse for outbound proxy connections |
| `net.core.busy_poll` | 50 µs | Reduced median latency |
| `net.ipv4.tcp_fastopen` | 3 | Save 1 RTT on first connection |
| THP | madvise | Eliminate compaction latency spikes |
| CPU governor | performance | Prevent frequency ramp-up latency |
| NIC ring buffers | max | No packet drops under burst |
| RPS/RFS | all CPUs | Software multi-queue on single-queue NICs |
| IRQ affinity | 1 core/queue | Eliminate cross-core interrupt bounce |

To persist across reboots, copy the sysctl lines to `/etc/sysctl.d/99-tunnel.conf`.

### In-code Optimisations (already applied)

| Optimisation | Location | Benefit |
|---|---|---|
| `SO_REUSEPORT` + backlog=4096 | `transport/listener.rs` | 4–8× multi-core accept scaling |
| `SO_KEEPALIVE` + `TCP_USER_TIMEOUT` | `transport/tcp_params.rs` | Fast dead-connection detection |
| 4 MB default socket buffers | `transport/tcp_params.rs` | High-BDP link throughput |
| BBR congestion control (QUIC) | `transport/quic.rs` | Lower queue, better WAN throughput |
| 4/32 MB QUIC windows | `transport/quic.rs` | No flow-control stalls |
| `target-cpu=native` | `.cargo/config.toml` | Hardware AES-NI / AVX2 / SHA-NI |
| `lto=fat` + `panic=abort` | `Cargo.toml` | ~5–15% cross-crate inlining gain |
| jemalloc global allocator | `server/main.rs`, `client/main.rs` | Lower fragmentation under load |
| `PeerKind` enum dispatch | `proxy/peers.rs` | Zero `Box<dyn>` on hot path |
| H1 header batch write | `protocol/driver/h1.rs` | 15–20 syscalls → 1 per response |
| `BytesMut` single-alloc read | `proxy/core.rs`, `h1.rs` | Eliminates double-copy on recv |
| `parking_lot::RwLock` + `CachePadded` | `server/registry.rs` | Reduces false-sharing |
| SoA `ClientGroup` layout | `server/registry.rs` | Hot path touches only connections |
| `send_message` single `write_all` | `models/msg.rs` | 3 syscalls → 1 per frame |
| QUIC GSO/GRO (auto) | `quinn-udp` | Batch UDP I/O on Linux ≥ 5.4 |

## References

- [frp](https://github.com/fatedier/frp) - Original inspiration
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [TLS SNI RFC 6066](https://datatracker.ietf.org/doc/html/rfc6066#section-3)

## License

MIT
