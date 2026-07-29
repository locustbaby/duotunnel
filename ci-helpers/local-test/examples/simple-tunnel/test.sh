#!/bin/bash
# Test script for bidirectional tunnel (unified control-plane topology)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/../../../../"
cd "$ROOT"

echo "=== DuoTunnel Bidirectional Test ==="
echo ""

# Build binaries if needed
if [ ! -f "$ROOT/target/release/duotunnel-ctld" ] || \
   [ ! -f "$ROOT/target/release/duotunnel-server" ] || \
   [ ! -f "$ROOT/target/release/duotunnel-client" ]; then
    echo "Building binaries..."
    cd "$ROOT" && cargo build --release
    echo "✓ Build completed"
else
    echo "✓ Binaries found"
fi

CTLD="$ROOT/target/release/duotunnel-ctld"
SERVER="$ROOT/target/release/duotunnel-server"
CLIENT="$ROOT/target/release/duotunnel-client"

mkdir -p "$SCRIPT_DIR/data"

cleanup() {
    kill "${CTLD_PID:-}" "${SERVER_PID:-}" "${CLIENT_PID:-}" 2>/dev/null || true
}
trap cleanup EXIT

# ── Start ctld ────────────────────────────────────────────────────────────────
echo "Starting duotunnel-ctld..."
"$CTLD" --config "$SCRIPT_DIR/ctld.yaml" > /tmp/duotunnel-ctld.log 2>&1 &
CTLD_PID=$!
echo "ctld started (PID: $CTLD_PID)"

for i in $(seq 1 20); do
    grep -q "starting duotunnel-ctld" /tmp/duotunnel-ctld.log 2>/dev/null && break
    sleep 0.2
done

# Create token
TOKEN=$("$CTLD" --config "$SCRIPT_DIR/ctld.yaml" \
    client create test-group | grep '^Token:' | awk '{print $2}')
if [ -z "$TOKEN" ]; then
    echo "ERROR: failed to obtain token"
    exit 1
fi
echo "✓ Token created"

# ── Start server ──────────────────────────────────────────────────────────────
echo "Starting tunnel server..."
"$SERVER" --config "$SCRIPT_DIR/server.yaml" --ctld-addr 127.0.0.1:7799 \
    > /tmp/duotunnel-server.log 2>&1 &
SERVER_PID=$!
echo "Server started (PID: $SERVER_PID)"

for i in $(seq 1 20); do
    grep -q "QUIC server listening" /tmp/duotunnel-server.log 2>/dev/null && break
    sleep 0.3
done

# ── Start client ──────────────────────────────────────────────────────────────
echo "Starting tunnel client..."
DUOTUNNEL_CLIENT__AUTH_TOKEN="$TOKEN" \
  "$CLIENT" --config "$SCRIPT_DIR/client.yaml" > /tmp/duotunnel-client.log 2>&1 &
CLIENT_PID=$!
echo "Client started (PID: $CLIENT_PID)"

for i in $(seq 1 30); do
    grep -q "Login successful" /tmp/duotunnel-client.log 2>/dev/null && break
    sleep 0.5
done

if ! grep -q "Login successful" /tmp/duotunnel-client.log; then
    echo "ERROR: client did not log in"
    echo "=== ctld ===" && cat /tmp/duotunnel-ctld.log
    echo "=== server ===" && cat /tmp/duotunnel-server.log
    echo "=== client ===" && cat /tmp/duotunnel-client.log
    exit 1
fi
echo "✓ Client connected"
echo ""

# ── Test 1: Ingress ───────────────────────────────────────────────────────────
echo "=== Test 1: Ingress (Server:8001 → QUIC → Client → beeceptor) ==="
RESPONSE=$(curl -sf --max-time 10 -H "Host: localhost" http://localhost:8001/ 2>&1 || true)

if echo "$RESPONSE" | grep -qi "beeceptor\|echo"; then
    echo "✅ Ingress PASSED"
else
    echo "❌ Ingress FAILED"
    echo "Response: $RESPONSE"
fi

sleep 0.5

# ── Test 2: Egress ────────────────────────────────────────────────────────────
echo ""
echo "=== Test 2: Egress (Client:8002 → QUIC → Server → beeceptor) ==="
RESPONSE=$(curl -sf --max-time 10 \
    -H "Host: echo.free.beeceptor.com" http://localhost:8002/ 2>&1 || true)

if echo "$RESPONSE" | grep -qi "beeceptor\|echo"; then
    echo "✅ Egress PASSED"
else
    echo "❌ Egress FAILED"
    echo "Response: $RESPONSE"
fi

echo ""
echo "Logs: /tmp/duotunnel-{ctld,server,client}.log"
