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
            "cpu": round(100.0 - ct.idle, 1),
            "cpu_per_core": [round(v, 1) for v in per_core],
            "cpu_usr": round(ct.user, 1), "cpu_sys": round(ct.system, 1),
            "loadavg_1": round(load1, 2), "ctx_switches": int(cs_per_s), "interrupts": int(intr_per_s),
            "mem_pct": round(vm.percent, 1), "mem_used_mb": round(vm.used / 1048576, 0),
        }

        # PSI & TCP (Linux/Advanced)
        psi = _read_psi()
        if psi: sys_out["psi"] = psi
        tcp = _read_tcp_summary()
        if tcp: sys_out["tcp"] = tcp

        # Disk & Network deltas
        if prev_disk_rb is not None and dt_net > 0:
            try:
                _d = psutil.disk_io_counters()
                sys_out.update({
                    "disk_read_kbs": round((_d.read_bytes - prev_disk_rb) / dt_net / 1024, 1),
                    "disk_write_kbs": round((_d.write_bytes - prev_disk_wb) / dt_net / 1024, 1),
                })
                prev_disk_rb, prev_disk_wb = _d.read_bytes, _d.write_bytes
            except Exception: pass

        cur_net = psutil.net_io_counters(pernic=True)
        rx_b = tx_b = 0
        for nic, cnt in cur_net.items():
            if nic in _NET_EXCLUDE or nic.startswith("veth"): continue
            prev = prev_net.get(nic)
            if prev and dt_net > 0:
                rx_b += cnt.bytes_recv - prev.bytes_recv
                tx_b += cnt.bytes_sent - prev.bytes_sent
        sys_out.update({"net_rx_kbs": round(rx_b/dt_net/1024, 1), "net_tx_kbs": round(tx_b/dt_net/1024, 1)})
        prev_net, prev_net_t = cur_net, t0

        # Process groups
        acc = {g: {"cpu": 0.0, "rss": 0.0, "vms": 0.0, "rk": 0.0, "wk": 0.0, "fds": 0} for g in ALL_GROUPS}
        new_io, pid_cpu, pid_rss = {}, [], []

        for p in psutil.process_iter(["pid", "name", "cpu_percent", "memory_info", "io_counters", "num_fds"]):
            try:
                i = p.info
                g = get_group(i["name"] or "")
                cpu = (i["cpu_percent"] or 0.0) / nproc
                acc[g]["cpu"] += cpu
                if cpu > 0.5: pid_cpu.append({"pid": i["pid"], "name": i["name"], "cpu": round(cpu, 1)})
                if i["memory_info"]:
                    rss = i["memory_info"].rss / 1048576
                    acc[g]["rss"] = max(acc[g]["rss"], rss)
                    if rss > 10: pid_rss.append({"pid": i["pid"], "name": i["name"], "rss": round(rss, 1)})
                if i["io_counters"]:
                    new_io[i["pid"]] = (i["io_counters"].read_bytes, i["io_counters"].write_bytes, t0)
                    if i["pid"] in prev_io:
                        pr, pw, pt = prev_io[i["pid"]]
                        dt = t0 - pt
                        if dt > 0:
                            acc[g]["rk"] += (i["io_counters"].read_bytes - pr) / dt / 1024
                            acc[g]["wk"] += (i["io_counters"].write_bytes - pw) / dt / 1024
                acc[g]["fds"] += i.get("num_fds") or 0
            except (psutil.NoSuchProcess, psutil.AccessDenied): continue
        prev_io = new_io

        procs_out = {g: {k: round(v, 1) for k, v in d.items() if v > 0} for g, d in acc.items()}
        procs_out = {g: v for g, v in procs_out.items() if v}
        
        out = {"t": now_t, "sys": sys_out, "procs": procs_out}
        if pid_cpu: out["top_cpu"] = sorted(pid_cpu, key=lambda x: x["cpu"], reverse=True)[:5]
        print(json.dumps(out, separators=(",", ":")), flush=True)

        elapsed = time.monotonic() - t0
        time.sleep(max(0, interval - elapsed))

# --- Subcommand: Parse (Raw Monitoring to JSON) ---

