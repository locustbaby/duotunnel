#!/usr/bin/env python3
"""
Background metrics collector — replaces mpstat / pidstat / sar.

Usage:
    python3 collect-metrics.py --interval 2 --output /tmp/metrics-collected.json

Runs until SIGTERM/SIGINT, then writes collected timeseries to --output.

Process tracking strategy:
  - PINNED processes (by name prefix) are always tracked individually
  - Remaining processes are ranked by CPU%; top (TOP_N - len(pinned)) are tracked
    individually; the rest are summed into the "other" bucket
  - This guarantees server/client/k6 are never dropped regardless of load

Output schema:
  {
    "nproc": int,
    "system": {
      "cpu_pct": [{t, v}],   cpu_usr, cpu_sys, cpu_iowait, cpu_steal,
      "mem_used_mb": [...],   mem_avail_mb,
      "net_rx_kbs": [...],    net_tx_kbs,
      "page_faults_s": [...],
      "tcp_estab": [...],     tcp_timewait
    },
    "psi": { "cpu": [{t,v}], "mem": [...], "io": [...] },
    "processes": {
      "<name>": {
        "cpu_pct": [{t,v}],  rss_mb, fds, threads,
        "io_read_kbs": [...], io_write_kbs,
        "ctx_vol": [...],     ctx_invol
      }
    }
  }
"""

import argparse
import json
import os
import signal
import sys
import time

try:
    import psutil
except ImportError:
    print("psutil not found — install with: pip install psutil", file=sys.stderr)
    sys.exit(1)

# Always tracked individually, regardless of CPU rank
PINNED = {"server", "client", "tunnel-ctld", "k6", "http-echo-server", "ws-echo-server", "grpc-echo-server", "frps", "frpc", "ci-test-client"}

# Match process name to a stable display key
def _pinned_key(name: str) -> str | None:
    for p in PINNED:
        if name == p or name.startswith(p):
            return p.replace("-", "_")
    return None

TOP_N = 20  # max individually tracked processes (pinned + top CPU)


def _read_psi(resource: str) -> float:
    try:
        with open(f"/proc/pressure/{resource}") as f:
            for line in f:
                if line.startswith("full"):
                    for tok in line.split():
                        if tok.startswith("avg10="):
                            return float(tok[6:])
    except Exception:
        pass
    return 0.0


def _net_bytes():
    rx = tx = 0
    try:
        for iface, c in psutil.net_io_counters(pernic=True).items():
            if iface != "lo":
                rx += c.bytes_recv
                tx += c.bytes_sent
    except Exception:
        pass
    return rx, tx


def _tcp_counts():
    estab = timewait = 0
    try:
        for c in psutil.net_connections(kind="tcp"):
            if c.status == "ESTABLISHED":
                estab += 1
            elif c.status == "TIME_WAIT":
                timewait += 1
    except Exception:
        pass
    return estab, timewait


def _page_faults():
    try:
        with open("/proc/vmstat") as f:
            for line in f:
                if line.startswith("pgfault "):
                    return int(line.split()[1])
    except Exception:
        pass
    return 0


def _pt(d, key, t, v):
    d.setdefault(key, []).append({"t": round(t, 1), "v": round(v, 2)})


