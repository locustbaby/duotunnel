#!/bin/bash
set -eo pipefail

CASE_NAME=$1
GITHUB_SHA=$2
BUILD_PROFILE=${3:-dial9}
WORKER_THREADS=${4:-0}
export WORKER_THREADS

source ci-helpers/cpu-contract.sh
cpu_contract_resolve "${DUOTUNNEL_BENCH_CPU_MODE:-isolate}" "$WORKER_THREADS" "${STRESS_CPU_QUOTA:-100%}"
mapfile -t SERVER_CPU_ARGS < <(cpu_contract_scope_args server)
mapfile -t CLIENT_CPU_ARGS < <(cpu_contract_scope_args client)

SUFFIX="${CASE_NAME}"
echo "==> Running Case: ${CASE_NAME} (Profile: ${BUILD_PROFILE}, WorkerThreads: ${WORKER_THREADS})"

COLLECT_PID=""
cleanup_trace_case() {
  if [ -n "${COLLECT_PID:-}" ]; then
    kill -9 "${COLLECT_PID}" 2>/dev/null || true
  fi
  sudo systemctl stop duotunnel-client.scope 2>/dev/null || true
  sudo systemctl stop duotunnel-server.scope 2>/dev/null || true
  sudo systemctl kill -s KILL duotunnel-client.scope 2>/dev/null || true
  sudo systemctl kill -s KILL duotunnel-server.scope 2>/dev/null || true
}
trap cleanup_trace_case EXIT

verify_dial9_started() {
  local log_path="$1"
  local trace_path="$2"
  for _ in $(seq 1 20); do
    if grep -q -E "dial9 trace started|dial9_worker: worker started" "$log_path" 2>/dev/null || \
      trace_file_exists "$trace_path"; then
      return 0
    fi
    sleep 0.5
  done
  echo "ERROR: Dial9 did not report startup for ${trace_path}" >&2
  tail -40 "$log_path" 2>/dev/null || true
  sudo find "${trace_path%/*}" -maxdepth 1 -type f \
    -name "$(basename "${trace_path%.bin}").*" -ls 2>/dev/null || true
  return 1
}

trace_file_exists() {
  local trace_path="$1"
  local trace_stem="${trace_path%.bin}"
  sudo test -e "${trace_path}.active" || \
    sudo test -e "${trace_stem}.0.bin.active" || \
    sudo test -e "${trace_stem}.0.bin"
}

trace_file_active() {
  local trace_path="$1"
  local trace_stem="${trace_path%.bin}"
  sudo test -e "${trace_path}.active" || \
    sudo test -e "${trace_stem}.0.bin.active"
}

find_trace_output() {
  local trace_path="$1"
  local trace_dir="${trace_path%/*}"
  local trace_stem="$(basename "${trace_path%.bin}")"
  sudo find "$trace_dir" -maxdepth 1 -type f \
    -name "${trace_stem}.*.bin.gz" -size +0c -printf '%T@ %p\n' 2>/dev/null |
    sort -nr | sed -n '1s/^[^ ]* //p'
}

TOKIO_ENV=()
if [ "${WORKER_THREADS}" != "0" ]; then
  TOKIO_ENV=(-E TOKIO_WORKER_THREADS="${WORKER_THREADS}")
fi

# 1. Cleanup server/client scopes from previous case
rm -f /tmp/bench-results.json
sudo rm -f /tmp/server-trace.bin.active /tmp/client-trace.bin.active \
  /tmp/server-trace.*.bin /tmp/client-trace.*.bin \
  /tmp/server-trace.*.bin.active /tmp/client-trace.*.bin.active \
  /tmp/server-trace.*.bin.gz /tmp/client-trace.*.bin.gz
