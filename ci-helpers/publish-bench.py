#!/usr/bin/env python3
import argparse
import json
import os

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--result", required=True)
    p.add_argument("--data", required=True)
    p.add_argument("--sha", required=True)
    p.add_argument("--msg", default="")
    p.add_argument("--url", default="")
    p.add_argument("--run-url", default="")
    p.add_argument("--resources", default="")
    p.add_argument("--frp-result", default="")
    p.add_argument("--frp-k6-offset", type=int, default=0)
    p.add_argument("--trace-server", default="")
    p.add_argument("--trace-client", default="")
    p.add_argument("--trace-artifact-url", default="")
    # Per-case traces: path to JSON file with list of {"case": "...", "server": "url", "client": "url"}
    p.add_argument("--trace-cases-file", default="")
    p.add_argument("--max-entries", type=int, default=50)
    args = p.parse_args()

    with open(args.result) as f:
        entry = json.load(f)

    entry["commit"] = {"id": args.sha, "message": args.msg, "url": args.url}
    if args.run_url:
        entry["run_url"] = args.run_url

    if args.resources and os.path.exists(args.resources):
        with open(args.resources) as f:
            entry.setdefault("resources_per_case", {})["core"] = json.load(f)
    if args.trace_server:
        entry.setdefault("artifacts", {})["trace_server"] = args.trace_server
    if args.trace_client:
        entry.setdefault("artifacts", {})["trace_client"] = args.trace_client
    if args.trace_artifact_url:
        entry.setdefault("artifacts", {})["trace_artifact_url"] = args.trace_artifact_url
    if args.trace_cases_file and os.path.exists(args.trace_cases_file):
        with open(args.trace_cases_file) as f:
            cases_list = json.load(f)
        if isinstance(cases_list, list) and cases_list:
            entry.setdefault("artifacts", {})["trace_cases"] = cases_list

    if args.frp_result and os.path.exists(args.frp_result):
        with open(args.frp_result) as f:
            frp = json.load(f)
        for sc in frp.get("scenarios", []):
            sc["tunnel"] = "frp"
            entry["scenarios"].append(sc)
        for ph in frp.get("phases", []):
            ph["tunnel"] = "frp"
            entry.setdefault("phases", []).append(ph)

    PREFIX = "window.BENCHMARK_DATA = "
    SUFFIX = ";"

    entries = []
    if os.path.exists(args.data):
        with open(args.data) as f:
            raw = f.read().strip()
        if raw.startswith(PREFIX):
            json_str = raw[len(PREFIX):]
            if json_str.endswith(SUFFIX):
                json_str = json_str[:-len(SUFFIX)]
            try:
                obj = json.loads(json_str)
                if isinstance(obj, dict):
                    entries = obj.get("entries", [])
                    if not isinstance(entries, list):
                        entries = []
            except json.JSONDecodeError:
                entries = []

    sha7 = args.sha[:7]
    bench_dir = os.path.dirname(os.path.abspath(args.data))
    detail_dir = os.path.join(bench_dir, "data")
    os.makedirs(detail_dir, exist_ok=True)
    detail_path = os.path.join(detail_dir, f"{sha7}.json")
    with open(detail_path, "w") as f:
        f.write(json.dumps(entry, separators=(",", ":")) + "\n")

    index_entry = {
        "commit": entry["commit"],
        "timestamp": entry.get("timestamp", ""),
        "summary": entry.get("summary"),
        "artifacts": entry.get("artifacts"),
        "run_url": entry.get("run_url", ""),
        "scenarios": [
            {k: s[k] for k in (
                "name", "category", "phase", "label", "payloadBytes",
                "timeRange", "thresholdSpec", "includeInTotalRps",
                "targetRate",
                "p50", "p95", "rps", "requests", "err",
                "protocol", "direction", "tunnel",
            ) if k in s}
            for s in entry.get("scenarios", [])
        ],
    }

    entries.append(index_entry)
    entries = entries[-args.max_entries:]

    with open(args.data, "w") as f:
        f.write(PREFIX + json.dumps({"entries": entries}, separators=(",", ":")) + SUFFIX + "\n")

    kept_shas = set()
    for e in entries:
        c = e.get("commit")
        sid = (c.get("id") if isinstance(c, dict) else c) or ""
        if sid:
            kept_shas.add(sid[:7])
    for name in os.listdir(detail_dir):
        stem, ext = os.path.splitext(name)
        if ext == ".json" and stem not in kept_shas:
            os.remove(os.path.join(detail_dir, name))

    traces_dir = os.path.join(bench_dir, "traces")
    if os.path.isdir(traces_dir):
        keep_traces = set()
        marker = "/traces/"
        for e in entries:
            arts = e.get("artifacts") or {}
            for key in ("trace_server", "trace_client"):
                pth = arts.get(key)
                if isinstance(pth, str) and marker in pth:
                    base = pth.split(marker, 1)[1]
                    # Could be a shard base or a plain filename; keep all variants
                    keep_traces.add(base)
                    keep_traces.add(base + "-meta.json")
                    keep_traces.add(base + "-events.json.gz")
                    keep_traces.add(base + "-cpu.json.gz")
            for tc in arts.get("trace_cases") or []:
                for key in ("server", "client"):
                    pth = tc.get(key) if isinstance(tc, dict) else None
                    if isinstance(pth, str) and marker in pth:
                        base = pth.split(marker, 1)[1]
                        # Keep all 3 shards for this base
                        keep_traces.add(base + "-meta.json")
                        keep_traces.add(base + "-events.json.gz")
                        keep_traces.add(base + "-cpu.json.gz")
        for name in os.listdir(traces_dir):
            fp = os.path.join(traces_dir, name)
            if os.path.isfile(fp) and name not in keep_traces:
                os.remove(fp)

    print(f"Published entry {sha7}, total entries: {len(entries)}")

if __name__ == "__main__":
    main()
