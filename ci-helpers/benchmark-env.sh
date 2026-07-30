#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOL="${ROOT_DIR}/ci-helpers/bench-tool.py"
source "${ROOT_DIR}/ci-helpers/cpu-contract.sh"

die() {
  echo "benchmark environment error: $*" >&2
  exit 1
}

write_plan() {
  local plan_path="$1"
  local env_path="$2"
  local mode="$3"
  local case_name="$4"
  local cgroup_v2="$5"
  local available="$6"
  local server_cpus="$7"
  local client_cpus="$8"
  local load_cpus="$9"
  local enforced="${10}"
  local service_width="${11}"
  local quota_width="${12}"
  local worker_threads="${13}"

  PLAN_PATH="$plan_path" MODE="$mode" CASE_NAME="$case_name" CGROUP_V2="$cgroup_v2" \
    AVAILABLE="$available" SERVER_CPUS="$server_cpus" CLIENT_CPUS="$client_cpus" \
    LOAD_CPUS="$load_cpus" ENFORCED="$enforced" SERVICE_WIDTH="$service_width" \
    QUOTA_WIDTH="$quota_width" WORKER_THREADS="$worker_threads" \
    python3 - <<'PY'
import json
import os

def parse(value):
    return [int(item) for item in value.split(",") if item]

contract = {
    "mode": os.environ["MODE"],
    "case": os.environ["CASE_NAME"],
    "cgroup_v2": os.environ["CGROUP_V2"] == "1",
    "enforced": os.environ["ENFORCED"] == "1",
    "available_cpus": parse(os.environ["AVAILABLE"]),
    "server_cpus": parse(os.environ["SERVER_CPUS"]),
    "client_cpus": parse(os.environ["CLIENT_CPUS"]),
    "frp_server_cpus": parse(os.environ["SERVER_CPUS"]),
    "frp_client_cpus": parse(os.environ["CLIENT_CPUS"]),
    "load_cpus": parse(os.environ["LOAD_CPUS"]),
    "load_cpu_count": len(parse(os.environ["LOAD_CPUS"])),
    "service_width": int(os.environ["SERVICE_WIDTH"]),
    "quota_width": int(os.environ["QUOTA_WIDTH"] or 0) or None,
    "worker_threads_request": int(os.environ["WORKER_THREADS"] or 0),
    "quota_semantics": "isolate clears CPUQuota and uses AllowedCPUs; observe preserves CPUQuota",
    "comparable": os.environ["ENFORCED"] == "1" and os.environ["CGROUP_V2"] == "1",
}
with open(os.environ["PLAN_PATH"], "w", encoding="utf-8") as f:
    json.dump({"schema_version": 1, "cpu_contract": contract}, f, indent=2)
PY

  printf 'DUOTUNNEL_BENCH_PLAN=%s\nDUOTUNNEL_BENCH_LOAD_CPUS=%s\nDUOTUNNEL_BENCH_CPU_ENFORCED=%s\n' \
    "$plan_path" "$load_cpus" "$enforced" > "$env_path"
}

pin_pid_to_load() {
  local env_path="$1"
  local pid="$2"
  [ -s "$env_path" ] || return 0
  # shellcheck disable=SC1090
  source "$env_path"
  [ "${DUOTUNNEL_BENCH_CPU_ENFORCED:-0}" = "1" ] || return 0
  [ -n "${DUOTUNNEL_BENCH_LOAD_CPUS:-}" ] || return 0
  if kill -0 "$pid" 2>/dev/null; then
    taskset -pc "$DUOTUNNEL_BENCH_LOAD_CPUS" "$pid" >/dev/null
  fi
}

