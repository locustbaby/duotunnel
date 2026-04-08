#!/usr/bin/env bash
# ci-helpers/local-integration-test.sh
# Mirrors CI integration-test job using *.localtest.com domains
# (wss/ws/grpc.localtest.com all resolve to 127.0.0.1 — no /etc/hosts sudo needed)
#
# Usage (run from repo root):
#   bash ci-helpers/local-integration-test.sh
#   bash ci-helpers/local-integration-test.sh --section 2
#   bash ci-helpers/local-integration-test.sh --keep
#
# Sections:
#   1 — External (continue-on-error)
#   2 — HTTP/1.1 ingress + egress
#   3 — HTTP/2 h2c ingress + egress
#   4 — WebSocket ingress
#   5 — gRPC (Health, Echo, ServerStream, BidiStream)
#   6 — Concurrent & mixed-protocol
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$REPO/target/release"
CLIENT="$BIN/ci-test-client"
CFGDIR="$REPO/ci-helpers"

PASS=0; FAIL=0; SKIP=0
KEEP=0
ONLY_SECTION="all"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep)    KEEP=1 ;;
    --section) ONLY_SECTION="$2"; shift ;;
    *) echo "unknown arg: $1"; exit 2 ;;
  esac
  shift
done

# ─── Colors ──────────────────────────────────────────────────────────────────
G='\033[0;32m'; R='\033[0;31m'; Y='\033[1;33m'; C='\033[0;36m'; N='\033[0m'
log()  { echo -e "${C}[+]${N} $*"; }
hdr()  { echo -e "\n${C}══ $* ══${N}"; }
_pass(){ echo -e " ${G}PASS${N}"; PASS=$((PASS+1)); }
_fail(){ echo -e " ${R}FAIL${N}"; echo "    └─ $(echo "$*" | head -3)"; FAIL=$((FAIL+1)); }
_skip(){ echo -e " ${Y}SKIP${N}"; SKIP=$((SKIP+1)); }

should_run() { [[ "$ONLY_SECTION" == "all" ]] || [[ "$ONLY_SECTION" == "$1" ]]; }

# ─── Portable timeout ─────────────────────────────────────────────────────────
# macOS ships without GNU timeout; use Perl as a portable fallback.
if command -v timeout &>/dev/null; then
  _timeout() { timeout "$@"; }
elif command -v gtimeout &>/dev/null; then
  _timeout() { gtimeout "$@"; }
else
  _timeout() {
    local secs="$1"; shift
    perl -e 'alarm shift; exec @ARGV or exit 1' "$secs" "$@"
  }
fi

# ─── Test runner ─────────────────────────────────────────────────────────────
run() {
  local name="$1"; shift
  local coe=0
  [[ "${1:-}" == "-c" ]] && { coe=1; shift; }
  printf "  %-62s" "$name"
  local out rc=0
  out=$("$@" 2>&1) || rc=$?
  if   [[ $rc -eq 0 ]];  then _pass
  elif [[ $coe -eq 1 ]]; then _skip
  else _fail "$out"; fi
}

# ─── Background process tracking ─────────────────────────────────────────────
BGPIDS=()

cleanup() {
  if [[ "$KEEP" -eq 0 ]]; then
    log "Stopping background processes..."
    for p in "${BGPIDS[@]}"; do kill "$p" 2>/dev/null || true; done
  fi
  echo ""
  echo "══════════════════════════════════════"
  printf " PASS=%-4s FAIL=%-4s SKIP=%s\n" "$PASS" "$FAIL" "$SKIP"
  echo "══════════════════════════════════════"
  [[ "$FAIL" -gt 0 ]] && exit 1 || exit 0
}
trap cleanup EXIT INT TERM

# ─── Preflight ───────────────────────────────────────────────────────────────
log "Checking binaries in $BIN ..."
for b in server client tunnel-ctld http-echo-server ws-echo-server grpc-echo-server ci-test-client; do
  [[ -x "$BIN/$b" ]] || { echo "MISSING: $BIN/$b  →  cargo build --release --workspace"; exit 1; }
done

