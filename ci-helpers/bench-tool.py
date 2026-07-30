#!/usr/bin/env python3
import argparse
import hashlib
import json
import math
import os
import pathlib
import platform
import re
import subprocess
import sys
import time
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

SNAPSHOT_SCHEMA_VERSION = 1
BENCHMARK_SCHEMA_VERSION = 1

def get_group(name):
    for prefix, g in PROC_GROUPS:
        if name == prefix or name.startswith(prefix):
            return g
    return "other"


def _read_text(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return None


def _sha256_bytes(value):
    return hashlib.sha256(value).hexdigest()


def _redact_text(value):
    if value is None:
        return None
    lines = []
    secret_line = re.compile(
        r"^(\s*(?:auth_token|token|password|secret|private_key|client_secret)\s*:\s*).*$",
        re.IGNORECASE,
    )
    secret_assignment = re.compile(
        r"(DUOTUNNEL_[A-Z0-9_]*(?:TOKEN|SECRET|PASSWORD)\s*[=:]\s*)[^\s]+",
        re.IGNORECASE,
    )
    for line in value.splitlines():
        line = secret_line.sub(r"\1<redacted>", line)
        line = secret_assignment.sub(r"\1<redacted>", line)
        lines.append(line)
    suffix = "\n" if value.endswith("\n") else ""
    return "\n".join(lines) + suffix


def _run_capture(command):
    try:
        completed = subprocess.run(
            command,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=5,
        )
        return {"status": completed.returncode, "output": completed.stdout.strip()}
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {"status": None, "output": str(exc)}


def _cgroup_value(name):
    for root in ("/sys/fs/cgroup", "/sys/fs/cgroup/unified"):
        value = _read_text(os.path.join(root, name))
        if value is not None:
            return value.strip()
    return None


def _parse_pid_file_spec(spec):
    if "=" in spec:
        label, path = spec.split("=", 1)
    else:
        label, path = pathlib.Path(spec).stem, spec
    return label, path


def _proc_environ(pid):
    selected = {
        "TOKIO_WORKER_THREADS",
        "DUOTUNNEL_CLIENT__AUTH_TOKEN",
        "STRESS_CPU_QUOTA",
        "DUOTUNNEL_BENCH_CPU_MODE",
        "DUOTUNNEL_BENCH_CPUSET",
    }
    try:
        raw = pathlib.Path(f"/proc/{pid}/environ").read_bytes()
    except OSError:
        return {}
    result = {}
    for field in raw.split(b"\0"):
        if b"=" not in field:
            continue
        key, value = field.split(b"=", 1)
        key = key.decode("utf-8", "replace")
        if key not in selected:
            continue
        result[key] = "<redacted>" if "TOKEN" in key else value.decode("utf-8", "replace")
    return result


def _process_snapshot(pid):
    status = _read_text(f"/proc/{pid}/status") or ""
    allowed = None
    for line in status.splitlines():
        if line.startswith("Cpus_allowed_list:"):
            allowed = line.split(":", 1)[1].strip()
            break
    cmdline = _read_text(f"/proc/{pid}/cmdline")
    return {
        "pid": pid,
        "cmdline": cmdline.replace("\0", " ").strip() if cmdline else None,
        "cpus_allowed_list": allowed,
        "cgroup": _read_text(f"/proc/{pid}/cgroup"),
        "environment": _proc_environ(pid),
    }


def _systemd_scope_snapshot(scope):
    result = _run_capture(
        [
            "systemctl",
            "show",
            scope,
            "--property=ControlGroup,AllowedCPUs,CPUQuotaPerSecUSec,CPUUsageNSec,TasksCurrent",
            "--no-pager",
        ]
    )
    properties = {}
    for line in result.get("output", "").splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            properties[key] = value
    return {"scope": scope, "status": result.get("status"), "properties": properties}


def _config_snapshot(path):
    absolute = os.path.abspath(path)
    raw = _read_text(absolute)
    if raw is None:
        return {"path": path, "exists": False}
    raw_bytes = raw.encode("utf-8", "replace")
    redacted = _redact_text(raw)
    return {
        "path": os.path.relpath(absolute, os.getcwd()),
        "exists": True,
        "sha256": _sha256_bytes(raw_bytes),
        "redacted_sha256": _sha256_bytes((redacted or "").encode("utf-8")),
        "content": redacted,
    }


def _log_evidence(path):
    absolute = os.path.abspath(path)
    raw = _read_text(absolute)
    if raw is None:
        return {"path": path, "exists": False}
    patterns = re.compile(
        r"effective|resolved|worker|shard|connection|buffer|pending|admission|parallel|config",
        re.IGNORECASE,
    )
    matched = [line for line in raw.splitlines() if patterns.search(line)]
    return {
        "path": os.path.relpath(absolute, os.getcwd()),
        "exists": True,
        "sha256": _sha256_bytes(raw.encode("utf-8", "replace")),
        "matched_lines": [_redact_text(line) for line in matched[-200:]],
        "complete": False,
    }


def run_snapshot(args):
    plan = {}
    if args.plan:
        try:
            with open(args.plan, "r", encoding="utf-8") as f:
                plan = json.load(f)
        except (OSError, json.JSONDecodeError) as exc:
            print(f"failed to read benchmark CPU plan: {exc}", file=sys.stderr)
            return 2

    configs = [_config_snapshot(path) for path in args.config]
    env_names = [
        "BENCH_PROFILE",
        "BENCH_CASE",
        "WORKER_THREADS",
        "TOKIO_WORKER_THREADS",
        "STRESS_CPU_QUOTA",
        "DUOTUNNEL_BENCH_CPU_MODE",
        "DUOTUNNEL_BENCH_CPUSET",
        "K6_CORE_STRESS_RATE",
        "DUOTUNNEL_COLLECT_RESOURCE_METRICS",
    ]
    environment = {name: os.environ[name] for name in env_names if name in os.environ}

    pid_entries = {}
    for spec in args.pid_file:
        label, path = _parse_pid_file_spec(spec)
        value = _read_text(path)
        try:
            pid = int((value or "").strip())
        except ValueError:
            pid = None
        pid_entries[label] = {
            "pid_file": path,
            "pid": _process_snapshot(pid) if pid and os.path.exists(f"/proc/{pid}") else None,
        }

    scopes = [_systemd_scope_snapshot(scope) for scope in args.scope]
    log_evidence = [_log_evidence(path) for path in args.log_file]
    cgroup_type = _run_capture(["stat", "-fc", "%T", "/sys/fs/cgroup"])
    affinity = None
    try:
        affinity = sorted(os.sched_getaffinity(0))
    except (AttributeError, OSError):
        pass

    config_fingerprint_payload = {
        "configs": [
            {
                "path": item.get("path"),
                "sha256": item.get("sha256"),
                "redacted_sha256": item.get("redacted_sha256"),
            }
            for item in configs
        ],
        "plan": plan.get("cpu_contract", plan),
        "environment": environment,
    }
    config_fingerprint = _sha256_bytes(
        json.dumps(config_fingerprint_payload, sort_keys=True, separators=(",", ":")).encode()
    )
    cpu_contract = plan.get("cpu_contract", {})
    cpu_fingerprint = _sha256_bytes(
        json.dumps(cpu_contract, sort_keys=True, separators=(",", ":")).encode()
    )
    comparable = bool(cpu_contract.get("enforced") and cpu_contract.get("cgroup_v2"))
    snapshot = {
        "schema_version": SNAPSHOT_SCHEMA_VERSION,
        "case": args.case_name,
        "created_at": datetime.utcnow().isoformat(timespec="milliseconds") + "Z",
        "git_sha": os.environ.get("GITHUB_SHA"),
        "config": {
            "files": configs,
            "fingerprint": config_fingerprint,
            "runtime_log_evidence": log_evidence,
            "runtime_values_complete": any(item.get("matched_lines") for item in log_evidence),
        },
        "cpu": {
            "platform": platform.platform(),
            "logical_cpus": os.cpu_count(),
            "process_affinity": affinity,
            "cgroup_filesystem": cgroup_type.get("output"),
            "cpuset_cpus_effective": _cgroup_value("cpuset.cpus.effective"),
            "cpuset_cpus": _cgroup_value("cpuset.cpus"),
            "cpu_max": _cgroup_value("cpu.max"),
            "cpu_stat": _cgroup_value("cpu.stat"),
        },
        "cpu_contract": {
            **cpu_contract,
            "fingerprint": cpu_fingerprint,
            "comparable": comparable,
        },
        "environment": environment,
        "processes": pid_entries,
        "scopes": scopes,
    }
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(snapshot, f, ensure_ascii=False, indent=2)
    print(f"Wrote benchmark snapshot: {args.output}")


def run_attach(args):
    with open(args.result, "r", encoding="utf-8") as f:
        entry = json.load(f)
    with open(args.metadata, "r", encoding="utf-8") as f:
        metadata = json.load(f)
    cases = entry.get("cases") or {}
    if args.case_name == "all":
        for name, case in cases.items():
            case["benchmark"] = metadata
        print(f"Attached benchmark snapshot to all cases in {args.result}")
    elif args.case_name not in cases:
        print(f"benchmark result has no case {args.case_name!r}", file=sys.stderr)
        return 2
    else:
        cases[args.case_name]["benchmark"] = metadata
        print(f"Attached benchmark snapshot to {args.result} case[{args.case_name!r}]")
    with open(args.result, "w", encoding="utf-8") as f:
        json.dump(entry, f, ensure_ascii=False, indent=2)


def _number(value):
    if value is None or isinstance(value, bool):
        return None
    try:
        number = float(value)
    except (TypeError, ValueError):
        return None
    return number if math.isfinite(number) else None


def _case_dropped_iterations(case, root_load):
    perf = case.get("perf") or {}
    if perf.get("droppedIterations") is not None:
        return _number(perf["droppedIterations"])
    return _number((root_load or {}).get("droppedIterations"))


def _validate_benchmark_result(result, require_comparable=False):
    failures = []
    cases = result.get("cases") or {}
    if not cases:
        failures.append("result has no cases")
    load = result.get("load") or {}
    for field in ("droppedIterations", "iterations"):
        value = _number(load.get(field))
        if value is None:
            failures.append(f"result.load.{field} is missing or not a finite number")
        elif value < 0:
            failures.append(f"result.load.{field} must be non-negative")
    for name, case in cases.items():
        perf = case.get("perf") or {}
        if not perf:
            continue
        for field in ("p50", "p95", "p99", "p99_9", "rps", "err", "requests", "droppedIterations"):
            if field not in perf:
                failures.append(f"case {name}: perf.{field} is missing")
                continue
            value = _number(perf.get(field))
            if value is None:
                failures.append(f"case {name}: perf.{field} is not a finite number")
            elif value < 0:
                failures.append(f"case {name}: perf.{field} must be non-negative")
        err = _number(perf.get("err"))
        if err is not None and err > 100:
            failures.append(f"case {name}: perf.err must be at most 100")
        benchmark = case.get("benchmark") or {}
        if benchmark.get("schema_version") != SNAPSHOT_SCHEMA_VERSION:
            failures.append(f"case {name}: benchmark snapshot schema is missing or unsupported")
        if not benchmark.get("config", {}).get("fingerprint"):
            failures.append(f"case {name}: effective config fingerprint is missing")
        contract = benchmark.get("cpu_contract") or {}
        if not contract.get("fingerprint"):
            failures.append(f"case {name}: CPU contract fingerprint is missing")
        if require_comparable and not contract.get("comparable"):
            failures.append(f"case {name}: CPU contract is not comparable")
    return failures


def _compare_benchmark_results(current, baseline, args):
    failures = []
    comparisons = []
    current_cases = current.get("cases") or {}
    baseline_cases = baseline.get("cases") or {}
    for name, case in current_cases.items():
        current_perf = case.get("perf") or {}
        if not current_perf:
            continue
        if name not in baseline_cases:
            failures.append(f"baseline is missing case {name}")
            continue
        old = baseline_cases[name]
        current_benchmark = case.get("benchmark") or {}
        baseline_benchmark = old.get("benchmark") or {}
        if current_benchmark.get("config", {}).get("fingerprint") != baseline_benchmark.get("config", {}).get("fingerprint"):
            failures.append(f"case {name}: effective config fingerprint changed")
        if current_benchmark.get("cpu_contract", {}).get("fingerprint") != baseline_benchmark.get("cpu_contract", {}).get("fingerprint"):
            failures.append(f"case {name}: CPU contract fingerprint changed")
        baseline_perf = old.get("perf") or {}
        row = {"case": name, "checks": []}
        for metric, limit in (("p95", args.max_p95_regression_pct), ("p99", args.max_p99_regression_pct), ("p99_9", args.max_p99_9_regression_pct)):
            now = _number(current_perf.get(metric))
            old_value = _number(baseline_perf.get(metric))
            if now is None or old_value is None or old_value <= 0:
                failures.append(f"case {name}: cannot compare {metric}")
                continue
            allowed = old_value * (1 + limit / 100.0)
            row["checks"].append({"metric": metric, "current": now, "baseline": old_value, "allowed": allowed})
            if now > allowed:
                failures.append(f"case {name}: {metric} regression {now:.2f} > {allowed:.2f}")
        now_rps = _number(current_perf.get("rps"))
        old_rps = _number(baseline_perf.get("rps"))
        if now_rps is None or old_rps is None or old_rps <= 0:
            failures.append(f"case {name}: cannot compare rps")
        else:
            allowed = old_rps * (1 - args.max_rps_drop_pct / 100.0)
            row["checks"].append({"metric": "rps", "current": now_rps, "baseline": old_rps, "allowed": allowed})
            if now_rps < allowed:
                failures.append(f"case {name}: rps regression {now_rps:.2f} < {allowed:.2f}")
        now_err = _number(current_perf.get("err"))
        old_err = _number(baseline_perf.get("err"))
        if now_err is None or old_err is None:
            failures.append(f"case {name}: cannot compare err")
        elif now_err > old_err + args.max_error_increase_pct_points:
            failures.append(f"case {name}: error rate {now_err:.2f}% > {old_err + args.max_error_increase_pct_points:.2f}%")
        current_drop = _case_dropped_iterations(case, current.get("load"))
        baseline_drop = _case_dropped_iterations(old, baseline.get("load"))
        if current_drop is None or baseline_drop is None:
            failures.append(f"case {name}: cannot compare dropped iterations")
        elif current_drop > baseline_drop + args.max_dropped_increase:
            failures.append(f"case {name}: dropped iterations {current_drop:g} > {baseline_drop + args.max_dropped_increase:g}")
        comparisons.append(row)
    return failures, comparisons


def run_gate(args):
    with open(args.result, "r", encoding="utf-8") as f:
        result = json.load(f)
    failures = _validate_benchmark_result(result, args.require_comparable)
    load = result.get("load") or {}
    if args.max_dropped_iterations is not None:
        dropped = _number(load.get("droppedIterations"))
        if dropped is None:
            failures.append("cannot enforce dropped-iterations limit without result.load.droppedIterations")
        elif dropped > args.max_dropped_iterations:
            failures.append(f"dropped iterations {dropped:g} exceed limit {args.max_dropped_iterations:g}")

    comparisons = []
    if args.baseline:
        if not os.path.isfile(args.baseline):
            failures.append(f"baseline file does not exist: {args.baseline}")
        else:
            with open(args.baseline, "r", encoding="utf-8") as f:
                baseline = json.load(f)
            baseline_failures = _validate_benchmark_result(baseline, args.require_comparable)
            failures.extend(f"baseline: {failure}" for failure in baseline_failures)
            if not baseline_failures:
                compare_failures, comparisons = _compare_benchmark_results(result, baseline, args)
                failures.extend(compare_failures)

    report = {
        "schema_version": BENCHMARK_SCHEMA_VERSION,
        "status": "fail" if failures else "pass",
        "result": args.result,
        "baseline": args.baseline or None,
        "failures": failures,
        "comparisons": comparisons,
    }
    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
    if failures:
        print(json.dumps(report, ensure_ascii=False, indent=2), file=sys.stderr)
        return 1
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0

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

        out = {
            "t": now_t,
            "timestamp_ms": time.time_ns() // 1_000_000,
            "sys": sys_out,
            "procs": procs_out,
        }
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
    selected_samples = 0
    total_samples = 0

    if os.path.exists(args.input):
        with open(args.input) as f:
            for line in f:
                try:
                    row = json.loads(line)
                    total_samples += 1
                    timestamp_ms = row.get("timestamp_ms")
                    if args.k6_start_ms is not None or args.k6_end_ms is not None:
                        if timestamp_ms is None:
                            continue
                        if args.k6_start_ms is not None and timestamp_ms < args.k6_start_ms:
                            continue
                        if args.k6_end_ms is not None and timestamp_ms > args.k6_end_ms:
                            continue
                        t = round((timestamp_ms - (args.k6_start_ms or timestamp_ms)) / 1000, 2)
                    else:
                        t = row["t"]
                    selected_samples += 1
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
    result["sampling_window"] = {
        "start_ms": args.k6_start_ms,
        "end_ms": args.k6_end_ms,
        "duration_ms": (
            args.k6_end_ms - args.k6_start_ms
            if args.k6_start_ms is not None and args.k6_end_ms is not None
            else None
        ),
        "samples": selected_samples,
        "total_samples": total_samples,
    }
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

    cases = entry.setdefault("cases", {})
    if args.case_name not in cases:
        cases[args.case_name] = {"label": args.case_name, "resources": None}
    cases[args.case_name]["resources"] = res_data
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
    run_id = args.run_id if getattr(args, "run_id", None) else ""
    if not run_id and args.run_url:
        m = re.search(r"/runs/(\d+)", args.run_url)
        if m:
            run_id = m.group(1)

    run_key = f"{sha7}-{run_id}" if run_id else sha7
    entry["id"] = run_key

    detail_dir = os.path.join(os.path.dirname(os.path.abspath(args.data)), "data")
    os.makedirs(detail_dir, exist_ok=True)
    with open(os.path.join(detail_dir, f"{run_key}.json"), "w") as f:
        json.dump(entry, f, separators=(",", ":"))

    index_entry = {k: entry.get(k) for k in ["id", "commit", "timestamp", "summary", "catalog", "run_url", "artifacts"]}
    index_entry["scenarios"] = [
        {"name": name, **{k: c.get(k) for k in ["label", "protocol", "direction", "category", "tunnel"]},
         **{k: (c.get("perf") or {}).get(k) for k in ["p50", "p95", "p99", "p99_9", "rps", "err", "requests"]}}
        for name, c in entry.get("cases", {}).items()
    ]
    entries.append(index_entry)
    entries = entries[-args.max_entries:]

    with open(args.data, "w") as f:
        f.write(PREFIX + json.dumps({"entries": entries}, separators=(",", ":")) + SUFFIX + "\n")
    print(f"Published {run_key}, total: {len(entries)}")

    # Prune old detail files & traces chronologically
    valid_ids = set()
    for e in entries:
        v_id = e.get("id") or (e.get("commit", {}).get("id", "")[:7])
        if v_id:
            valid_ids.add(v_id)

    # Prune data/
    if os.path.exists(detail_dir):
        for filename in os.listdir(detail_dir):
            if filename.endswith(".json"):
                name = filename[:-5]
                if name not in valid_ids:
                    try:
                        os.remove(os.path.join(detail_dir, filename))
                        print(f"Pruned old detail file: {filename}")
                    except Exception as ex:
                        print(f"Failed to prune detail file {filename}: {ex}", file=sys.stderr)

    # Prune traces/
    latest_ids_for_traces = []
    for e in reversed(entries):
        v_id = e.get("id") or (e.get("commit", {}).get("id", "")[:7])
        if v_id and v_id not in latest_ids_for_traces:
            latest_ids_for_traces.append(v_id)
            if len(latest_ids_for_traces) >= args.max_traces:
                break
    latest_ids_for_traces = set(latest_ids_for_traces)

    traces_dir = os.path.join(os.path.dirname(os.path.abspath(args.data)), "traces")
    if os.path.exists(traces_dir):
        for filename in os.listdir(traces_dir):
            matched = False
            for v_id in latest_ids_for_traces:
                if filename.startswith(f"{v_id}-"):
                    matched = True
                    break
            if not matched:
                try:
                    os.remove(os.path.join(traces_dir, filename))
                    print(f"Pruned old trace file: {filename}")
                except Exception as ex:
                    print(f"Failed to prune trace file {filename}: {ex}", file=sys.stderr)

# --- CLI Entry Point ---

def main():
    p = argparse.ArgumentParser(description="DuoTunnel Benchmark Tool")
    sub = p.add_subparsers(dest="cmd", required=True)

    # Collect
    c = sub.add_parser("collect")
    c.add_argument("interval", type=float, nargs="?", default=1.0)

    s = sub.add_parser("snapshot")
    s.add_argument("--output", required=True)
    s.add_argument("--case-name", required=True)
    s.add_argument("--plan", default="")
    s.add_argument("--config", action="append", default=[])
    s.add_argument("--scope", action="append", default=[])
    s.add_argument("--pid-file", action="append", default=[])
    s.add_argument("--log-file", action="append", default=[])

    # Parse
    r = sub.add_parser("parse")
    r.add_argument("--input", required=True)
    r.add_argument("--output", required=True)
    r.add_argument("--k6-offset", type=int, default=0)
    r.add_argument("--k6-start-ms", type=int)
    r.add_argument("--k6-end-ms", type=int)

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
    b.add_argument("--run-id", default="")
    b.add_argument("--max-traces", type=int, default=3)

    i = sub.add_parser("inject")
    i.add_argument("--result", required=True)
    i.add_argument("--resources", required=True)
    i.add_argument("--case-name", required=True)

    a = sub.add_parser("attach")
    a.add_argument("--result", required=True)
    a.add_argument("--metadata", required=True)
    a.add_argument("--case-name", required=True)

    g = sub.add_parser("gate")
    g.add_argument("--result", required=True)
    g.add_argument("--baseline", default="")
    g.add_argument("--output", default="")
    g.add_argument("--require-comparable", action="store_true")
    g.add_argument("--max-dropped-iterations", type=float, default=None)
    g.add_argument("--max-p95-regression-pct", type=float, default=10.0)
    g.add_argument("--max-p99-regression-pct", type=float, default=10.0)
    g.add_argument("--max-p99-9-regression-pct", type=float, default=15.0)
    g.add_argument("--max-rps-drop-pct", type=float, default=5.0)
    g.add_argument("--max-error-increase-pct-points", type=float, default=1.0)
    g.add_argument("--max-dropped-increase", type=float, default=0.0)

    args = p.parse_args()
    if args.cmd == "collect": run_collect(args)
    elif args.cmd == "snapshot": return_code = run_snapshot(args)
    elif args.cmd == "parse": run_parse(args)
    elif args.cmd == "inject": run_inject(args)
    elif args.cmd == "attach": return_code = run_attach(args)
    elif args.cmd == "gate": return_code = run_gate(args)
    elif args.cmd == "publish": run_publish(args)
    else: return_code = 0
    if "return_code" in locals():
        sys.exit(return_code)

if __name__ == "__main__":
    main()
