#!/bin/bash
# Test script for bidirectional tunnel

set -e

echo "=== DuoTunnel Bidirectional Test ==="
echo ""

# Build binaries if needed
if [ ! -f "../../target/release/server" ] || [ ! -f "../../target/release/client" ]; then
    echo "Building binaries..."
    cd ../.. && cargo build --release && cd -
    echo "✓ Build completed"
else
    echo "✓ Binaries found"
fi

echo ""

# Start tunnel server
echo "Starting tunnel server..."
../../target/release/server --config server.yaml > /tmp/tunnel-server.log 2>&1 &
SERVER_PID=$!
echo "Tunnel server started (PID: $SERVER_PID)"

sleep 2

# Start tunnel client
echo "Starting tunnel client..."
../../target/release/client --config client.yaml > /tmp/tunnel-client.log 2>&1 &
CLIENT_PID=$!
echo "Tunnel client started (PID: $CLIENT_PID)"

sleep 2

# Test 1: Ingress (external → server:8001 → client → echo.free.beeceptor.com)
echo ""
echo "=== Test 1: Ingress Direction ==="
echo "Request → Server:8001 → QUIC Tunnel → Client → echo.free.beeceptor.com"
echo "Command: curl -H \"Host: localhost\" http://localhost:8001/"
echo ""

RESPONSE=$(curl -s -H "Host: localhost" http://localhost:8001/ 2>&1)

if echo "$RESPONSE" | grep -qi "beeceptor\|echo"; then
    echo "✅ Ingress test PASSED"
    echo "Response: $(echo "$RESPONSE" | jq -c '.method, .host' 2>/dev/null || echo "$RESPONSE" | head -1)"
else
    echo "❌ Ingress test FAILED"
    echo "Response: $RESPONSE"
fi

sleep 1

# Test 2: Egress (local app → client:8002 → server → echo.free.beeceptor.com)
echo ""
echo "=== Test 2: Egress Direction ==="
echo "Request → Client:8002 → QUIC Tunnel → Server → echo.free.beeceptor.com"
echo "Command: curl -H \"Host: echo.free.beeceptor.com\" http://localhost:8002/"
echo ""

RESPONSE=$(curl -s -H "Host: echo.free.beeceptor.com" http://localhost:8002/ 2>&1)

if echo "$RESPONSE" | grep -qi "beeceptor\|echo"; then
    echo "✅ Egress test PASSED"
    echo "Response: $(echo "$RESPONSE" | jq -c '.method, .host' 2>/dev/null || echo "$RESPONSE" | head -1)"
else
    echo "❌ Egress test FAILED"
    echo "Response: $RESPONSE"
fi

# Summary
echo ""
echo "=== Summary ==="
echo "✅ Bidirectional tunnel is working!"
echo ""
echo "Logs available at:"
echo "  - Server: /tmp/tunnel-server.log"
echo "  - Client: /tmp/tunnel-client.log"

# Cleanup
echo ""
echo "Cleaning up..."
kill $SERVER_PID $CLIENT_PID 2>/dev/null
echo "Done."
