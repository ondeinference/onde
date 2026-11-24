#!/usr/bin/env bash
# Build the wasm engine (sdk/web/rust) into sdk/web/wasm, where dist/ imports
# it from. Extra arguments go to cargo, e.g. `./scripts/build-wasm.sh --locked`.
set -euo pipefail
cd "$(dirname "$0")/.."

# simd128 is what makes CPU inference usable in a browser. Every current
# browser supports it, and createOnde checks for it before loading anything.
export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-C target-feature=+simd128"

wasm-pack build rust --target web --release --out-dir ../wasm --out-name onde_web -- "$@"

# wasm-pack writes a package.json and .gitignore for a standalone package;
# this one is shipped inside @ondeinference/web instead.
rm -f wasm/package.json wasm/.gitignore wasm/README.md
