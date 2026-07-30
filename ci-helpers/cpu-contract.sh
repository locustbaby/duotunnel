#!/usr/bin/env bash

cpu_contract_die() {
  echo "benchmark CPU contract error: $*" >&2
  return 1
}

cpu_contract_expand_cpu_list() {
  local value="${1:-}"
  local part start end cpu
  IFS=',' read -ra parts <<< "$value"
  for part in "${parts[@]}"; do
    [ -n "$part" ] || continue
    if [[ "$part" == *-* ]]; then
      start="${part%%-*}"
      end="${part##*-}"
      [[ "$start" =~ ^[0-9]+$ && "$end" =~ ^[0-9]+$ && start -le end ]] || \
        cpu_contract_die "invalid CPU set: $value"
      for ((cpu = start; cpu <= end; cpu++)); do
        echo "$cpu"
      done
    else
      [[ "$part" =~ ^[0-9]+$ ]] || cpu_contract_die "invalid CPU set: $value"
      echo "$part"
    fi
  done
}

cpu_contract_join() {
  local value=""
  local cpu
  for cpu in "$@"; do
    if [ -n "$value" ]; then
      value="${value},${cpu}"
    else
      value="$cpu"
    fi
  done
  printf '%s' "$value"
}

cpu_contract_quota_width() {
  local quota="${1:-100%}"
  if [[ "$quota" == "none" || "$quota" == "unlimited" || "$quota" == "0" || "$quota" == "0%" ]]; then
    printf '%s' ""
    return 0
  fi
  [[ "$quota" =~ ^[0-9]+%$ ]] || cpu_contract_die "invalid STRESS_CPU_QUOTA: $quota"
  local percent="${quota%%%}"
  printf '%s' "$(( (percent + 99) / 100 ))"
}

cpu_contract_resolve() {
  local mode="${1:-${DUOTUNNEL_BENCH_CPU_MODE:-isolate}}"
  local worker_threads="${2:-${WORKER_THREADS:-0}}"
  local quota="${3:-${STRESS_CPU_QUOTA:-100%}}"
  local requested_set="${DUOTUNNEL_BENCH_CPUSET:-auto}"

  case "$mode" in
    isolate|observe) ;;
    *) cpu_contract_die "DUOTUNNEL_BENCH_CPU_MODE must be isolate or observe: $mode" ;;
  esac
  [[ "$worker_threads" =~ ^[0-9]+$ ]] || worker_threads=0

  local effective_set="$requested_set"
  if [ "$effective_set" = "auto" ]; then
    effective_set="$(cat /sys/fs/cgroup/cpuset.cpus.effective 2>/dev/null || true)"
  fi
  [ -n "$effective_set" ] || effective_set="0-$(( $(nproc) - 1 ))"

  local -a available_cpus
  mapfile -t available_cpus < <(cpu_contract_expand_cpu_list "$effective_set")
  [ "${#available_cpus[@]}" -gt 0 ] || cpu_contract_die "no effective CPUs found"

  local quota_width
  quota_width="$(cpu_contract_quota_width "$quota")"
  local load_width="${DUOTUNNEL_BENCH_LOAD_WIDTH:-3}"
  [[ "$load_width" =~ ^[1-9][0-9]*$ ]] || cpu_contract_die "DUOTUNNEL_BENCH_LOAD_WIDTH must be a positive integer: $load_width"
  local service_width
  if [ -n "$quota_width" ]; then
    service_width="$quota_width"
    [ "$worker_threads" -gt 0 ] && [ "$worker_threads" -lt "$service_width" ] && service_width="$worker_threads"
  elif [ "$worker_threads" -gt 0 ]; then
    service_width="$worker_threads"
  elif [ "$mode" = "observe" ]; then
    service_width="${#available_cpus[@]}"
  else
    service_width=1
  fi
  [ "$service_width" -gt 0 ] || service_width=1

  local available_csv
  available_csv="$(cpu_contract_join "${available_cpus[@]}")"
  CPU_CONTRACT_MODE="$mode"
  CPU_CONTRACT_AVAILABLE_CPUS="$available_csv"
  CPU_CONTRACT_QUOTA_WIDTH="${quota_width:-}"
  CPU_CONTRACT_SERVICE_WIDTH="$service_width"
  CPU_CONTRACT_SERVER_CPUS="$available_csv"
  CPU_CONTRACT_CLIENT_CPUS="$available_csv"
  CPU_CONTRACT_LOAD_CPUS="$available_csv"
  CPU_CONTRACT_ENFORCED=0

  if [ "$mode" = "isolate" ]; then
    [ "$(stat -fc '%T' /sys/fs/cgroup 2>/dev/null || true)" = "cgroup2fs" ] || \
      cpu_contract_die "isolate mode requires a cgroup v2 filesystem"
    [ "${#available_cpus[@]}" -ge "$((service_width + load_width))" ] || \
      cpu_contract_die "isolate mode needs at least $((service_width + load_width)) effective CPUs; found ${#available_cpus[@]}"
    CPU_CONTRACT_SERVER_CPUS="$(cpu_contract_join "${available_cpus[@]:0:service_width}")"
    CPU_CONTRACT_CLIENT_CPUS="$CPU_CONTRACT_SERVER_CPUS"
    CPU_CONTRACT_LOAD_CPUS="$(cpu_contract_join "${available_cpus[@]:service_width:load_width}")"
    CPU_CONTRACT_ENFORCED=1
  fi
}

cpu_contract_scope_args() {
  local role="$1"
  local mode="${CPU_CONTRACT_MODE:-${DUOTUNNEL_BENCH_CPU_MODE:-isolate}}"
  if [ "$mode" = "isolate" ]; then
    case "$role" in
      server|frp-server) printf '%s\n' "-p" "AllowedCPUs=${CPU_CONTRACT_SERVER_CPUS}" ;;
      client|frp-client) printf '%s\n' "-p" "AllowedCPUs=${CPU_CONTRACT_CLIENT_CPUS}" ;;
      load) printf '%s\n' "-p" "AllowedCPUs=${CPU_CONTRACT_LOAD_CPUS}" ;;
      *) cpu_contract_die "unknown CPU contract role: $role" ;;
    esac
  else
    local quota="${STRESS_CPU_QUOTA:-100%}"
    case "$quota" in
      none|unlimited|0|0%) return 0 ;;
      *) printf '%s\n' "-p" "CPUQuota=${quota}" ;;
    esac
  fi
}
