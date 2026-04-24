#!/bin/bash
set -eo pipefail

CASE_NAME=$1
GITHUB_SHA=$2
FRP_SERVER_PORT=7700
FRP_HTTP_PORT=7800

cleanup_frp() {
  sudo systemctl stop frp-client.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-client.scope 2>/dev/null || true
  sudo systemctl stop frp-server.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-server.scope 2>/dev/null || true
  sudo systemctl reset-failed frp-client.scope frp-server.scope 2>/dev/null || true
  sudo pkill -x frpc 2>/dev/null || true
  sudo pkill -x frps 2>/dev/null || true
  sudo fuser -k "${FRP_SERVER_PORT}/tcp" 2>/dev/null || true
  sudo fuser -k "${FRP_HTTP_PORT}/tcp" 2>/dev/null || true
}

wait_for_frp_ports_free() {
  for i in $(seq 1 20); do
    ss -tln | grep -qE ":${FRP_SERVER_PORT}|:${FRP_HTTP_PORT}" || return 0
    sleep 0.5
  done
  return 1
}

dump_frp_port_diag() {
  echo "=== frp port diagnostics ==="
  sudo ss -tlnp "( sport = :${FRP_SERVER_PORT} or sport = :${FRP_HTTP_PORT} )" 2>/dev/null || true
  sudo lsof -nP -iTCP:"${FRP_SERVER_PORT}" -iTCP:"${FRP_HTTP_PORT}" -sTCP:LISTEN 2>/dev/null || true
}

start_frp_stack() {
  : > /tmp/frps.log
  : > /tmp/frpc.log

  sudo systemd-run --scope --unit=frp-server --collect \
    -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
    -- frps -c ci-helpers/configs/frps.toml >> /tmp/frps.log 2>&1 &

  for i in $(seq 1 10); do
    if sudo systemctl is-failed frp-server.scope > /dev/null 2>&1; then
      return 1
    fi
    if ss -tln | grep -q ":${FRP_SERVER_PORT}"; then
      break
    fi
    sleep 0.3
  done

  sudo systemd-run --scope --unit=frp-client --collect \
    -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
    -- frpc -c ci-helpers/configs/frpc.toml >> /tmp/frpc.log 2>&1 &

  for i in $(seq 1 20); do
    if sudo systemctl is-failed frp-server.scope > /dev/null 2>&1; then
      return 1
    fi
    if sudo systemctl is-failed frp-client.scope > /dev/null 2>&1; then
      return 1
    fi
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 2 -H "Host: echo.local" "http://127.0.0.1:${FRP_HTTP_PORT}/" 2>/dev/null || true)
    [[ "$STATUS" =~ ^[0-9]+$ ]] && [ "$STATUS" -ge 200 ] && return 0
    sleep 0.5
  done

  return 1
}

IS_FRP=0
[[ "$CASE_NAME" == frp_* ]] && IS_FRP=1

if [ "$IS_FRP" -eq 1 ]; then
  cleanup_frp
  if ! wait_for_frp_ports_free; then
    echo "WARNING: frp ports are still occupied after cleanup, skipping $CASE_NAME"
    dump_frp_port_diag
    exit 0
  fi

  FRP_READY=0
  for attempt in $(seq 1 3); do
    if start_frp_stack; then
      FRP_READY=1
      break
    fi
    cleanup_frp
    wait_for_frp_ports_free || true
    sleep 1
  done

  if [ "$FRP_READY" -eq 0 ]; then
    echo "WARNING: frp did not become ready, skipping $CASE_NAME"
    dump_frp_port_diag
    echo "=== frp-server scope ===" && sudo systemctl status frp-server.scope 2>&1 | tail -20 || true
    echo "=== frp-client scope ===" && sudo systemctl status frp-client.scope 2>&1 | tail -20 || true
    echo "=== frps ===" && cat /tmp/frps.log || true
    echo "=== frpc ===" && cat /tmp/frpc.log || true
    exit 0
  fi
fi

python3 ci-helpers/bench-tool.py collect 1 > "/tmp/collect-${CASE_NAME}.jsonl" 2>/dev/null &
echo $! > "/tmp/collect-${CASE_NAME}.pid"

(cd ci-helpers/k6 && k6 run \
  -e GITHUB_SHA="${GITHUB_SHA}" \
  -e BENCH_CASE="${CASE_NAME}" \
  -e BENCH_RESULT_PATH="/tmp/bench-results-${CASE_NAME}.json" \
  bench.js)

kill -9 "$(cat "/tmp/collect-${CASE_NAME}.pid" 2>/dev/null)" 2>/dev/null || true
sleep 0.5

python3 ci-helpers/bench-tool.py parse \
  --input  "/tmp/collect-${CASE_NAME}.jsonl" \
  --k6-offset 0 \
  --output "/tmp/resource-${CASE_NAME}.json" || true

if [ -s "/tmp/resource-${CASE_NAME}.json" ] && [ -s "/tmp/bench-results-${CASE_NAME}.json" ]; then
  python3 ci-helpers/bench-tool.py inject \
    --result    "/tmp/bench-results-${CASE_NAME}.json" \
    --resources "/tmp/resource-${CASE_NAME}.json" \
    --case-name "${CASE_NAME}" || true
fi

if [ "$IS_FRP" -eq 1 ]; then
  cleanup_frp
  wait_for_frp_ports_free || true
fi
