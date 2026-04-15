#!/usr/bin/env python3
import argparse
import json
import os
import sys
import time
import platform
import re
from datetime import datetime

# --- Unified Configuration (Loaded from schema.json) ---

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
SCHEMA_PATH = os.path.join(BASE_DIR, "schema.json")
try:
    with open(SCHEMA_PATH, "r") as f:
        SCHEMA = json.load(f)
except Exception as e:
    print(f"Error loading schema.json: {e}", file=sys.stderr)
    SCHEMA = {"categories": [], "metrics": [], "groups": [], "process_mapping": []}

PROC_GROUPS = [(m["prefix"], m["group"]) for m in SCHEMA.get("process_mapping", [])]
ALL_GROUPS = [g["id"] for g in SCHEMA.get("groups", [])] + ["other"]

def get_group(name):
    for prefix, g in PROC_GROUPS:
        if name == prefix or name.startswith(prefix):
            return g
    return "other"

# --- Subcommand: Collect (Resource Monitoring) ---

def run_collect(args):
    try:
        import psutil
    except ImportError:
        print("psutil not found, attempting auto-install...", file=sys.stderr)
        os.execvp("sh", ["sh", "-c", "pip install -q psutil && exec python3 " + " ".join(sys.argv)])

    interval = args.interval
    nproc = psutil.cpu_count(logical=True) or 1
    _IS_LINUX = platform.system() == "Linux"
    _NET_EXCLUDE = {"lo", "docker0", "virbr0"}

    def _read_udp_snmp():
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
        except Exception: pass
        return 0, 0

    def _read_psi():
        res = {}
        for kind in ["cpu", "memory", "io"]:
            try:
                with open(f"/proc/pressure/{kind}") as f:
                    # avg10=0.00 avg60=0.00 avg300=0.00 total=0
                    line = f.readline()
                    m = re.search(r"avg10=([0-9.]+)", line)
                    if m: res[kind] = float(m.group(1))
            except Exception: pass
        return res

    def _read_tcp_summary():
        # Using psutil.net_connections() can be slow if thousands of sockets exist.
        # Kind="inet" is usually sufficient for our tunnel benchmarks.
        counts = {}
        try:
            for conn in psutil.net_connections(kind="inet"):
                s = conn.status
                counts[s] = counts.get(s, 0) + 1
        except Exception: pass
        return counts

    # Prime counters
    psutil.cpu_times_percent(interval=None)
    psutil.cpu_percent(percpu=True)
    for p in psutil.process_iter():
        try: p.cpu_percent(interval=None)
        except (psutil.NoSuchProcess, psutil.AccessDenied): pass

    prev_net = psutil.net_io_counters(pernic=True)
    _cpu_st = psutil.cpu_stats()
    prev_cs, prev_intr = _cpu_st.ctx_switches, _cpu_st.interrupts
    prev_net_t = time.monotonic()
    try:
        _disk0 = psutil.disk_io_counters()
        prev_disk_rb, prev_disk_wb = _disk0.read_bytes, _disk0.write_bytes
        prev_disk_rc, prev_disk_wc = _disk0.read_count, _disk0.write_count
    except Exception:
        prev_disk_rb = prev_disk_wb = prev_disk_rc = prev_disk_wc = None
    prev_udp_inerr, prev_udp_buferr = _read_udp_snmp() if _IS_LINUX else (None, None)

    prev_io, prev_ctx = {}, {}
    start_ts = time.monotonic()
    time.sleep(interval)

    while True:
        t0 = time.monotonic()
        now_t = round(t0 - start_ts, 2)
        ct = psutil.cpu_times_percent(interval=None)
        per_core = psutil.cpu_percent(percpu=True)
        load1, load5, load15 = psutil.getloadavg()
        _cpu_st = psutil.cpu_stats()
        dt_net = t0 - prev_net_t
        cs_per_s = round((_cpu_st.ctx_switches - prev_cs) / dt_net, 0) if dt_net > 0 else 0
        intr_per_s = round((_cpu_st.interrupts - prev_intr) / dt_net, 0) if dt_net > 0 else 0
        prev_cs, prev_intr = _cpu_st.ctx_switches, _cpu_st.interrupts
        vm, swap = psutil.virtual_memory(), psutil.swap_memory()

        sys_out = {
            "cpu":          round(100.0 - ct.idle, 1),
            "cpu_per_core": [round(v, 1) for v in per_core],
            "cpu_usr":      round(ct.user, 1),
            "cpu_sys":      round(ct.system, 1),
            "cpu_irq":      round(getattr(ct, "irq",     0.0), 1),
            "cpu_soft":     round(getattr(ct, "softirq", 0.0), 1),
            "cpu_iowait":   round(getattr(ct, "iowait",  0.0), 1),
            "cpu_steal":    round(getattr(ct, "steal",   0.0), 1),
            "loadavg_1":    round(load1,  2),
            "loadavg_5":    round(load5,  2),
            "loadavg_15":   round(load15, 2),
            "ctx_switches": int(cs_per_s),
            "interrupts":   int(intr_per_s),
            "mem_pct":      round(vm.percent, 1),
            "mem_used_mb":  round(vm.used / 1048576, 0),
            "swap_used_mb": round(swap.used / 1048576, 0),
        }

        # PSI (Linux)
        psi = _read_psi()
        if psi: sys_out["psi"] = psi

        # TCP snapshot: both per-status dict and aggregate estab/timewait
        tcp = _read_tcp_summary()
        if tcp:
            sys_out["tcp"] = tcp
            sys_out["tcp_estab"]    = tcp.get("ESTABLISHED", 0)
            sys_out["tcp_timewait"] = tcp.get("TIME_WAIT", 0)

        # Disk deltas
        if prev_disk_rb is not None and dt_net > 0:
            try:
                _d = psutil.disk_io_counters()
                sys_out.update({
                    "disk_read_kbs":   round((_d.read_bytes  - prev_disk_rb) / dt_net / 1024, 1),
                    "disk_write_kbs":  round((_d.write_bytes - prev_disk_wb) / dt_net / 1024, 1),
                    "disk_read_iops":  round((_d.read_count  - prev_disk_rc) / dt_net, 1),
                    "disk_write_iops": round((_d.write_count - prev_disk_wc) / dt_net, 1),
                })
                prev_disk_rb, prev_disk_wb = _d.read_bytes, _d.write_bytes
                prev_disk_rc, prev_disk_wc = _d.read_count, _d.write_count
            except Exception: pass

        # UDP errors (Linux)
        udp_rx_err = udp_buf_err = None
        if _IS_LINUX and prev_udp_inerr is not None and dt_net > 0:
            cur_ue, cur_be = _read_udp_snmp()
            udp_rx_err  = round((cur_ue - prev_udp_inerr)  / dt_net, 2)
            udp_buf_err = round((cur_be - prev_udp_buferr) / dt_net, 2)
            prev_udp_inerr, prev_udp_buferr = cur_ue, cur_be
        if udp_rx_err is not None:
            sys_out["udp_rx_err"]  = udp_rx_err
            sys_out["udp_buf_err"] = udp_buf_err

        # Network deltas
        cur_net = psutil.net_io_counters(pernic=True)
        rx_b = tx_b = rx_pkts = tx_pkts = drop_in = drop_out = err_in = err_out = 0
        for nic, cnt in cur_net.items():
            if nic in _NET_EXCLUDE or nic.startswith("veth"): continue
            prev = prev_net.get(nic)
            if prev and dt_net > 0:
                rx_b     += cnt.bytes_recv   - prev.bytes_recv
                tx_b     += cnt.bytes_sent   - prev.bytes_sent
                rx_pkts  += cnt.packets_recv - prev.packets_recv
                tx_pkts  += cnt.packets_sent - prev.packets_sent
                drop_in  += cnt.dropin       - prev.dropin
                drop_out += cnt.dropout      - prev.dropout
                err_in   += cnt.errin        - prev.errin
                err_out  += cnt.errout       - prev.errout
        if dt_net > 0:
            sys_out.update({
                "net_rx_kbs":   round(rx_b    / dt_net / 1024, 1),
                "net_tx_kbs":   round(tx_b    / dt_net / 1024, 1),
                "net_rx_pkts":  round(rx_pkts / dt_net, 0),
                "net_tx_pkts":  round(tx_pkts / dt_net, 0),
                "net_drop_in":  round(drop_in  / dt_net, 2),
                "net_drop_out": round(drop_out / dt_net, 2),
                "net_err_in":   round(err_in   / dt_net, 2),
                "net_err_out":  round(err_out  / dt_net, 2),
            })
        prev_net, prev_net_t = cur_net, t0

        # Process groups
        acc = {g: {"cpu": 0.0, "rss": 0.0, "vms": 0.0, "rk": 0.0, "wk": 0.0,
                   "cswch": 0.0, "nvcswch": 0.0, "fds": 0} for g in ALL_GROUPS}
        new_io, new_ctx, pid_cpu, pid_rss = {}, {}, [], []

        for p in psutil.process_iter(["pid", "name", "cpu_percent", "memory_info",
                                       "io_counters", "num_ctx_switches", "num_fds"]):
            try:
                i = p.info
                g = get_group(i["name"] or "")
                cpu = (i["cpu_percent"] or 0.0) / nproc
                acc[g]["cpu"] += cpu
                if cpu > 0.5: pid_cpu.append({"pid": i["pid"], "name": i["name"], "cpu": round(cpu, 1)})
                if i["memory_info"]:
                    rss = i["memory_info"].rss / 1048576
                    vms = i["memory_info"].vms / 1048576
                    acc[g]["rss"] = max(acc[g]["rss"], rss)
                    acc[g]["vms"] = max(acc[g]["vms"], vms)
                    if rss > 1: pid_rss.append({"pid": i["pid"], "name": i["name"], "rss": round(rss, 1)})
                if i["io_counters"]:
                    new_io[i["pid"]] = (i["io_counters"].read_bytes, i["io_counters"].write_bytes, t0)
                    if i["pid"] in prev_io:
                        pr, pw, pt = prev_io[i["pid"]]
                        dt = t0 - pt
                        if dt > 0:
                            acc[g]["rk"] += (i["io_counters"].read_bytes - pr) / dt / 1024
                            acc[g]["wk"] += (i["io_counters"].write_bytes - pw) / dt / 1024
                ctx = i.get("num_ctx_switches")
                if ctx is not None:
                    new_ctx[i["pid"]] = (ctx.voluntary, ctx.involuntary, t0)
                    if i["pid"] in prev_ctx:
                        pv, pi2, pt = prev_ctx[i["pid"]]
                        dt = t0 - pt
                        if dt > 0:
                            acc[g]["cswch"]  += (ctx.voluntary   - pv)  / dt
                            acc[g]["nvcswch"] += (ctx.involuntary - pi2) / dt
                acc[g]["fds"] += i.get("num_fds") or 0
            except (psutil.NoSuchProcess, psutil.AccessDenied): continue
        prev_io, prev_ctx = new_io, new_ctx

        procs_out = {}
        for g, d in acc.items():
            entry = {}
            if d["cpu"]:    entry["cpu"]     = round(d["cpu"],    2)
            if d["rss"]:    entry["rss"]     = round(d["rss"],    1)
            if d["vms"]:    entry["vms"]     = round(d["vms"],    1)
            if d["rk"]:     entry["rk"]      = round(d["rk"],     1)
            if d["wk"]:     entry["wk"]      = round(d["wk"],     1)
            if d["cswch"]:  entry["cswch"]   = round(d["cswch"],  1)
            if d["nvcswch"]:entry["nvcswch"] = round(d["nvcswch"],1)
            if d["fds"]:    entry["fds"]     = d["fds"]
            if entry:
                procs_out[g] = entry

        out = {"t": now_t, "sys": sys_out, "procs": procs_out}
        if pid_cpu: out["top_cpu"] = sorted(pid_cpu, key=lambda x: x["cpu"], reverse=True)[:10]
        if pid_rss: out["top_rss"] = sorted(pid_rss, key=lambda x: x["rss"], reverse=True)[:10]
        print(json.dumps(out, separators=(",", ":")), flush=True)

        elapsed = time.monotonic() - t0
        time.sleep(max(0, interval - elapsed))

