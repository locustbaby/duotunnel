# DuoTunnel Benchmark Specification

This document is the contract for CI benchmark jobs, k6 case catalogs, resource sampling, and GitHub Actions `workflow_dispatch` parameters. Implementation lives under `ci-helpers/` and `.github/workflows/ci.yml`.

## 1. Architecture Overview

Three tiers:

1. **Orchestration & load (k6)** — JSON case catalogs in `ci-helpers/k6/cases/*.json`, executed via `ci-helpers/k6/bench.js`.
2. **Collection & processing (`bench-tool.py`)** — psutil sampling, alignment with k6 windows, inject into per-case JSON.
3. **Visualization (Bench UI)** — static dashboard on gh-pages (`bench/index.html`, `data.js`, `data/[sha7].json`).

### 1.1 CI jobs (current)

| Job | Trigger (main push) | Cases |
|-----|---------------------|-------|
| `unit-tests` | always (path filter) | `cargo test`, clippy, audit |
| `integration-test` | always | `ci-test-client` protocol matrix |
| `bench-basic` | main push | k6 profiles `basic` + `body_size` |
| `bench-3k` | main push | `ingress_3000qps`, `egress_3000qps`, `*_multihost` (3k), optional frp |
| `bench-6k` | main push | `*_6000qps` cases, optional frp |
| `bench-8k` | main push | dial9 trace cases @ 8k RPS + optional frp @ 8k |
| `publish-bench` | main push | merge artifacts → gh-pages |

`workflow_dispatch` can disable jobs via boolean inputs (see §4).

---

## 2. Configuration Schemas

### 2.1 Metadata dictionary (`schema.json`)

Single source of truth for dashboard labels, units, and process grouping (`ci-helpers/schema.json`).

### 2.2 K6 case catalogs

| File | Profile filter (`BENCH_PROFILE`) | Notes |
|------|----------------------------------|-------|
| `cases/defaults.json` | all | Ports, thresholds, scenario defaults |
| `cases/basic.json` | `basic` | Ramp HTTP/WS/gRPC/bidir |
| `cases/body_size.json` | `body_size` | Payload scaling |
| `cases/stress.json` | `core` (excludes `*_8000qps`) | 3k/6k fixed-rate HTTP |
| `cases/frp.json` | `frp` or per-case `frp_*` | frp baseline; looser error threshold |

**Case selection**

- `BENCH_PROFILE` — filters catalog (`bench.js` `filterCases`).
- `BENCH_CASE` — run exactly one case (`run-bench-case.sh`, `run-trace-8k.sh`).

**Multihost**

- k6 uses `MULTIHOST_COUNT = 50` hosts: `echo-01.local` … `echo-50.local` (`bench.js`).
- Routing in `ci-helpers/configs/server.yaml` and `frpc.toml` covers `echo-01` … `echo-50`.
- `/etc/hosts` also lists `echo-51` … `echo-60` for future expansion; they are **not** used by current cases.

**Thresholds (actual, not legacy comments)**

- Default: `p(95)<60000` ms and `http_req_failed rate<0.05` (`defaults.json`).
- frp cases: `http_req_failed rate<0.20`.
- CI does **not** enforce `p99 < 500ms`; k6 fails only on configured thresholds.

### 2.3 Runtime config patched at CI startup

`start-infra` and `run-trace-8k.sh` rewrite `ci-helpers/configs/client.yaml` at runtime:

| Field | Repo default | CI effective | Set by |
|-------|--------------|--------------|--------|
| `auth_token` | placeholder | ctld-issued token | `start-infra` / `run-trace-8k.sh` |
| `quic.connections` | `0` (auto) | `effective_runtime_parallelism()` | runtime resolver (not patched in CI) |

**Parallelism anchor** — single workflow input `worker_threads` (`0` = auto):

`effective_runtime_parallelism()` = `min(requested, cgroup_cpu_limit)` where `requested` is `TOKIO_WORKER_THREADS` or host logical CPUs. **systemd `CPUQuota=100%` = 1 CPU** (not 100% of all cores); the runtime reads cgroup `cpu.max` / `cpu.cfs_quota_us` so derived values match the quota.

| Derived field | YAML path | CI default (`worker_threads=0`, `CPUQuota=100%`) |
|---------------|-----------|-----------------------------------------------------|
| Tokio workers | (runtime builder) | **1** |
| `accept_workers` | `server.accept_workers` / `entry.accept_workers` | **1** |
| `quic.shards` | `server.quic.shards` / `quic.shards` | **1** |
| `quic.connections` | `quic.connections` (`0` = auto) | **1** |

Examples: `CPUQuota=400%` → anchor **4** on a quota that allows 4 CPUs; `worker_threads=2` with `CPUQuota=100%` → anchor **1** (quota caps explicit pin). Set `stress_cpu_quota` to `none` / `0` to disable cgroup cap and use full host CPU count.

Explicit yaml overrides (e.g. `connections: 1`) are honored without a separate Actions input.

**Ports** (`defaults.json` ↔ configs)

| Role | Port | Config |
|------|------|--------|
| Ingress HTTP | 8080 | server listener |
| Client egress entry | 8082 | `client.yaml` `entry.port` |
| QUIC tunnel | 10086 | both |
| Metrics | 9090 / 9092 / 9091 | server / client / ctld |

---

