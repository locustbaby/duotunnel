#!/bin/bash
set -eo pipefail

CASE_NAME=$1
GITHUB_SHA=$2

IS_FRP=0
[[ "$CASE_NAME" == frp_* ]] && IS_FRP=1

if [ "$IS_FRP" -eq 1 ]; then
  sudo systemctl stop frp-client.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-client.scope 2>/dev/null || true
  sudo systemctl stop frp-server.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-server.scope 2>/dev/null || true
  sudo fuser -k 7700/tcp 2>/dev/null || true
  sudo fuser -k 7800/tcp 2>/dev/null || true
  for i in $(seq 1 20); do
    ss -tlnp | grep -qE ':7700|:7800' || break
    sleep 0.5
  done

  sudo systemd-run --scope --unit=frp-server --collect \
    -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
    -- frps -c ci-helpers/configs/frps.toml >> /tmp/frps.log 2>&1 &
  sleep 1
  sudo systemd-run --scope --unit=frp-client --collect \
    -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
    -- frpc -c ci-helpers/configs/frpc.toml >> /tmp/frpc.log 2>&1 &
  FRP_READY=0
  for i in $(seq 1 20); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 2 -H "Host: echo.local" http://127.0.0.1:7800/ 2>/dev/null || true)
    [[ "$STATUS" =~ ^[0-9]+$ ]] && [ "$STATUS" -ge 200 ] && FRP_READY=1 && break
    sleep 0.5
  done
  if [ "$FRP_READY" -eq 0 ]; then
    echo "WARNING: frp did not become ready, skipping $CASE_NAME"
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
  sudo systemctl stop frp-client.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-client.scope 2>/dev/null || true
  sudo systemctl stop frp-server.scope 2>/dev/null || true
  sudo systemctl kill -s KILL frp-server.scope 2>/dev/null || true
  for i in $(seq 1 20); do
    ss -tlnp | grep -qE ':7700|:7800' || break
    sleep 0.5
  done
fi