# --- Subcommand: Parse (Raw Monitoring to JSON) ---

_PROC_FIELDS = {"cpu", "rss", "vms", "read_kbs", "write_kbs", "cswch", "nvcswch", "fds"}

def run_parse(args):
    result = {"processes": {}, "system": {}, "top_cpu": {}, "top_rss": {}}
    cpu_per_core_series = []

    if os.path.exists(args.input):
        with open(args.input) as f:
            for line in f:
                try:
                    row = json.loads(line)
                    t = row["t"]
                    sys_d = row.get("sys", {})
                    for k, v in sys_d.items():
                        if k == "cpu_per_core":
                            if isinstance(v, list):
                                while len(cpu_per_core_series) < len(v):
                                    cpu_per_core_series.append([])
                                for i, cv in enumerate(v):
                                    cpu_per_core_series[i].append([t, cv])
                        elif k == "psi" and isinstance(v, dict):
                            for sub_k, sub_v in v.items():
                                result["system"].setdefault(f"psi_{sub_k}", []).append([t, sub_v])
                        elif k == "tcp" and isinstance(v, dict):
                            for sub_k, sub_v in v.items():
                                result["system"].setdefault(f"tcp_{sub_k}", []).append([t, sub_v])
                        elif not isinstance(v, (list, dict)):
                            result["system"].setdefault(k, []).append([t, v])

                    procs_d = row.get("procs", {})
                    for g, gd in procs_d.items():
                        for k, v in gd.items():
                            fld = "read_kbs" if k == "rk" else ("write_kbs" if k == "wk" else k)
                            if fld in _PROC_FIELDS and not isinstance(v, (list, dict)):
                                result["processes"].setdefault(g, {}).setdefault(fld, []).append([t, v])

                    for entry in row.get("top_cpu", []):
                        name = entry.get("name") or "?"
                        result["top_cpu"].setdefault(name, []).append([t, entry.get("cpu", 0)])
                    for entry in row.get("top_rss", []):
                        name = entry.get("name") or "?"
                        result["top_rss"].setdefault(name, []).append([t, entry.get("rss", 0)])
                except Exception: continue

    if cpu_per_core_series:
        result["system"]["cpu_per_core"] = cpu_per_core_series

    result["k6_offset"] = args.k6_offset
    result["processes"] = {g: v for g, v in result["processes"].items() if any(v.values())}
    if not result["top_cpu"]: del result["top_cpu"]
    if not result["top_rss"]: del result["top_rss"]

    with open(args.output, "w") as f:
        json.dump(result, f, indent=2)
    print(f"Parsed {args.input} -> {args.output}")

