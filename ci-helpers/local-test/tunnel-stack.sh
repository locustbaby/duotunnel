#!/usr/bin/env bash
# ci-helpers/local-test/tunnel-stack.sh
#
# Shared functions for starting and stopping the tunnel stack
# (backends + ctld + server + client).
#
# Source this file, then call the functions:
#
#   source ci-helpers/local-test/tunnel-stack.sh
#
#   # Required env vars (set defaults before sourcing if needed):
#   #   BIN        — directory containing compiled binaries (e.g. ./target/release)
#   #   CFGDIR     — directory containing yaml configs    (e.g. ./ci-helpers/local-test)
#   #   LOG_PREFIX — prefix for /tmp log files            (default: ci)
#   #   CLIENT_GROUP — ctld group name                   (default: ci-group)
#   #   QUIC_CONNECTIONS — value to patch into client.yaml (default: not patched)
#
#   stack_start_backends
#   stack_start_ctld
#   stack_start_server
#   stack_create_token
#   stack_start_client
#
#   stack_stop_all    # clean shutdown (SIGTERM + wait + SIGKILL)
#   stack_kill_all    # immediate kill (SIGKILL, for use in cleanup traps)

# ── Defaults ────────────────────────────────────────────────────────────────
: "${BIN:=./target/release}"
: "${CFGDIR:=./ci-helpers/local-test}"
: "${LOG_PREFIX:=ci}"
: "${CLIENT_GROUP:=ci-group}"

# ── Colors / logging helpers (only if not already defined) ───────────────────
if ! declare -f log > /dev/null 2>&1; then
  _C='\033[0;36m'; _N='\033[0m'
  log() { echo -e "${_C}[+]${_N} $*"; }
fi

# ── Backend servers ───────────────────────────────────────────────────────────
stack_start_backends() {
  log "Starting backend servers (http-echo:9999  ws-echo:8765  grpc-echo:50051) ..."
  "$BIN/http-echo-server" 9999  > "/tmp/${LOG_PREFIX}-http-echo.log"  2>&1 &
  echo $! > /tmp/http-echo.pid
  "$BIN/ws-echo-server"   8765  > "/tmp/${LOG_PREFIX}-ws-echo.log"    2>&1 &
  echo $! > /tmp/ws-echo.pid
  "$BIN/grpc-echo-server" 50051 > "/tmp/${LOG_PREFIX}-grpc-echo.log"  2>&1 &
  echo $! > /tmp/grpc-echo.pid

  # Wait for http-echo to be ready; ws/grpc get a grace period
  for i in $(seq 1 20); do
    curl -sf --max-time 1 http://127.0.0.1:9999/ > /dev/null 2>&1 && break
    sleep 0.2
  done
  sleep 0.5
  log "Backends ready"
}

# ── tunnel-ctld ───────────────────────────────────────────────────────────────
stack_start_ctld() {
  local cfg="${CTLD_CONFIG:-$CFGDIR/ctld.yaml}"
  log "Starting tunnel-ctld (config: $cfg) ..."
  mkdir -p data
  "$BIN/tunnel-ctld" --config "$cfg" > "/tmp/${LOG_PREFIX}-ctld.log" 2>&1 &
  echo $! > /tmp/ctld.pid

  for i in $(seq 1 60); do
    curl -sf --max-time 1 http://127.0.0.1:9091/healthz > /dev/null 2>&1 && break
    sleep 0.5
  done
  curl -sf --max-time 2 http://127.0.0.1:9091/healthz > /dev/null 2>&1 || {
    echo "ERROR: ctld healthz never became ready"
    cat "/tmp/${LOG_PREFIX}-ctld.log"
    return 1
  }
  log "ctld ready"
}

# ── tunnel server ─────────────────────────────────────────────────────────────
stack_start_server() {
  local cfg="${SERVER_CONFIG:-$CFGDIR/server.yaml}"
  local ctld_addr="${CTLD_ADDR:-127.0.0.1:7788}"
  log "Starting tunnel server (config: $cfg, ctld: $ctld_addr) ..."
  "$BIN/server" --config "$cfg" --ctld-addr "$ctld_addr" \
    > "/tmp/${LOG_PREFIX}-server.log" 2>&1 &
  echo $! > /tmp/server.pid

  for i in $(seq 1 60); do
    curl -sf --max-time 1 http://127.0.0.1:9090/healthz > /dev/null 2>&1 && break
    sleep 0.5
  done
  curl -sf --max-time 2 http://127.0.0.1:9090/healthz > /dev/null 2>&1 || {
    echo "ERROR: server healthz never became ready"
    cat "/tmp/${LOG_PREFIX}-server.log"
    return 1
  }
  log "Tunnel server ready"
}

