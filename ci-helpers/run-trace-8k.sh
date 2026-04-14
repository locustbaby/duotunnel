#!/bin/bash
set -eo pipefail

CASE_NAME=$1
GITHUB_SHA=$2
QUIC_CONNS=${3:-4}
BUILD_PROFILE=${4:-dial9}
TOKIO_ENV=${5:-""}

SUFFIX="${CASE_NAME}"
echo "==> Running Case: ${CASE_NAME} (QUIC: ${QUIC_CONNS}, Profile: ${BUILD_PROFILE})"

# 1. Cleanup before case
rm -f /tmp/bench-results.json
sudo rm -f /tmp/server-trace.*.bin /tmp/client-trace.*.bin /tmp/server-trace.*.bin.active /tmp/client-trace.*.bin.active /tmp/server-trace.*.bin.gz /tmp/client-trace.*.bin.gz
rm -f /tmp/collect.jsonl /tmp/collect-resources-err.log /tmp/collect-resources.pid /tmp/ss-timeseries.log /tmp/ss-loop.pid

# 2. Start backends
./target/release/http-echo-server 9999  > "/tmp/http-echo-${SUFFIX}.log"  2>&1 & echo $! > /tmp/http-echo.pid
./target/release/ws-echo-server   8765  > "/tmp/ws-echo-${SUFFIX}.log"    2>&1 & echo $! > /tmp/ws-echo.pid
./target/release/grpc-echo-server 50051 > "/tmp/grpc-echo-${SUFFIX}.log"  2>&1 & echo $! > /tmp/grpc-echo.pid

for i in $(seq 1 30); do
  curl -sf --max-time 2 http://127.0.0.1:9999/ > /dev/null && break
  sleep 0.5
done
sleep 0.5

# 3. Start Tunnel
mkdir -p data
./target/release/tunnel-ctld --config ci-helpers/configs/ctld.yaml > "/tmp/ci-ctld-${SUFFIX}.log" 2>&1 &
echo $! > /tmp/ctld.pid
for i in $(seq 1 60); do
  curl -sf --max-time 1 http://127.0.0.1:9091/healthz > /dev/null 2>&1 && break
  sleep 0.5
done

# Start Server
sudo systemd-run --scope --unit=duotunnel-server --collect \
  -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  -E DIAL9_TRACE_PATH=/tmp/server-trace.bin \
  ${TOKIO_ENV:+-E $TOKIO_ENV} \
  -- ./target/release/server --config ci-helpers/configs/server.yaml \
  --ctld-addr 127.0.0.1:7788 >> "/tmp/ci-server-${SUFFIX}.log" 2>&1 &

for i in $(seq 1 60); do
  curl -sf --max-time 1 http://127.0.0.1:9090/healthz > /dev/null 2>&1 && break
  sleep 0.5
done

# Token
TOKEN=$(./target/release/tunnel-ctld --config ci-helpers/configs/ctld.yaml \
  client create-client ci-group 2>/dev/null | grep '^Token:' | awk '{print $2}' \
  || ./target/release/tunnel-ctld --config ci-helpers/configs/ctld.yaml \
  client rotate-token ci-group 2>/dev/null | awk '{print $NF}')

python3 -c "import re,sys; f='ci-helpers/configs/client.yaml'; t=sys.argv[1]; n=sys.argv[2]; c=open(f).read(); c=re.sub(r'^auth_token:.*','auth_token: \"'+t+'\"',c,flags=re.MULTILINE); c=re.sub(r'^(\\s*connections:)\\s*\\d+', r'\\1 '+n, c, flags=re.MULTILINE); open(f,'w').write(c)" "$TOKEN" "$QUIC_CONNS"
sleep 5

# Start Client
sudo systemd-run --scope --unit=duotunnel-client --collect \
  -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  -E DIAL9_TRACE_PATH=/tmp/client-trace.bin \
  ${TOKIO_ENV:+-E $TOKIO_ENV} \
  -- ./target/release/client --config ci-helpers/configs/client.yaml >> "/tmp/ci-client-${SUFFIX}.log" 2>&1 &

