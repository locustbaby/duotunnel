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
#   #   DUOTUNNEL_BIN        — directory containing compiled binaries (e.g. ./target/release)
#   #   DUOTUNNEL_CONFIG_DIR     — directory containing yaml configs    (e.g. ./ci-helpers/local-test)
#   #   DUOTUNNEL_LOG_PREFIX — prefix for /tmp log files            (default: ci)
#   #   DUOTUNNEL_CLIENT_GROUP — ctld group name                   (default: ci-group)
#   #   DUOTUNNEL_QUIC_CONNECTIONS — value to patch into client.yaml (default: not patched)
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
: "${DUOTUNNEL_BIN:=./target/release}"
: "${DUOTUNNEL_CONFIG_DIR:=./ci-helpers/local-test}"
: "${DUOTUNNEL_LOG_PREFIX:=ci}"
: "${DUOTUNNEL_CLIENT_GROUP:=ci-group}"

# ── Colors / logging helpers (only if not already defined) ───────────────────
if ! declare -f log > /dev/null 2>&1; then
  _C='\033[0;36m'; _N='\033[0m'
  log() { echo -e "${_C}[+]${_N} $*"; }
fi

# ── Backend servers ───────────────────────────────────────────────────────────
stack_start_backends() {
  log "Starting backend servers (http-echo:9999  ws-echo:8765  grpc-echo:50051) ..."
  "$DUOTUNNEL_BIN/http-echo-server" 9999  > "/tmp/${DUOTUNNEL_LOG_PREFIX}-http-echo.log"  2>&1 &
  echo $! > /tmp/http-echo.pid
  "$DUOTUNNEL_BIN/ws-echo-server"   8765  > "/tmp/${DUOTUNNEL_LOG_PREFIX}-ws-echo.log"    2>&1 &
  echo $! > /tmp/ws-echo.pid
  "$DUOTUNNEL_BIN/grpc-echo-server" 50051 > "/tmp/${DUOTUNNEL_LOG_PREFIX}-grpc-echo.log"  2>&1 &
  echo $! > /tmp/grpc-echo.pid

  # Wait for http-echo to be ready; ws/grpc get a grace period
  for i in $(seq 1 20); do
    curl -sf --max-time 1 http://127.0.0.1:9999/ > /dev/null 2>&1 && break
    sleep 0.2
  done
  sleep 0.5
  log "Backends ready"
}

# ── duotunnel-ctld ───────────────────────────────────────────────────────────────
stack_start_ctld() {
  local cfg="${DUOTUNNEL_CTLD_CONFIG:-$DUOTUNNEL_CONFIG_DIR/ctld.yaml}"
  log "Starting duotunnel-ctld (config: $cfg) ..."
  mkdir -p data
  "$DUOTUNNEL_BIN/duotunnel-ctld" --config "$cfg" > "/tmp/${DUOTUNNEL_LOG_PREFIX}-ctld.log" 2>&1 &
  echo $! > /tmp/ctld.pid

  for i in $(seq 1 60); do
    curl -sf --max-time 1 http://127.0.0.1:9091/healthz > /dev/null 2>&1 && break
    sleep 0.5
  done
  curl -sf --max-time 2 http://127.0.0.1:9091/healthz > /dev/null 2>&1 || {
    echo "ERROR: ctld healthz never became ready"
    cat "/tmp/${DUOTUNNEL_LOG_PREFIX}-ctld.log"
    return 1
  }
  log "ctld ready"
}

# ── tunnel server ─────────────────────────────────────────────────────────────
stack_start_server() {
  local cfg="${DUOTUNNEL_SERVER_CONFIG:-$DUOTUNNEL_CONFIG_DIR/server.yaml}"
  local ctld_addr="${DUOTUNNEL_CTLD_ADDR:-127.0.0.1:7788}"
  log "Starting tunnel server (config: $cfg, ctld: $ctld_addr) ..."
  "$DUOTUNNEL_BIN/duotunnel-server" --config "$cfg" --ctld-addr "$ctld_addr" \
    > "/tmp/${DUOTUNNEL_LOG_PREFIX}-server.log" 2>&1 &
  echo $! > /tmp/server.pid

  for i in $(seq 1 60); do
    curl -sf --max-time 1 http://127.0.0.1:9090/healthz > /dev/null 2>&1 && break
    sleep 0.5
  done
  curl -sf --max-time 2 http://127.0.0.1:9090/healthz > /dev/null 2>&1 || {
    echo "ERROR: server healthz never became ready"
    cat "/tmp/${DUOTUNNEL_LOG_PREFIX}-server.log"
    return 1
  }
  log "Tunnel server ready"
}

# ── Create/rotate token, patch client.yaml ────────────────────────────────────
# After calling this, DUOTUNNEL_TOKEN is exported.
stack_create_token() {
  local cfg="${DUOTUNNEL_CTLD_CONFIG:-$DUOTUNNEL_CONFIG_DIR/ctld.yaml}"
  log "Creating/rotating token for group '${DUOTUNNEL_CLIENT_GROUP}' ..."
  DUOTUNNEL_TOKEN=$(
    "$DUOTUNNEL_BIN/duotunnel-ctld" --config "$cfg" \
      client create "$DUOTUNNEL_CLIENT_GROUP" 2>/dev/null \
      | grep '^Token:' | awk '{print $2}' \
    || "$DUOTUNNEL_BIN/duotunnel-ctld" --config "$cfg" \
      client rotate "$DUOTUNNEL_CLIENT_GROUP" 2>/dev/null \
      | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g'
  )
  DUOTUNNEL_TOKEN=$(echo "$DUOTUNNEL_TOKEN" | tr -cd '[:print:]')
  [[ -n "$DUOTUNNEL_TOKEN" ]] || { echo "ERROR: failed to get token"; return 1; }
  export DUOTUNNEL_TOKEN

  # Wait for ctld to publish the new token before the client connects.
  sleep 3
  log "Token ready"
}

# ── tunnel client ─────────────────────────────────────────────────────────────
stack_start_client() {
  local cfg="${DUOTUNNEL_CLIENT_CONFIG:-$DUOTUNNEL_CONFIG_DIR/client.yaml}"
  local healthz_port="${DUOTUNNEL_CLIENT_HEALTHZ_PORT:-9092}"
  log "Starting tunnel client (config: $cfg) ..."
  local -a client_env=("DUOTUNNEL_CLIENT__AUTH_TOKEN=$DUOTUNNEL_TOKEN")
  if [[ -n "${DUOTUNNEL_QUIC_CONNECTIONS:-}" ]]; then
    client_env+=("DUOTUNNEL_CLIENT__QUIC__CONNECTIONS=$DUOTUNNEL_QUIC_CONNECTIONS")
  fi
  env "${client_env[@]}" "$DUOTUNNEL_BIN/duotunnel-client" --config "$cfg" \
    > "/tmp/${DUOTUNNEL_LOG_PREFIX}-client.log" 2>&1 &
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
    cat "/tmp/${DUOTUNNEL_LOG_PREFIX}-client.log"
    return 1
  fi
  log "Client connected"
}

# ── Dump logs on failure ──────────────────────────────────────────────────────
stack_dump_logs() {
  for comp in ctld server client; do
    local f="/tmp/${DUOTUNNEL_LOG_PREFIX}-${comp}.log"
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
