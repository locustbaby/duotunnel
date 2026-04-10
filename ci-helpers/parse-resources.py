#!/usr/bin/env python3
import json
import re
import argparse
import os
from datetime import datetime


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Map process name (as seen in pidstat Command column, max 15 chars) to group key.
PROC_GROUPS = [
    ("server",          "server"),
    ("client",          "client"),
    ("http-echo-serve", "http_echo"),  # truncated by kernel to 15 chars
    ("ws-echo-server",  "ws_echo"),
    ("grpc-echo-serve", "grpc_echo"),  # truncated
    ("k6",              "k6"),
    ("frps",            "frps"),
    ("frpc",            "frpc"),
]


def _proc_group(cmd):
    for name, group in PROC_GROUPS:
        if cmd == name or cmd.startswith(name):
            return group
    return "other"


def _parse_hms(tok):
    tok = tok.strip()
    for fmt in ("%I:%M:%S %p", "%H:%M:%S"):
        try:
            return datetime.strptime(tok, fmt)
        except ValueError:
            pass
    return None


def _ts_from_parts(parts):
    if len(parts) >= 2 and parts[1].upper() in ("AM", "PM"):
        return _parse_hms(f"{parts[0]} {parts[1]}")
    return _parse_hms(parts[0])


def _to_relative(ts, epoch):
    if ts is None or epoch is None:
        return None
    return round((ts - epoch).total_seconds(), 1)


def _read_nproc(path="/tmp/nproc"):
    try:
        with open(path) as f:
            return max(1, int(f.read().strip()))
    except Exception:
        try:
            import multiprocessing
            return multiprocessing.cpu_count()
        except Exception:
            return 1


def _split_blocks(lines):
    """Split blank-line-delimited blocks."""
    blocks, current = [], []
    for line in lines:
        s = line.strip()
        if not s:
            if current:
                blocks.append(current)
                current = []
            continue
        current.append(s)
    if current:
        blocks.append(current)
    return blocks


# ---------------------------------------------------------------------------
# Parsers
# ---------------------------------------------------------------------------

def parse_pidstat_ur(path, nproc=1):
    """Parse `pidstat -p ALL -u -r <interval>` — CPU and RSS per process group.

    With -u and -r combined, pidstat outputs two separate block types each
    interval, distinguished by their header lines:
      CPU block header contains: %usr
      Memory block header contains: minflt/s (or RSS)

    CPU columns:  Time [AM/PM] UID PID %usr %system %guest %wait %CPU CPU Command
    Mem columns:  Time [AM/PM] UID PID minflt/s majflt/s VSZ RSS %MEM Command

    %CPU is already normalised to a single-CPU basis (i.e. can exceed 100% on
    multi-core).  We divide by nproc to express as % of total machine capacity,
    matching the old proc-cpu behaviour.
    RSS is in KB; we convert to MB.
    """
    ALL_GROUPS = ["server", "client", "http_echo", "ws_echo", "grpc_echo", "k6", "frps", "frpc", "other"]
    groups = {g: {"cpu": [], "rss": []} for g in ALL_GROUPS}

    if not os.path.exists(path):
        return groups

    with open(path) as f:
        blocks = _split_blocks(f.readlines())

    cpu_epoch = None
    mem_epoch = None

    for block in blocks:
        # Identify block type from header
        is_cpu_block = any("%usr" in l for l in block)
        is_mem_block = any("minflt" in l or ("RSS" in l and "VSZ" in l) for l in block)
        if not is_cpu_block and not is_mem_block:
            continue

        # Determine column indices dynamically from header line
        col_map = {}
        if is_cpu_block:
            for line in block:
                if "%usr" in line:
                    parts = line.split()
                    for i, p in enumerate(parts):
                        if p == "%CPU":
                            col_map["cpu"] = i
                        elif p == "Command":
                            col_map["cmd"] = i
                    break
        else:
            for line in block:
                if "minflt" in line or ("RSS" in line and "VSZ" in line):
                    parts = line.split()
                    for i, p in enumerate(parts):
                        if p == "RSS":
                            col_map["rss"] = i
                        elif p == "Command":
                            col_map["cmd"] = i
                    break

        if not col_map:
            continue

        other_by_t: dict = {}

        for line in block:
            if "UID" in line or line.startswith("#") or \
               line.startswith("Linux") or line.startswith("Average") or \
               "%usr" in line or "minflt" in line or ("RSS" in line and "VSZ" in line):
                continue
            parts = line.split()
            if len(parts) < 5:
                continue
            ts = _ts_from_parts(parts)
            if ts is None:
                continue
            cmd_idx = col_map.get("cmd")
            if cmd_idx is None or cmd_idx >= len(parts):
                continue
            cmd = parts[cmd_idx]
            group = _proc_group(cmd)

            if is_cpu_block:
                if cpu_epoch is None:
                    cpu_epoch = ts
                t = _to_relative(ts, cpu_epoch)
                cpu_idx = col_map.get("cpu")
                if cpu_idx is None or cpu_idx >= len(parts):
                    continue
                try:
                    cpu_pct = float(parts[cpu_idx]) / nproc
                except ValueError:
                    continue
                if cpu_pct <= 0:
                    continue
                if group == "other":
                    other_by_t[t] = other_by_t.get(t, 0.0) + cpu_pct
                else:
                    groups[group]["cpu"].append({"t": t, "v": round(cpu_pct, 2)})
            else:
                if mem_epoch is None:
                    mem_epoch = ts
                t = _to_relative(ts, mem_epoch)
                rss_idx = col_map.get("rss")
                if rss_idx is None or rss_idx >= len(parts):
                    continue
                try:
                    rss_mb = int(parts[rss_idx]) / 1024.0
                except ValueError:
                    continue
                if rss_mb <= 0:
                    continue
                if group != "other":
                    # Keep max RSS per group per timestamp
                    existing = groups[group]["rss"]
                    if existing and existing[-1]["t"] == t:
                        existing[-1]["v"] = round(max(existing[-1]["v"], rss_mb), 1)
                    else:
                        groups[group]["rss"].append({"t": t, "v": round(rss_mb, 1)})

        if is_cpu_block:
            for t, v in sorted(other_by_t.items()):
                groups["other"]["cpu"].append({"t": t, "v": round(v, 2)})

    return groups


