import json
import os
import sys
from copy import deepcopy

def r2(v):
    return round(float(v), 2)

def merge_extra(merged, sc_by_name, extra_path, label, include_in_total_rps):
    if not extra_path or not os.path.isfile(extra_path):
        return
    with open(extra_path, "r", encoding="utf-8") as f:
        extra = json.load(f)
    if not extra:
        return

    name_map = {}
    for s in extra.get("scenarios", []):
        if not isinstance(s, dict):
            continue
        ns = deepcopy(s)
        old_name = str(ns.get("name", "") or "")
        if old_name:
            new_name = f"{old_name} ({label})"
            name_map[old_name] = new_name
            ns["name"] = new_name
        if not include_in_total_rps:
            ns["includeInTotalRps"] = False
        if "name" in ns:
            sc_by_name[ns["name"]] = ns

    # phases: keep original start/end (0-based per case), just rename
    for p in extra.get("phases", []):
        if not isinstance(p, dict):
            continue
        np = deepcopy(p)
        phase_name = str(np.get("name", ""))
        np["name"] = f"{phase_name} ({label})" if phase_name else label
        if isinstance(np.get("scenarios"), list):
            np["scenarios"] = [name_map.get(n, n) for n in np.get("scenarios", [])]
        merged["phases"].append(np)

    # carry per-case resource data
    for case_name, res_data in (extra.get("resources_per_case") or {}).items():
        merged.setdefault("resources_per_case", {})[f"{case_name} ({label})"] = res_data

    core_catalog = merged.get("catalog") or {}
    extra_catalog = extra.get("catalog") or {}
    core_case_order = list(core_catalog.get("caseOrder") or [])
    for n in extra_catalog.get("caseOrder") or []:
        nn = name_map.get(n, n)
        if nn not in core_case_order:
            core_case_order.append(nn)
    core_catalog["caseOrder"] = core_case_order

    categories = []
    seen_cat = set()
    for c in (core_catalog.get("categories") or []) + (extra_catalog.get("categories") or []):
        if not isinstance(c, dict):
            continue
        cid = c.get("id")
        if cid in seen_cat:
            continue
        seen_cat.add(cid)
        categories.append(c)
    core_catalog["categories"] = categories
    merged["catalog"] = core_catalog

def main():
    if len(sys.argv) < 3:
        print("Usage: merge-results.py <core_json> <out_json> [extra_json:label:includeInTotalRps ...]")
        sys.exit(1)

    core_path = sys.argv[1]
    out_path = sys.argv[2]
    extras = sys.argv[3:]

    with open(core_path, "r", encoding="utf-8") as f:
        core = json.load(f)

    merged = deepcopy(core)
    if merged.get("scenarios") is None:
        merged["scenarios"] = []
    if merged.get("phases") is None:
        merged["phases"] = []

    sc_by_name = {}
    for s in merged["scenarios"]:
        if isinstance(s, dict) and "name" in s:
            sc_by_name[s["name"]] = s

    for extra_arg in extras:
        # Format: path:label:includeInTotalRps
        parts = extra_arg.split(":")
        if len(parts) != 3:
            continue
        merge_extra(merged, sc_by_name, parts[0], parts[1], parts[2].lower() == "true")

    merged["scenarios"] = list(sc_by_name.values())

    total_requests = 0.0
    total_errors = 0.0
    total_rps = 0.0
    for s in merged["scenarios"]:
        req = float(s.get("requests", 0) or 0)
        err = float(s.get("err", 0) or 0)
        rps = float(s.get("rps", 0) or 0)
        total_requests += req
        total_errors += req * err / 100.0
        if bool(s.get("includeInTotalRps", False)):
            total_rps += rps

    total_err = 0.0 if total_requests <= 0 else (total_errors / total_requests * 100.0)
    merged["summary"] = {
        "totalRPS": r2(total_rps),
        "totalErr": r2(total_err),
        "totalRequests": int(round(total_requests)),
    }

    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(merged, f, ensure_ascii=False, indent=2)

if __name__ == "__main__":
    main()
