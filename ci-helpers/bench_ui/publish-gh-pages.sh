#!/usr/bin/env bash
# ci-helpers/bench/publish-gh-pages.sh
set -euo pipefail

# This script is intended to be run from GitHub Actions.
# It handles cloning gh-pages, copying assets, processing traces, and pushing updates.

[[ -z "$GITHUB_TOKEN" ]] && { echo "GITHUB_TOKEN not set"; exit 1; }
[[ -z "$GITHUB_WORKSPACE" ]] && { echo "GITHUB_WORKSPACE not set"; exit 1; }

REPO_URL="https://x-access-token:${GITHUB_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
WORKDIR="/tmp/gh-pages-repo"

echo "==> Preparing gh-pages repository"
rm -rf "$WORKDIR"
if git clone --branch gh-pages --single-branch --depth 1 --filter=blob:none --no-checkout "$REPO_URL" "$WORKDIR" 2>/dev/null; then
  cd "$WORKDIR"
  git sparse-checkout init --cone
  git sparse-checkout set bench
  git checkout
else
  mkdir -p "$WORKDIR" && cd "$WORKDIR"
  git init && git checkout --orphan gh-pages
fi

echo "==> Updating bench assets"
mkdir -p bench
cp "$GITHUB_WORKSPACE/ci-helpers/bench_ui/index.html" bench/index.html
cp "$GITHUB_WORKSPACE/ci-helpers/bench_ui/style.css" bench/style.css
cp "$GITHUB_WORKSPACE/ci-helpers/bench_ui/app.js" bench/app.js
rm -rf bench/viewer && cp -r "$GITHUB_WORKSPACE/ci-helpers/bench_ui/viewer" bench/viewer

echo "==> Pruning old traces"
if [ -d bench/traces ]; then
  find bench/traces -name "*.html" -delete 2>/dev/null || true
  SHAS=$(ls bench/traces/ 2>/dev/null | grep -oE '^[0-9a-f]{7}' | sort -u)
  SHA_COUNT=$(echo "$SHAS" | grep -c . || true)
  if [ "$SHA_COUNT" -gt 3 ]; then
    OLD_SHAS=$(echo "$SHAS" | head -n $(( SHA_COUNT - 3 )))
    for OLD in $OLD_SHAS; do
      find bench/traces -maxdepth 1 -name "${OLD}-*" -exec rm -rf {} +
    done
  fi
fi

FRP_ARG=""
FRP_OFFSET_ARG=""
if [ -s /tmp/core/bench-results-frp.json ]; then
  FRP_ARG="--frp-result /tmp/core/bench-results-frp.json"
  if [ -f /tmp/core/frp_k6_start_epoch ] && [ -f /tmp/core/sampling_start_epoch ]; then
    FRP_K6_OFFSET=$(( $(cat /tmp/core/frp_k6_start_epoch) - $(cat /tmp/core/sampling_start_epoch) ))
    FRP_OFFSET_ARG="--frp-k6-offset $FRP_K6_OFFSET"
  fi
fi

RES_ARG=""
if [ -s /tmp/core/resource-data.json ]; then
  RES_ARG="--resources /tmp/core/resource-data.json"
fi

echo "==> Processing per-case traces"
SHORT_SHA="$(echo "$GITHUB_SHA" | cut -c1-7)"
REPO_NAME="$(echo "$GITHUB_REPOSITORY" | cut -d/ -f2)"
BASE_URL="https://${GITHUB_REPOSITORY_OWNER}.github.io/${REPO_NAME}/bench/traces"
mkdir -p bench/traces

TRACE_CASES_JSON='['
FIRST=1
for CASE_NAME in ingress_8000qps egress_8000qps ingress_multihost_8000qps egress_multihost_8000qps; do
  SRV="/tmp/dial9-traces-q4/trace-${CASE_NAME}/server-trace.bin.gz"
  CLI="/tmp/dial9-traces-q4/trace-${CASE_NAME}/client-trace.bin.gz"
  SRV_URL=""
  CLI_URL=""
  
  # URLs are passed via environment variables (TRACE_ARTIFACT_...)
  # We use indirect expansion or just check the specific vars
  VAR_SRV="TRACE_ARTIFACT_${CASE_NAME^^}_SERVER_URL"
  VAR_CLI="TRACE_ARTIFACT_${CASE_NAME^^}_CLIENT_URL"
  SRV_ARTIFACT_URL="${!VAR_SRV:-}"
  CLI_ARTIFACT_URL="${!VAR_CLI:-}"

  if [ -s "$SRV" ]; then
    node "$GITHUB_WORKSPACE/ci-helpers/bench_ui/viewer/bin2json.js" \
      "$SRV" "bench/traces/${SHORT_SHA}-${CASE_NAME}-server"
    SRV_URL="${BASE_URL}/${SHORT_SHA}-${CASE_NAME}-server"
  fi
  if [ -s "$CLI" ]; then
    node "$GITHUB_WORKSPACE/ci-helpers/bench_ui/viewer/bin2json.js" \
      "$CLI" "bench/traces/${SHORT_SHA}-${CASE_NAME}-client"
    CLI_URL="${BASE_URL}/${SHORT_SHA}-${CASE_NAME}-client"
  fi
  if [ -n "$SRV_URL" ] || [ -n "$CLI_URL" ] || [ -n "$SRV_ARTIFACT_URL" ] || [ -n "$CLI_ARTIFACT_URL" ]; then
    [ "$FIRST" = "1" ] && FIRST=0 || TRACE_CASES_JSON="${TRACE_CASES_JSON},"
    TRACE_CASES_JSON="${TRACE_CASES_JSON}{\"case\":\"${CASE_NAME}\",\"server\":\"${SRV_URL}\",\"client\":\"${CLI_URL}\",\"server_download\":\"${SRV_ARTIFACT_URL}\",\"client_download\":\"${CLI_ARTIFACT_URL}\"}"
  fi
done
TRACE_CASES_JSON="${TRACE_CASES_JSON}]"

TRACE_CASES_ARG=""
if [ "$TRACE_CASES_JSON" != "[]" ]; then
  echo "$TRACE_CASES_JSON" > /tmp/trace-cases.json
  TRACE_CASES_ARG="--trace-cases-file /tmp/trace-cases.json"
fi

echo "==> Running bench-tool.py publish"
python3 "$GITHUB_WORKSPACE/ci-helpers/bench-tool.py" publish \
  --result  /tmp/bench-results-merged.json \
  --data    bench/data.js \
  --sha     "$GITHUB_SHA" \
  --msg     "$(git -C "$GITHUB_WORKSPACE" log -1 --pretty=%s)" \
  --url     "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/commit/${GITHUB_SHA}" \
  --run-url "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}" \
  $RES_ARG $FRP_ARG $FRP_OFFSET_ARG $TRACE_CASES_ARG

echo "==> Committing and pushing to gh-pages"
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add bench/
git diff --cached --quiet && { echo "No changes to commit"; exit 0; }
git commit -m "bench: update data for $SHORT_SHA"
git push --force origin gh-pages

echo "==> Done"