# --- Subcommand: Inject (Embed resource-data into bench-results) ---

def run_inject(args):
    with open(args.result) as f:
        entry = json.load(f)
    with open(args.resources) as f:
        res_data = json.load(f)

    if args.core:
        entry["core_resources"] = res_data
        print(f"Injected {args.resources} -> {args.result} as core_resources")
    else:
        found = False
        for phase in entry.get("phases", {}).values():
            if args.case_name in phase.get("cases", {}):
                phase["cases"][args.case_name]["resources"] = res_data
                found = True
                break
        if not found:
            print(f"WARNING: case '{args.case_name}' not found in phases", file=sys.stderr)
        else:
            print(f"Injected {args.resources} -> {args.result} case[{args.case_name!r}].resources")

    with open(args.result, "w") as f:
        json.dump(entry, f, separators=(",", ":"))

# --- Subcommand: Publish (Merge & Update Data.js) ---

def run_publish(args):
    with open(args.result) as f:
        entry = json.load(f)

    entry["commit"] = {"id": args.sha, "message": args.msg, "url": args.url}
    if args.run_url:
        entry["run_url"] = args.run_url

    if args.trace_cases_file and os.path.exists(args.trace_cases_file):
        with open(args.trace_cases_file) as f:
            trace_cases = json.load(f)
        if isinstance(trace_cases, list) and trace_cases:
            entry.setdefault("artifacts", {})["trace_cases"] = trace_cases

    # Update index data.js
    PREFIX, SUFFIX = "window.BENCHMARK_DATA = ", ";"
    entries = []
    if os.path.exists(args.data):
        with open(args.data) as f:
            raw = f.read().strip()
            if raw.startswith(PREFIX):
                try: entries = json.loads(raw[len(PREFIX):-len(SUFFIX)]).get("entries", [])
                except Exception: pass

    sha7 = args.sha[:7]
    detail_dir = os.path.join(os.path.dirname(os.path.abspath(args.data)), "data")
    os.makedirs(detail_dir, exist_ok=True)
    with open(os.path.join(detail_dir, f"{sha7}.json"), "w") as f:
        json.dump(entry, f, separators=(",", ":"))

    index_entry = {k: entry.get(k) for k in ["commit", "timestamp", "summary", "catalog", "run_url", "artifacts"]}
    index_entry["scenarios"] = [
        {"name": name, **{k: c.get(k) for k in ["label", "protocol", "direction", "category", "tunnel"]},
         **{k: (c.get("perf") or {}).get(k) for k in ["p50", "p95", "rps", "err", "requests"]}}
        for phase in entry.get("phases", {}).values()
        for name, c in phase.get("cases", {}).items()
    ]
    entries.append(index_entry)
    entries = entries[-args.max_entries:]

    with open(args.data, "w") as f:
        f.write(PREFIX + json.dumps({"entries": entries}, separators=(",", ":")) + SUFFIX + "\n")
    print(f"Published {sha7}, total: {len(entries)}")

