#!/bin/bash
# Test script for bidirectional tunnel (ctld-managed mode)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/../../.."

echo "=== DuoTunnel Bidirectional Test ==="
echo ""

# Build binaries if needed
if [ ! -f "$ROOT/target/release/tunnel-ctld" ] || \
   [ ! -f "$ROOT/target/release/server" ] || \
   [ ! -f "$ROOT/target/release/client" ]; then
    echo "Building binaries..."
    cd "$ROOT" && cargo build --release
    echo "✓ Build completed"
else
    echo "✓ Binaries found"
fi

cd "$SCRIPT_DIR"

CTLD="$ROOT/target/release/tunnel-ctld"
SERVER="$ROOT/target/release/server"
CLIENT="$ROOT/target/release/client"

mkdir -p "$SCRIPT_DIR/data"

cleanup() {
    kill "${CTLD_PID:-}" "${SERVER_PID:-}" "${CLIENT_PID:-}" 2>/dev/null || true
}
trap cleanup EXIT

# ── Start ctld ────────────────────────────────────────────────────────────────
echo "Starting tunnel-ctld..."
# Use a temp ctld config pointing at a local DB
cat > /tmp/example-ctld.yaml <<'EOF'
database_url: "sqlite:///tmp/duotunnel-example.db?mode=rwc"
watch_addr: "127.0.0.1:7799"
log_level: "info"
EOF

"$CTLD" --config /tmp/example-ctld.yaml > /tmp/tunnel-ctld.log 2>&1 &
CTLD_PID=$!
echo "ctld started (PID: $CTLD_PID)"

for i in $(seq 1 20); do
    grep -q "starting tunnel-ctld" /tmp/tunnel-ctld.log 2>/dev/null && break
    sleep 0.2
done

# Seed routing
"$CTLD" --config /tmp/example-ctld.yaml \
    client import-routing --from "$SCRIPT_DIR/server.yaml"
echo "✓ Routing seeded"

# Create token
TOKEN=$("$CTLD" --config /tmp/example-ctld.yaml \
    client create-client test-group | grep '^Token:' | awk '{print $2}')
if [ -z "$TOKEN" ]; then
    echo "ERROR: failed to obtain token"
    exit 1
fi
echo "✓ Token created"

# ── Start server ──────────────────────────────────────────────────────────────
echo "Starting tunnel server..."
"$SERVER" --config "$SCRIPT_DIR/server.yaml" --ctld-addr 127.0.0.1:7799 \
    > /tmp/tunnel-server.log 2>&1 &
SERVER_PID=$!
echo "Server started (PID: $SERVER_PID)"

for i in $(seq 1 20); do
    grep -q "QUIC server listening" /tmp/tunnel-server.log 2>/dev/null && break
    sleep 0.3
done

# ── Start client ──────────────────────────────────────────────────────────────
echo "Starting tunnel client..."
# Patch auth_token in a temp copy of client.yaml
TEMP_CLIENT=$(mktemp /tmp/example-client-XXXX.yaml)
python3 -c "
import re, sys
src = open('$SCRIPT_DIR/client.yaml').read()
src = re.sub(r'^auth_token:.*', 'auth_token: \"$TOKEN\"', src, flags=re.MULTILINE)
open(sys.argv[1], 'w').write(src)
" "$TEMP_CLIENT"

"$CLIENT" --config "$TEMP_CLIENT" > /tmp/tunnel-client.log 2>&1 &
CLIENT_PID=$!
echo "Client started (PID: $CLIENT_PID)"

for i in $(seq 1 30); do
    grep -q "Login successful" /tmp/tunnel-client.log 2>/dev/null && break
    sleep 0.5
done

if ! grep -q "Login successful" /tmp/tunnel-client.log; then
    echo "ERROR: client did not log in"
    echo "=== ctld ===" && cat /tmp/tunnel-ctld.log
    echo "=== server ===" && cat /tmp/tunnel-server.log
    echo "=== client ===" && cat /tmp/tunnel-client.log
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
echo "Logs: /tmp/tunnel-{ctld,server,client}.log"
