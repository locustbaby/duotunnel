#!/usr/bin/env bash
# ci-helpers/local-test/test.sh
# Local integration test for the tunnel stack.
# It intentionally uses *.localtest.com domains, while CI uses ws.local / grpc.local.
#
# Usage (run from repo root):
#   bash ci-helpers/local-test/test.sh
#   bash ci-helpers/local-test/test.sh --section 2
#   bash ci-helpers/local-test/test.sh --keep
#
# Sections:
#   1 — External (continue-on-error)
#   2 — HTTP/1.1 ingress + egress
#   3 — HTTP/2 h2c
#   4 — WebSocket ingress
#   5 — gRPC (Health, Echo, ServerStream, BidiStream)
#   6 — Mixed-protocol smoke tests
set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
export BIN="$REPO/target/release"
export CFGDIR="$REPO/ci-helpers/local-test"
export LOG_PREFIX="lt"

# Local-test overrides for tunnel-stack.sh
export CTLD_CONFIG="$CFGDIR/ctld.yaml"
export SERVER_CONFIG="$CFGDIR/server.yaml"
export CLIENT_CONFIG="$CFGDIR/client.yaml"
export CLIENT_GROUP="local-group"
export CLIENT_HEALTHZ_PORT="9092"
export LOCAL_HTTP_HOST="wss.localtest.com"
export LOCAL_WS_HOST="ws.localtest.com"
export LOCAL_GRPC_HOST="grpc.localtest.com"

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

# ─── Colors + helpers ─────────────────────────────────────────────────────────
G='\033[0;32m'; R='\033[0;31m'; Y='\033[1;33m'; C='\033[0;36m'; N='\033[0m'
log()  { echo -e "${C}[+]${N} $*"; }
hdr()  { echo -e "\n${C}══ $* ══${N}"; }
_pass(){ echo -e " ${G}PASS${N}"; PASS=$((PASS+1)); }
_fail(){ echo -e " ${R}FAIL${N}"; echo "    └─ $(echo "$*" | head -3)"; FAIL=$((FAIL+1)); }
_skip(){ echo -e " ${Y}SKIP${N}"; SKIP=$((SKIP+1)); }
PASS=0; FAIL=0; SKIP=0

should_run() { [[ "$ONLY_SECTION" == "all" ]] || [[ "$ONLY_SECTION" == "$1" ]]; }

# ─── Portable timeout ─────────────────────────────────────────────────────────
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

resolve_ipv4() {
  local host="$1" ip=""

  if command -v dig &>/dev/null; then
    ip=$(dig +short "$host" 2>/dev/null | awk '/^[0-9]+\./ { print; exit }') || true
  fi

  if [[ -z "$ip" ]] && command -v getent &>/dev/null; then
    ip=$(getent hosts "$host" 2>/dev/null | awk 'NR == 1 { print $1 }') || true
  fi

  if [[ -z "$ip" ]] && command -v python3 &>/dev/null; then
    ip=$(python3 - "$host" <<'PY'
import socket
import sys

try:
    infos = socket.getaddrinfo(sys.argv[1], None, family=socket.AF_INET, type=socket.SOCK_STREAM)
except socket.gaierror:
    raise SystemExit(0)

for info in infos:
    print(info[4][0])
    break
PY
) || true
  fi

  printf '%s' "$ip"
}

warn_if_domain_not_loopback() {
  local domain="$1"
  local ip
  ip="$(resolve_ipv4 "$domain")"

  if [[ -z "$ip" ]]; then
    echo ""
    echo "WARNING: unable to resolve $domain in preflight"
    echo "  Continuing; protocol sections will verify connectivity directly."
    echo ""
    return
  fi

  if [[ "$ip" != "127.0.0.1" ]]; then
    echo ""
    echo "WARNING: $domain resolves to '$ip', not 127.0.0.1"
    echo "  Sections using $domain may fail if your resolver is not pinned locally."
    echo ""
  fi
}

# ─── Source shared tunnel-stack functions ────────────────────────────────────
# shellcheck source=ci-helpers/local-test/tunnel-stack.sh
source "$CFGDIR/tunnel-stack.sh"

# ─── Cleanup ─────────────────────────────────────────────────────────────────
cleanup() {
  if [[ "$KEEP" -eq 0 ]]; then
    stack_stop_all 2>/dev/null || true
  fi
  echo ""
  echo "══════════════════════════════════════"
  printf " PASS=%-4s FAIL=%-4s SKIP=%s\n" "$PASS" "$FAIL" "$SKIP"
  echo "══════════════════════════════════════"
  [[ "$FAIL" -gt 0 ]] && exit 1 || exit 0
}
trap cleanup EXIT INT TERM

# ─── Preflight: clippy & binaries ─────────────────────────────────────────────
log "Running cargo clippy ..."
cargo clippy --workspace || exit 1

log "Checking binaries in $BIN ..."
for b in server client tunnel-ctld http-echo-server ws-echo-server grpc-echo-server ci-test-client; do
  [[ -x "$BIN/$b" ]] || {
    echo "MISSING: $BIN/$b  →  cargo build --release --workspace"
    exit 1
  }
done

# ─── Preflight: local-only domain resolution ─────────────────────────────────
for domain in "$LOCAL_WS_HOST" "$LOCAL_GRPC_HOST"; do
  warn_if_domain_not_loopback "$domain"
done

# Kill any leftover processes from a previous run
pkill -f "http-echo-server 9999"          2>/dev/null || true
pkill -f "ws-echo-server 8765"            2>/dev/null || true
pkill -f "grpc-echo-server 50051"         2>/dev/null || true
pkill -f "tunnel-ctld.*local-test-ctld"   2>/dev/null || true
pkill -f "server.*local-test-server"      2>/dev/null || true
pkill -f "client.*local-test-client"      2>/dev/null || true
sleep 0.5

