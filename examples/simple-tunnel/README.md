# Simple Bidirectional Tunnel Example

This is the minimal configuration to test DuoTunnel's **bidirectional** functionality - both ingress and egress in a single setup.

## Scenario

DuoTunnel supports traffic in **both directions** simultaneously:

**Ingress** (reverse proxy):
```
External Request → Server:8001 → QUIC Tunnel → Client → echo.free.beeceptor.com
```

**Egress** (forward proxy):
```
Local App → Client:8002 → QUIC Tunnel → Server → echo.free.beeceptor.com
```

## Quick Start

### Automated Test

```bash
cd examples/simple-tunnel
bash test.sh
```

The script will:
1. Build binaries if needed
2. Start server and client
3. Test both ingress and egress directions
4. Show results and clean up

### Manual Setup

1. **Start the tunnel server**:
   ```bash
   ./target/release/server --config examples/simple-tunnel/server.yaml
   ```

2. **Start the tunnel client** (in another terminal):
   ```bash
   ./target/release/client --config examples/simple-tunnel/client.yaml
   ```

3. **Test ingress** (server → client):
   ```bash
   curl -H "Host: localhost" http://localhost:8001/
   ```

4. **Test egress** (client → server):
   ```bash
   curl -H "Host: echo.free.beeceptor.com" http://localhost:8002/
   ```

## How It Works

### Architecture

```
┌─────────────┐         QUIC Tunnel          ┌─────────────┐
│   Server    │◄────────────────────────────►│   Client    │
│             │                               │             │
│ :8001 Entry │  Ingress: external → client  │ Forward to  │
│ :10086 QUIC │                               │ backend     │
│             │  Egress: client → external   │             │
│ Forward to  │                               │ :8002 Entry │
│ backend     │                               │             │
└─────────────┘                               └─────────────┘
       │                                              │
       │                                              │
       ▼                                              ▼
echo.free.beeceptor.com                echo.free.beeceptor.com
```

### Request Flow Details

#### Ingress Flow (Reverse Proxy)

**Example:** `curl -H "Host: localhost" http://localhost:8001/`

```
┌─────────┐    ①     ┌─────────┐    ③     ┌─────────┐    ⑤     ┌──────────────┐
│  curl   │─────────►│ Server  │─────────►│ Client  │─────────►│ echo.free.   │
│         │          │ :8001   │          │         │          │ beeceptor.com│
└─────────┘          └─────────┘          └─────────┘          └──────────────┘
     ▲                    │                     │                      │
     │                    │ ② VHost Match       │ ④ Server Config     │
     │                    │    "localhost"      │    Applied          │
     │                    │    → test-group     │                     │
     │                    ▼                     ▼                     ▼
     │               QUIC Stream           Forward HTTP          HTTPS :443
     │               Created               Request               Connection
     │                                                                │
     └────────────────────────────────────────────────────────────────┘
                              ⑥ Response flows back
```

**Step-by-step:**

1. **Request arrives at Server**
   - TCP connection accepted on `:8001`
   - HTTP/1.1 request received: `GET / HTTP/1.1\r\nHost: localhost\r\n...`
   - Protocol detected: HTTP/1.1 (via initial bytes inspection)

2. **Server VHost Routing**
   - Extract `Host: localhost` header
   - Match against `server_ingress_routing.rules.vhost`
   - Rule matched: `match_host: "localhost"` → `action_client_group: "test-group"`
   - Select client from group (round-robin if multiple clients)

3. **QUIC Stream Creation**
   - Server opens new QUIC bidirectional stream to selected client
   - Send `RoutingInfo` message (protocol type, target host, metadata)
   - Forward HTTP request bytes over QUIC stream

4. **Client Processing**
   - Client accepts QUIC stream from server
   - Receive `RoutingInfo` message
   - Apply server-distributed `client_egress_routing` config for `test-group`
   - Match `Host: localhost` against client's vhost rules
   - Rule matched: `match_host: "localhost"` → `action_upstream: "echo_service"`
   - Resolve upstream: `echo.free.beeceptor.com:443`

5. **Backend Connection**
   - Client establishes HTTPS connection to `echo.free.beeceptor.com:443`
   - Rewrite `Host` header (if needed) or forward as-is
   - Send HTTP request to backend
   - Receive HTTP response

