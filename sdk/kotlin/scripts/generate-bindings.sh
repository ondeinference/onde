#!/usr/bin/env bash
# generate-bindings.sh
#
# Generates Kotlin UniFFI bindings from the compiled Android arm64 library.
# Output goes to lib/src/generated/kotlin/ and is gitignored.
#
# Run this after build-android.sh whenever the Rust API changes.
#
# Prerequisites
# -------------
#   cargo-ndk and Android targets installed (see build-android.sh)
#   uniffi-bindgen binary built (pinned to =0.31.0):
#     cargo build --manifest-path ../../uniffi-bindgen/Cargo.toml --release
#
# Usage
#   ./scripts/generate-bindings.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$(cd "$SDK_DIR/../.." && pwd)"
OUT_DIR="$SDK_DIR/lib/src/generated/kotlin"
BINDGEN="$REPO_ROOT/uniffi-bindgen/target/release/uniffi-bindgen"
LIBRARY="$REPO_ROOT/target/aarch64-linux-android/release/libonde.so"

echo "=== Generating Kotlin UniFFI bindings ==="

# Build the bindgen binary if it doesn't exist or is stale
if [[ ! -f "$BINDGEN" ]]; then
    echo "  → Building uniffi-bindgen…"
    cargo build --manifest-path "$REPO_ROOT/uniffi-bindgen/Cargo.toml" --release
fi

# The library must exist — run build-android.sh first
if [[ ! -f "$LIBRARY" ]]; then
    echo "Error: $LIBRARY not found."
    echo "Run ./scripts/build-android.sh first to compile the Rust library."
    exit 1
fi

mkdir -p "$OUT_DIR"

echo "  → Generating from: $LIBRARY"
"$BINDGEN" generate "$LIBRARY" \
    --language kotlin \
    --out-dir "$OUT_DIR"

echo ""
echo "Generated bindings: $OUT_DIR"
echo "Generated files:"
ls -lh "$OUT_DIR"
