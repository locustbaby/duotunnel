import json
import os
import sys
from copy import deepcopy

def r2(v):
    return round(float(v), 2)

def _max_t(res):
    pts = []
    for v in (res.get("system") or {}).values():
        if isinstance(v, list) and v and not isinstance(v[0], list):
            pts.extend(p[0] for p in v if isinstance(p, list) and len(p) >= 2)
        elif isinstance(v, list) and v and isinstance(v[0], list):
            for sub in v:
                pts.extend(p[0] for p in sub if isinstance(p, list) and len(p) >= 2)
    for proc in (res.get("processes") or {}).values():
        for series in proc.values():
            if isinstance(series, list):
                pts.extend(p[0] for p in series if isinstance(p, list) and len(p) >= 2)
    return max(pts) if pts else 0

def append_resource_to_core(core_res, extra_res, gap=10):
    base = _max_t(core_res) + gap

    def shift(series):
        if not series:
            return series
        if isinstance(series[0], list) and series[0] and isinstance(series[0][0], list):
            return [[p[0]+base, p[1]] for sub in series for p in sub]
        return [[p[0]+base, p[1]] for p in series if isinstance(p, list) and len(p) >= 2]

    sys_src = extra_res.get("system") or {}
    sys_dst = core_res.setdefault("system", {})
    for k, v in sys_src.items():
        if k == "cpu_per_core" and isinstance(v, list):
            dst_cores = sys_dst.setdefault("cpu_per_core", [])
            for i, sub in enumerate(v):
                while len(dst_cores) <= i:
                    dst_cores.append([])
                dst_cores[i].extend([[p[0]+base, p[1]] for p in sub if isinstance(p, list) and len(p) >= 2])
        elif isinstance(v, list) and v and isinstance(v[0], list):
            dst = sys_dst.setdefault(k, [])
            dst.extend([[p[0]+base, p[1]] for p in v if isinstance(p, list) and len(p) >= 2])

    proc_src = extra_res.get("processes") or {}
    proc_dst = core_res.setdefault("processes", {})
    for grp, metrics in proc_src.items():
        dst_grp = proc_dst.setdefault(grp, {})
        for k, v in metrics.items():
            if isinstance(v, list) and v and isinstance(v[0], list):
                dst_grp.setdefault(k, []).extend([[p[0]+base, p[1]] for p in v if isinstance(p, list) and len(p) >= 2])

    return base

def merge_extra(merged, sc_by_name, extra_path, label, include_in_total_rps, resource_path=""):
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

    SKIP_RPC = {"catalog", "nproc", "k6OffsetSeconds"}
    res_offset = None

    def _get_target_res():
        rpc = merged.get("resources_per_case") or {}
        key = next((k for k in rpc if k not in SKIP_RPC), None)
        return rpc.get(key) if key else None

    if resource_path and os.path.isfile(resource_path):
        with open(resource_path, "r", encoding="utf-8") as f:
            extra_res = json.load(f)
        target = _get_target_res()
        if target is not None:
            res_offset = append_resource_to_core(target, extra_res)
        else:
            merged.setdefault("resources_per_case", {})["core"] = extra_res

    extra_rpc = extra.get("resources_per_case") or {}
    extra_rpc_vals = [v for k, v in extra_rpc.items() if k not in SKIP_RPC and isinstance(v, dict)]
    if extra_rpc_vals:
        target = _get_target_res()
        if target is not None:
            base = _max_t(target) + 10
            for extra_res in extra_rpc_vals:
                append_resource_to_core(target, extra_res)
            res_offset = base
        else:
            merged.setdefault("resources_per_case", {})["core"] = extra_rpc_vals[0]
            target = merged["resources_per_case"]["core"]
            res_offset = 0
            for extra_res in extra_rpc_vals[1:]:
                append_resource_to_core(target, extra_res)

    for p in extra.get("phases", []):
        if not isinstance(p, dict):
            continue
        np = deepcopy(p)
        phase_name = str(np.get("name", ""))
        np["name"] = f"{phase_name} ({label})" if phase_name else label
        if isinstance(np.get("scenarios"), list):
            np["scenarios"] = [name_map.get(n, n) for n in np.get("scenarios", [])]
        np["resourceCase"] = "core"
        if res_offset is not None:
            np["start"] = (np.get("start") or 0) + res_offset
            np["end"]   = (np.get("end")   or 0) + res_offset
        merged["phases"].append(np)

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
    import argparse as _ap
    parser = _ap.ArgumentParser(add_help=False)
    parser.add_argument("core_path")
    parser.add_argument("out_path")
    parser.add_argument("extras", nargs="*")
    parser.add_argument("--core-resource", default="")
    parser.add_argument("--core-label", default="ingress_8000qps")
    known, unknown = parser.parse_known_args()

    core_path = known.core_path
    out_path = known.out_path
    extras = known.extras + unknown

    with open(core_path, "r", encoding="utf-8") as f:
        core = json.load(f)

    merged = deepcopy(core)
    if known.core_resource and os.path.isfile(known.core_resource):
        with open(known.core_resource, "r", encoding="utf-8") as f:
            res_data = json.load(f)
        merged.setdefault("resources_per_case", {})[known.core_label] = res_data
        for p in merged.get("phases") or []:
            if isinstance(p, dict) and "resourceCase" not in p:
                p["resourceCase"] = known.core_label
    if merged.get("scenarios") is None:
        merged["scenarios"] = []
    if merged.get("phases") is None:
        merged["phases"] = []

    sc_by_name = {}
    for s in merged["scenarios"]:
        if isinstance(s, dict) and "name" in s:
            sc_by_name[s["name"]] = s

    for extra_arg in extras:
        # Format: path:label:includeInTotalRps[:resourcePath]
        parts = extra_arg.split(":")
        if len(parts) < 3:
            continue
        res_path = parts[3] if len(parts) >= 4 else ""
        merge_extra(merged, sc_by_name, parts[0], parts[1], parts[2].lower() == "true", res_path)

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