# Check that test domains resolve to 127.0.0.1 (requires /etc/hosts entries).
# Add them with: sudo tee -a /etc/hosts << 'EOF'
#   127.0.0.1  ws.localtest.com
#   127.0.0.1  grpc.localtest.com
# EOF
for domain in ws.localtest.com grpc.localtest.com; do
  ip=$(dig +short "$domain" 2>/dev/null | grep -m1 '^[0-9]' || getent hosts "$domain" 2>/dev/null | awk '{print $1}')
  if [[ "$ip" != "127.0.0.1" ]]; then
    echo ""
    echo "WARNING: $domain resolves to '${ip:-<nothing>}', not 127.0.0.1"
    echo "  Sections 4 (WS) and 5 (gRPC) will fail."
    echo "  Fix: sudo tee -a /etc/hosts <<< '127.0.0.1  $domain'"
    echo ""
  fi
done

# Kill any leftover test processes from a previous run
pkill -f "http-echo-server 9999"         2>/dev/null || true
pkill -f "ws-echo-server 8765"           2>/dev/null || true
pkill -f "grpc-echo-server 50051"        2>/dev/null || true
pkill -f "tunnel-ctld.*local-test-ctld"  2>/dev/null || true
pkill -f "server.*local-test-server"     2>/dev/null || true
pkill -f "client.*local-test-client"     2>/dev/null || true
sleep 0.5

# Remove stale DBs so ctld seeds routing fresh
rm -f "$REPO/data/local-test.db" "$REPO/data/duotunnel-local-test.db"

# ─── Start backends ──────────────────────────────────────────────────────────
log "Starting backend servers ..."
mkdir -p "$REPO/data"

"$BIN/http-echo-server" 9999  > /tmp/lt-http-echo.log  2>&1 & BGPIDS+=($!)
"$BIN/ws-echo-server"   8765  > /tmp/lt-ws-echo.log    2>&1 & BGPIDS+=($!)
"$BIN/grpc-echo-server" 50051 > /tmp/lt-grpc-echo.log  2>&1 & BGPIDS+=($!)

for i in $(seq 1 30); do
  curl -sf http://127.0.0.1:9999/ > /dev/null 2>&1 && break; sleep 0.2
done
sleep 0.5
log "Backends ready  (http:9999  ws:8765  grpc:50051)"

# ─── Start tunnel-ctld ───────────────────────────────────────────────────────
log "Starting tunnel-ctld ..."
"$BIN/tunnel-ctld" --config "$CFGDIR/local-test-ctld.yaml" > /tmp/lt-ctld.log 2>&1 & BGPIDS+=($!)
for i in $(seq 1 60); do
  curl -sf --max-time 1 http://127.0.0.1:9091/healthz > /dev/null 2>&1 && break; sleep 0.5
done
curl -sf --max-time 2 http://127.0.0.1:9091/healthz > /dev/null 2>&1 || {
  echo "ERROR: ctld not ready"; cat /tmp/lt-ctld.log; exit 1
}
log "ctld ready"

# ─── Start tunnel server ─────────────────────────────────────────────────────
log "Starting tunnel server ..."
"$BIN/server" --config "$CFGDIR/local-test-server.yaml" \
  --ctld-addr 127.0.0.1:7788 > /tmp/lt-server.log 2>&1 & BGPIDS+=($!)
for i in $(seq 1 60); do
  curl -sf --max-time 1 http://127.0.0.1:9090/healthz > /dev/null 2>&1 && break; sleep 0.5
done
curl -sf --max-time 2 http://127.0.0.1:9090/healthz > /dev/null 2>&1 || {
  echo "ERROR: server not ready"; cat /tmp/lt-server.log; exit 1
}
log "Server ready"

# ─── Create/rotate token, patch client config ────────────────────────────────
log "Getting client token ..."
TOKEN=$("$BIN/tunnel-ctld" --config "$CFGDIR/local-test-ctld.yaml" \
  client create-client local-group 2>/dev/null | grep '^Token:' | awk '{print $2}' \
  || "$BIN/tunnel-ctld" --config "$CFGDIR/local-test-ctld.yaml" \
  client rotate-token local-group 2>/dev/null | awk '{print $NF}')
