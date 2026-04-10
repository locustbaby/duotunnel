#!/usr/bin/env python3
"""Unified resource collector — replaces mpstat + pidstat-ur + pidstat-ctxsw + pidstat-io.

Outputs one JSON line per interval to stdout.

sys fields:
  cpu            — total CPU % (all cores averaged)
  cpu_per_core   — list of per-core CPU %
  cpu_usr/sys/irq/soft/iowait/steal — breakdown
  loadavg_1/5/15 — load average
  ctx_switches   — system-wide context switches/s (delta)
  interrupts     — hardware interrupts/s (delta)
  mem_pct        — RAM used %
  mem_used_mb    — RAM used MB
  swap_used_mb   — swap used MB (> 0 = bad)
  disk_read_kbs  — system disk read KB/s (all disks, delta)
  disk_write_kbs — system disk write KB/s (all disks, delta)
  disk_read_iops — disk read operations/s
  disk_write_iops— disk write operations/s
  net_rx_kbs     — network recv KB/s (sum all NICs)
  net_tx_kbs     — network sent KB/s (sum all NICs)
  net_rx_pkts    — recv packets/s
  net_tx_pkts    — sent packets/s
  net_drop_in    — recv drop/s
  net_drop_out   — send drop/s
  net_err_in     — recv errors/s
  net_err_out    — send errors/s
  tcp_estab      — TCP connections in ESTABLISHED state
  tcp_timewait   — TCP connections in TIME_WAIT state
  udp_rx_err     — UDP receive errors/s (Linux /proc/net/snmp only)
  udp_buf_err    — UDP buffer overflow errors/s (Linux only)

procs fields per group:
  cpu       — % of all CPUs (sum across matching PIDs)
  rss       — MB (max across matching PIDs)
  vms       — MB virtual memory (max across matching PIDs)
  read_kbs  — KB/s (sum)
  write_kbs — KB/s (sum)
  cswch     — voluntary context switches/s (sum)
  nvcswch   — involuntary context switches/s (sum)
  fds       — open file descriptors (sum)

top_cpu / top_rss: list of top-10 PIDs by CPU / RSS

Usage:
  python3 collect-resources.py [interval_seconds] > /tmp/collect.jsonl
  Default interval: 1.0
"""

import json
import os
import sys
import time
import platform

try:
    import psutil
except ImportError:
    os.execvp("sh", ["sh", "-c",
        "pip install -q psutil && exec python3 " + " ".join(sys.argv)])

PROC_GROUPS = [
    ("server",           "server"),
    ("client",           "client"),
    ("http-echo-server", "http_echo"),
    ("http-echo-serve",  "http_echo"),
    ("ws-echo-server",   "ws_echo"),
    ("ws-echo-serve",    "ws_echo"),
    ("grpc-echo-server", "grpc_echo"),
    ("grpc-echo-serve",  "grpc_echo"),
    ("k6",               "k6"),
    ("frps",             "frps"),
    ("frpc",             "frpc"),
]
ALL_GROUPS = ["server", "client", "http_echo", "ws_echo", "grpc_echo",
              "k6", "frps", "frpc", "other"]

# NICs to exclude from net I/O (loopback + virtual)
_NET_EXCLUDE = {"lo", "docker0", "virbr0"}

_IS_LINUX = platform.system() == "Linux"


def _read_udp_snmp():
    """Read UDP error counters from /proc/net/snmp (Linux only).
    Returns (InErrors, RcvbufErrors) as absolute counters.
    """
    try:
        with open("/proc/net/snmp") as f:
            content = f.read()
        lines = content.splitlines()
        for i, line in enumerate(lines):
            if line.startswith("Udp:") and i + 1 < len(lines):
                keys = line.split()
                vals = lines[i + 1].split()
                kv = dict(zip(keys[1:], vals[1:]))
                return int(kv.get("InErrors", 0)), int(kv.get("RcvbufErrors", 0))
    except Exception:
        pass
    return 0, 0


def _group(name):
    for prefix, g in PROC_GROUPS:
        if name == prefix or name.startswith(prefix):
            return g
    return "other"


