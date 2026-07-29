# Simple Bidirectional Tunnel Example

Minimal setup demonstrating DuoTunnel's bidirectional traffic — ingress and egress on a single QUIC connection.

## Scenario

**Ingress** (reverse proxy — external request routed through tunnel to a local backend):
```
External → Server:8001 → QUIC tunnel → Client → echo.free.beeceptor.com
```

**Egress** (forward proxy — local app tunnelled out through the server):
```
Local app → Client:8002 → QUIC tunnel → Server → echo.free.beeceptor.com
```

## Quick Start

### Automated test

```bash
cd ci-helpers/local-test/examples/simple-tunnel
bash test.sh
```

The script builds binaries if needed, starts ctld + server + client, tests both directions, then cleans up.

### Manual setup

**1. Start the control daemon:**
```bash
./target/release/duotunnel-ctld --config ci-helpers/local-test/examples/simple-tunnel/ctld.yaml
```

**2. Create a token:**
```bash
TOKEN=$(./target/release/duotunnel-ctld --config ci-helpers/local-test/examples/simple-tunnel/ctld.yaml \
  client create test-group | grep '^Token:' | awk '{print $2}')
```

**3. Start the server:**
```bash
./target/release/duotunnel-server \
  --config ci-helpers/local-test/examples/simple-tunnel/server.yaml \
  --ctld-addr 127.0.0.1:7799
```

**4. Start the client** (in another terminal):
```bash
DUOTUNNEL_CLIENT__AUTH_TOKEN="$TOKEN" \
  ./target/release/duotunnel-client --config ci-helpers/local-test/examples/simple-tunnel/client.yaml
```

**5. Test ingress** (server → client):
```bash
curl -H "Host: localhost" http://localhost:8001/
```

**6. Test egress** (client → server):
```bash
curl -H "Host: echo.free.beeceptor.com" http://localhost:8002/
```

## How It Works

```
┌─────────────┐    watch stream    ┌─────────────────┐
│ duotunnel-ctld │◄──────────────►│ duotunnel-server │
│  ctld :7799    │ routing+tokens │ :10086 QUIC      │
└─────────────┘                    │  :8001 ingress  │
                                   └────────┬────────┘
                                            │ QUIC tunnel
                                   ┌────────▼────────┐
                                   │ duotunnel-client │
                                   │  :8002 egress   │
                                   └────────┬────────┘
                                            │
                                   echo.free.beeceptor.com
```

### Ingress flow

```
curl → Server:8001 → VHost match "localhost" → QUIC stream → Client
                                                               → echo.free.beeceptor.com:443
```

1. Request arrives at server ingress listener `:8001`
2. Server matches `Host: localhost` → `test-group` / `echo_service` (from ctld routing)
3. Server opens a QUIC stream to the client, sends routing metadata
4. Client resolves `echo_service` upstream → `echo.free.beeceptor.com:443`
5. Client forwards request; response travels back over the same QUIC stream

### Egress flow

```
curl → Client:8002 → QUIC stream → Server → echo.free.beeceptor.com:443
```

1. Request arrives at client's local HTTP entry `:8002`
2. Client opens a QUIC stream to the server, sends routing metadata
3. Server matches `Host: echo.free.beeceptor.com` → `echo_backend` upstream (from ctld routing)
4. Server connects to `echo.free.beeceptor.com:443` and forwards the request

## Configuration Files

**[server.yaml](server.yaml)** — server tuning only (routing lives in ctld):
- `server.tunnel_port: 10086` — QUIC port clients connect to

**[routing.yaml](routing.yaml)** — YAML base layer for ingress and egress routing.

**[client.yaml](client.yaml)** — connects to server, exposes egress on `:8002`

**[ctld.yaml](ctld.yaml)** — control daemon (SQLite + YAML source + watch address)

## Key Observations

- **Single QUIC connection** — both ingress and egress streams share port 10086
- **Stream multiplexing** — each request is an independent QUIC stream (no head-of-line blocking)
- **Hot routing** — routing changes in ctld are pushed to all connected servers without restart
- **Config distribution** — server pushes client upstream config to clients on login
- **Protocol detection** — server and client auto-detect HTTP/1.1, HTTP/2, TLS SNI, WebSocket

## Troubleshooting

- Ensure ports `7788`, `8001`, `8002`, `10086` are free before starting
- Check logs: `/tmp/duotunnel-ctld.log`, `/tmp/duotunnel-server.log`, `/tmp/duotunnel-client.log`
- Ingress test requires `Host: localhost` to match the vhost rule
- Egress test requires `Host: echo.free.beeceptor.com` to match the server egress rule