[[ -n "$TOKEN" ]] || { echo "ERROR: failed to get token"; exit 1; }
python3 - "$TOKEN" <<'PY'
import re, sys
f = sys.argv[2] if len(sys.argv) > 2 else "ci-helpers/local-test-client.yaml"
# path injected at shell expansion time below
PY
python3 -c "
import re, sys
f = '$CFGDIR/local-test-client.yaml'
c = open(f).read()
c = re.sub(r'^auth_token:.*', 'auth_token: \"' + sys.argv[1] + '\"', c, flags=re.MULTILINE)
open(f, 'w').write(c)
" "$TOKEN"
sleep 3   # wait for ctld to push new token to server
log "Token ready"

# ─── Start tunnel client ─────────────────────────────────────────────────────
log "Starting tunnel client ..."
"$BIN/client" --config "$CFGDIR/local-test-client.yaml" > /tmp/lt-client.log 2>&1 &
CLI_PID=$!
BGPIDS+=($CLI_PID)

for i in $(seq 1 60); do
  kill -0 "$CLI_PID" 2>/dev/null || break
  curl -sf --max-time 1 http://127.0.0.1:9092/healthz > /dev/null 2>&1 && break
  sleep 0.5
done

if ! kill -0 "$CLI_PID" 2>/dev/null; then
  echo "ERROR: client process exited"
  cat /tmp/lt-server.log; cat /tmp/lt-client.log; exit 1
fi
if ! curl -sf --max-time 2 http://127.0.0.1:9092/healthz > /dev/null 2>&1; then
  echo "ERROR: client healthz not ready"
  cat /tmp/lt-client.log; exit 1
fi
log "Client connected  (ingress: *.localtest.com:8081  egress: 127.0.0.1:8082)"

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 1 — External backends (continue-on-error)
# ═════════════════════════════════════════════════════════════════════════════
if should_run 1; then
hdr "SECTION 1 — External (continue-on-error)"
run "[Ext-HTTP] GET egress → echo.free.beeceptor.com" -c \
  "$CLIENT" http "http://127.0.0.1:8082/" \
    --header "Host: echo.free.beeceptor.com" --allow-status 429
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 2 — HTTP/1.1 ingress + egress
# ═════════════════════════════════════════════════════════════════════════════
if should_run 2; then
hdr "SECTION 2 — HTTP/1.1"
H="http://wss.localtest.com:8081"
E="http://127.0.0.1:8082"
EH="--header Host:wss.localtest.com"

run "[HTTP] GET ingress"                  "$CLIENT" http "$H/"
run "[HTTP] POST + body verify ingress"   "$CLIENT" http "$H/" --method POST \
  --body '{"tunnel":"ci","test":"post-body-123"}' --expect-body "post-body-123"
run "[HTTP] HEAD ingress"                 "$CLIENT" http "$H/" --method HEAD --allow-status 200
run "[HTTP] DELETE ingress — method echo" "$CLIENT" http "$H/" --method DELETE --expect-body "DELETE"
run "[HTTP] Custom header ingress"        "$CLIENT" http "$H/" \
  --header "X-Tunnel-Test: reliable" --expect-body "x-tunnel-test"

printf "  %-62s" "[HTTP] Stability (10 sequential, >= 8 pass)"
OK=0
for i in $(seq 1 10); do
  "$CLIENT" http "$H/" > /dev/null 2>&1 && OK=$((OK+1)) || true
done
[[ "$OK" -ge 8 ]] && _pass || _fail "$OK/10 succeeded"

run "[HTTP] GET egress"                   "$CLIENT" http "$E/" --header "Host: wss.localtest.com"
run "[HTTP] POST + body verify egress"    "$CLIENT" http "$E/" --method POST \
  --header "Host: wss.localtest.com" \
  --body '{"tunnel":"ci","test":"egress-post-456"}' --expect-body "egress-post-456"
run "[HTTP] DELETE egress"                "$CLIENT" http "$E/" --method DELETE \
  --header "Host: wss.localtest.com" --expect-body "DELETE"