def main():
    interval = float(sys.argv[1]) if len(sys.argv) > 1 else 1.0
    nproc = psutil.cpu_count(logical=True) or 1

    # Prime CPU percent counters (first call always returns 0.0 — discard)
    psutil.cpu_times_percent(interval=None)
    psutil.cpu_percent(percpu=True)
    for p in psutil.process_iter():
        try:
            p.cpu_percent(interval=None)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass

    # Prime network / ctx-switch / disk deltas
    prev_net  = psutil.net_io_counters(pernic=True)
    _cpu_st   = psutil.cpu_stats()
    prev_cs   = _cpu_st.ctx_switches
    prev_intr = _cpu_st.interrupts
    prev_net_t = time.monotonic()
    try:
        _disk0        = psutil.disk_io_counters()
        prev_disk_rb  = _disk0.read_bytes
        prev_disk_wb  = _disk0.write_bytes
        prev_disk_rc  = _disk0.read_count
        prev_disk_wc  = _disk0.write_count
    except Exception:
        prev_disk_rb = prev_disk_wb = prev_disk_rc = prev_disk_wc = None
    if _IS_LINUX:
        prev_udp_inerr, prev_udp_buferr = _read_udp_snmp()
    else:
        prev_udp_inerr = prev_udp_buferr = None

    prev_io:  dict = {}   # pid -> (read_bytes, write_bytes, monotonic_ts)
    prev_ctx: dict = {}   # pid -> (voluntary, involuntary, monotonic_ts)

    start_ts = time.monotonic()
    time.sleep(interval)

    while True:
        t0 = time.monotonic()
        now_t = round(t0 - start_ts, 2)

        # ── system-wide CPU ───────────────────────────────────────────────
        ct       = psutil.cpu_times_percent(interval=None)
        per_core = psutil.cpu_percent(percpu=True)
        load1, load5, load15 = psutil.getloadavg()

        # system ctx switches + interrupts delta /s
        _cpu_st  = psutil.cpu_stats()
        cur_cs   = _cpu_st.ctx_switches
        cur_intr = _cpu_st.interrupts
        dt_net   = t0 - prev_net_t
        cs_per_s   = round((cur_cs   - prev_cs)   / dt_net, 0) if dt_net > 0 else 0
        intr_per_s = round((cur_intr - prev_intr) / dt_net, 0) if dt_net > 0 else 0
        prev_cs   = cur_cs
        prev_intr = cur_intr

        vm   = psutil.virtual_memory()
        swap = psutil.swap_memory()

        # disk I/O delta
        disk_read_kbs = disk_write_kbs = disk_read_iops = disk_write_iops = None
        if prev_disk_rb is not None and dt_net > 0:
            try:
                _disk = psutil.disk_io_counters()
                disk_read_kbs   = round((_disk.read_bytes  - prev_disk_rb) / dt_net / 1024, 1)
                disk_write_kbs  = round((_disk.write_bytes - prev_disk_wb) / dt_net / 1024, 1)
                disk_read_iops  = round((_disk.read_count  - prev_disk_rc) / dt_net, 1)
                disk_write_iops = round((_disk.write_count - prev_disk_wc) / dt_net, 1)
                prev_disk_rb = _disk.read_bytes
                prev_disk_wb = _disk.write_bytes
                prev_disk_rc = _disk.read_count
                prev_disk_wc = _disk.write_count
            except Exception:
                pass

        # UDP error deltas (Linux only)
        udp_rx_err = udp_buf_err = None
        if _IS_LINUX and prev_udp_inerr is not None and dt_net > 0:
            cur_ue, cur_be = _read_udp_snmp()
            udp_rx_err  = round((cur_ue - prev_udp_inerr)  / dt_net, 2)
            udp_buf_err = round((cur_be - prev_udp_buferr) / dt_net, 2)
            prev_udp_inerr  = cur_ue
            prev_udp_buferr = cur_be

        # TCP connection counts (snapshot)
        tcp_estab = tcp_timewait = None
        try:
            conns = psutil.net_connections(kind="tcp")
            tcp_estab    = sum(1 for c in conns if c.status == "ESTABLISHED")
            tcp_timewait = sum(1 for c in conns if c.status == "TIME_WAIT")
        except Exception:
            pass

        # network I/O delta
        cur_net = psutil.net_io_counters(pernic=True)
        rx_bytes = tx_bytes = rx_pkts = tx_pkts = drop_in = drop_out = err_in = err_out = 0
        for nic, cnt in cur_net.items():
            if nic in _NET_EXCLUDE or nic.startswith("veth"):
                continue
            prev = prev_net.get(nic)
            if prev and dt_net > 0:
                rx_bytes  += cnt.bytes_recv    - prev.bytes_recv
                tx_bytes  += cnt.bytes_sent    - prev.bytes_sent
                rx_pkts   += cnt.packets_recv  - prev.packets_recv
                tx_pkts   += cnt.packets_sent  - prev.packets_sent
                drop_in   += cnt.dropin        - prev.dropin
                drop_out  += cnt.dropout       - prev.dropout
                err_in    += cnt.errin         - prev.errin
                err_out   += cnt.errout        - prev.errout
        prev_net   = cur_net
        prev_net_t = t0

        sys_out = {
            "cpu":          round(100.0 - ct.idle, 1),
            "cpu_per_core": [round(v, 1) for v in per_core],
            "cpu_usr":      round(ct.user,                    1),
            "cpu_sys":      round(ct.system,                  1),
            "cpu_irq":      round(getattr(ct, "irq",     0.0), 1),
            "cpu_soft":     round(getattr(ct, "softirq", 0.0), 1),
            "cpu_iowait":   round(getattr(ct, "iowait",  0.0), 1),
            "cpu_steal":    round(getattr(ct, "steal",   0.0), 1),
            "loadavg_1":    round(load1,  2),
            "loadavg_5":    round(load5,  2),
            "loadavg_15":   round(load15, 2),
            "ctx_switches": int(cs_per_s),
            "interrupts":   int(intr_per_s),
            "mem_pct":      round(vm.percent,       1),
            "mem_used_mb":  round(vm.used / 1048576, 0),
            "swap_used_mb": round(swap.used / 1048576, 0),
        }
        if disk_read_kbs is not None:
            sys_out.update({
                "disk_read_kbs":   disk_read_kbs,
                "disk_write_kbs":  disk_write_kbs,
                "disk_read_iops":  disk_read_iops,
                "disk_write_iops": disk_write_iops,
            })
        if tcp_estab is not None:
            sys_out["tcp_estab"]    = tcp_estab
            sys_out["tcp_timewait"] = tcp_timewait
        if udp_rx_err is not None:
            sys_out["udp_rx_err"]  = udp_rx_err
            sys_out["udp_buf_err"] = udp_buf_err
        if dt_net > 0:
            sys_out.update({
                "net_rx_kbs":  round(rx_bytes  / dt_net / 1024, 1),
                "net_tx_kbs":  round(tx_bytes  / dt_net / 1024, 1),
                "net_rx_pkts": round(rx_pkts   / dt_net, 0),
                "net_tx_pkts": round(tx_pkts   / dt_net, 0),
                "net_drop_in": round(drop_in   / dt_net, 2),
                "net_drop_out":round(drop_out  / dt_net, 2),
                "net_err_in":  round(err_in    / dt_net, 2),
                "net_err_out": round(err_out   / dt_net, 2),
            })

        # ── per-process ───────────────────────────────────────────────────
        acc_cpu  = {g: 0.0 for g in ALL_GROUPS}
        acc_rss  = {g: 0.0 for g in ALL_GROUPS}
        acc_vms  = {g: 0.0 for g in ALL_GROUPS}
        acc_rkbs = {g: 0.0 for g in ALL_GROUPS}
        acc_wkbs = {g: 0.0 for g in ALL_GROUPS}
        acc_cs   = {g: 0.0 for g in ALL_GROUPS}
        acc_nvcs = {g: 0.0 for g in ALL_GROUPS}
        acc_fds  = {g: 0   for g in ALL_GROUPS}

        new_io:  dict = {}
        new_ctx: dict = {}

        pid_cpu: list = []
        pid_rss: list = []

        for p in psutil.process_iter(
            ["pid", "name", "cpu_percent", "memory_info",
             "io_counters", "num_ctx_switches", "num_fds"]
        ):
            try:
                info = p.info
                pid  = info["pid"]
                name = info["name"] or ""
                g    = _group(name)

                cpu = (info["cpu_percent"] or 0.0) / nproc
                acc_cpu[g] = round(acc_cpu[g] + cpu, 2)
                if cpu > 0:
                    pid_cpu.append((round(cpu, 2), pid, name))

                mem = info["memory_info"]
                if mem:
                    rss_mb = mem.rss / 1048576
                    vms_mb = mem.vms / 1048576
                    if rss_mb > acc_rss[g]:
                        acc_rss[g] = rss_mb
                    if vms_mb > acc_vms[g]:
                        acc_vms[g] = vms_mb
                    if rss_mb > 0:
                        pid_rss.append((round(rss_mb, 1), pid, name))

                ioc = info["io_counters"]
                if ioc is not None:
                    new_io[pid] = (ioc.read_bytes, ioc.write_bytes, t0)
                    if pid in prev_io:
                        pr, pw, pt = prev_io[pid]
                        dt = t0 - pt
                        if dt > 0:
                            acc_rkbs[g] = round(acc_rkbs[g] + (ioc.read_bytes  - pr) / dt / 1024, 1)
                            acc_wkbs[g] = round(acc_wkbs[g] + (ioc.write_bytes - pw) / dt / 1024, 1)

                ctx = info["num_ctx_switches"]
                if ctx is not None:
                    new_ctx[pid] = (ctx.voluntary, ctx.involuntary, t0)
                    if pid in prev_ctx:
                        pv, pi, pt = prev_ctx[pid]
                        dt = t0 - pt
                        if dt > 0:
                            acc_cs[g]   = round(acc_cs[g]   + (ctx.voluntary   - pv) / dt, 1)
                            acc_nvcs[g] = round(acc_nvcs[g] + (ctx.involuntary - pi) / dt, 1)

                fds = info.get("num_fds") or 0
                acc_fds[g] += fds

            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                continue

        prev_io  = new_io
        prev_ctx = new_ctx

        top_cpu = [{"pid": pid, "name": name, "cpu": cpu}
                   for cpu, pid, name in sorted(pid_cpu, reverse=True)[:10]]
        top_rss = [{"pid": pid, "name": name, "rss": rss}
                   for rss, pid, name in sorted(pid_rss, reverse=True)[:10]]

        procs_out = {}
        for g in ALL_GROUPS:
            entry = {}
            if acc_cpu[g]:   entry["cpu"]       = round(acc_cpu[g],  2)
            if acc_rss[g]:   entry["rss"]        = round(acc_rss[g],  1)
            if acc_vms[g]:   entry["vms"]        = round(acc_vms[g],  1)
            if acc_rkbs[g]:  entry["read_kbs"]   = round(acc_rkbs[g], 1)
            if acc_wkbs[g]:  entry["write_kbs"]  = round(acc_wkbs[g], 1)
            if acc_cs[g]:    entry["cswch"]      = round(acc_cs[g],   1)
            if acc_nvcs[g]:  entry["nvcswch"]    = round(acc_nvcs[g], 1)
            if acc_fds[g]:   entry["fds"]        = acc_fds[g]
            if entry:
                procs_out[g] = entry

        out = {"t": now_t, "sys": sys_out, "procs": procs_out}
        if top_cpu:
            out["top_cpu"] = top_cpu
        if top_rss:
            out["top_rss"] = top_rss
        print(json.dumps(out, separators=(",", ":")), flush=True)

        elapsed = time.monotonic() - t0
        rem = interval - elapsed
        if rem > 0:
            time.sleep(rem)


if __name__ == "__main__":
    main()
