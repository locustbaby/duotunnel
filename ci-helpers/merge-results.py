import json
import os
import sys


def r2(v):
    return round(float(v), 2)


def merge_phases(merged, extra_path):
    if not extra_path or not os.path.isfile(extra_path):
        return
    with open(extra_path, "r", encoding="utf-8") as f:
        extra = json.load(f)

    merged_phases = merged.setdefault("phases", {})
    for phase_name, phase in (extra.get("phases") or {}).items():
        key = phase_name
        i = 2
        while key in merged_phases:
            key = f"{phase_name} ({i})"
            i += 1
        merged_phases[key] = phase

    core_cats = {c["id"]: c for c in (merged.get("catalog") or {}).get("categories") or []}
    for c in (extra.get("catalog") or {}).get("categories") or []:
        if c.get("id") and c["id"] not in core_cats:
            merged.setdefault("catalog", {}).setdefault("categories", []).append(c)


def recalc_summary(merged):
    total_rps = 0.0
    total_requests = 0.0
    total_errors = 0.0
    for phase in (merged.get("phases") or {}).values():
        for case in (phase.get("cases") or {}).values():
            perf = case.get("perf") or {}
            req = float(perf.get("requests") or 0)
            err = float(perf.get("err") or 0)
            rps = float(perf.get("rps") or 0)
            total_requests += req
            total_errors += req * err / 100.0
            if perf.get("includeInTotalRps"):
                total_rps += rps
    total_err = 0.0 if total_requests <= 0 else (total_errors / total_requests * 100.0)
    merged["summary"] = {
        "totalRPS": r2(total_rps),
        "totalErr": r2(total_err),
        "totalRequests": int(round(total_requests)),
    }


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("core_path")
    parser.add_argument("out_path")
    parser.add_argument("extras", nargs="*")
    args = parser.parse_args()

    with open(args.core_path, "r", encoding="utf-8") as f:
        merged = json.load(f)

    for extra_path in args.extras:
        merge_phases(merged, extra_path)

    recalc_summary(merged)

    with open(args.out_path, "w", encoding="utf-8") as f:
        json.dump(merged, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