# Remove stale DBs so ctld seeds routing fresh
rm -f "$REPO/data/local-test.db"* "$REPO/data/duotunnel-local-test.db"*

# ─── Start the full tunnel stack (backends + ctld + server + client) ──────────
stack_start_backends
stack_start_ctld
stack_start_server
stack_create_token
stack_start_client

log "Tunnel ready  (ingress: $LOCAL_HTTP_HOST/$LOCAL_WS_HOST/$LOCAL_GRPC_HOST on :8081, egress: $LOCAL_HTTP_HOST:8082)"

CLIENT="$BIN/ci-test-client"

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
H="http://$LOCAL_HTTP_HOST:8081"
E="http://$LOCAL_HTTP_HOST:8082"

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

run "[HTTP] GET egress"                   "$CLIENT" http "$E/"
run "[HTTP] POST + body verify egress"    "$CLIENT" http "$E/" --method POST \
  --body '{"tunnel":"ci","test":"egress-post-456"}' --expect-body "egress-post-456"
run "[HTTP] DELETE egress"                "$CLIENT" http "$E/" --method DELETE \
  --expect-body "DELETE"
run "[HTTP] Custom header egress"         "$CLIENT" http "$E/" \
  --header "X-Egress-Test: forwarded" --expect-body "x-egress-test"
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 3 — HTTP/2 h2c
# ═════════════════════════════════════════════════════════════════════════════
if should_run 3; then
hdr "SECTION 3 — HTTP/2 h2c"
H="http://$LOCAL_HTTP_HOST:8081"
E="http://$LOCAL_HTTP_HOST:8082"

run "[HTTP2] GET h2c ingress"             "$CLIENT" http2 "$H/"
run "[HTTP2] POST h2c + body ingress"     "$CLIENT" http2 "$H/" --method POST \
  --body '{"h2":"post-test"}' --expect-body "post-test"
run "[HTTP2] Custom header h2c ingress"   "$CLIENT" http2 "$H/" \
  --header "X-H2-Test: h2c-header" --expect-body "x-h2-test"
run "[HTTP2] GET h2c egress"              "$CLIENT" http2 "$E/"
run "[HTTP2] POST h2c + body egress"      "$CLIENT" http2 "$E/" --method POST \
  --body '{"h2":"egress-post-test"}' --expect-body "egress-post-test"
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 4 — WebSocket ingress
# Uses the local-test WebSocket hostname, separate from CI's ws.local.
# ═════════════════════════════════════════════════════════════════════════════
if should_run 4; then
hdr "SECTION 4 — WebSocket"
run "[WS] Echo ingress"                   "$CLIENT" ws "ws://$LOCAL_WS_HOST:8081" \
  --message "hello-ws-tunnel"
run "[WS] Custom message echo"            "$CLIENT" ws "ws://$LOCAL_WS_HOST:8081" \
  --message "ci-msg-two"
run "[WS] Large message (4096 bytes)"     "$CLIENT" ws "ws://$LOCAL_WS_HOST:8081" \
  --message "$(python3 -c "print('x'*4096)")"
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 5 — gRPC
# Uses the local-test gRPC hostname, separate from CI's grpc.local.
# ═════════════════════════════════════════════════════════════════════════════
if should_run 5; then
hdr "SECTION 5 — gRPC"
G5="$LOCAL_GRPC_HOST:8081"

run "[gRPC] Health/Check SERVING"         "$CLIENT" grpc              "$G5" --service ""
run "[gRPC] EchoService/Echo body"        "$CLIENT" grpc-echo         "$G5" --ping "ci-local-echo-test"
run "[gRPC] EchoService/Echo headers"     "$CLIENT" grpc-echo         "$G5" --ping "ci-header-check"
run "[gRPC] ServerStreamEcho (5 msgs)"    "$CLIENT" grpc-server-stream "$G5" \
  --ping "ci-stream" --count 5
run "[gRPC] BidiStreamEcho (5 msgs)"      "$CLIENT" grpc-bidi-stream  "$G5" \
  --ping "ci-bidi" --count 5
fi

# ═════════════════════════════════════════════════════════════════════════════
# SECTION 6 — Mixed-protocol sequential smoke tests
# ═════════════════════════════════════════════════════════════════════════════
if should_run 6; then
hdr "SECTION 6 — Mixed-protocol smoke tests"

run "[Mix] H1 GET ingress"            "$CLIENT" http      "http://$LOCAL_HTTP_HOST:8081/"
run "[Mix] H2 GET ingress"            "$CLIENT" http2     "http://$LOCAL_HTTP_HOST:8081/"
run "[Mix] H2c prior GET ingress"     "$CLIENT" h2c-prior "http://$LOCAL_HTTP_HOST:8081/"
run "[Mix] WS echo ingress"           "$CLIENT" ws        "ws://$LOCAL_WS_HOST:8081" --message "mix-ws"
run "[Mix] gRPC Health ingress"       "$CLIENT" grpc      "$LOCAL_GRPC_HOST:8081" --service ""
run "[Mix] gRPC Echo ingress"         "$CLIENT" grpc-echo "$LOCAL_GRPC_HOST:8081" --ping "mix-grpc"
run "[Mix] H1 GET egress"             "$CLIENT" http      "http://$LOCAL_HTTP_HOST:8082/"
run "[Mix] H2 GET egress"             "$CLIENT" http2     "http://$LOCAL_HTTP_HOST:8082/"
fi
