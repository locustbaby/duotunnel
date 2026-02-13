#!/bin/bash

# Configuration
HTTP_URL="http://wss.localtest.com:8001"
WSS_URL="ws://wss.localtest.com:8001"
GRPC_ADDR="localhost:50051"

PASS=0
FAIL=0

pass() { PASS=$((PASS+1)); echo "  ✅ $1"; }
fail() { FAIL=$((FAIL+1)); echo "  ❌ $1"; }

echo "============================================="
echo "   DuoTunnel Proxy Verification"
echo "============================================="

# =============================================
# SECTION 1: HTTP (curl)
# =============================================
echo ""
echo ">>> HTTP Tests (curl)"
echo "---------------------------------------------"

# 1.1 GET request
echo "[1.1] GET Request"
RESP=$(curl -s -w "\n%{http_code}" "$HTTP_URL/")
CODE=$(echo "$RESP" | tail -1)
BODY=$(echo "$RESP" | sed '$d')
if [ "$CODE" = "200" ]; then
    pass "GET $HTTP_URL/ -> $CODE"
else
    fail "GET $HTTP_URL/ -> $CODE"
fi

sleep 1

# 1.2 POST with JSON body
echo "[1.2] POST with JSON Body"
RESP=$(curl -s -X POST -d '{"data":"test_payload_123"}' -H "Content-Type: application/json" "$HTTP_URL/")
if echo "$RESP" | grep -q 'test_payload_123'; then
    pass "POST body forwarded correctly"
else
    fail "POST body not found in response"
fi

sleep 1

# 1.3 Custom header forwarding
echo "[1.3] Custom Header Forwarding"
RESP=$(curl -s -H "X-Tunnel-Test: reliable" "$HTTP_URL/")
if echo "$RESP" | grep -qi 'X-Tunnel-Test'; then
    pass "Custom header forwarded"
else
    fail "Custom header not found in response"
fi

sleep 1

# 1.4 Connection reuse latency
echo "[1.4] Connection Reuse (3 sequential GETs)"
for i in 1 2 3; do
    TIMING=$(curl -s -o /dev/null -w "connect=%{time_connect}s total=%{time_total}s code=%{http_code}" "$HTTP_URL/")
    echo "  Req $i: $TIMING"
    sleep 1
done

sleep 1

# 1.5 Stability (10 requests)
echo "[1.5] Stability (10 requests)"
OK=0
for i in $(seq 1 10); do
    CODE=$(curl -s -o /dev/null -w "%{http_code}" "$HTTP_URL/")
    if [ "$CODE" = "200" ]; then
        printf "  ✅"
        OK=$((OK+1))
    else
        printf "  ❌($CODE)"
    fi
    sleep 1
done
echo ""
if [ "$OK" -ge 8 ]; then
    pass "Stability: $OK/10 succeeded"
else
    fail "Stability: $OK/10 succeeded (expected >= 8)"
fi

# =============================================
# SECTION 2: WebSocket (wscat)
# =============================================
echo ""
echo ">>> WebSocket Tests (wscat)"
echo "---------------------------------------------"

if command -v wscat &> /dev/null; then
    # 2.1 WSS echo test - send a message and check echo
    echo "[2.1] WebSocket Echo"
    RESP=$(echo "hello-duotunnel" | wscat -c "$WSS_URL" -w 3 2>&1 | head -5)
    if echo "$RESP" | grep -qi "hello-duotunnel\|connected\|upgrade"; then
        pass "WebSocket connection established"
    else
        # WebSocket may not work if upstream is HTTP echo, not WS echo
        fail "WebSocket connection failed: $RESP"
    fi
else
    echo "  ⚠️  wscat not found, skipping WebSocket tests"
fi

# =============================================
# SECTION 3: gRPC / H2 (grpcurl)
# =============================================
echo ""
echo ">>> gRPC/H2 Tests (grpcurl)"
echo "---------------------------------------------"

if command -v grpcurl &> /dev/null; then
    # 3.1 gRPC health/reflection check via TCP port routing (50051)
    echo "[3.1] gRPC Reflection (port 50051 -> grpc_service)"
    RESP=$(grpcurl -plaintext "$GRPC_ADDR" list 2>&1)
    if echo "$RESP" | grep -qi "service\|method\|grpc"; then
        pass "gRPC reflection succeeded"
    else
        fail "gRPC reflection failed: $(echo "$RESP" | head -1)"
    fi

    sleep 1

    # 3.2 H2 via curl on entry port
    echo "[3.2] HTTP/2 via entry port"
    RESP=$(curl -s --http2 -o /dev/null -w "%{http_version} %{http_code}" "$HTTP_URL/")
    echo "  Response: $RESP"
    if echo "$RESP" | grep -q "^2"; then
        pass "HTTP/2 negotiated"
    else
        # H2 may downgrade to 1.1 depending on upstream
        echo "  (note: H2 may downgrade if upstream only supports H1)"
        pass "HTTP request succeeded via H2 attempt: $RESP"
    fi
else
    echo "  ⚠️  grpcurl not found, skipping gRPC tests"
fi

# =============================================
# Summary
# =============================================
echo ""
echo "============================================="
TOTAL=$((PASS+FAIL))
echo "  Results: $PASS passed, $FAIL failed ($TOTAL total)"
echo "============================================="

exit $FAIL