for i in $(seq 1 60); do
  systemctl is-failed duotunnel-client.scope > /dev/null 2>&1 && break
  curl -sf --max-time 1 http://127.0.0.1:9092/healthz > /dev/null 2>&1 && break
  sleep 0.5
done

# 4. Warmup
chmod +x ci-helpers/warmup.sh
./ci-helpers/warmup.sh ctld 8080

# 5. Collect & k6
python3 ci-helpers/bench-tool.py collect 1 > /tmp/collect.jsonl 2>/tmp/collect-resources-err.log &
echo $! > /tmp/collect-resources.pid

(cd ci-helpers/k6 && k6 run -e GITHUB_SHA=${GITHUB_SHA} -e BENCH_PROFILE=8k -e BENCH_CASE="${CASE_NAME}" bench.js)

# 6. Stop resource sampling
kill -9 "$(cat /tmp/collect-resources.pid 2>/dev/null)" 2>/dev/null || true
pkill -9 -f "bench-tool.py collect" 2>/dev/null || true
sleep 0.5

# 7. Parse & Cleanup
python3 ci-helpers/bench-tool.py parse --input /tmp/collect.jsonl --k6-offset 0 --output "/tmp/resource-data-${CASE_NAME}.json" || true

sudo systemctl stop duotunnel-client.scope 2>/dev/null || true
sudo systemctl stop duotunnel-server.scope 2>/dev/null || true
for i in $(seq 1 60); do
  [ ! -f /tmp/server-trace.0.bin.active ] && [ ! -f /tmp/client-trace.0.bin.active ] && break
  sleep 0.5
done
sudo systemctl kill -s KILL duotunnel-client.scope 2>/dev/null || true
sudo systemctl kill -s KILL duotunnel-server.scope 2>/dev/null || true
for i in $(seq 1 30); do
  ! systemctl is-active duotunnel-server.scope > /dev/null 2>&1 && \
  ! systemctl is-active duotunnel-client.scope > /dev/null 2>&1 && break
  sleep 0.5
done

# 8. Archive artifacts
mkdir -p "/tmp/trace-${SUFFIX}"
cp /tmp/bench-results.json "/tmp/trace-${SUFFIX}/bench-results.json"
[ -f "/tmp/resource-data-${CASE_NAME}.json" ] && cp "/tmp/resource-data-${CASE_NAME}.json" "/tmp/trace-${SUFFIX}/resource-data.json" || true

if [ "$BUILD_PROFILE" != "release" ]; then
  if [ -s /tmp/server-trace.0.bin ] && [ -s /tmp/client-trace.0.bin ]; then
    sudo cp /tmp/server-trace.0.bin "/tmp/trace-${SUFFIX}/server-trace.bin.gz"
    sudo cp /tmp/client-trace.0.bin "/tmp/trace-${SUFFIX}/client-trace.bin.gz"
    sudo chown runner:runner "/tmp/trace-${SUFFIX}/server-trace.bin.gz" "/tmp/trace-${SUFFIX}/client-trace.bin.gz"
  fi
fi

cp "/tmp/ci-server-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-server.log"
cp "/tmp/ci-client-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-client.log"
cp "/tmp/ci-ctld-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-ctld.log"
cp "/tmp/http-echo-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/http-echo.log"
cp "/tmp/ws-echo-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ws-echo.log"
cp "/tmp/grpc-echo-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/grpc-echo.log"

# Cleanup for next case
kill "$(cat /tmp/ctld.pid 2>/dev/null)" 2>/dev/null || true
kill "$(cat /tmp/http-echo.pid 2>/dev/null)" 2>/dev/null || true
kill "$(cat /tmp/ws-echo.pid 2>/dev/null)" 2>/dev/null || true
kill "$(cat /tmp/grpc-echo.pid 2>/dev/null)" 2>/dev/null || true
