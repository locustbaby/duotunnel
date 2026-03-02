#!/usr/bin/env bash
# pgo-build.sh — Profile-Guided Optimization (PGO) build for tunnel server/client
#
# PGO feeds real traffic profiles back to LLVM so it can make better inlining
# and branch-prediction decisions.  Typical gain: 10–20% throughput.
#
# Requirements:
#   - llvm-profdata in PATH  (apt: llvm / brew: llvm)
#   - A running server + client for the profiling step (see below)
#
# Usage:
#   bash scripts/pgo-build.sh
#
# Output:
#   ./target/release/server   (PGO-optimised)
#   ./target/release/client   (PGO-optimised)

set -euo pipefail

PROFILE_DIR="$(pwd)/target/pgo-profiles"
MERGED="$PROFILE_DIR/merged.profdata"

# ── Step 1: instrumented build ────────────────────────────────────────────────
echo "==> [1/4] Building instrumented binaries…"
rm -rf "$PROFILE_DIR"
mkdir -p "$PROFILE_DIR"

RUSTFLAGS="-Cprofile-generate=$PROFILE_DIR" \
    cargo build --release 2>&1

echo "    Instrumented binaries ready."

# ── Step 2: collect profiles ──────────────────────────────────────────────────
echo ""
echo "==> [2/4] Profiling step"
echo "    Start the server and client with the instrumented binaries, run your"
echo "    workload for at least 30 seconds, then press ENTER to continue."
echo ""
echo "    Example (in separate terminals):"
echo "      ./target/release/server --config config/server.yaml"
echo "      ./target/release/client --config config/client.yaml"
echo "      bash verify_proxy.sh   # or run wrk / curl for more coverage"
echo ""
read -r -p "    Press ENTER when profiling is complete…"

# ── Step 3: merge profiles ────────────────────────────────────────────────────
echo "==> [3/4] Merging profile data…"
if ! command -v llvm-profdata &>/dev/null; then
    # Try versioned binaries (common on Debian/Ubuntu)
    for v in 18 17 16 15 14; do
        if command -v "llvm-profdata-$v" &>/dev/null; then
            ln -sf "$(command -v llvm-profdata-$v)" /tmp/llvm-profdata
            export PATH="/tmp:$PATH"
            break
        fi
    done
fi

if ! command -v llvm-profdata &>/dev/null; then
    echo "ERROR: llvm-profdata not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install llvm"
    echo "  macOS:         brew install llvm && export PATH=/opt/homebrew/opt/llvm/bin:\$PATH"
    exit 1
fi

llvm-profdata merge \
    --output="$MERGED" \
    "$PROFILE_DIR"/*.profraw 2>/dev/null || \
llvm-profdata merge \
    --output="$MERGED" \
    "$PROFILE_DIR"

echo "    Profile data merged → $MERGED"

# ── Step 4: PGO-optimised build ───────────────────────────────────────────────
echo "==> [4/4] Building PGO-optimised release binaries…"

RUSTFLAGS="-Cprofile-use=$MERGED -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --release 2>&1

echo ""
echo "==> PGO build complete."
echo "    server: ./target/release/server"
echo "    client: ./target/release/client"
echo ""
echo "    Profile data kept in: $PROFILE_DIR"
echo "    To rebuild without re-profiling: RUSTFLAGS=\"-Cprofile-use=$MERGED\" cargo build --release"