# --- CLI Entry Point ---

def main():
    p = argparse.ArgumentParser(description="DuoTunnel Benchmark Tool")
    sub = p.add_subparsers(dest="cmd", required=True)

    # Collect
    c = sub.add_parser("collect")
    c.add_argument("interval", type=float, nargs="?", default=1.0)

    # Parse
    r = sub.add_parser("parse")
    r.add_argument("--input", required=True)
    r.add_argument("--output", required=True)
    r.add_argument("--k6-offset", type=int, default=0)

    # Publish
    b = sub.add_parser("publish")
    b.add_argument("--result", required=True)
    b.add_argument("--data", required=True)
    b.add_argument("--sha", required=True)
    b.add_argument("--msg", default="")
    b.add_argument("--url", default="")
    b.add_argument("--run-url", default="")
    b.add_argument("--trace-cases-file", default="")
    b.add_argument("--max-entries", type=int, default=50)

    i = sub.add_parser("inject")
    i.add_argument("--result", required=True)
    i.add_argument("--resources", required=True)
    i_group = i.add_mutually_exclusive_group(required=True)
    i_group.add_argument("--core", action="store_true")
    i_group.add_argument("--case-name")

    args = p.parse_args()
    if args.cmd == "collect": run_collect(args)
    elif args.cmd == "parse": run_parse(args)
    elif args.cmd == "inject": run_inject(args)
    elif args.cmd == "publish": run_publish(args)

if __name__ == "__main__":
    main()