run "[HTTP] Custom header egress"         "$CLIENT" http "$E/" \
  --header "Host: wss.localtest.com" --header "X-Egress-Test: forwarded" \
  --expect-body "x-egress-test"
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 3 — HTTP/2 h2c
# ═════════════════════════════════════════════════════════════════════════════
if should_run 3; then
hdr "SECTION 3 — HTTP/2 h2c"
H="http://wss.localtest.com:8081"
E="http://127.0.0.1:8082"

run "[HTTP2] GET h2c ingress"             "$CLIENT" http2 "$H/"
run "[HTTP2] POST h2c + body ingress"     "$CLIENT" http2 "$H/" --method POST \
  --body '{"h2":"post-test"}' --expect-body "post-test"
run "[HTTP2] Custom header h2c ingress"   "$CLIENT" http2 "$H/" \
  --header "X-H2-Test: h2c-header" --expect-body "x-h2-test"
run "[HTTP2] GET h2c egress"              "$CLIENT" http2 "$E/" --header "Host: wss.localtest.com"
run "[HTTP2] POST h2c + body egress"      "$CLIENT" http2 "$E/" --method POST \
  --header "Host: wss.localtest.com" \
  --body '{"h2":"egress-post-test"}' --expect-body "egress-post-test"
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 4 — WebSocket
# ═════════════════════════════════════════════════════════════════════════════
# Requires ws.localtest.com → 127.0.0.1 in /etc/hosts.
if should_run 4; then
hdr "SECTION 4 — WebSocket"
run "[WS] Echo ingress"                   "$CLIENT" ws "ws://ws.localtest.com:8081" \
  --message "hello-ws-tunnel"
run "[WS] Custom message echo"            "$CLIENT" ws "ws://ws.localtest.com:8081" \
  --message "ci-msg-two"
run "[WS] Large message (4096 bytes)"     "$CLIENT" ws "ws://ws.localtest.com:8081" \
  --message "$(python3 -c "print('x'*4096)")"
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 5 — gRPC
# ═════════════════════════════════════════════════════════════════════════════
# Requires grpc.localtest.com → 127.0.0.1 in /etc/hosts.
if should_run 5; then
hdr "SECTION 5 — gRPC"
G5="grpc.localtest.com:8081"

run "[gRPC] Health/Check SERVING"         "$CLIENT" grpc        "$G5" --service ""
run "[gRPC] EchoService/Echo body"        "$CLIENT" grpc-echo   "$G5" --ping "ci-local-echo-test"
run "[gRPC] EchoService/Echo headers"     "$CLIENT" grpc-echo   "$G5" --ping "ci-header-check"
run "[gRPC] ServerStreamEcho (5 msgs)"    "$CLIENT" grpc-server-stream "$G5" \
  --ping "ci-stream" --count 5
run "[gRPC] BidiStreamEcho (5 msgs)"      "$CLIENT" grpc-bidi-stream "$G5" \
  --ping "ci-bidi" --count 5
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 6 — Mixed-protocol sequential smoke tests
# ═════════════════════════════════════════════════════════════════════════════
if should_run 6; then
hdr "SECTION 6 — Mixed-protocol smoke tests"

run "[Mix] H1 GET ingress"            "$CLIENT" http      "http://wss.localtest.com:8081/"
run "[Mix] H2 GET ingress"            "$CLIENT" http2     "http://wss.localtest.com:8081/"
run "[Mix] H2c prior GET ingress"     "$CLIENT" h2c-prior "http://wss.localtest.com:8081/"
run "[Mix] WS echo ingress"           "$CLIENT" ws        "ws://ws.localtest.com:8081" --message "mix-ws"
run "[Mix] gRPC Health ingress"       "$CLIENT" grpc      "grpc.localtest.com:8081" --service ""
run "[Mix] gRPC Echo ingress"         "$CLIENT" grpc-echo "grpc.localtest.com:8081" --ping "mix-grpc"
run "[Mix] H1 GET egress"             "$CLIENT" http      "http://127.0.0.1:8082/" \
  --header "Host: wss.localtest.com"
run "[Mix] H2 GET egress"             "$CLIENT" http2     "http://127.0.0.1:8082/" \
  --header "Host: wss.localtest.com"
fi
