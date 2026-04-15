#!/bin/bash
set -eo pipefail

CASE_NAME=$1
GITHUB_SHA=$2
QUIC_CONNS=${3:-4}
BUILD_PROFILE=${4:-dial9}
TOKIO_ENV=${5:-""}

SUFFIX="${CASE_NAME}"
echo "==> Running Case: ${CASE_NAME} (QUIC: ${QUIC_CONNS}, Profile: ${BUILD_PROFILE})"

# 1. Cleanup server/client scopes from previous case
rm -f /tmp/bench-results.json
sudo rm -f /tmp/server-trace.*.bin /tmp/client-trace.*.bin /tmp/server-trace.*.bin.active /tmp/client-trace.*.bin.active /tmp/server-trace.*.bin.gz /tmp/client-trace.*.bin.gz
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
  ./target/release/tunnel-ctld --config ci-helpers/configs/ctld.yaml > /tmp/ci-ctld.log 2>&1 &
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

  TOKEN=$(./target/release/tunnel-ctld --config ci-helpers/configs/ctld.yaml \
    client create-client ci-group 2>/dev/null | grep '^Token:' | awk '{print $2}' \
    || ./target/release/tunnel-ctld --config ci-helpers/configs/ctld.yaml \
    client rotate-token ci-group 2>/dev/null | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
  TOKEN=$(echo "$TOKEN" | tr -cd '[:print:]')

  python3 -c "import re,sys; f='ci-helpers/configs/client.yaml'; t=sys.argv[1]; n=sys.argv[2]; c=open(f).read(); c=re.sub(r'^auth_token:.*','auth_token: \"'+t+'\"',c,flags=re.MULTILINE); c=re.sub(r'^(\\s*connections:)\\s*\\d+', r'\\1 '+n, c, flags=re.MULTILINE); open(f,'w').write(c)" "$TOKEN" "$QUIC_CONNS"

  touch /tmp/trace-8k-initialized
fi

# 3. Start Server
sudo systemd-run --scope --unit=duotunnel-server --collect \
  -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  -E DIAL9_TRACE_PATH=/tmp/server-trace.bin \
  ${TOKIO_ENV:+-E $TOKIO_ENV} \
  -- ./target/release/server --config ci-helpers/configs/server.yaml \
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

# 4. Start Client
# sleep 3: allow server to re-register with ctld and receive the token before client connects
sleep 3

sudo systemd-run --scope --unit=duotunnel-client --collect \
  -p CPUQuota=50% -p CPUWeight=1024 -p MemoryMax=2G -p MemoryLow=256M \
  -E DIAL9_TRACE_PATH=/tmp/client-trace.bin \
  ${TOKIO_ENV:+-E $TOKIO_ENV} \
  -- ./target/release/client --config ci-helpers/configs/client.yaml >> "/tmp/ci-client-${SUFFIX}.log" 2>&1 &

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

# 5. Warmup
chmod +x ci-helpers/warmup.sh
./ci-helpers/warmup.sh ctld 8080 "/tmp/ci-server-${SUFFIX}.log" "/tmp/ci-client-${SUFFIX}.log"

# 6. Collect & k6
python3 ci-helpers/bench-tool.py collect 1 > /tmp/collect.jsonl 2>/tmp/collect-resources-err.log &
echo $! > /tmp/collect-resources.pid

(cd ci-helpers/k6 && k6 run -e GITHUB_SHA=${GITHUB_SHA} -e BENCH_PROFILE=8k -e BENCH_CASE="${CASE_NAME}" bench.js)

# 7. Stop resource sampling
kill -9 "$(cat /tmp/collect-resources.pid 2>/dev/null)" 2>/dev/null || true
pkill -9 -f "bench-tool.py collect" 2>/dev/null || true
sleep 0.5

# 8. Parse
python3 ci-helpers/bench-tool.py parse --input /tmp/collect.jsonl --k6-offset 0 --output "/tmp/resource-data-${CASE_NAME}.json" || true

# 8b. Inject resource into bench-results.json case
if [ -s "/tmp/resource-data-${CASE_NAME}.json" ] && [ -s /tmp/bench-results.json ]; then
  python3 ci-helpers/bench-tool.py inject \
    --result    /tmp/bench-results.json \
    --resources "/tmp/resource-data-${CASE_NAME}.json" \
    --case-name "${CASE_NAME}" || true
fi

# 9. Stop server/client, wait for trace flush
sudo systemctl stop duotunnel-client.scope 2>/dev/null || true
sudo systemctl stop duotunnel-server.scope 2>/dev/null || true
for i in $(seq 1 60); do
  [ ! -f /tmp/server-trace.0.bin.active ] && [ ! -f /tmp/client-trace.0.bin.active ] && break
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
  if [ -s /tmp/server-trace.0.bin ] && [ -s /tmp/client-trace.0.bin ]; then
    sudo cp /tmp/server-trace.0.bin "/tmp/trace-${SUFFIX}/server-trace.bin.gz"
    sudo cp /tmp/client-trace.0.bin "/tmp/trace-${SUFFIX}/client-trace.bin.gz"
    sudo chown runner:runner "/tmp/trace-${SUFFIX}/server-trace.bin.gz" "/tmp/trace-${SUFFIX}/client-trace.bin.gz"
  fi
fi

cp "/tmp/ci-server-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-server.log"
cp "/tmp/ci-client-${SUFFIX}.log" "/tmp/trace-${SUFFIX}/ci-client.log"
cp /tmp/ci-ctld.log "/tmp/trace-${SUFFIX}/ci-ctld.log"
cp /tmp/http-echo.log "/tmp/trace-${SUFFIX}/http-echo.log"
cp /tmp/ws-echo.log "/tmp/trace-${SUFFIX}/ws-echo.log"
cp /tmp/grpc-echo.log "/tmp/trace-${SUFFIX}/grpc-echo.log"
