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

def parse_proc_cpu(path, nproc=1):
    """Parse /proc/PID/stat snapshot log — CPU and RSS per process group.

    Log format:
        Line 1: CLK_TCK=<value>
        Then repeating blocks separated by '---':
            <timestamp> <pid> <comm> <utime> <stime> <rss_kb>

    CPU% = (delta_utime + delta_stime) / (delta_t * CLK_TCK) * 100 / nproc
    RSS from VmRSS in kB, converted to MB.
    """
    ALL_GROUPS = ["server", "client", "http_echo", "ws_echo", "grpc_echo", "k6", "frps", "frpc", "other"]
    groups = {g: {"cpu": [], "rss": []} for g in ALL_GROUPS}

    if not os.path.exists(path):
        return groups

    clk_tck = 100
    snapshots = []
    current = []

    with open(path) as f:
        for line in f:
            s = line.strip()
            if not s:
                continue
            if s.startswith("CLK_TCK="):
                try:
                    clk_tck = int(s.split("=", 1)[1])
                except ValueError:
                    pass
                continue
            if s == "---":
                if current:
                    snapshots.append(current)
                    current = []
                continue
            current.append(s)
    if current:
        snapshots.append(current)

    if len(snapshots) < 2:
        return groups

    def parse_snap(lines):
        ts = None
        procs = {}
        for line in lines:
            parts = line.split()
            if len(parts) < 6:
                continue
            try:
                t = float(parts[0])
                pid = parts[1]
                comm = parts[2]
                utime = int(parts[3])
                stime = int(parts[4])
                rss_kb = int(parts[5])
            except (ValueError, IndexError):
                continue
            if ts is None:
                ts = t
            procs[pid] = (comm, utime, stime, rss_kb)
        return ts, procs

    prev_ts, prev_procs = parse_snap(snapshots[0])
    first_ts = prev_ts

    for snap in snapshots[1:]:
        cur_ts, cur_procs = parse_snap(snap)
        if cur_ts is None or prev_ts is None:
            prev_ts, prev_procs = cur_ts, cur_procs
            continue
        dt = cur_ts - prev_ts
        if dt <= 0:
            prev_ts, prev_procs = cur_ts, cur_procs
            continue

        t_rel = round(cur_ts - first_ts, 1)
        group_cpu = {g: 0.0 for g in ALL_GROUPS}
        group_rss = {g: 0.0 for g in ALL_GROUPS}

        for pid, (comm, utime, stime, rss_kb) in cur_procs.items():
            group = _proc_group(comm)
            if pid in prev_procs:
                _, p_utime, p_stime, _ = prev_procs[pid]
                dticks = (utime - p_utime) + (stime - p_stime)
                if dticks > 0:
                    cpu_pct = dticks / (dt * clk_tck) * 100.0 / nproc
                    group_cpu[group] += cpu_pct
            rss_mb = rss_kb / 1024.0
            group_rss[group] = max(group_rss[group], rss_mb)

        for g in ALL_GROUPS:
            if group_cpu[g] > 0:
                groups[g]["cpu"].append({"t": t_rel, "v": round(group_cpu[g], 2)})
            if group_rss[g] > 0:
                groups[g]["rss"].append({"t": t_rel, "v": round(group_rss[g], 1)})

        prev_ts, prev_procs = cur_ts, cur_procs

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
# Main
# ---------------------------------------------------------------------------

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--proc-cpu",      required=True)
    p.add_argument("--mpstat",        required=True)
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

    nproc        = _read_nproc(args.nproc)
    proc_groups  = parse_proc_cpu(args.proc_cpu, nproc)
    mpstat_data  = parse_mpstat(args.mpstat)
    io_groups    = parse_pidstat_io(args.pidstat_io)       if args.pidstat_io   else {}
    ctxsw_groups = parse_pidstat_ctxsw(args.pidstat_ctxsw) if args.pidstat_ctxsw else {}
    net          = parse_sar_net(args.sar_net)             if args.sar_net      else {}
    paging       = parse_sar_paging(args.sar_paging)       if args.sar_paging   else {}
    tcp          = parse_ss_timeseries(args.ss)            if args.ss           else {}
    psi          = parse_psi(args.psi)                     if args.psi          else {}

    # Merge CPU/RSS + IO + ctxsw into per-process entries; only emit groups with data
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
