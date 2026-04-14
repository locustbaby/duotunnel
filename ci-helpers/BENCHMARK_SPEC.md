# DuoTunnel Benchmark Specification

This document defines the architectural conventions, data formats, and configuration schemas used in the DuoTunnel benchmark infrastructure.

## 1. Architecture Overview

The system follows a three-tier architecture:
1.  **Orchestration & Load (K6)**: JSON-driven stress tests that generate load and latency metrics.
2.  **Collection & Processing (`bench-tool.py`)**: A Python CLI that samples system resources (CPU, Mem, PSI, TCP) and merges them with K6 results.
3.  **Visualization (Bench UI)**: A static data-driven dashboard that renders trends and detailed resource charts.

---

## 2. Configuration Schemas

### 2.1 Metadata Dictionary (`schema.json`)
Located at `ci-helpers/schema.json`, this is the **Single Source of Truth** for display labels, units, and process grouping.

- **`categories`**: Maps internal IDs (e.g., `stress`) to display labels and descriptions.
- **`metrics`**: Maps raw data keys (e.g., `net_rx_kbs`) to human-readable names (`Network RX`) and units (`KB/s`).
- **`groups`**: Defines the logical grouping of processes (e.g., `server`, `client`).
- **`process_mapping`**: Rules for mapping process name prefixes to group IDs.

### 2.2 K6 Scenario Definitions
Scenarios are defined in `ci-helpers/k6/cases/*.json`.
- **`defaults.json`**: Global parameter inheritance (VUs, durations, thresholds).
- **Domain JSONs** (`basic.json`, `stress.json`, `frp.json`): Scenario-specific parameters and execution phases.

---

## 3. Data Format (The Contract)

### 3.1 Global Index (`data.js`)
Used by the dashboard overview page. It exports a global `BENCHMARK_DATA` object.

```json
{
  "entries": [
    {
      "commit": { "id": "sha7", "message": "...", "url": "..." },
      "timestamp": "ISO-8601",
      "summary": { "totalRPS": 8000, "totalErr": 0.01, "totalRequests": 100000 },
      "scenarios": [
        { "name": "...", "p95": 10.5, "rps": 3000, "tunnel": "duotunnel|frp" }
      ]
    }
  ]
}
```

### 3.2 Detailed Report (`data/[sha7].json`)
High-fidelity data for a specific run.

- **`scenarios`**: Full list of scenario execution metrics.
- **`resources_per_case`**: System resource time-series keyed by scenario name.
  - `system`: Global metrics (CPU, Memory, Net).
  - `processes`: Metrics broken down by group (Server, Client).
- **`catalog`**: A snapshot of `schema.json` to ensure the report is self-describing.

---

## 4. Deployment Conventions (GitHub Pages)

The dashboard is served as a static site with the following directory structure:

- `/index.html`: The main React/Vanilla entry point.
- `/data.js`: The central index of all historical runs.
- `/data/[sha7].json`: Metadata and resource traces for a specific commit.
- `/schema.json`: The active metadata dictionary.

### URL Persistence
The UI uses **Relative Paths** and **Hash Routing** (`#overview`, `#[sha]`) to ensure compatibility with subpath hosting (e.g., `github.io/duotunnel/`).

---

## 5. CLI Usage

- **Collect**: `python3 bench-tool.py collect <interval_sec>`
- **Parse**: `python3 bench-tool.py parse --input <raw_log> --output <detail_json>`
- **Publish**: `python3 bench-tool.py publish --result <k6_json> --resources <parsed_json>`
