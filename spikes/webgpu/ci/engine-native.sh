#!/usr/bin/env bash
# Native CPU benches for both engines on one runner. An engine that fails
# records its exit code instead of failing the job.
#   engine-native.sh <out-dir>   (expects MODEL_DIR and the release binaries)
set -uo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
out="$1"; mkdir -p "$out"
m="$MODEL_DIR/model.gguf"; t="$MODEL_DIR/tokenizer.json"

bench() { # <label> <binary> <cfg> [device]
  local label="$1" bin="$2" cfg="$3"
  "$bin" "$m" "$t" "$here/common/$cfg.json" ${4:+"$4"} > "$out/$label-$cfg.jsonl" 2> "$out/$label-$cfg.err"
  echo $? > "$out/$label-$cfg.exit"
}

for cfg in bench-tiny bench-quick bench; do
  bench cpu      "$here/cpu-baseline/target-native-cpu/release/native-bench" "$cfg" cpu
  bench vectoron "$here/vectoron-cpu/target-native/release/native-bench"     "$cfg"
done
grep -H . "$out"/*.exit
# Fail only if neither engine produced the full bench.
[ -s "$out/cpu-bench.jsonl" ] || [ -s "$out/vectoron-bench.jsonl" ]
