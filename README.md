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
client_group_id: "group-a"
auth_token: "your-token-here"
http_entry_port: 8003       # Optional: egress entry
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

## References

- [frp](https://github.com/fatedier/frp) - Original inspiration
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [TLS SNI RFC 6066](https://datatracker.ietf.org/doc/html/rfc6066#section-3)

## License

MIT
