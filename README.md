# Tunnel - QUIC-Based Bidirectional Tunnel Proxy

## Overview

Tunnel is a high-performance bidirectional tunnel proxy system built on the QUIC protocol, supporting HTTP and WebSocket Secure (WSS) protocol forwarding.

### Core Features

- 🚀 **QUIC-Based**: Leverages QUIC protocol's multiplexing, low latency, and connection migration capabilities
- 🔄 **Bidirectional Forwarding**: Supports both Server → Client and Client → Server traffic forwarding
- 🌐 **HTTP Support**: Complete HTTP/1.1 bidirectional proxy functionality
- 🔌 **WSS Support**: WebSocket Secure (Server → Client) forwarding
- 📊 **Smart Routing**: Host-based rule matching and traffic routing
- 🎯 **Connection Pooling**: High-performance connection pool management with connection reuse

## Supported Protocols

### ✅ Implemented and Tested

| Protocol | Direction | Status | Description |
|----------|-----------|--------|-------------|
| HTTP | Bidirectional | ✅ Tested | Supports both Server → Client and Client → Server |
| WSS | Server → Client | ✅ Tested | WebSocket Secure forwarding |

### ❌ No Longer Supported

| Protocol | Reason |
|----------|--------|
| gRPC | No longer planned for support |
| WSS (Client → Server) | Reverse WSS no longer planned for support |

## Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      External Clients                            │
└────────────┬────────────────────────────────────┬────────────────┘
             │                                    │
             │ HTTP/WSS                           │ HTTP
             ▼                                    ▼
┌─────────────────────┐                 ┌─────────────────────┐
│   Tunnel Server     │                 │   Tunnel Client     │
│  ┌───────────────┐  │                 │  ┌───────────────┐  │
│  │ HTTP Ingress  │  │                 │  │ HTTP Ingress  │  │
│  │ WSS Ingress   │  │                 │  └───────────────┘  │
│  └───────────────┘  │                 │         │           │
│         │           │                 │         ▼           │
│         ▼           │                 │  ┌───────────────┐  │
│  ┌───────────────┐  │    QUIC Tunnel  │  │   Forwarder   │  │
│  │ Rule Matcher  │  │ ◄─────────────► │  └───────────────┘  │
│  └───────────────┘  │                 │         │           │
│         │           │                 │         ▼           │
│         ▼           │                 │  ┌───────────────┐  │
│  ┌───────────────┐  │                 │  │  Egress Pool  │  │
│  │Client Selector│  │                 │  └───────────────┘  │
│  └───────────────┘  │                 └─────────┬───────────┘
│         │           │                           │
│         ▼           │                           │
│  ┌───────────────┐  │                           │
│  │ Data Stream   │  │                           │
│  └───────────────┘  │                           │
└─────────┬───────────┘                           │
          │                                       │
          │ HTTP (Egress)                         │ HTTP
          ▼                                       ▼
┌─────────────────────┐                 ┌─────────────────────┐
│  External Backend   │                 │  Local Service      │
└─────────────────────┘                 └─────────────────────┘
```

### Data Flow

#### 1. HTTP Bidirectional Forwarding

**Server → Client (Reverse Proxy)**
```
External Client → Server Ingress → Rule Match → Select Client 
→ QUIC Tunnel → Client Forwarder → Local Service
```

**Client → Server (Forward Proxy)**
```
Local Client → Client Ingress → QUIC Tunnel → Server Data Stream 
→ External Backend
```

#### 2. WSS Forwarding (Server → Client)

```
External WSS Client → Server WSS Ingress → Rule Match → Select Client 
→ QUIC Tunnel → Client WSS Handler → Local WSS Service
```

## Quick Start

### Prerequisites

- Rust 1.70+
- OpenSSL or rustls

### Build

```bash
cargo build --release
```

### Configuration

#### Server Configuration (`config/server.yaml`)

```yaml
server:
  config_version: "v1.0.0"
  tunnel_port: 10086          # QUIC tunnel port
  http_entry_port: 8001       # HTTP entry port
  wss_entry_port: 8444        # WSS entry port (optional)
  log_level: "info"

# Server egress upstream (Server → External Backend)
server_egress_upstream:
  upstreams:
    backend:
      servers:
        - address: "https://www.google.com"
          resolve: true
      lb_policy: "round_robin"
  
  rules:
    http:
      - match_host: "example.com"
        action_upstream: "backend"

