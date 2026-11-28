#!/usr/bin/env bash
# Native benches on one runner, three configs each. A device that cannot start
# (Metal inside a VM, say) records its error instead of failing the job.
#   native.sh <out-dir>   (expects MODEL_DIR and the release binaries)
set -uo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
out="$1"; mkdir -p "$out"
m="$MODEL_DIR/model.gguf"; t="$MODEL_DIR/tokenizer.json"

bench() { # <label> <binary> <device> <cfg>
  local label="$1" bin="$2" dev="$3" cfg="$4"
  if [ ! -x "$bin" ]; then
    : > "$out/$label-$cfg.jsonl"; echo "binary missing, its build failed: $bin" > "$out/$label-$cfg.err"; echo 127 > "$out/$label-$cfg.exit"
    return
  fi
  "$bin" "$m" "$t" "$here/common/$cfg.json" "$dev" > "$out/$label-$cfg.jsonl" 2> "$out/$label-$cfg.err"
  echo $? > "$out/$label-$cfg.exit"
}

for cfg in bench-tiny bench-quick bench; do
  bench cpu   "$here/cpu-baseline/target-native-cpu/release/native-bench"   cpu   "$cfg"
  bench metal "$here/cpu-baseline/target-native-metal/release/native-bench" metal "$cfg"
  bench wgpu  "$here/wgpu-candle/target-native/release/native-bench"        wgpu  "$cfg"
done
grep -H . "$out"/*.exit
