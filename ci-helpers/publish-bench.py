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
    p.add_argument("--resources", default="")
    p.add_argument("--max-entries", type=int, default=50)
    args = p.parse_args()

    with open(args.result) as f:
        entry = json.load(f)

    entry["commit"] = {"id": args.sha, "message": args.msg, "url": args.url}

    if args.resources and os.path.exists(args.resources):
        with open(args.resources) as f:
            entry["resources"] = json.load(f)

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

    entries.append(entry)
    entries = entries[-args.max_entries:]

    with open(args.data, "w") as f:
        f.write(PREFIX + json.dumps({"entries": entries}, indent=2) + SUFFIX + "\n")

    print(f"Published entry {args.sha[:7]}, total entries: {len(entries)}")

if __name__ == "__main__":
    main()
