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
    p.add_argument("--trace-artifact-url", default="")
    p.add_argument("--max-entries", type=int, default=50)
    args = p.parse_args()

    with open(args.result) as f:
        entry = json.load(f)

    entry["commit"] = {"id": args.sha, "message": args.msg, "url": args.url}
    if args.run_url:
        entry["run_url"] = args.run_url

    if args.resources and os.path.exists(args.resources):
        with open(args.resources) as f:
            entry["resources"] = json.load(f)
    if args.trace_artifact_url:
        entry.setdefault("artifacts", {})["trace_artifact_url"] = args.trace_artifact_url

    if args.frp_result and os.path.exists(args.frp_result):
        with open(args.frp_result) as f:
            frp = json.load(f)
        for sc in frp.get("scenarios", []):
            sc["tunnel"] = "frp"
            entry["scenarios"].append(sc)
        frp_offset = args.frp_k6_offset
        for ph in frp.get("phases", []):
            ph["tunnel"] = "frp"
            ph["start"] += frp_offset
            ph["end"] += frp_offset
            entry.setdefault("phases", []).append(ph)
    if isinstance(entry.get("phases"), list):
        entry["phases"] = sorted(
            entry["phases"],
            key=lambda p: (
                p.get("start", 0) if isinstance(p, dict) else 0,
                p.get("end", 0) if isinstance(p, dict) else 0,
                p.get("name", "") if isinstance(p, dict) else "",
            ),
        )

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
        f.write(json.dumps(entry, indent=2) + "\n")

    index_entry = {
        "commit": entry["commit"],
        "timestamp": entry.get("timestamp", ""),
        "summary": entry.get("summary"),
        "artifacts": entry.get("artifacts"),
        "run_url": entry.get("run_url", ""),
        "scenarios": [
            {k: s[k] for k in ("name", "category", "p50", "p95", "rps", "requests", "err", "protocol", "direction", "tunnel", "includeInTotalRps") if k in s}
            for s in entry.get("scenarios", [])
        ],
    }

    entries.append(index_entry)
    entries = entries[-args.max_entries:]

    with open(args.data, "w") as f:
        f.write(PREFIX + json.dumps({"entries": entries}, indent=2) + SUFFIX + "\n")

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

    print(f"Published entry {sha7}, total entries: {len(entries)}")

if __name__ == "__main__":
    main()