def parse_pidstat_io(path):
    """Parse `pidstat -p ALL -d 2` — disk read/write KB/s per process group."""
    groups = {g: {"read_kbs": [], "write_kbs": []}
              for g in ["server", "client", "http_echo", "ws_echo", "grpc_echo", "k6", "frps", "frpc", "other"]}

    if not os.path.exists(path):
        return groups

    with open(path) as f:
        blocks = _split_blocks(f.readlines())

    epoch = None
    other_read_by_t: dict = {}
    other_write_by_t: dict = {}

    for block in blocks:
        if not any("kB_rd" in l or "kB_wr" in l for l in block):
            continue
        for line in block:
            if "UID" in line or line.startswith("#") or \
               line.startswith("Linux") or line.startswith("Average"):
                continue
            parts = line.split()
            if len(parts) < 6:
                continue
            cmd = parts[-1]
            ts = _ts_from_parts(parts)
            if ts is None:
                continue
            if epoch is None:
                epoch = ts
            t = _to_relative(ts, epoch)
            group = _proc_group(cmd)
            try:
                # layout: Time [AM/PM] UID PID kB_rd/s kB_wr/s kB_ccwr/s iodelay Command
                read_kb = float(parts[-5])
                write_kb = float(parts[-4])
            except (ValueError, IndexError):
                read_kb = write_kb = 0.0

            if group == "other":
                other_read_by_t[t] = other_read_by_t.get(t, 0.0) + read_kb
                other_write_by_t[t] = other_write_by_t.get(t, 0.0) + write_kb
            else:
                groups[group]["read_kbs"].append({"t": t, "v": round(read_kb, 1)})
                groups[group]["write_kbs"].append({"t": t, "v": round(write_kb, 1)})

    for t, v in sorted(other_read_by_t.items()):
        groups["other"]["read_kbs"].append({"t": t, "v": round(v, 1)})
    for t, v in sorted(other_write_by_t.items()):
        groups["other"]["write_kbs"].append({"t": t, "v": round(v, 1)})

    return groups


