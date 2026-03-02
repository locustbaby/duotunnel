#!/usr/bin/env bash
# tune-os.sh — Linux kernel & NIC tuning for high-throughput QUIC/TCP proxy
#
# Run as root on the server host BEFORE starting the tunnel server.
# Safe to re-run; all settings are applied with sysctl -w (runtime only).
# To make permanent, copy the sysctl block into /etc/sysctl.d/99-tunnel.conf.
#
# Usage:
#   sudo bash scripts/tune-os.sh [NIC]
#   sudo bash scripts/tune-os.sh eth0
#
# If NIC is omitted the script auto-detects the default-route interface.

set -euo pipefail

NIC="${1:-$(ip route show default 2>/dev/null | awk '/default/{print $5; exit}')}"

if [[ -z "$NIC" ]]; then
    echo "ERROR: could not determine network interface. Pass it explicitly: $0 eth0" >&2
    exit 1
fi

if [[ $EUID -ne 0 ]]; then
    echo "ERROR: this script must be run as root (sudo)." >&2
    exit 1
fi

echo "==> Tuning kernel for tunnel proxy (NIC: $NIC)"

# ── Socket buffers ────────────────────────────────────────────────────────────
# 16 MB max — matches TcpParams default of 4 MB (kernel doubles the set value,
# so effective window is 8 MB; the max allows room above that).
sysctl -w net.core.rmem_max=16777216
sysctl -w net.core.wmem_max=16777216
sysctl -w net.core.rmem_default=4194304
sysctl -w net.core.wmem_default=4194304
sysctl -w net.ipv4.tcp_rmem="4096 4194304 16777216"
sysctl -w net.ipv4.tcp_wmem="4096 4194304 16777216"

# UDP (QUIC transport layer)
sysctl -w net.core.netdev_max_backlog=65536
sysctl -w net.ipv4.udp_rmem_min=65536
sysctl -w net.ipv4.udp_wmem_min=65536

# ── Connection queue ──────────────────────────────────────────────────────────
sysctl -w net.core.somaxconn=65535
sysctl -w net.ipv4.tcp_max_syn_backlog=65535
sysctl -w net.ipv4.ip_local_port_range="1024 65535"

# ── TIME_WAIT / FIN handling ──────────────────────────────────────────────────
# Proxy makes many outbound connections; aggressively reuse TIME_WAIT sockets.
sysctl -w net.ipv4.tcp_tw_reuse=1
sysctl -w net.ipv4.tcp_fin_timeout=15

# ── Busy-poll (low-latency mode) ─────────────────────────────────────────────
# Poll for 50 µs before sleeping — cuts median latency at the cost of ~1 CPU%.
sysctl -w net.core.busy_poll=50
sysctl -w net.core.busy_read=50

# ── TCP Fast Open ─────────────────────────────────────────────────────────────
sysctl -w net.ipv4.tcp_fastopen=3

# ── Transparent Huge Pages ───────────────────────────────────────────────────
# Disable THP compaction to reduce latency spikes.
if [[ -f /sys/kernel/mm/transparent_hugepage/enabled ]]; then
    echo madvise > /sys/kernel/mm/transparent_hugepage/enabled
fi
if [[ -f /sys/kernel/mm/transparent_hugepage/defrag ]]; then
    echo defer+madvise > /sys/kernel/mm/transparent_hugepage/defrag
fi

# ── NUMA balancing ────────────────────────────────────────────────────────────
# Short-lived proxy connections thrash NUMA auto-balance; disable it.
sysctl -w kernel.numa_balancing=0 2>/dev/null || true

# ── CPU frequency scaling ────────────────────────────────────────────────────
# Force performance governor to prevent frequency ramp-up latency (~50–200 µs).
if command -v cpupower &>/dev/null; then
    cpupower frequency-set -g performance 2>/dev/null || true
else
    for gov in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        [[ -f "$gov" ]] && echo performance > "$gov" || true
    done
fi

# ── NIC ring buffers ──────────────────────────────────────────────────────────
if command -v ethtool &>/dev/null; then
    MAX_RX=$(ethtool -g "$NIC" 2>/dev/null | awk '/^RX:/{print $2; exit}')
    MAX_TX=$(ethtool -g "$NIC" 2>/dev/null | awk '/^TX:/{print $2; exit}')
    if [[ -n "$MAX_RX" && -n "$MAX_TX" ]]; then
        ethtool -G "$NIC" rx "$MAX_RX" tx "$MAX_TX" 2>/dev/null || true
        echo "    NIC ring buffers: rx=$MAX_RX tx=$MAX_TX"
    fi
fi

# ── RPS / RFS (software multi-queue) ────────────────────────────────────────
# Spread packet processing across all CPUs.
CPU_COUNT=$(nproc)
CPU_MASK=$(printf '%x' $(( (1 << CPU_COUNT) - 1 )))
for rps_file in /sys/class/net/"$NIC"/queues/rx-*/rps_cpus; do
    [[ -f "$rps_file" ]] && echo "$CPU_MASK" > "$rps_file" || true
done

RPS_FLOW_ENTRIES=32768
sysctl -w net.core.rps_sock_flow_entries=$RPS_FLOW_ENTRIES
for rfs_file in /sys/class/net/"$NIC"/queues/rx-*/rps_flow_cnt; do
    [[ -f "$rfs_file" ]] && echo 4096 > "$rfs_file" || true
done

# ── IRQ affinity (multi-queue NICs) ──────────────────────────────────────────
# Bind hardware queue interrupts to individual physical cores (not hyperthreads)
# so NIC interrupts and Tokio worker threads land on the same NUMA node.
HW_QUEUES=$(ls -d /sys/class/net/"$NIC"/queues/rx-* 2>/dev/null | wc -l)
if [[ $HW_QUEUES -gt 1 ]]; then
    echo "    Binding $HW_QUEUES NIC IRQs to cores 0–$((HW_QUEUES-1))"
    IRQ_LIST=$(grep -l "$NIC" /proc/irq/*/node 2>/dev/null | awk -F/ '{print $4}' | head -"$HW_QUEUES" || true)
    CORE=0
    for IRQ in $IRQ_LIST; do
        echo "$CORE" > /proc/irq/"$IRQ"/smp_affinity_list 2>/dev/null || true
        CORE=$(( (CORE + 1) % CPU_COUNT ))
    done
fi

echo "==> Done. To persist sysctl settings across reboots:"
echo "    sudo bash scripts/tune-os.sh && sysctl --system"
echo "    (or copy sysctl lines to /etc/sysctl.d/99-tunnel.conf)"
