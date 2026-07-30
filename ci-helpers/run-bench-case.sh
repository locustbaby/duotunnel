#!/bin/bash
set -eo pipefail

CASE_NAME=$1
GITHUB_SHA=$2
FRP_SERVER_PORT=${FRP_SERVER_PORT:-}
FRP_HTTP_PORT=${FRP_HTTP_PORT:-}
REQUESTED_FRP_SERVER_PORT="${FRP_SERVER_PORT}"
REQUESTED_FRP_HTTP_PORT="${FRP_HTTP_PORT}"
FRP_TMP_DIR=""
FRPS_CONFIG="ci-helpers/configs/frps.toml"
FRPC_CONFIG="ci-helpers/configs/frpc.toml"
source ci-helpers/cpu-contract.sh

cpu_contract_resolve "${DUOTUNNEL_BENCH_CPU_MODE:-isolate}" "${WORKER_THREADS:-0}" "${STRESS_CPU_QUOTA:-100%}"
mapfile -t FRP_SERVER_CPU_ARGS < <(cpu_contract_scope_args frp-server)
mapfile -t FRP_CLIENT_CPU_ARGS < <(cpu_contract_scope_args frp-client)

pick_frp_ports() {
  if [ -n "${FRP_SERVER_PORT}" ] && [ -n "${FRP_HTTP_PORT}" ]; then
    return 0
  fi

  read -r FRP_SERVER_PORT FRP_HTTP_PORT < <(python3 - <<'PY'
import socket

sockets = []
for _ in range(2):
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind(("127.0.0.1", 0))
    sockets.append(s)

print(*(s.getsockname()[1] for s in sockets))
PY
)
}

prepare_frp_configs() {
  rm -rf "${FRP_TMP_DIR:-}"
  FRP_TMP_DIR=$(mktemp -d /tmp/duotunnel-frp.XXXXXX)
  FRPS_CONFIG="${FRP_TMP_DIR}/frps.toml"
  FRPC_CONFIG="${FRP_TMP_DIR}/frpc.toml"
  sed \
    -e "s/^bindPort = .*/bindPort = ${FRP_SERVER_PORT}/" \
    -e "s/^vhostHTTPPort = .*/vhostHTTPPort = ${FRP_HTTP_PORT}/" \
    ci-helpers/configs/frps.toml > "${FRPS_CONFIG}"
  sed \
    -e "s/^serverPort = .*/serverPort = ${FRP_SERVER_PORT}/" \
    ci-helpers/configs/frpc.toml > "${FRPC_CONFIG}"
}

cleanup_frp_tmp() {
  rm -rf "${FRP_TMP_DIR:-}"
  FRP_TMP_DIR=""
}

cleanup_frp() {
  sudo systemctl stop frp-client.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-client.scope 2>/dev/null || true
  sudo systemctl stop frp-server.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-server.scope 2>/dev/null || true
  sudo systemctl reset-failed frp-client.scope frp-server.scope 2>/dev/null || true
  sudo pkill -x frpc 2>/dev/null || true
  sudo pkill -x frps 2>/dev/null || true
  if [ -n "${FRP_SERVER_PORT}" ]; then sudo fuser -k "${FRP_SERVER_PORT}/tcp" 2>/dev/null || true; fi
  if [ -n "${FRP_HTTP_PORT}" ]; then sudo fuser -k "${FRP_HTTP_PORT}/tcp" 2>/dev/null || true; fi
}

COLLECT_PID=""
cleanup_case() {
  if [ -n "${COLLECT_PID:-}" ]; then
    kill -9 "${COLLECT_PID}" 2>/dev/null || true
  fi
  if [ "${IS_FRP:-0}" -eq 1 ]; then
    cleanup_frp
    cleanup_frp_tmp
  fi
}
trap cleanup_case EXIT

SERVER_LOG_PATH="/tmp/ci-server.log"
CLIENT_LOG_PATH="/tmp/ci-client.log"

port_is_listening() {
  ss -H -tln "sport = :$1" 2>/dev/null | grep -q .
}