def parse_pidstat_ctxsw(path):
    """Parse `pidstat -p ALL -w 2` — voluntary/involuntary context switches.

    Returns per-group {cswch: [{t,v}], nvcswch: [{t,v}]}.
    'other' is summed across all unrecognised processes.
    """
    groups = {g: {"cswch": [], "nvcswch": []}
              for g in ["server", "client", "http_echo", "ws_echo", "grpc_echo", "k6", "frps", "frpc", "other"]}

    if not os.path.exists(path):
        return groups

    with open(path) as f:
        blocks = _split_blocks(f.readlines())

    epoch = None

    for block in blocks:
        if not any("cswch" in l for l in block):
            continue

        other_cs_by_t: dict = {}
        other_nvcs_by_t: dict = {}

        for line in block:
            if "UID" in line or line.startswith("#") or \
               line.startswith("Linux") or line.startswith("Average"):
                continue
            parts = line.split()
            if len(parts) < 5:
                continue
            cmd = parts[-1]
            ts = _ts_from_parts(parts)
            if ts is None:
                continue
            if epoch is None:
                epoch = ts
            t = _to_relative(ts, epoch)
            group = _proc_group(cmd)
            try:
                # layout: Time [AM/PM] UID PID cswch/s nvcswch/s Command
                cswch = float(parts[-3])
                nvcswch = float(parts[-2])
            except (ValueError, IndexError):
                cswch = nvcswch = 0.0

            if group == "other":
                other_cs_by_t[t] = other_cs_by_t.get(t, 0.0) + cswch
                other_nvcs_by_t[t] = other_nvcs_by_t.get(t, 0.0) + nvcswch
            else:
                groups[group]["cswch"].append({"t": t, "v": round(cswch, 1)})
                groups[group]["nvcswch"].append({"t": t, "v": round(nvcswch, 1)})

        for t, v in sorted(other_cs_by_t.items()):
            groups["other"]["cswch"].append({"t": t, "v": round(v, 1)})
        for t, v in sorted(other_nvcs_by_t.items()):
            groups["other"]["nvcswch"].append({"t": t, "v": round(v, 1)})

    return groups


def parse_mpstat(path):
    """Parse `mpstat 2` — whole-system CPU utilisation with per-component breakdown.

    Returns a dict of lists: cpu (total used), cpu_usr, cpu_sys, cpu_irq,
    cpu_soft, cpu_iowait, cpu_steal.  Column positions are determined
    dynamically from the header line so sysstat version differences don't matter.
    """
    result = {k: [] for k in ("cpu", "cpu_usr", "cpu_sys", "cpu_irq",
                               "cpu_soft", "cpu_iowait", "cpu_steal")}
    if not os.path.exists(path):
        return result

    # header columns we care about (mpstat uses %xxx names)
    WANT = {"%usr": "cpu_usr", "%sys": "cpu_sys", "%irq": "cpu_irq",
            "%soft": "cpu_soft", "%iowait": "cpu_iowait", "%steal": "cpu_steal",
            "%idle": "_idle"}
    col_map = {}   # index → result key (or "_idle")
    epoch = None

    with open(path) as f:
        for line in f:
            s = line.strip()
            if not s or s.startswith("Linux") or s.startswith("Average"):
                continue
            parts = s.split()
            # header line contains "%usr" (or similar) — re-parse column indices each time
            if any(p in WANT for p in parts):
                col_map = {}
                for i, p in enumerate(parts):
                    if p in WANT:
                        col_map[i] = WANT[p]
                continue
            if not col_map:
                continue
            # data line — must contain "all" in position 1 or 2
            if not (len(parts) > 1 and (parts[1] == "all" or
                    (len(parts) > 2 and parts[2] == "all"))):
                continue
            ts = _ts_from_parts(parts)
            if ts is None:
                continue
            if epoch is None:
                epoch = ts
            t = _to_relative(ts, epoch)
            idle = None
            for idx, key in col_map.items():
                if idx >= len(parts):
                    continue
                try:
                    v = float(parts[idx])
                except ValueError:
                    continue
                if key == "_idle":
                    idle = v
                else:
                    result[key].append({"t": t, "v": round(v, 1)})
            if idle is not None:
                result["cpu"].append({"t": t, "v": round(100.0 - idle, 1)})

    return result


def parse_machine_info(path):
    """Read /tmp/machine-info: '<nproc> <mem_total_kb>'."""
    try:
        with open(path) as f:
            parts = f.read().strip().split()
        return {
            "cpu_cores": int(parts[0]),
            "mem_total_mb": round(int(parts[1]) / 1024),
        }
    except Exception:
        return None


