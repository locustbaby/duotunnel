#!/usr/bin/env bash
# ci-helpers/monitor.sh
#
# Shared functions for starting and stopping resource monitoring during benchmarks.
#
# Source this file, then call:
#
#   source ci-helpers/monitor.sh
#
#   # Control which samplers run (all default to "true"):
#   #   MONITOR_CPU     — collect-resources.py via psutil
#   #   MONITOR_NET     — sar -n DEV
#   #   MONITOR_PAGING  — sar -B
#   #   MONITOR_TCP     — ss snapshot loop
#   #   MONITOR_PSI     — /proc/pressure/* loop
#
#   monitor_start   # starts all enabled samplers
#   monitor_stop    # stops all samplers, flushes data
#
# Outputs (in /tmp/):
#   collect.jsonl          — raw psutil JSONL stream
#   sar-net.log            — sar network output
#   sar-paging.log         — sar paging output
#   ss-timeseries.log      — ss TCP state snapshots
#   psi-timeseries.log     — PSI pressure metrics
#   nproc                  — CPU count
#   machine-info           — "<ncpu> <memkb>"
#   sampling_start_epoch   — unix timestamp when sampling started

: "${MONITOR_CPU:=true}"
: "${MONITOR_NET:=false}"
: "${MONITOR_PAGING:=false}"
: "${MONITOR_TCP:=true}"
: "${MONITOR_PSI:=false}"

: "${MONITOR_INTERVAL:=1}"   # seconds between samples (collect-resources.py)
: "${SAR_INTERVAL:=2}"       # seconds between sar samples

# Workspace root — override if needed
: "${WORKSPACE:=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

if ! declare -f log > /dev/null 2>&1; then
  _C='\033[0;36m'; _N='\033[0m'
  log() { echo -e "${_C}[+]${_N} $*"; }
fi

monitor_start() {
  log "Starting resource samplers (cpu=$MONITOR_CPU net=$MONITOR_NET tcp=$MONITOR_TCP psi=$MONITOR_PSI paging=$MONITOR_PAGING) ..."

  nproc > /tmp/nproc
  echo "$(nproc) $(grep MemTotal /proc/meminfo | awk '{print $2}')" > /tmp/machine-info
  date +%s > /tmp/sampling_start_epoch

  if [[ "$MONITOR_CPU" == "true" ]]; then
    pip install -q psutil 2>/dev/null || true
    python3 "$WORKSPACE/ci-helpers/collect-resources.py" "$MONITOR_INTERVAL" \
      > /tmp/collect.jsonl 2>/tmp/collect-resources-err.log &
    echo $! > /tmp/collect-resources.pid
  fi

  if [[ "$MONITOR_NET" == "true" ]]; then
    sar -n DEV "$SAR_INTERVAL" > /tmp/sar-net.log 2>&1 &
    echo $! > /tmp/sar-net.pid
  fi

  if [[ "$MONITOR_PAGING" == "true" ]]; then
    sar -B "$SAR_INTERVAL" > /tmp/sar-paging.log 2>&1 &
    echo $! > /tmp/sar-paging.pid
  fi

  if [[ "$MONITOR_TCP" == "true" ]]; then
    (while true; do
      echo "$(date +%s) $(ss -s 2>/dev/null | grep -E 'estab|timewait|closed' | tr '\n' '|')"
      sleep "$SAR_INTERVAL"
    done) > /tmp/ss-timeseries.log 2>&1 &
    echo $! > /tmp/ss-loop.pid
  fi

  if [[ "$MONITOR_PSI" == "true" ]] && [[ -f /proc/pressure/cpu ]]; then
    (while true; do
      TS=$(date +%s)
      CPU=$(awk '/^full/{for(i=1;i<=NF;i++)if($i~/^avg10=/)print substr($i,7)}' /proc/pressure/cpu 2>/dev/null)
      MEM=$(awk '/^full/{for(i=1;i<=NF;i++)if($i~/^avg10=/)print substr($i,7)}' /proc/pressure/memory 2>/dev/null)
      IO=$(awk '/^full/{for(i=1;i<=NF;i++)if($i~/^avg10=/)print substr($i,7)}' /proc/pressure/io 2>/dev/null)
      echo "$TS cpu=$CPU mem=$MEM io=$IO"
      sleep "$SAR_INTERVAL"
    done) > /tmp/psi-timeseries.log 2>&1 &
    echo $! > /tmp/psi-loop.pid
  else
    echo "PSI not available" > /tmp/psi-timeseries.log
  fi
}

monitor_stop() {
  log "Stopping resource samplers ..."
  kill -9 "$(cat /tmp/collect-resources.pid 2>/dev/null)" 2>/dev/null || true
  kill -9 "$(cat /tmp/sar-net.pid 2>/dev/null)"           2>/dev/null || true
  kill -9 "$(cat /tmp/sar-paging.pid 2>/dev/null)"        2>/dev/null || true
  kill -9 "$(cat /tmp/ss-loop.pid 2>/dev/null)"           2>/dev/null || true
  kill -9 "$(cat /tmp/psi-loop.pid 2>/dev/null)"          2>/dev/null || true
  pkill -9 -f "collect-resources.py" 2>/dev/null || true
  pkill -9 -f "sar -n"               2>/dev/null || true
  pkill -9 -f "sar -B"               2>/dev/null || true
  sleep 0.5
}

# Parse collected data into resource-data.json.
# Call after monitor_stop and after k6_start_epoch has been recorded.
#   monitor_parse <k6_start_epoch_file> <output_json>
monitor_parse() {
  local k6_epoch_file="${1:-/tmp/k6_start_epoch}"
  local output="${2:-/tmp/resource-data.json}"

  local k6_offset=0
  if [[ -f /tmp/sampling_start_epoch ]] && [[ -f "$k6_epoch_file" ]]; then
    local s k
    s=$(cat /tmp/sampling_start_epoch)
    k=$(cat "$k6_epoch_file")
    k6_offset=$(( k - s ))
  fi

  python3 "$WORKSPACE/ci-helpers/parse-resources.py" \
    --collect      /tmp/collect.jsonl \
    --sar-net      /tmp/sar-net.log \
    --sar-paging   /tmp/sar-paging.log \
    --ss           /tmp/ss-timeseries.log \
    --psi          /tmp/psi-timeseries.log \
    --k6-offset    "$k6_offset" \
    --nproc        /tmp/nproc \
    --machine-info /tmp/machine-info \
    --output       "$output" \
    || echo "Resource parsing failed (non-fatal)"
}