6. **Response Path**
   - Client writes response bytes to QUIC stream
   - Server reads from QUIC stream
   - Server forwards response to original TCP connection on `:8001`
   - curl receives and displays response

---

#### Egress Flow (Forward Proxy)

**Example:** `curl -H "Host: echo.free.beeceptor.com" http://localhost:8002/`

```
┌─────────┐    ①     ┌─────────┐    ③     ┌─────────┐    ⑤     ┌──────────────┐
│  curl   │─────────►│ Client  │─────────►│ Server  │─────────►│ echo.free.   │
│         │          │ :8002   │          │         │          │ beeceptor.com│
└─────────┘          └─────────┘          └─────────┘          └──────────────┘
     ▲                    │                     │                      │
     │                    │ ② No local match    │ ④ VHost Match       │
     │                    │    Forward to       │    "echo.free..."   │
     │                    │    server           │    → echo_backend   │
     │                    ▼                     ▼                     ▼
     │               QUIC Stream           Server Egress          HTTPS :443
     │               Created               Routing                Connection
     │                                                                │
     └────────────────────────────────────────────────────────────────┘
                              ⑥ Response flows back
```

**Step-by-step:**

1. **Request arrives at Client**
   - TCP connection accepted on `:8002` (client's `http_entry_port`)
   - HTTP/1.1 request received: `GET / HTTP/1.1\r\nHost: echo.free.beeceptor.com\r\n...`
   - Protocol detected: HTTP/1.1

2. **Client Decision**
   - Extract `Host: echo.free.beeceptor.com` header
   - Check client's local egress routing rules (from server-distributed config)
   - No matching rule found (client config has empty vhost rules)
   - Decision: Forward to server via QUIC tunnel

3. **QUIC Stream Creation**
   - Client opens new QUIC bidirectional stream to server
   - Send `RoutingInfo` message (protocol=HTTP, host=echo.free.beeceptor.com)
   - Forward HTTP request bytes over QUIC stream

4. **Server Egress Routing**
   - Server accepts QUIC stream from client
   - Receive `RoutingInfo` message
   - Apply `server_egress_upstream.rules.http` routing
   - Match `Host: echo.free.beeceptor.com` against rules
   - Rule matched: `match_host: "echo.free.beeceptor.com"` → `action_upstream: "echo_backend"`
   - Resolve upstream: `echo.free.beeceptor.com:443`

5. **Backend Connection**
   - Server establishes HTTPS connection to `echo.free.beeceptor.com:443`
   - Forward HTTP request to backend
   - Receive HTTP response

6. **Response Path**
   - Server writes response bytes to QUIC stream
   - Client reads from QUIC stream
   - Client forwards response to original TCP connection on `:8002`
   - curl receives and displays response

---

### Key Observations

- **Single QUIC Connection**: Both ingress and egress use the same persistent QUIC connection (port 10086)
- **Stream Multiplexing**: Each HTTP request creates a new QUIC stream (no head-of-line blocking)
- **Zero-RTT Streams**: After initial handshake, new streams have ~0 RTT overhead
- **Unified Routing**: VHost rules work consistently across both directions
- **Config Distribution**: Server pushes client routing config via `LoginResp` message
- **Protocol Detection**: Both server and client auto-detect HTTP/1.1, HTTP/2, TLS SNI, WebSocket
- **Transparent Proxying**: Original HTTP semantics preserved (headers, methods, status codes)

## Configuration Highlights

**Server** ([server.yaml](server.yaml)):
- Ingress routing: `localhost` → `test-group` clients
- Egress routing: `echo.free.beeceptor.com` → `echo_backend` upstream
- Distributes client egress config to clients in `test-group`

**Client** ([client.yaml](client.yaml)):
- Connects to server's QUIC tunnel on port 10086
- Listens on port 8002 for local egress requests
- Receives and applies server-distributed routing config

## Troubleshooting

- Ensure ports 8001, 8002, and 10086 are free
- Check logs: `/tmp/tunnel-server.log`, `/tmp/tunnel-client.log`
- Verify Host header matches vhost rules
- For ingress: use `Host: localhost`
- For egress: use `Host: echo.free.beeceptor.com`

## Key Features Demonstrated

- ✅ **Bidirectional** traffic (ingress + egress)
- ✅ **QUIC-based** multiplexing (single connection)
- ✅ **VHost routing** (Host header-based)
- ✅ **Server-to-client config distribution**
- ✅ **External service backends** (no localhost dependencies)
