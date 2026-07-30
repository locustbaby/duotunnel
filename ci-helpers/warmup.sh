#!/bin/bash
set -e

MODE=${1:-ctld}
PORT=${2:-8080}
SERVER_LOG=${3:-/tmp/ci-server.log}
CLIENT_LOG=${4:-/tmp/ci-client.log}
RETRIES=10

source ci-helpers/cpu-contract.sh
cpu_contract_resolve "${DUOTUNNEL_BENCH_CPU_MODE:-isolate}" "${WORKER_THREADS:-0}" "${STRESS_CPU_QUOTA:-100%}"
mapfile -t SERVER_CPU_ARGS < <(cpu_contract_scope_args server)
mapfile -t CLIENT_CPU_ARGS < <(cpu_contract_scope_args client)

TOKIO_ENV=()
if [ "${WORKER_THREADS:-0}" != "0" ]; then
  TOKIO_ENV=(-E TOKIO_WORKER_THREADS="${WORKER_THREADS}")
fi

SERVER_TRACE_ENV=()
CLIENT_TRACE_ENV=()
if [ -n "${DIAL9_SERVER_TRACE_PATH:-}" ]; then
  SERVER_TRACE_ENV=(-E DIAL9_TRACE_PATH="${DIAL9_SERVER_TRACE_PATH}")
fi
if [ -n "${DIAL9_CLIENT_TRACE_PATH:-}" ]; then
  CLIENT_TRACE_ENV=(-E DIAL9_TRACE_PATH="${DIAL9_CLIENT_TRACE_PATH}")
fi

wait_unit_gone() {
  for i in $(seq 1 30); do
    S=$(systemctl is-active duotunnel-server.scope 2>/dev/null || echo "gone")
    C=$(systemctl is-active duotunnel-client.scope 2>/dev/null || echo "gone")
    if [ "$S" != "active" ] && [ "$S" != "activating" ] && [ "$S" != "deactivating" ] && \
       [ "$C" != "active" ] && [ "$C" != "activating" ] && [ "$C" != "deactivating" ]; then
      return 0
    fi
    sleep 0.5
  done
}

warmup_probe() {
  echo "Probing ingress (echo.local:$PORT)..."
  for i in $(seq 1 $RETRIES); do
    if curl -sf --max-time 3 "http://echo.local:$PORT/" > /dev/null 2>&1; then
      echo "  ingress OK after probe $i"
      break
    fi
    [ "$i" -eq "$RETRIES" ] && return 1
    sleep 0.5
  done

  echo "Probing egress (127.0.0.1:8082)..."
  for i in $(seq 1 $RETRIES); do
    if curl -sf --max-time 3 -H "Host: echo.local" "http://127.0.0.1:8082/" > /dev/null 2>&1; then
      echo "  egress OK after probe $i"
      break
    fi
    [ "$i" -eq "$RETRIES" ] && return 2
    sleep 0.5
  done
  return 0
}

if warmup_probe; then
  echo "Tunnel warm and serving"
  exit 0
fi

echo "WARNING: warmup failed, restarting client then server..."
echo "=== client log ===" && tail -30 "$CLIENT_LOG" 2>/dev/null || true
echo "=== server log ===" && tail -30 "$SERVER_LOG" 2>/dev/null || true

sudo systemctl stop duotunnel-client.scope 2>/dev/null || true
sudo systemctl kill -s KILL duotunnel-client.scope 2>/dev/null || true
sudo systemctl stop duotunnel-server.scope 2>/dev/null || true
sudo systemctl kill -s KILL duotunnel-server.scope 2>/dev/null || true
wait_unit_gone

echo "Restarting server..."
sudo systemd-run --scope --unit=duotunnel-server --collect \
  "${SERVER_CPU_ARGS[@]}" -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  "${TOKIO_ENV[@]}" \
  "${SERVER_TRACE_ENV[@]}" \
  -- ./target/release/duotunnel-server --config ci-helpers/configs/server.yaml \
  --ctld-addr 127.0.0.1:7788 >> "$SERVER_LOG" 2>&1 &

SERVER_UP=0
for i in $(seq 1 60); do
  curl -sf --max-time 1 http://127.0.0.1:9090/healthz > /dev/null 2>&1 && SERVER_UP=1 && break
  sleep 0.5
done
if [ "$SERVER_UP" -eq 0 ]; then
  echo "FAIL: server did not restart"
  tail -20 "$SERVER_LOG" 2>/dev/null || true
  exit 1
fi

echo "Restarting client..."
CLIENT_TOKEN=$(cat /tmp/ci-token 2>/dev/null || true)
if [ -z "$CLIENT_TOKEN" ]; then
  echo "FAIL: /tmp/ci-token is missing; cannot restart client without its ctld token"
  exit 1
fi
sudo systemd-run --scope --unit=duotunnel-client --collect \
  "${CLIENT_CPU_ARGS[@]}" -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  "${TOKIO_ENV[@]}" \
  "${CLIENT_TRACE_ENV[@]}" \
  -E DUOTUNNEL_CLIENT__AUTH_TOKEN="$CLIENT_TOKEN" \
  -- ./target/release/duotunnel-client --config ci-helpers/configs/client.yaml >> "$CLIENT_LOG" 2>&1 &

CLIENT_UP=0
for i in $(seq 1 60); do
  if systemctl is-failed duotunnel-client.scope > /dev/null 2>&1; then break; fi
  curl -sf --max-time 1 http://127.0.0.1:9092/healthz > /dev/null 2>&1 && CLIENT_UP=1 && break
  sleep 0.5
done
if [ "$CLIENT_UP" -eq 0 ]; then
  echo "FAIL: client did not restart"
  tail -20 "$CLIENT_LOG" 2>/dev/null || true
  exit 1
fi

if warmup_probe; then
  echo "Tunnel warm and serving (after restart)"
  exit 0
fi

echo "FAIL: tunnel not serving after restart"
exit 1