configure() {
  local case_name="${1:?case name required}"
  local server_log="${2:-/tmp/ci-server.log}"
  local client_log="${3:-/tmp/ci-client.log}"
  local mode="${DUOTUNNEL_BENCH_CPU_MODE:-isolate}"
  local requires_duotunnel=1
  if [[ "$case_name" == frp_* ]]; then
    requires_duotunnel=0
  fi
  local plan_path="/tmp/duotunnel-bench-plan-${case_name}.json"
  local env_path="/tmp/duotunnel-bench-plan-${case_name}.env"

  case "$mode" in
    isolate|observe) ;;
    *) die "DUOTUNNEL_BENCH_CPU_MODE must be isolate or observe: $mode" ;;
  esac

  local worker_threads="${WORKER_THREADS:-0}"
  cpu_contract_resolve "$mode" "$worker_threads" "${STRESS_CPU_QUOTA:-100%}" || die "unable to resolve CPU contract"
  local available_csv="$CPU_CONTRACT_AVAILABLE_CPUS"
  local server_cpus="$CPU_CONTRACT_SERVER_CPUS"
  local client_cpus="$CPU_CONTRACT_CLIENT_CPUS"
  local load_cpus="$CPU_CONTRACT_LOAD_CPUS"
  local enforced="$CPU_CONTRACT_ENFORCED"
  local service_width="$CPU_CONTRACT_SERVICE_WIDTH"
  local quota_width="$CPU_CONTRACT_QUOTA_WIDTH"

  if [ "$mode" = "isolate" ]; then
    command -v systemctl >/dev/null || die "isolate mode requires systemctl"
    if [ "$requires_duotunnel" = "1" ]; then
      systemctl is-active --quiet duotunnel-server.scope 2>/dev/null || die "duotunnel-server.scope is not active"
      systemctl is-active --quiet duotunnel-client.scope 2>/dev/null || die "duotunnel-client.scope is not active"
      sudo systemctl set-property --runtime duotunnel-server.scope "AllowedCPUs=${server_cpus}"
      sudo systemctl set-property --runtime duotunnel-client.scope "AllowedCPUs=${client_cpus}"
    fi
    for scope in frp-server.scope frp-client.scope; do
      if systemctl is-active --quiet "$scope" 2>/dev/null; then
        case "$scope" in
          frp-server.scope)
            sudo systemctl set-property --runtime "$scope" "AllowedCPUs=${server_cpus}"
            ;;
          frp-client.scope)
            sudo systemctl set-property --runtime "$scope" "AllowedCPUs=${client_cpus}"
            ;;
        esac
      fi
    done
    if [ "${DUOTUNNEL_BENCH_PRESERVE_CPU_QUOTA:-0}" != "1" ]; then
      for scope in duotunnel-server.scope duotunnel-client.scope frp-server.scope frp-client.scope; do
        if systemctl is-active --quiet "$scope" 2>/dev/null; then
          sudo systemctl set-property --runtime "$scope" CPUQuota=
        fi
      done
    fi
    enforced=1
  else
    server_cpus="$available_csv"
    client_cpus="$available_csv"
    load_cpus="$available_csv"
  fi

  write_plan "$plan_path" "$env_path" "$mode" "$case_name" \
    "$([ "$(stat -fc '%T' /sys/fs/cgroup 2>/dev/null || true)" = "cgroup2fs" ] && echo 1 || echo 0)" \
    "$available_csv" "$server_cpus" "$client_cpus" "$load_cpus" "$enforced" \
    "$service_width" "$quota_width" "$worker_threads"

  pid_files=(/tmp/http-echo.pid /tmp/ws-echo.pid /tmp/grpc-echo.pid)
  if [ "$requires_duotunnel" = "1" ]; then
    pid_files+=(/tmp/ctld.pid)
  fi
  for pid_file in "${pid_files[@]}"; do
    if [ -s "$pid_file" ]; then
      pin_pid_to_load "$env_path" "$(cat "$pid_file")"
    fi
  done

  scope_args=(--scope frp-server.scope --scope frp-client.scope)
  if [ "$requires_duotunnel" = "1" ]; then
    scope_args+=(--scope duotunnel-server.scope --scope duotunnel-client.scope)
  fi

  python3 "$TOOL" snapshot \
    --output "/tmp/benchmark-snapshot-${case_name}.json" \
    --case-name "$case_name" \
    --plan "$plan_path" \
    --config ci-helpers/configs/ctld.yaml \
    --config ci-helpers/configs/server.yaml \
    --config ci-helpers/configs/client.yaml \
    --config ci-helpers/configs/routing.yaml \
    "${scope_args[@]}" \
    --log-file "$server_log" \
    --log-file "$client_log" \
    --log-file /tmp/ci-ctld.log

  echo "Benchmark CPU contract: /tmp/benchmark-snapshot-${case_name}.json"
}

run_load() {
  local case_name="${1:?case name required}"
  shift
  local env_path="/tmp/duotunnel-bench-plan-${case_name}.env"
  if [ -s "$env_path" ]; then
    # shellcheck disable=SC1090
    source "$env_path"
  fi
  if [ "${DUOTUNNEL_BENCH_CPU_ENFORCED:-0}" = "1" ] && [ -n "${DUOTUNNEL_BENCH_LOAD_CPUS:-}" ]; then
    exec taskset -c "$DUOTUNNEL_BENCH_LOAD_CPUS" "$@"
  fi
  exec "$@"
}

case "${1:-}" in
  configure)
    shift
    configure "$@"
    ;;
  pin-pid)
    shift
    case_name="${1:?case name required}"
    pid="${2:?pid required}"
    pin_pid_to_load "/tmp/duotunnel-bench-plan-${case_name}.env" "$pid"
    ;;
  run-load)
    shift
    run_load "$@"
    ;;
  *)
    die "usage: $0 {configure CASE [SERVER_LOG CLIENT_LOG]|pin-pid CASE PID|run-load CASE COMMAND ...}"
    ;;
esac