wait_for_frp_ports_free() {
  for i in $(seq 1 20); do
    if ! port_is_listening "${FRP_SERVER_PORT}" && ! port_is_listening "${FRP_HTTP_PORT}"; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

dump_frp_port_diag() {
  echo "=== frp port diagnostics ==="
  echo "frp server port: ${FRP_SERVER_PORT:-unset}, http port: ${FRP_HTTP_PORT:-unset}"
  sudo ss -tlnp "( sport = :${FRP_SERVER_PORT} or sport = :${FRP_HTTP_PORT} )" 2>/dev/null || true
  sudo ss -tanp "( sport = :${FRP_SERVER_PORT} or sport = :${FRP_HTTP_PORT} )" 2>/dev/null || true
  sudo lsof -nP -iTCP:"${FRP_SERVER_PORT}" -iTCP:"${FRP_HTTP_PORT}" -sTCP:LISTEN 2>/dev/null || true
}

start_frp_stack() {
  : > /tmp/frps.log
  : > /tmp/frpc.log

  sudo systemd-run --scope --unit=frp-server --collect \
    "${FRP_SERVER_CPU_ARGS[@]}" -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
    -- frps -c "${FRPS_CONFIG}" >> /tmp/frps.log 2>&1 &

  for i in $(seq 1 20); do
    if sudo systemctl is-failed frp-server.scope > /dev/null 2>&1; then
      return 1
    fi
    if port_is_listening "${FRP_SERVER_PORT}" && port_is_listening "${FRP_HTTP_PORT}"; then
      break
    fi
    sleep 0.3
  done
  if ! port_is_listening "${FRP_SERVER_PORT}" || ! port_is_listening "${FRP_HTTP_PORT}"; then
    return 1
  fi

  sudo systemd-run --scope --unit=frp-client --collect \
    "${FRP_CLIENT_CPU_ARGS[@]}" -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
    -- frpc -c "${FRPC_CONFIG}" >> /tmp/frpc.log 2>&1 &

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
if [ "$IS_FRP" -eq 0 ]; then
  bash ci-helpers/warmup.sh ctld 8080 "$SERVER_LOG_PATH" "$CLIENT_LOG_PATH"
fi

if [ "$IS_FRP" -eq 1 ]; then
  FRP_READY=0
  for attempt in $(seq 1 3); do
    cleanup_frp
    FRP_SERVER_PORT="${REQUESTED_FRP_SERVER_PORT}"
    FRP_HTTP_PORT="${REQUESTED_FRP_HTTP_PORT}"
    pick_frp_ports
    prepare_frp_configs
    echo "Starting frp attempt ${attempt} with server port ${FRP_SERVER_PORT}, http port ${FRP_HTTP_PORT}"
    if ! wait_for_frp_ports_free; then
      echo "WARNING: selected frp ports are occupied before start"
      dump_frp_port_diag
      cleanup_frp_tmp
      FRP_SERVER_PORT=""
      FRP_HTTP_PORT=""
      continue
    fi
    if start_frp_stack; then
      FRP_READY=1
      break
    fi
    cleanup_frp
    wait_for_frp_ports_free || true
    cleanup_frp_tmp
    FRP_SERVER_PORT=""
    FRP_HTTP_PORT=""
    sleep 1
  done

  if [ "$FRP_READY" -eq 0 ]; then
    echo "WARNING: frp did not become ready, skipping $CASE_NAME"
    dump_frp_port_diag
    echo "=== frp-server scope ===" && sudo systemctl status frp-server.scope 2>&1 | tail -20 || true
    echo "=== frp-client scope ===" && sudo systemctl status frp-client.scope 2>&1 | tail -20 || true
    echo "=== frps ===" && cat /tmp/frps.log || true
    echo "=== frpc ===" && cat /tmp/frpc.log || true
    cleanup_frp_tmp
    exit 1
  fi
fi

bash ci-helpers/benchmark-env.sh configure "$CASE_NAME" "$SERVER_LOG_PATH" "$CLIENT_LOG_PATH"

python3 ci-helpers/bench-tool.py collect 1 > "/tmp/collect-${CASE_NAME}.jsonl" 2>/dev/null &
echo $! > "/tmp/collect-${CASE_NAME}.pid"
COLLECT_PID="$(cat "/tmp/collect-${CASE_NAME}.pid")"
COLLECT_ENABLED=1
if [ "${DUOTUNNEL_COLLECT_RESOURCE_METRICS:-1}" = "0" ] || [ "${DUOTUNNEL_COLLECT_RESOURCE_METRICS}" = "false" ]; then
  kill -9 "${COLLECT_PID}" 2>/dev/null || true
  rm -f "/tmp/collect-${CASE_NAME}.jsonl" "/tmp/collect-${CASE_NAME}.pid"
  COLLECT_ENABLED=0
else
  bash ci-helpers/benchmark-env.sh pin-pid "$CASE_NAME" "$COLLECT_PID" 2>/dev/null || true
fi

K6_ENV=(
  -e GITHUB_SHA="${GITHUB_SHA}"
  -e BENCH_CASE="${CASE_NAME}"
  -e FRP_HTTP_PORT="${FRP_HTTP_PORT}"
  -e BENCH_RESULT_PATH="/tmp/bench-results-${CASE_NAME}.json"
)
if [ -n "${K6_CORE_STRESS_RATE}" ]; then
  K6_ENV+=(-e K6_CORE_STRESS_RATE="${K6_CORE_STRESS_RATE}")
fi

K6_START_MS=$(date +%s%3N)
set +e
(cd ci-helpers/k6 && bash ../../ci-helpers/benchmark-env.sh run-load "$CASE_NAME" k6 run "${K6_ENV[@]}" bench.js)
K6_STATUS=$?
set -e
K6_END_MS=$(date +%s%3N)

python3 ci-helpers/bench-tool.py attach \
  --result "/tmp/bench-results-${CASE_NAME}.json" \
  --metadata "/tmp/benchmark-snapshot-${CASE_NAME}.json" \
  --case-name "$CASE_NAME"

if [ "${COLLECT_ENABLED}" = "1" ]; then
  kill -9 "$(cat "/tmp/collect-${CASE_NAME}.pid" 2>/dev/null)" 2>/dev/null || true
  sleep 0.5

  python3 ci-helpers/bench-tool.py parse \
    --input  "/tmp/collect-${CASE_NAME}.jsonl" \
    --k6-start-ms "${K6_START_MS}" \
    --k6-end-ms "${K6_END_MS}" \
    --output "/tmp/resource-${CASE_NAME}.json" || true

  if [ -s "/tmp/resource-${CASE_NAME}.json" ] && [ -s "/tmp/bench-results-${CASE_NAME}.json" ]; then
    python3 ci-helpers/bench-tool.py inject \
      --result    "/tmp/bench-results-${CASE_NAME}.json" \
      --resources "/tmp/resource-${CASE_NAME}.json" \
      --case-name "${CASE_NAME}" || true
  fi
  COLLECT_PID=""
fi

if [ "$IS_FRP" -eq 1 ]; then
  cleanup_frp
  wait_for_frp_ports_free || true
  cleanup_frp_tmp
fi

GATE_ARGS=(
  --result "/tmp/bench-results-${CASE_NAME}.json"
  --output "/tmp/benchmark-gate-${CASE_NAME}.json"
)
if [ "${DUOTUNNEL_BENCHMARK_REQUIRE_COMPARABLE:-1}" = "1" ]; then
  GATE_ARGS+=(--require-comparable)
fi
MAX_DROPPED_ITERATIONS="${DUOTUNNEL_BENCHMARK_MAX_DROPPED_ITERATIONS:-3000}"
if [ "$MAX_DROPPED_ITERATIONS" != "-1" ]; then
  GATE_ARGS+=(--max-dropped-iterations "$MAX_DROPPED_ITERATIONS")
fi
if ! python3 ci-helpers/bench-tool.py gate "${GATE_ARGS[@]}"; then
  exit 1
fi

exit "$K6_STATUS"