def collect(interval: float, output: str):
    running = [True]

    def _stop(sig, frame):
        running[0] = False

    signal.signal(signal.SIGTERM, _stop)
    signal.signal(signal.SIGINT, _stop)

    cpu_count = psutil.cpu_count(logical=True) or 1
    result = {
        "nproc":     cpu_count,
        "system":    {},
        "psi":       {"cpu": [], "mem": [], "io": []},
        "processes": {},
    }
    sys_data  = result["system"]
    psi_data  = result["psi"]
    proc_data = result["processes"]

    # delta state
    prev_net_rx, prev_net_tx = _net_bytes()
    prev_pgf                  = _page_faults()
    prev_io:  dict[int, tuple[int, int]] = {}
    prev_ctx: dict[int, tuple[int, int]] = {}

    # prime cpu_percent (first call returns 0)
    psutil.cpu_percent(interval=None)
    for p in psutil.process_iter():
        try:
            p.cpu_percent(interval=None)
        except Exception:
            pass

    t0 = time.monotonic()
    time.sleep(interval)

    while running[0]:
        t = round(time.monotonic() - t0, 1)

        # ── system CPU ────────────────────────────────────────────────────────
        ct = psutil.cpu_times_percent(interval=None)
        _pt(sys_data, "cpu_pct",    t, 100.0 - ct.idle)
        _pt(sys_data, "cpu_usr",    t, ct.user)
        _pt(sys_data, "cpu_sys",    t, ct.system)
        _pt(sys_data, "cpu_iowait", t, getattr(ct, "iowait", 0.0))
        _pt(sys_data, "cpu_steal",  t, getattr(ct, "steal",  0.0))

        # ── memory ────────────────────────────────────────────────────────────
        mem = psutil.virtual_memory()
        _pt(sys_data, "mem_used_mb",  t, mem.used      / 1048576)
        _pt(sys_data, "mem_avail_mb", t, mem.available / 1048576)

        # ── network ───────────────────────────────────────────────────────────
        rx, tx = _net_bytes()
        _pt(sys_data, "net_rx_kbs", t, max(0.0, (rx - prev_net_rx) / 1024 / interval))
        _pt(sys_data, "net_tx_kbs", t, max(0.0, (tx - prev_net_tx) / 1024 / interval))
        prev_net_rx, prev_net_tx = rx, tx

        # ── page faults ───────────────────────────────────────────────────────
        pgf = _page_faults()
        _pt(sys_data, "page_faults_s", t, max(0.0, (pgf - prev_pgf) / interval))
        prev_pgf = pgf

        # ── TCP ───────────────────────────────────────────────────────────────
        estab, tw = _tcp_counts()
        _pt(sys_data, "tcp_estab",    t, estab)
        _pt(sys_data, "tcp_timewait", t, tw)

        # ── PSI ───────────────────────────────────────────────────────────────
        psi_data["cpu"].append({"t": t, "v": _read_psi("cpu")})
        psi_data["mem"].append({"t": t, "v": _read_psi("memory")})
        psi_data["io"].append( {"t": t, "v": _read_psi("io")})

        # ── per-process: snapshot all, then rank ──────────────────────────────
        snapshot: list[dict] = []
        for proc in psutil.process_iter(
            ["pid", "name", "cpu_percent", "memory_info",
             "num_fds", "num_threads"]
        ):
            try:
                pid  = proc.pid
                name = proc.info["name"] or ""
                cpu  = proc.cpu_percent(interval=None) / cpu_count
                rss  = (proc.info["memory_info"].rss if proc.info["memory_info"] else 0) / 1048576
                fds  = proc.info.get("num_fds") or 0
                thr  = proc.info.get("num_threads") or 0

                io_r = io_w = 0.0
                try:
                    ioc = proc.io_counters()
                    if pid in prev_io:
                        pr, pw = prev_io[pid]
                        io_r = max(0.0, (ioc.read_bytes  - pr) / 1024 / interval)
                        io_w = max(0.0, (ioc.write_bytes - pw) / 1024 / interval)
                    prev_io[pid] = (ioc.read_bytes, ioc.write_bytes)
                except (psutil.AccessDenied, AttributeError):
                    pass

                cv = iv = 0.0
                try:
                    ctx = proc.num_ctx_switches()
                    if pid in prev_ctx:
                        pv, pi = prev_ctx[pid]
                        cv = max(0.0, (ctx.voluntary   - pv) / interval)
                        iv = max(0.0, (ctx.involuntary - pi) / interval)
                    prev_ctx[pid] = (ctx.voluntary, ctx.involuntary)
                except (psutil.AccessDenied, AttributeError):
                    pass

                snapshot.append({
                    "pid": pid, "name": name, "cpu": cpu, "rss": rss,
                    "fds": fds, "thr": thr,
                    "io_r": io_r, "io_w": io_w,
                    "cv": cv, "iv": iv,
                })
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue

        # separate pinned vs non-pinned
        pinned_procs = [p for p in snapshot if _pinned_key(p["name"])]
        other_procs  = [p for p in snapshot if not _pinned_key(p["name"])]

        # from non-pinned, pick top (TOP_N - pinned_count) by CPU
        slots_left = max(0, TOP_N - len(pinned_procs))
        other_procs.sort(key=lambda p: p["cpu"], reverse=True)
        tracked_others = other_procs[:slots_left]
        rest           = other_procs[slots_left:]

        # accumulate into proc_data
        def _record(key: str, p: dict):
            d = proc_data.setdefault(key, {})
            if p["cpu"]  > 0: _pt(d, "cpu_pct",     t, p["cpu"])
            if p["rss"]  > 0: _pt(d, "rss_mb",      t, p["rss"])
            if p["fds"]  > 0: _pt(d, "fds",         t, p["fds"])
            if p["thr"]  > 0: _pt(d, "threads",     t, p["thr"])
            if p["io_r"] > 0: _pt(d, "io_read_kbs", t, p["io_r"])
            if p["io_w"] > 0: _pt(d, "io_write_kbs",t, p["io_w"])
            if p["cv"]   > 0: _pt(d, "ctx_vol",     t, p["cv"])
            if p["iv"]   > 0: _pt(d, "ctx_invol",   t, p["iv"])

        for p in pinned_procs:
            _record(_pinned_key(p["name"]), p)

        for p in tracked_others:
            _record(p["name"], p)

        # sum the rest into "other"
        if rest:
            other_agg = {
                "pid": -1, "name": "other",
                "cpu":  sum(p["cpu"]  for p in rest),
                "rss":  sum(p["rss"]  for p in rest),
                "fds":  sum(p["fds"]  for p in rest),
                "thr":  sum(p["thr"]  for p in rest),
                "io_r": sum(p["io_r"] for p in rest),
                "io_w": sum(p["io_w"] for p in rest),
                "cv":   sum(p["cv"]   for p in rest),
                "iv":   sum(p["iv"]   for p in rest),
            }
            _record("other", other_agg)

        time.sleep(interval)

    # ── write output atomically ───────────────────────────────────────────────
    tmp = output + ".tmp"
    with open(tmp, "w") as f:
        json.dump(result, f)
    os.replace(tmp, output)
    n = len(sys_data.get("cpu_pct", []))
    print(f"collect-metrics: {n} samples → {output}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--interval", type=float, default=2.0)
    ap.add_argument("--top",      type=int,   default=20,
                    help="max individually tracked non-pinned processes")
    ap.add_argument("--output",   required=True)
    args = ap.parse_args()
    global TOP_N
    TOP_N = args.top
    collect(args.interval, args.output)


if __name__ == "__main__":
    main()