rm -f /tmp/collect.jsonl /tmp/collect-resources-err.log /tmp/collect-resources.pid /tmp/ss-timeseries.log /tmp/ss-loop.pid
sudo systemctl stop duotunnel-client.scope 2>/dev/null || true
sudo systemctl stop duotunnel-server.scope 2>/dev/null || true
sudo systemctl kill -s KILL duotunnel-client.scope 2>/dev/null || true
sudo systemctl kill -s KILL duotunnel-server.scope 2>/dev/null || true
for i in $(seq 1 20); do
  S=$(systemctl is-active duotunnel-server.scope 2>/dev/null || echo "gone")
  C=$(systemctl is-active duotunnel-client.scope 2>/dev/null || echo "gone")
  if [ "$S" != "active" ] && [ "$S" != "activating" ] && [ "$S" != "deactivating" ] && \
     [ "$C" != "active" ] && [ "$C" != "activating" ] && [ "$C" != "deactivating" ]; then
    break
  fi
  sleep 0.5
done

# 2. Start backends + ctld + token (first case only; reused across cases)
if [ ! -f /tmp/trace-8k-initialized ]; then
  ./target/release/http-echo-server 9999  > /tmp/http-echo.log  2>&1 & echo $! > /tmp/http-echo.pid
  ./target/release/ws-echo-server   8765  > /tmp/ws-echo.log    2>&1 & echo $! > /tmp/ws-echo.pid
  ./target/release/grpc-echo-server 50051 > /tmp/grpc-echo.log  2>&1 & echo $! > /tmp/grpc-echo.pid

  for i in $(seq 1 30); do
    curl -sf --max-time 2 http://127.0.0.1:9999/ > /dev/null && break
    sleep 0.5
  done
  sleep 0.5

  mkdir -p data
  ./target/release/duotunnel-ctld --config ci-helpers/configs/ctld.yaml > /tmp/ci-ctld.log 2>&1 &
  echo $! > /tmp/ctld.pid
  for i in $(seq 1 60); do
    curl -sf --max-time 1 http://127.0.0.1:9091/healthz > /dev/null 2>&1 && break
    sleep 0.5
  done
  curl -sf --max-time 2 http://127.0.0.1:9091/healthz > /dev/null 2>&1 || {
    echo "ERROR: ctld did not start"
    tail -20 /tmp/ci-ctld.log || true
    exit 1
  }

  TOKEN=$(./target/release/duotunnel-ctld --config ci-helpers/configs/ctld.yaml \
    client create ci-group 2>/dev/null | grep '^Token:' | awk '{print $2}' \
    || ./target/release/duotunnel-ctld --config ci-helpers/configs/ctld.yaml \
    client rotate ci-group 2>/dev/null | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
  TOKEN=$(echo "$TOKEN" | tr -cd '[:print:]')
  printf '%s' "$TOKEN" > /tmp/trace-8k-token

  touch /tmp/trace-8k-initialized
fi

TOKEN=$(cat /tmp/trace-8k-token)

# 3. Start Server
sudo systemd-run --scope --unit=duotunnel-server --collect \
  "${SERVER_CPU_ARGS[@]}" -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  -E DIAL9_TRACE_PATH=/tmp/server-trace.bin \
  "${TOKIO_ENV[@]}" \
  -- ./target/release/duotunnel-server --config ci-helpers/configs/server.yaml \
  --ctld-addr 127.0.0.1:7788 >> "/tmp/ci-server-${SUFFIX}.log" 2>&1 &

SERVER_UP=0
for i in $(seq 1 60); do
  curl -sf --max-time 1 http://127.0.0.1:9090/healthz > /dev/null 2>&1 && SERVER_UP=1 && break
  sleep 0.5
done
if [ "$SERVER_UP" -eq 0 ]; then
  echo "ERROR: server did not start (healthz timeout)"
  echo "=== systemd unit status ===" && systemctl status duotunnel-server.scope 2>&1 | tail -20 || true
  echo "=== server log ===" && tail -30 "/tmp/ci-server-${SUFFIX}.log" 2>/dev/null || true
  exit 1
fi
if [ "$BUILD_PROFILE" != "release" ]; then
  verify_dial9_started "/tmp/ci-server-${SUFFIX}.log" /tmp/server-trace.bin
fi

# 4. Start Client
# sleep 3: allow server to re-register with ctld and receive the token before client connects
sleep 3

sudo systemd-run --scope --unit=duotunnel-client --collect \
  "${CLIENT_CPU_ARGS[@]}" -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  -E DIAL9_TRACE_PATH=/tmp/client-trace.bin \
  "${TOKIO_ENV[@]}" \
  -E DUOTUNNEL_CLIENT__AUTH_TOKEN="$TOKEN" \
  -- ./target/release/duotunnel-client --config ci-helpers/configs/client.yaml >> "/tmp/ci-client-${SUFFIX}.log" 2>&1 &

CLIENT_UP=0
for i in $(seq 1 60); do
  if systemctl is-failed duotunnel-client.scope > /dev/null 2>&1; then break; fi
  curl -sf --max-time 1 http://127.0.0.1:9092/healthz > /dev/null 2>&1 && CLIENT_UP=1 && break
  sleep 0.5
done
if [ "$CLIENT_UP" -eq 0 ]; then
  echo "ERROR: client did not start (healthz timeout or scope failed)"
  echo "=== systemd unit status ===" && systemctl status duotunnel-client.scope 2>&1 | tail -20 || true
  echo "=== client log ===" && tail -30 "/tmp/ci-client-${SUFFIX}.log" 2>/dev/null || true
  exit 1
fi
if [ "$BUILD_PROFILE" != "release" ]; then
  verify_dial9_started "/tmp/ci-client-${SUFFIX}.log" /tmp/client-trace.bin
fi

# 5. Warmup
chmod +x ci-helpers/warmup.sh
DIAL9_SERVER_TRACE_PATH=/tmp/server-trace.bin \
DIAL9_CLIENT_TRACE_PATH=/tmp/client-trace.bin \
  ./ci-helpers/warmup.sh ctld 8080 "/tmp/ci-server-${SUFFIX}.log" "/tmp/ci-client-${SUFFIX}.log"

bash ci-helpers/benchmark-env.sh configure "$CASE_NAME" \
  "/tmp/ci-server-${SUFFIX}.log" "/tmp/ci-client-${SUFFIX}.log"

# 6. Collect & k6
COLLECT_ENABLED=1
if [ "${DUOTUNNEL_COLLECT_RESOURCE_METRICS:-1}" = "0" ] || [ "${DUOTUNNEL_COLLECT_RESOURCE_METRICS}" = "false" ]; then
  COLLECT_ENABLED=0
fi
if [ "${COLLECT_ENABLED}" = "1" ]; then
  python3 ci-helpers/bench-tool.py collect 1 > /tmp/collect.jsonl 2>/tmp/collect-resources-err.log &
  COLLECT_PID=$!
  echo "${COLLECT_PID}" > /tmp/collect-resources.pid
  bash ci-helpers/benchmark-env.sh pin-pid "$CASE_NAME" "${COLLECT_PID}" 2>/dev/null || true
fi

K6_START_MS=$(date +%s%3N)
set +e
(cd ci-helpers/k6 && bash ../../ci-helpers/benchmark-env.sh run-load "$CASE_NAME" k6 run -e GITHUB_SHA="${GITHUB_SHA}" -e BENCH_PROFILE=8k -e BENCH_CASE="${CASE_NAME}" bench.js)
K6_STATUS=$?
set -e
K6_END_MS=$(date +%s%3N)

if [ -s /tmp/bench-results.json ]; then
  python3 ci-helpers/bench-tool.py attach \
    --result /tmp/bench-results.json \
    --metadata "/tmp/benchmark-snapshot-${CASE_NAME}.json" \
    --case-name "$CASE_NAME"
fi

# 7. Stop resource sampling
if [ "${COLLECT_ENABLED}" = "1" ]; then
  kill -9 "$(cat /tmp/collect-resources.pid 2>/dev/null)" 2>/dev/null || true
  COLLECT_PID=""
  pkill -9 -f "bench-tool.py collect" 2>/dev/null || true
  sleep 0.5

  # 8. Parse
  python3 ci-helpers/bench-tool.py parse \
    --input /tmp/collect.jsonl \
    --k6-start-ms "${K6_START_MS}" \
    --k6-end-ms "${K6_END_MS}" \
    --output "/tmp/resource-data-${CASE_NAME}.json" || true

  # 8b. Inject resource into bench-results.json case
  if [ -s "/tmp/resource-data-${CASE_NAME}.json" ] && [ -s /tmp/bench-results.json ]; then
    python3 ci-helpers/bench-tool.py inject \
      --result    /tmp/bench-results.json \
      --resources "/tmp/resource-data-${CASE_NAME}.json" \
      --case-name "${CASE_NAME}" || true
  fi
fi

# 9. Stop server/client, wait for trace flush
sudo systemctl stop duotunnel-client.scope 2>/dev/null || true
sudo systemctl stop duotunnel-server.scope 2>/dev/null || true
for i in $(seq 1 60); do
  ! trace_file_active /tmp/server-trace.bin && ! trace_file_active /tmp/client-trace.bin && break
  sleep 0.5
done
sudo systemctl kill -s KILL duotunnel-client.scope 2>/dev/null || true
sudo systemctl kill -s KILL duotunnel-server.scope 2>/dev/null || true
for i in $(seq 1 30); do
  S=$(systemctl is-active duotunnel-server.scope 2>/dev/null || echo "gone")
  C=$(systemctl is-active duotunnel-client.scope 2>/dev/null || echo "gone")
  if [ "$S" != "active" ] && [ "$S" != "activating" ] && [ "$S" != "deactivating" ] && \
     [ "$C" != "active" ] && [ "$C" != "activating" ] && [ "$C" != "deactivating" ]; then
    break
  fi
  sleep 0.5
done

# 10. Archive artifacts
mkdir -p "/tmp/trace-${SUFFIX}"
cp /tmp/bench-results.json "/tmp/trace-${SUFFIX}/bench-results.json"
[ -f "/tmp/resource-data-${CASE_NAME}.json" ] && cp "/tmp/resource-data-${CASE_NAME}.json" "/tmp/trace-${SUFFIX}/resource-data.json" || true

if [ "$BUILD_PROFILE" != "release" ]; then
  SERVER_TRACE_OUTPUT="$(find_trace_output /tmp/server-trace.bin)"
  CLIENT_TRACE_OUTPUT="$(find_trace_output /tmp/client-trace.bin)"
  if [ -n "$SERVER_TRACE_OUTPUT" ] && [ -n "$CLIENT_TRACE_OUTPUT" ]; then
    sudo cp "$SERVER_TRACE_OUTPUT" "/tmp/trace-${SUFFIX}/server-trace.bin.gz"
    sudo cp "$CLIENT_TRACE_OUTPUT" "/tmp/trace-${SUFFIX}/client-trace.bin.gz"
    sudo chown runner:runner "/tmp/trace-${SUFFIX}/server-trace.bin.gz" "/tmp/trace-${SUFFIX}/client-trace.bin.gz"
  else
    echo "ERROR: Expected trace files not found or empty!"
    [ -z "$SERVER_TRACE_OUTPUT" ] && echo "  - no non-empty server trace output found"
    [ -z "$CLIENT_TRACE_OUTPUT" ] && echo "  - no non-empty client trace output found"
    echo "  Contents of /tmp matching trace pattern:"
    ls -la /tmp/server-trace* /tmp/client-trace* 2>/dev/null || echo "    (none)"
    exit 1
  fi
fi

cp "/tmp/ci-server-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-server.log"
cp "/tmp/ci-client-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-client.log"
cp /tmp/ci-ctld.log "/tmp/trace-${SUFFIX}/ci-ctld.log"
cp /tmp/http-echo.log "/tmp/trace-${SUFFIX}/http-echo.log"
cp /tmp/ws-echo.log "/tmp/trace-${SUFFIX}/ws-echo.log"
cp /tmp/grpc-echo.log "/tmp/trace-${SUFFIX}/grpc-echo.log"

if [ -s /tmp/bench-results.json ]; then
  GATE_ARGS=(
    --result /tmp/bench-results.json
    --output "/tmp/trace-${SUFFIX}/benchmark-gate.json"
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
else
  echo "ERROR: k6 did not produce /tmp/bench-results.json"
  exit 1
fi

exit "${K6_STATUS}"
