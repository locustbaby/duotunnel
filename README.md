# DuoTunnel

A high-performance bidirectional tunnel proxy built on QUIC in Rust.

Inspired by [frp](https://github.com/fatedier/frp), but using QUIC's native stream multiplexing for lower latency and a centralised control plane for routing management.

## Why QUIC?

| Feature | frp (TCP + Yamux) | DuoTunnel (QUIC) |
|---------|-------------------|------------------|
| Stream creation | Client initiates TCP connection | Server calls `open_bi()` directly |
| Handshake roundtrips | 3 messages | 1 message |
| New stream latency | ≥ 1.5 RTT | ~0 RTT |
| Connection pool | Required (pre-created) | Not needed (on-demand) |
| Multiplexing | Yamux over TCP | Native QUIC streams |
| 0-RTT resumption | Not supported | Natively supported |
| Connection migration | Not supported | Natively supported |

## Features

- **Bidirectional** — ingress (server → client → backend) and egress (client → server → external) on one tunnel
- **Multi-protocol** — HTTP/1.1, HTTP/2, HTTPS/TLS-SNI, WebSocket, raw TCP
- **Centralised control plane** — `tunnel-ctld` owns all routing rules and token lifecycle; the server is a pure data-plane process
- **Hot reload** — routing changes pushed to all connected servers and clients without restart
- **VHost routing** — O(1) host-header dispatch with wildcard support
- **TLS termination** — dynamic self-signed certificate generation and caching
- **Group-based auth** — client groups with round-robin load balancing, token revocation, rotation
- **Prometheus metrics** — connections, auth, requests

## Architecture

```
                ┌─────────────────────────────────────┐
                │          tunnel-ctld                │
                │  ┌─────────────┐  ┌──────────────┐ │
                │  │  Rule store │  │  Token store │ │
                │  └─────────────┘  └──────────────┘ │
                │        watch stream (:7788)         │
                └──────────────┬──────────────────────┘
                               │ config push (on connect + change)
                               ▼
┌──────────────────────────────────────────────────────┐
│                  tunnel server                       │
│                                                      │
│  Ingress listeners (HTTP/TCP/TLS)                    │
│       │  VHost router  ◄──── routing snapshot        │
│       │  Client registry                             │
│       │                          QUIC :10086 ──────────────►  tunnel client
│       ▼                                              │            │
│  Server egress (forward to external)                 │       Local services
└──────────────────────────────────────────────────────┘
```

**Ingress** (reverse proxy — server → client → local service):
```
External → Server listener → VHost match → QUIC stream → Client → Backend
```

**Egress** (forward proxy — client → server → external):
```
Local app → Client HTTP entry → QUIC stream → Server → External service
```

## Components

| Binary | Description |
|--------|-------------|
| `tunnel-ctld` | Control daemon — routing rules, token management, watch stream |
| `server` | Data-plane tunnel server — QUIC endpoint, ingress listeners, egress forwarder |
| `client` | Tunnel client — connects to server, forwards to local backends |

## Quick Start

### Recommended: ctld-managed mode

This is the normal production setup. The control daemon owns all configuration; the server is stateless apart from its own tuning parameters.

**Step 1 — Start the control daemon:**
```bash
./target/release/tunnel-ctld --config ctld.yaml
```

**Step 2 — Seed routing config from your server.yaml (first boot):**
```bash
./target/release/tunnel-ctld --config ctld.yaml \
  client import-routing --from config/server.yaml
```

**Step 3 — Create a token for your client group:**
```bash
./target/release/tunnel-ctld --config ctld.yaml \
  client create-client my-group
# → Created client 'my-group'
# → Token: dt_xxxxxxxxxxxxxxxx
```

**Step 4 — Start the server pointing at ctld:**
```bash
./target/release/server --config config/server.yaml --ctld-addr 127.0.0.1:7788
```

**Step 5 — Start the client:**
```bash
./target/release/client --config config/client.yaml
```

---

### Standalone mode (single machine / dev)

The server reads routing config from its own SQLite database (seeded from `server.yaml` on first boot). No ctld required.

```bash
# Start server (self-contained — manages its own DB)
./target/release/server --config config/server.yaml

# Create a token
./target/release/server --config config/server.yaml token create --name my-group

# Start client
./target/release/client --config config/client.yaml
```

---

## Configuration

### `ctld.yaml` — control daemon

```yaml
database_url: "sqlite://./data/duotunnel.db?mode=rwc"
watch_addr: "0.0.0.0:7788"   # server connects here
log_level: "info"
```

### `config/server.yaml` — server tuning

In ctld-managed mode the server only reads the `server.*` block. The `tunnel_management` and `server_egress_upstream` blocks are used for standalone mode and for `import-routing`.

```yaml
server:
  tunnel_port: 10086          # QUIC tunnel port (clients connect here)
  log_level: "info"
  # database_url only needed in standalone mode
  database_url: "sqlite://./data/duotunnel.db?mode=rwc"

  # Optional tuning
  quic:
    max_concurrent_streams: 1000
    keepalive_secs: 20
    idle_timeout_secs: 60
  tcp:
    nodelay: true
  http_pool:
    idle_timeout_secs: 90
    max_idle_per_host: 10
  proxy_buffers:
    peek_buf_size: 16384
  pki:
    cert_cache_ttl_secs: 3600

# ── Routing sections (used in standalone mode + import-routing) ───────────────

server_egress_upstream:
  upstreams:
    ext_api:
      servers:
        - address: "api.example.com:443"
          resolve: true
      lb_policy: "round_robin"
  rules:
    vhost:
      - match_host: "api.example.com"
        action_upstream: "ext_api"

tunnel_management:
  server_ingress_routing:
    listeners:
      - port: 8001
        mode: http
        vhost:
          - match_host: "app.example.com"
            client_group: "my-group"
            proxy_name: "local_web"
      - port: 50051
        mode: tcp
        client_group: "my-group"
        proxy_name: "grpc_service"

  client_configs:
    groups:
      my-group:
        config_version: "v1.0.0"
        upstreams:
          local_web:
            servers:
              - address: "127.0.0.1:8080"
                resolve: false
            lb_policy: "round_robin"
          grpc_service:
            servers:
              - address: "127.0.0.1:9090"
                resolve: false
            lb_policy: "round_robin"
```

### `config/client.yaml` — tunnel client

```yaml
log_level: "info"
tls_skip_verify: true         # set false in production with a real cert
server_addr: "example.com"
server_port: 10086
auth_token: "dt_xxxxxxxxxxxxxxxx"

# Optional: expose a local HTTP entry for egress (forward proxy)
http_entry_port: 8003

# Optional tuning
quic:
  connections: 1
  max_concurrent_streams: 100
tcp:
  nodelay: true
http_pool:
  idle_timeout_secs: 90
  max_idle_per_host: 10
reconnect:
  initial_delay_ms: 1000
  max_delay_ms: 60000
  connect_timeout_ms: 10000
```

---

## Token management (`tunnel-ctld client ...`)

```bash
# Create a new client group and print its token
tunnel-ctld client create-client <name>

# Rotate (invalidate old + issue new)
tunnel-ctld client rotate-token <name>

# Revoke all tokens for a group
tunnel-ctld client revoke-client <name>

# List all clients and token status
tunnel-ctld client list-tokens

# Seed routing from a server.yaml
tunnel-ctld client import-routing --from config/server.yaml
```

---

## Build

**Prerequisites:** Rust 1.80+

```bash
cargo build --release
# Binaries: target/release/{tunnel-ctld,server,client}
```

---

## Project Structure

```
tunnel/
├── tunnel-lib/          # Core: QUIC transport, protocol detection, proxy engine
├── tunnel-store/        # Storage traits + SQLite implementations (auth + rules)
├── tunnel-service/      # tunnel-ctld binary — control daemon + CLI
├── server/              # server binary — data-plane tunnel server
│   ├── handlers/        # QUIC, HTTP, TCP ingress handlers
│   ├── egress.rs        # Server-side egress forwarder
│   ├── registry.rs      # Lock-free client registry (DashMap)
│   ├── listener_mgr.rs  # Ingress listener lifecycle management
│   └── control_client.rs # ctld watch stream consumer
├── client/              # client binary
│   ├── app.rs           # LocalProxyMap — client-side routing
│   ├── proxy.rs         # Stream handler
│   └── entry.rs         # Local HTTP egress entry listener
├── config/              # Example YAML configs
└── ci-helpers/          # CI configs and example setups
```

---

## Tech Stack

- [quinn](https://github.com/quinn-rs/quinn) — QUIC (ALPN: `tunnel-quic`)
- [rustls](https://github.com/rustls/rustls) — TLS 1.3
- [hyper](https://github.com/hyperium/hyper) — HTTP/1.1 & HTTP/2
- [tokio](https://github.com/tokio-rs/tokio) — async runtime
- [dashmap](https://github.com/xacrimon/dashmap) — lock-free concurrent maps
- [arc-swap](https://github.com/vorner/arc-swap) — lock-free routing snapshot swap
- [sqlx](https://github.com/launchbadge/sqlx) — SQLite (auth + rule stores)
- [rcgen](https://github.com/est31/rcgen) — dynamic certificate generation

---

## Use Cases

**Intranet penetration** — expose private services through a public server:
```
Internet → Server (public) → QUIC tunnel → Client (private) → Local service
```

**Forward proxy** — route internal traffic through the tunnel to external services:
```
Local app → Client → QUIC tunnel → Server → External API
```

**WebSocket / gRPC proxy** — protocol-aware forwarding with header rewriting:
```
WS/gRPC client → Server → QUIC tunnel → Client → Local WS/gRPC service
```

---

## Performance Tuning

### Optimised release build

```bash
cargo build --release
# .cargo/config.toml enables: target-cpu=native, lto=fat, codegen-units=1, panic=abort
```

### Profile-Guided Optimisation (+10–20% throughput)

```bash
bash scripts/pgo-build.sh
```

### OS / kernel tuning (+20–40% at high concurrency)

```bash
sudo bash scripts/tune-os.sh          # auto-detects NIC
sudo bash scripts/tune-os.sh eth0     # explicit NIC
```

Key settings applied:

| Setting | Value | Effect |
|---------|-------|--------|
| `net.core.rmem_max` | 16 MB | Larger UDP receive buffers for QUIC |
| `net.core.somaxconn` | 65535 | Large accept queue |
| `net.ipv4.tcp_tw_reuse` | 1 | Fast TIME_WAIT reuse for egress |
| `net.core.busy_poll` | 50 µs | Reduced median latency |
| THP | madvise | Eliminate compaction spikes |
| CPU governor | performance | Prevent frequency ramp-up latency |
| NIC ring buffers | max | No packet drops under burst |

### In-code optimisations (already applied)

| Optimisation | Location | Benefit |
|---|---|---|
| `SO_REUSEPORT` + backlog=4096 | `transport/listener.rs` | Multi-core accept scaling |
| BBR congestion control | `transport/quic.rs` | Lower queue, better WAN throughput |
| 4/32 MB QUIC flow-control windows | `transport/quic.rs` | No stalls on high-BDP links |
| `ArcSwap` routing snapshot | `server/main.rs` | Lock-free config hot-swap |
| `DashMap` client registry | `server/registry.rs` | Sharded concurrent access |
| `CachePadded` counters | `server/registry.rs` | Eliminate false sharing |
| `PeerKind` enum dispatch | `proxy/peers.rs` | Zero `Box<dyn>` on hot path |
| Peek-buffer pool | `server/main.rs` | Reuse fixed-size buffers, avoid per-conn alloc |
| jemalloc global allocator | `server/main.rs`, `client/main.rs` | Lower fragmentation under load |
| QUIC GSO/GRO (auto) | `quinn-udp` | Batched UDP I/O on Linux ≥ 5.4 |

---

## Comparison with frp

| Capability | frp | DuoTunnel |
|---|---|---|
| Routing config location | Client-side YAML | ctld (centralised, hot-reload) |
| New stream latency | ≥ 1.5 RTT | ~0 RTT (QUIC native) |
| Bidirectional (ingress + egress) | Ingress only | Both directions |
| Token management | Manual config | ctld CLI (create/rotate/revoke) |
| Config distribution | Manual file sync | Pushed to clients on login |
| Connection multiplexing | Yamux over TCP | Native QUIC streams |

---

## References

- [frp](https://github.com/fatedier/frp) — original inspiration
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [TLS SNI RFC 6066](https://datatracker.ietf.org/doc/html/rfc6066#section-3)

## License

Apache-2.0