def run_parse(args):
    result = {"processes": {g: {"cpu":[], "rss":[], "read_kbs":[], "fds":[]} for g in ALL_GROUPS}, "system": {}}
    
    if os.path.exists(args.input):
        with open(args.input) as f:
            for line in f:
                try:
                    row = json.loads(line)
                    t = row["t"]
                    sys_d = row.get("sys", {})
                    for k, v in sys_d.items():
                        if k in ["psi", "tcp"]:
                            # Flattern nested dicts: psi.cpu, tcp.ESTABLISHED, etc.
                            for sub_k, sub_v in v.items():
                                result["system"].setdefault(f"{k}_{sub_k}", []).append([t, sub_v])
                        else:
                            result["system"].setdefault(k, []).append([t, v])
                    
                    procs_d = row.get("procs", {})
                    for g, gd in procs_d.items():
                        for k, v in gd.items():
                            fld = "read_kbs" if k == "rk" else ("write_kbs" if k == "wk" else k)
                            if fld in ["cpu", "rss", "vms", "read_kbs", "write_kbs", "fds"]:
                                result["processes"].setdefault(g, {}).setdefault(fld, []).append([t, v])
                except Exception: continue

    result["k6OffsetSeconds"] = args.k6_offset
    result["processes"] = {g: v for g, v in result["processes"].items() if any(v.values())}
    result["catalog"] = {
        "categories": SCHEMA.get("categories", []),
        "metrics": SCHEMA.get("metrics", []),
        "groups": SCHEMA.get("groups", [])
    }
    
    with open(args.output, "w") as f:
        json.dump(result, f, indent=2)
    print(f"Parsed {args.input} -> {args.output}")

# --- Subcommand: Publish (Merge & Update Data.js) ---

def run_publish(args):
    with open(args.result) as f:
        entry = json.load(f)

    entry["commit"] = {"id": args.sha, "message": args.msg, "url": args.url}
    if args.run_url:
        entry["run_url"] = args.run_url

    k6_active_offset = entry.get("k6_active_offset", 0)
    if args.resources and os.path.exists(args.resources):
        with open(args.resources) as f:
            res_data = json.load(f)
            res_data["k6OffsetSeconds"] = res_data.get("k6OffsetSeconds", 0) + k6_active_offset
            entry.setdefault("resources_per_case", {})["core"] = res_data
            if isinstance(res_data.get("catalog"), dict):
                entry["resources_per_case"]["catalog"] = res_data["catalog"]

    if args.trace_cases_file and os.path.exists(args.trace_cases_file):
        with open(args.trace_cases_file) as f:
            trace_cases = json.load(f)
        if isinstance(trace_cases, list) and trace_cases:
            entry.setdefault("artifacts", {})["trace_cases"] = trace_cases

    # Append FRP results if present
    if args.frp_result and os.path.exists(args.frp_result):
        with open(args.frp_result) as f:
            frp = json.load(f)
        off = args.frp_k6_offset + frp.get("k6_active_offset", 0)
        for sc in frp.get("scenarios", []):
            sc["tunnel"] = "frp"
            entry["scenarios"].append(sc)
        for ph in frp.get("phases", []):
            entry.setdefault("phases", []).append({**ph, "tunnel": "frp", "resourceCase": "core",
                                                  "start": (ph.get("start") or 0) + off,
                                                  "end": (ph.get("end") or 0) + off})

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
    index_entry["scenarios"] = [{k: s[k] for k in s if k in ["name", "category", "phase", "label", "p50", "p95", "rps", "err", "protocol", "direction", "tunnel"]} 
                              for s in entry.get("scenarios", [])]
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
    b.add_argument("--resources", default="")
    b.add_argument("--frp-result", default="")
    b.add_argument("--frp-k6-offset", type=int, default=0)
    b.add_argument("--msg", default="")
    b.add_argument("--url", default="")
    b.add_argument("--run-url", default="")
    b.add_argument("--trace-cases-file", default="")
    b.add_argument("--max-entries", type=int, default=50)

    args = p.parse_args()
    if args.cmd == "collect": run_collect(args)
    elif args.cmd == "parse": run_parse(args)
    elif args.cmd == "publish": run_publish(args)

if __name__ == "__main__":
    main()
