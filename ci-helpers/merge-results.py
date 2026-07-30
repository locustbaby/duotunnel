import json
import os
import sys


def r2(v):
    return round(float(v), 2)


def number(value):
    if isinstance(value, bool) or value is None:
        return None
    try:
        result = float(value)
    except (TypeError, ValueError):
        return None
    return result if result == result and result not in (float("inf"), float("-inf")) else None


def normalize_result(result):
    load = result.setdefault("load", {})
    if number(load.get("droppedIterations")) is None:
        load["droppedIterations"] = 0

    for case in (result.get("cases") or {}).values():
        perf = case.get("perf")
        if not isinstance(perf, dict):
            continue
        if number(perf.get("droppedIterations")) is None:
            perf["droppedIterations"] = 0
        if number(perf.get("completedIterations")) is None:
            perf["completedIterations"] = 0

    iterations = number(load.get("iterations"))
    dropped = number(load.get("droppedIterations"))
    if iterations is not None and dropped is not None:
        total = iterations + dropped
        load["droppedIterationRate"] = 0 if total <= 0 else round(dropped / total * 100, 2)


def merge_cases(merged, extra_path):
    if not extra_path or not os.path.isfile(extra_path):
        return
    with open(extra_path, "r", encoding="utf-8") as f:
        extra = json.load(f)

    normalize_result(extra)
    merged_load = merged.setdefault("load", {})
    extra_load = extra.get("load") or {}
    for field in ("iterations", "droppedIterations"):
        values = [merged_load.get(field), extra_load.get(field)]
        numeric = [number(value) for value in values]
        numeric = [value for value in numeric if value is not None]
        if numeric:
            merged_load[field] = int(sum(numeric))
    normalize_result(merged)

    merged_cases = merged.setdefault("cases", {})
    for case_name, case in (extra.get("cases") or {}).items():
        key = case_name
        i = 2
        while key in merged_cases:
            key = f"{case_name} ({i})"
            i += 1
        merged_cases[key] = case

    core_cats = {c["id"]: c for c in (merged.get("catalog") or {}).get("categories") or []}
    for c in (extra.get("catalog") or {}).get("categories") or []:
        if c.get("id") and c["id"] not in core_cats:
            merged.setdefault("catalog", {}).setdefault("categories", []).append(c)


def recalc_summary(merged):
    total_rps = 0.0
    total_requests = 0.0
    total_errors = 0.0
    for case in (merged.get("cases") or {}).values():
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

    normalize_result(merged)
    for extra_path in args.extras:
        merge_cases(merged, extra_path)

    recalc_summary(merged)

    with open(args.out_path, "w", encoding="utf-8") as f:
        json.dump(merged, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