# ── Create/rotate token, patch client.yaml ────────────────────────────────────
# After calling this, TUNNEL_TOKEN is exported.
stack_create_token() {
  local cfg="${CTLD_CONFIG:-$CFGDIR/ctld.yaml}"
  local client_cfg="${CLIENT_CONFIG:-$CFGDIR/client.yaml}"
  log "Creating/rotating token for group '${CLIENT_GROUP}' ..."
  TUNNEL_TOKEN=$(
    "$BIN/tunnel-ctld" --config "$cfg" \
      client create-client "$CLIENT_GROUP" 2>/dev/null \
      | grep '^Token:' | awk '{print $2}' \
    || "$BIN/tunnel-ctld" --config "$cfg" \
      client rotate-token "$CLIENT_GROUP" 2>/dev/null \
      | awk '{print $NF}'
  )
  [[ -n "$TUNNEL_TOKEN" ]] || { echo "ERROR: failed to get token"; return 1; }
  export TUNNEL_TOKEN

  # Patch auth_token in client config
  python3 -c "
import re, sys
f, t = sys.argv[1], sys.argv[2]
c = open(f).read()
c = re.sub(r'^auth_token:.*', 'auth_token: \"' + t + '\"', c, flags=re.MULTILINE)
if sys.argv[3]:
    n = sys.argv[3]
    c = re.sub(r'^(\s*connections:)\s*\d+', r'\1 ' + n, c, flags=re.MULTILINE)
open(f, 'w').write(c)
" "$client_cfg" "$TUNNEL_TOKEN" "${QUIC_CONNECTIONS:-}"

  # Wait for ctld to push the new token to server (~1500ms poll interval)
  sleep 3
  log "Token ready"
}

# ── tunnel client ─────────────────────────────────────────────────────────────
stack_start_client() {
  local cfg="${CLIENT_CONFIG:-$CFGDIR/client.yaml}"
  local healthz_port="${CLIENT_HEALTHZ_PORT:-9092}"
  log "Starting tunnel client (config: $cfg) ..."
  "$BIN/client" --config "$cfg" > "/tmp/${LOG_PREFIX}-client.log" 2>&1 &
  local cli_pid=$!
  echo "$cli_pid" > /tmp/client.pid

  for i in $(seq 1 60); do
    kill -0 "$cli_pid" 2>/dev/null || break
    curl -sf --max-time 1 "http://127.0.0.1:${healthz_port}/healthz" > /dev/null 2>&1 && break
    sleep 0.5
  done

  if ! kill -0 "$cli_pid" 2>/dev/null; then
    echo "ERROR: client process exited"
    stack_dump_logs
    return 1
  fi
  if ! curl -sf --max-time 2 "http://127.0.0.1:${healthz_port}/healthz" > /dev/null 2>&1; then
    echo "ERROR: client healthz not ready"
    cat "/tmp/${LOG_PREFIX}-client.log"
    return 1
  fi
  log "Client connected"
}

# ── Dump logs on failure ──────────────────────────────────────────────────────
stack_dump_logs() {
  for comp in ctld server client; do
    local f="/tmp/${LOG_PREFIX}-${comp}.log"
    [[ -f "$f" ]] && { echo "=== ${comp} ===" && cat "$f"; } || true
  done
}

# ── Graceful stop (SIGTERM → wait up to 5s → SIGKILL) ────────────────────────
_graceful_kill() {
  local pid_file="$1"
  local pid
  pid="$(cat "$pid_file" 2>/dev/null || true)"
  [[ -n "$pid" ]] || return
  kill -TERM "$pid" 2>/dev/null || return
  for i in $(seq 1 10); do
    kill -0 "$pid" 2>/dev/null || return
    sleep 0.5
  done
  kill -9 "$pid" 2>/dev/null || true
}

stack_stop_all() {
  log "Stopping tunnel stack ..."
  _graceful_kill /tmp/client.pid
  _graceful_kill /tmp/server.pid
  _graceful_kill /tmp/ctld.pid
  kill -9 "$(cat /tmp/http-echo.pid  2>/dev/null)" 2>/dev/null || true
  kill -9 "$(cat /tmp/ws-echo.pid    2>/dev/null)" 2>/dev/null || true
  kill -9 "$(cat /tmp/grpc-echo.pid  2>/dev/null)" 2>/dev/null || true
}

stack_kill_all() {
  for f in /tmp/client.pid /tmp/server.pid /tmp/ctld.pid \
            /tmp/http-echo.pid /tmp/ws-echo.pid /tmp/grpc-echo.pid; do
    kill -9 "$(cat "$f" 2>/dev/null)" 2>/dev/null || true
  done
}
