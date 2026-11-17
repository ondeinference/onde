#!/usr/bin/env bash
# One browser cell: serve the harness and climb bench-tiny -> bench-quick ->
# bench, stopping at the first rung that fails (error, guard kill, timeout).
#   run-cell.sh <harness> <chrome|webkit> <out-dir> [drain-every]
# Expects MODEL_DIR (model.gguf + tokenizer.json) and the built www/pkg.
set -uo pipefail
harness="$1"; browser="$2"; here="$(cd "$(dirname "$0")/.." && pwd)"
drain="${4:-1}"
mkdir -p "$3"
out="$(cd "$3" && pwd)"
cd "$here"

export BASE="http://127.0.0.1:8123"
node serve.mjs 8123 > "$out/serve.log" 2>&1 &
server=$!
trap 'kill $server 2>/dev/null' EXIT
for _ in $(seq 1 50); do curl -fs "$BASE/common/bench-tiny.json" >/dev/null && break; sleep 0.2; done

# Per-rung wall-clock cap in seconds for the guard. Software WebGPU is slow.
cap() { case "$1" in bench-tiny) echo 240;; bench-quick) echo 420;; *) echo 900;; esac; }
reached=none
for cfg in bench-tiny bench-quick bench; do
  echo "::group::$harness/$browser $cfg"
  # Fresh Chrome profile per rung so Cache Storage seeding is exercised each time.
  rm -rf "$RUNNER_TEMP/profile-chrome"
  if MAX_SECONDS="$(cap "$cfg")" PROFILE_DIR="$RUNNER_TEMP/profile-chrome" \
     node run-page.mjs "/$harness/www/index.html?cfg=$cfg&drain=$drain" "$browser" --out "$out/$cfg.json" 2> "$out/$cfg.log"; then
    reached=$cfg
    echo "$cfg passed"
  else
    echo "$cfg failed"; tail -n 40 "$out/$cfg.log"
    echo "::endgroup::"
    break
  fi
  echo "::endgroup::"
done
echo "{\"harness\":\"$harness\",\"browser\":\"$browser\",\"runner\":\"${RUNNER_OS:-?}\",\"drain\":$drain,\"reached\":\"$reached\"}" > "$out/cell.json"
[ "$reached" = bench ]