def parse_sar_net(path):
    """Parse `sar -n DEV 2` — network RX/TX KB/s."""
    net = {"rx_kbs": [], "tx_kbs": []}
    if not os.path.exists(path):
        return net

    epoch = None
    with open(path) as f:
        for line in f:
            s = line.strip()
            if not s or s.startswith("Linux") or "IFACE" in s or s.startswith("Average"):
                continue
            parts = s.split()
            if len(parts) < 6:
                continue
            iface = parts[1] if not parts[1][0].isdigit() else parts[2]
            if iface not in ("eth0", "ens5", "ens33"):
                continue
            ts = _ts_from_parts(parts)
            if ts is None:
                continue
            try:
                rx_kbs = float(parts[4]) if len(parts) > 6 else float(parts[3])
                tx_kbs = float(parts[5]) if len(parts) > 6 else float(parts[4])
            except (ValueError, IndexError):
                continue
            if epoch is None:
                epoch = ts
            t = _to_relative(ts, epoch)
            net["rx_kbs"].append({"t": t, "v": round(rx_kbs, 1)})
            net["tx_kbs"].append({"t": t, "v": round(tx_kbs, 1)})

    return net


def parse_sar_paging(path):
    """Parse `sar -B 2` — page fault rates (majflt/s, minflt/s, pgfree/s)."""
    paging = {"majflt": [], "minflt": [], "pgfree": []}
    if not os.path.exists(path):
        return paging

    epoch = None
    with open(path) as f:
        for line in f:
            s = line.strip()
            if not s or s.startswith("Linux") or "pgpgin" in s or s.startswith("Average"):
                continue
            parts = s.split()
            if len(parts) < 8:
                continue
            ts = _ts_from_parts(parts)
            if ts is None:
                continue
            try:
                # sar -B layout: Time pgpgin/s pgpgout/s fault/s majflt/s pgfree/s ...
                # column indices vary; use header-based approach fallback to position
                majflt = float(parts[5])
                minflt = float(parts[4])   # fault/s (includes minor)
                pgfree = float(parts[6])
            except (ValueError, IndexError):
                continue
            if epoch is None:
                epoch = ts
            t = _to_relative(ts, epoch)
            paging["majflt"].append({"t": t, "v": round(majflt, 2)})
            paging["minflt"].append({"t": t, "v": round(minflt, 1)})
            paging["pgfree"].append({"t": t, "v": round(pgfree, 1)})

    return paging


def parse_ss_timeseries(path, sampling_epoch=None):
    """Parse our custom `ss -s` snapshot loop log.

    Each line: <unix_epoch> <ss -s output joined with |>
    Extracts: estab, timewait, closed TCP counts over time.
    """
    tcp = {"estab": [], "timewait": [], "closed": []}
    if not os.path.exists(path):
        return tcp

    epoch = None

    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or not line[0].isdigit():
                continue
            parts = line.split(" ", 1)
            if len(parts) < 2:
                continue
            try:
                ts_epoch = int(parts[0])
            except ValueError:
                continue

            if epoch is None:
                epoch = ts_epoch
            t = round(ts_epoch - epoch, 1)

            text = parts[1]
            # ss -s output looks like: "TCP:   42 (estab 10, closed 2, ...)"
            estab = timewait = closed = 0
            m = re.search(r'estab\s+(\d+)', text)
            if m:
                estab = int(m.group(1))
            m = re.search(r'timewait\s+(\d+)', text)
            if m:
                timewait = int(m.group(1))
            m = re.search(r'closed\s+(\d+)', text)
            if m:
                closed = int(m.group(1))

            tcp["estab"].append({"t": t, "v": estab})
            tcp["timewait"].append({"t": t, "v": timewait})
            tcp["closed"].append({"t": t, "v": closed})

    return tcp


def parse_psi(path, sampling_epoch=None):
    """Parse our PSI snapshot loop log.

    Each line: <unix_epoch> cpu=<pct> mem=<pct> io=<pct>
    Values are the 'total' stall time percentage over the last window.
    """
    psi = {"cpu": [], "mem": [], "io": []}
    if not os.path.exists(path):
        return psi

    first_line = ""
    with open(path) as f:
        first_line = f.readline().strip()
    if "not available" in first_line:
        return psi

    epoch = None
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or not line[0].isdigit():
                continue
            parts = line.split()
            if len(parts) < 4:
                continue
            try:
                ts_epoch = int(parts[0])
            except ValueError:
                continue

            if epoch is None:
                epoch = ts_epoch
            t = round(ts_epoch - epoch, 1)

            vals = {}
            for tok in parts[1:]:
                if "=" in tok:
                    k, v = tok.split("=", 1)
                    try:
                        vals[k] = float(v)
                    except ValueError:
                        pass

            psi["cpu"].append({"t": t, "v": vals.get("cpu", 0.0)})
            psi["mem"].append({"t": t, "v": vals.get("mem", 0.0)})
            psi["io"].append({"t": t, "v": vals.get("io", 0.0)})

    return psi