# Tunnel management
tunnel_management:
  # Server ingress routing (External → Server → Client)
  server_ingress_routing:
    rules:
      http:
        - match_host: "app.example.com"
          action_client_group: "group-a"
      
      wss:
        - match_host: "ws.example.com"
          action_client_group: "group-a"
  
  # Client configuration distribution
  client_configs:
    client_egress_routings:
      group-a:
        config_version: "v1.0.0"
        upstreams:
          local_service:
            servers:
              - address: "http://127.0.0.1:8080"
                resolve: false
            lb_policy: "round_robin"
          
          wss_service:
            servers:
              - address: "wss://echo.websocket.org"
                resolve: false
            lb_policy: "round_robin"
        
        rules:
          http:
            - match_host: "app.example.com"
              action_upstream: "local_service"
          
          wss:
            - match_host: "ws.example.com"
              action_upstream: "wss_service"
```

#### Client Configuration (`config/client.yaml`)

```yaml
log_level: "info"
server_addr: "127.0.0.1"
server_port: 10086
client_group_id: "group-a"
auth_token: "your-token-here"

# Optional: Enable HTTP listener (Client → Server)
http_entry_port: 8003

# Note: gRPC and WSS (Client → Server) are no longer supported
```

### Running

#### Start Server

```bash
./target/release/server --config config/server.yaml
```

#### Start Client

```bash
./target/release/client --config config/client.yaml
```

## Use Cases

### 1. Intranet Penetration (Server → Client)

Forward external traffic to internal services:

```
Internet → Server (Public IP) → QUIC Tunnel → Client (Private Network) → Local Service
```

**Configuration Example**:
- Server listens on `http_entry_port: 8001`
- Configure `server_ingress_routing` rules to match Host
- Client receives traffic and forwards to local service

### 2. Forward Proxy (Client → Server)

Internal clients access external services through the tunnel:

```
Local Client → Client Ingress → QUIC Tunnel → Server → External Service
```

**Configuration Example**:
- Client listens on `http_entry_port: 8003`
- Server configures `server_egress_upstream` to define external services
- Traffic is forwarded through the tunnel to external services

### 3. WebSocket Proxy

Forward WebSocket Secure connections:

```
WSS Client → Server WSS Ingress → QUIC Tunnel → Client → Local WSS Service
```

## Technical Details

### QUIC Transport Layer

- Uses `quinn` library for QUIC protocol implementation
- Supports multiplexing with multiple streams over a single connection
- Automatic reconnection and connection migration

### Protocol Frame Format

```rust
pub enum TunnelFrame {
    Register(RegisterPayload),      // Client registration
    Config(ConfigPayload),           // Configuration distribution
    Routing(RoutingInfo),            // Routing information
    Data(Vec<u8>),                   // Data transfer
    Close,                           // Connection close
}
```

### Connection Pool Management

- Supports HTTP/1.1 Keep-Alive
- Smart connection reuse
- Idle connection timeout cleanup
- Connection health checks

### Rule Matching

- Host-based exact matching
- O(1) lookup performance (using HashMap)
- Supports dynamic rule updates

## Performance Optimizations

- ✅ Lock-free concurrency using `DashMap`
- ✅ Lock-free QUIC connection access with `ArcSwapOption`
- ✅ Connection pool warmup and reuse
- ✅ Zero-copy data transfer
- ✅ Asynchronous I/O and streaming

## Logging and Monitoring

### Log Levels

```yaml
log_level: "info"  # trace, debug, info, warn, error
```

### Key Logs

- Client registration and connection status
- Rule matching results
- Traffic forwarding statistics
- Errors and exceptions

## Troubleshooting

### Common Issues

1. **Connection Failure**
   - Check if Server's `tunnel_port` is reachable
   - Verify firewall rules
   - Ensure QUIC UDP port is not blocked

2. **Rule Mismatch**
   - Check if Host header is correct
   - Verify `match_host` in rule configuration
   - Review rule matching results in logs

3. **Forwarding Timeout**
   - Check if upstream service is reachable
   - Verify connection pool configuration
   - Review network latency

## Roadmap

### Completed ✅

- [x] QUIC tunnel establishment
- [x] HTTP bidirectional forwarding
- [x] WSS Server → Client forwarding
- [x] Rule matching and routing
- [x] Connection pool management
- [x] Configuration hot reload

### No Longer Supported ❌

- [ ] gRPC support
- [ ] WSS Client → Server reverse forwarding

### Future Plans 🚧

- [ ] Monitoring and metrics collection
- [ ] Web management interface
- [ ] Multi-tenancy support
- [ ] Traffic rate limiting and QoS
- [ ] Additional load balancing strategies

## License

MIT License

## Contributing

Issues and Pull Requests are welcome!

## Contact

- GitHub Issues: [Project URL]
- Email: [Contact Email]