## 3. Data Format

### 3.1 Global index (`data.js`)

See prior `BENCHMARK_DATA.entries[]` shape — commit, summary, scenarios[].

### 3.2 Per-commit report (`data/[sha7].json`)

- `cases` / `scenarios` — latency, RPS, errors per case name.
- `resources_per_case` — time series keyed by case (when sampling + parse succeed).
- `catalog` — snapshot of `schema.json`.

### 3.3 Resource sampling alignment

- **Preferred**: record `sampling_start` and `k6_start`, pass `--k6-offset $((k6_start - sampling_start))` to `bench-tool.py parse` (`bench-basic` job).
- **Known gap**: `run-bench-case.sh` and `run-trace-8k.sh` currently use `--k6-offset 0` while collect starts before k6 — resource charts may be shifted vs latency window.

---

## 4. GitHub Actions `workflow_dispatch` parameters

Applies only to **manual** runs. **Push to `main`** uses fixed defaults in job YAML (not the form), listed in the “Effective default” column.

### 4.1 Build & test toggles

| Input | Type | Form default | Effective on push | Meaning |
|-------|------|--------------|-------------------|---------|
| `build_profile` | choice | `both` | `both` on main (dial9 build runs) | `release` \| `dial9` \| `both` — binaries for bench-8k trace |
| `run_unit` | bool | `true` | always | Unit tests + clippy |
| `run_integration` | bool | `true` | always | Integration matrix |
| `run_stress` | bool | `true` | bench jobs on main | All `bench-*` jobs |
| `run_stress_trace` | bool | `true` | same | `bench-8k` dial9 trace cases (`run-trace-8k.sh`) |
| `run_frp` | bool | `true` | same | frp baseline cases in bench-3k/6k/8k |
| `tune_kernel` | bool | `true` | `true` | `scripts/tune-os.sh` before bench |
| `publish_pages` | bool | `false` | `true` on main | gh-pages publish |

### 4.2 Runtime tuning (bench + integration when dispatched)

| Input | Type | Form default | Effective on push | Valid values | Wired to |
|-------|------|--------------|-------------------|--------------|----------|
| `worker_threads` | number | `0` | `0` | `0` = auto (`min(host CPUs, cgroup quota)`; with default `CPUQuota=100%` → derived **1**). `1`–`256` = pin `TOKIO_WORKER_THREADS` request (still capped by quota). | `start-infra`, `run-trace-8k.sh` → `effective_runtime_parallelism()` |
| `stress_core_target_rate` | number | `0` | `0` | `0` = use per-case `rate` in `stress.json` (3000 for `*_3000qps` / `ingress_multihost`). `1`–`50000` = override via `K6_CORE_STRESS_RATE` for **non-8k** stress cases only. | `K6_CORE_STRESS_RATE` env → `bench.js` |
| `stress_cpu_quota` | string | `100%` | `100%` | systemd `CPUQuota` per scope. **`100%` = 1 CPU** → derived parallelism **1**. `400%` ≈ 4 CPUs. `none` \| `unlimited` \| `0` = no cap. Same value for DuoTunnel and frp. | `STRESS_CPU_QUOTA` env |

### 4.3 Observability

| Input | Type | Form default | Effective on push | Meaning |
|-------|------|--------------|-------------------|---------|
| `collect_resource_metrics` | bool | `true` | `true` | Run `bench-tool.py collect` during k6 (`COLLECT_RESOURCE_METRICS` env: `1` / `0`) |

### 4.4 Environment variables (job `env`)

| Variable | Source | Consumers |
|----------|--------|-----------|
| `STRESS_CPU_QUOTA` | `inputs.stress_cpu_quota` or `100%` | systemd scopes |
| `K6_CORE_STRESS_RATE` | set only if `stress_core_target_rate > 0` | k6 `bench.js` via `run-bench-case.sh` |
| `COLLECT_RESOURCE_METRICS` | `1` on push; `0` when dispatch sets `collect_resource_metrics=false` | `run-bench-case.sh`, `run-trace-8k.sh`, `bench-basic` job |

---

## 5. Scripts

| Script | Role |
|--------|------|
| `run-bench-case.sh <case> <sha>` | Single k6 case + collect + parse + inject; starts frp when case name is `frp_*` |
| `run-trace-8k.sh <case> <sha> <profile> <threads>` | Restart server/client per case, dial9 trace, 8k case |
| `warmup.sh` | Ingress/egress probe; optional restart using `STRESS_CPU_QUOTA` |
| `merge-results.py` | Merge multiple `bench-results-*.json` |
| `bench_ui/publish-gh-pages.sh` | Publish dashboard |

---

## 6. CLI (`bench-tool.py`)

- **collect** `<interval_sec>` — JSONL stream to stdout.
- **parse** `--input` `--k6-offset` `--output` — trim to k6 window.
- **inject** `--result` `--resources` `--case-name` — attach resources to one case in result JSON.

---

## 7. Deployment (GitHub Pages)

- `/index.html`, `/data.js`, `/data/[sha7].json`, `/schema.json`
- Hash routing: `#overview`, `#[sha7]`

---

## 8. Related specs

- Product/runtime parameters: `docs/spec/parameters.md` (overload, QUIC, buffers — **not** CI-specific).
- Integration topology: comments in `.github/workflows/ci.yml` `integration-test` job.