# ---------------------------------------------------------------------------
# collect-resources.py JSON Lines parser
# ---------------------------------------------------------------------------

def parse_collect_jsonl(path):
    """Parse output of collect-resources.py (one JSON object per line).

    Each line: {"t": <float>, "sys": {...}, "procs": {<group>: {...}},
                "top_cpu": [...], "top_rss": [...]}

    Returns (proc_groups, mpstat_data, top_cpu_series, top_rss_series).

    top_cpu_series / top_rss_series: dict of name -> [{t, v}] for the top-10
    processes seen across all samples (union of names that ever appeared in top10).
    """
    ALL_GROUPS = ["server", "client", "http_echo", "ws_echo", "grpc_echo",
                  "k6", "frps", "frpc", "other"]
    proc_groups = {g: {"cpu": [], "rss": [], "vms": [], "read_kbs": [], "write_kbs": [],
                       "cswch": [], "nvcswch": []} for g in ALL_GROUPS}
    sys_keys = ("cpu", "cpu_usr", "cpu_sys", "cpu_irq", "cpu_soft", "cpu_iowait", "cpu_steal",
                "loadavg_1", "loadavg_5", "loadavg_15",
                "ctx_switches", "interrupts",
                "mem_pct", "mem_used_mb", "swap_used_mb",
                "disk_read_kbs", "disk_write_kbs", "disk_read_iops", "disk_write_iops",
                "net_rx_kbs", "net_tx_kbs", "net_rx_pkts", "net_tx_pkts",
                "net_drop_in", "net_drop_out", "net_err_in", "net_err_out",
                "tcp_estab", "tcp_timewait",
                "udp_rx_err", "udp_buf_err")
    mpstat_data = {k: [] for k in sys_keys}
    # cpu_per_core: list of series, one per core index
    cpu_per_core_series: list = []  # will be set from first sample

    # top-10 series: name -> {t -> value}  (last seen value per name per t)
    top_cpu_by_name: dict = {}
    top_rss_by_name: dict = {}

    if not os.path.exists(path):
        return proc_groups, mpstat_data, {}, {}

    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = row.get("t")
            if t is None:
                continue

            # system scalar metrics
            sys_d = row.get("sys") or {}
            for k in sys_keys:
                v = sys_d.get(k)
                if v is not None:
                    mpstat_data[k].append({"t": t, "v": v})

            # per-core CPU (variable length list)
            cores = sys_d.get("cpu_per_core")
            if isinstance(cores, list):
                # extend series list if more cores than seen before
                while len(cpu_per_core_series) < len(cores):
                    cpu_per_core_series.append([])
                for i, v in enumerate(cores):
                    cpu_per_core_series[i].append({"t": t, "v": v})

            # per-process groups
            procs_d = row.get("procs") or {}
            for g in ALL_GROUPS:
                gd = procs_d.get(g)
                if not gd:
                    continue
                for field in ("cpu", "rss", "vms", "read_kbs", "write_kbs", "cswch", "nvcswch", "fds"):
                    v = gd.get(field)
                    if v is not None and v != 0:
                        proc_groups[g][field].append({"t": t, "v": v})

            # top-10 CPU
            for entry in (row.get("top_cpu") or []):
                name = entry.get("name") or f"pid{entry.get('pid','?')}"
                v = entry.get("cpu", 0)
                if name not in top_cpu_by_name:
                    top_cpu_by_name[name] = []
                top_cpu_by_name[name].append({"t": t, "v": round(v, 2)})

            # top-10 RSS
            for entry in (row.get("top_rss") or []):
                name = entry.get("name") or f"pid{entry.get('pid','?')}"
                v = entry.get("rss", 0)
                if name not in top_rss_by_name:
                    top_rss_by_name[name] = []
                top_rss_by_name[name].append({"t": t, "v": round(v, 1)})

    if cpu_per_core_series:
        mpstat_data["cpu_per_core"] = cpu_per_core_series

    return proc_groups, mpstat_data, top_cpu_by_name, top_rss_by_name


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    p = argparse.ArgumentParser()
    # --collect supersedes all individual tool outputs
    p.add_argument("--collect",       default="")   # collect-resources.py jsonl
    # legacy individual-tool inputs (used when --collect not provided)
    p.add_argument("--pidstat-ur",    default="")
    p.add_argument("--proc-cpu",      default="")
    p.add_argument("--mpstat",        default="")
    p.add_argument("--pidstat-io",    default="")
    p.add_argument("--pidstat-ctxsw", default="")
    p.add_argument("--sar-net",       default="")
    p.add_argument("--sar-paging",    default="")
    p.add_argument("--ss",            default="")
    p.add_argument("--psi",           default="")
    p.add_argument("--k6-offset",     type=int, default=0)
    p.add_argument("--nproc",         default="/tmp/nproc")
    p.add_argument("--machine-info",  default="")
    p.add_argument("--output",        required=True)
    args = p.parse_args()

    nproc = _read_nproc(args.nproc)

    top_cpu_series: dict = {}
    top_rss_series: dict = {}

    if args.collect and os.path.exists(args.collect):
        # New unified path: one jsonl file replaces mpstat + pidstat-ur + pidstat-ctxsw + pidstat-io
        proc_groups, mpstat_data, top_cpu_series, top_rss_series = parse_collect_jsonl(args.collect)
        io_groups    = {}
        ctxsw_groups = {}
    else:
        # Legacy path
        if args.pidstat_ur and os.path.exists(args.pidstat_ur):
            proc_groups = parse_pidstat_ur(args.pidstat_ur, nproc)
        elif args.proc_cpu and os.path.exists(args.proc_cpu):
            proc_groups = parse_proc_cpu(args.proc_cpu, nproc)
        else:
            proc_groups = {g: {"cpu": [], "rss": []}
                           for g in ["server","client","http_echo","ws_echo",
                                     "grpc_echo","k6","frps","frpc","other"]}
        mpstat_data  = parse_mpstat(args.mpstat) if args.mpstat else \
                       {k: [] for k in ("cpu","cpu_usr","cpu_sys","cpu_irq","cpu_soft","cpu_iowait","cpu_steal")}
        io_groups    = parse_pidstat_io(args.pidstat_io)       if args.pidstat_io   else {}
        ctxsw_groups = parse_pidstat_ctxsw(args.pidstat_ctxsw) if args.pidstat_ctxsw else {}

    net    = parse_sar_net(args.sar_net)     if args.sar_net    else {}
    paging = parse_sar_paging(args.sar_paging) if args.sar_paging else {}
    tcp    = parse_ss_timeseries(args.ss)    if args.ss         else {}
    psi    = parse_psi(args.psi)             if args.psi        else {}

    # Merge per-process fields; only emit groups with data
    processes = {}
    for g in ["server", "client", "http_echo", "ws_echo", "grpc_echo", "k6", "frps", "frpc", "other"]:
        entry = {**proc_groups.get(g, {})}
        for src in (io_groups.get(g, {}), ctxsw_groups.get(g, {})):
            for k, v in src.items():
                if v:
                    entry[k] = v
        if any(v for v in entry.values()):
            processes[g] = entry

    system = {k: v for k, v in mpstat_data.items() if v}

    result = {
        "processes": processes,
        "system":    system,
        "nproc":     nproc,
        "k6OffsetSeconds": args.k6_offset,
    }
    machine = parse_machine_info(args.machine_info) if args.machine_info else None
    if machine:
        result["machine"] = machine
    if net and (net.get("rx_kbs") or net.get("tx_kbs")):
        result["network"] = net
    if paging and any(paging.values()):
        result["paging"] = paging
    if tcp and any(tcp.values()):
        result["tcp_conns"] = tcp
    if psi and any(psi.values()):
        result["psi"] = psi
    if top_cpu_series:
        result["top_cpu"] = top_cpu_series
    if top_rss_series:
        result["top_rss"] = top_rss_series

    with open(args.output, "w") as f:
        json.dump(result, f, indent=2)

    counts = {g: len(processes.get(g, {}).get("cpu", [])) for g in processes}
    print(
        f"nproc={nproc}, system_cpu={len(mpstat_data.get('cpu',[]))}, processes={counts}, "
        f"tcp_estab={len(tcp.get('estab',[]))}, psi_pts={len(psi.get('cpu',[]))}, "
        f"paging_pts={len(paging.get('majflt',[]))}"
    )


if __name__ == "__main__":
    main()
