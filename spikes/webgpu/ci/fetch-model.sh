#!/usr/bin/env bash
# Download the spike model into $1 (default ./model) and check its SHA-256.
# Skips files already present (the workflow restores $1 from actions/cache).
set -euo pipefail
dir="${1:-model}"
mkdir -p "$dir"
want=6eb923e7d26e9cea28811e1a8e852009b21242fb157b26149d3b188f3a8c8653

if [ ! -f "$dir/model.gguf" ]; then
  curl -fL --retry 3 -o "$dir/model.gguf" \
    https://huggingface.co/bartowski/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/Qwen2.5-0.5B-Instruct-Q4_K_M.gguf
fi
if [ ! -f "$dir/tokenizer.json" ]; then
  curl -fL --retry 3 -o "$dir/tokenizer.json" \
    https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/main/tokenizer.json
fi

got=$(shasum -a 256 "$dir/model.gguf" | cut -d' ' -f1)
if [ "$got" != "$want" ]; then
  echo "model sha256 mismatch: got $got, want $want" >&2
  rm -f "$dir/model.gguf"
  exit 1
fi
echo "model.gguf sha256 ok ($got)"
echo "tokenizer.json sha256 $(shasum -a 256 "$dir/tokenizer.json" | cut -d' ' -f1)"
