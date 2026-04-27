#!/usr/bin/env bash
# build-ios.sh
#
# Builds the Onde Rust crate as static libraries for iOS targets.
# The C header (onde_c_api.h) is already checked into the repository at
# lib/src/nativeInterop/cinterop/onde_c_api.h; this script only compiles
# the Rust static libraries that Kotlin/Native links against via cinterop.
#
# Output:
#   lib/src/iosArm64Main/libs/libonde.a         (device, arm64)
#   lib/src/iosSimulatorArm64Main/libs/libonde.a (simulator, arm64)
#
# Prerequisites:
#   - Rust stable toolchain with iOS targets:
#       rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#
# Usage:
#   ./scripts/build-ios.sh              # both targets, release
#   ./scripts/build-ios.sh --debug      # both targets, debug
#   FILTER_TARGET=device ./scripts/build-ios.sh   # device only
#   FILTER_TARGET=simulator ./scripts/build-ios.sh # simulator only

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SDK_DIR/../.." && pwd)"

PROFILE="release"
for arg in "$@"; do
    if [[ "$arg" == "--debug" ]]; then
        PROFILE="debug"
    fi
done

CARGO_ARGS=(rustc --manifest-path "$REPO_ROOT/Cargo.toml" --lib --crate-type staticlib)
if [[ "$PROFILE" == "release" ]]; then
    CARGO_ARGS+=(--release)
fi

echo "=== Building Onde for iOS (profile: $PROFILE) ==="
echo "Repository root: $REPO_ROOT"

BUILD_DEVICE=true
BUILD_SIM=true

if [[ "${FILTER_TARGET:-}" == "device" ]]; then
    BUILD_SIM=false
elif [[ "${FILTER_TARGET:-}" == "simulator" ]]; then
    BUILD_DEVICE=false
fi

# ── Build device (arm64) ──────────────────────────────────────────────────────

if [[ "$BUILD_DEVICE" == "true" ]]; then
    echo "  → Building aarch64-apple-ios (device)…"
    cargo "${CARGO_ARGS[@]}" --target aarch64-apple-ios

    DEVICE_LIB_DIR="$SDK_DIR/lib/src/iosArm64Main/libs"
    mkdir -p "$DEVICE_LIB_DIR"
    cp "$REPO_ROOT/target/aarch64-apple-ios/$PROFILE/libonde.a" "$DEVICE_LIB_DIR/libonde.a"
    echo "  ✓ Device lib → $DEVICE_LIB_DIR/libonde.a"
fi

# ── Build simulator (arm64) ──────────────────────────────────────────────────

if [[ "$BUILD_SIM" == "true" ]]; then
    echo "  → Building aarch64-apple-ios-sim (simulator)…"
    cargo "${CARGO_ARGS[@]}" --target aarch64-apple-ios-sim

    SIM_LIB_DIR="$SDK_DIR/lib/src/iosSimulatorArm64Main/libs"
    mkdir -p "$SIM_LIB_DIR"
    cp "$REPO_ROOT/target/aarch64-apple-ios-sim/$PROFILE/libonde.a" "$SIM_LIB_DIR/libonde.a"
    echo "  ✓ Simulator lib → $SIM_LIB_DIR/libonde.a"
fi

# ── Verify C header exists ───────────────────────────────────────────────────

HEADER="$SDK_DIR/lib/src/nativeInterop/cinterop/onde_c_api.h"
if [[ ! -f "$HEADER" ]]; then
    echo ""
    echo "⚠  Warning: C header not found at $HEADER"
    echo "   The cinterop definition expects onde_c_api.h to be present."
    echo "   This header is checked into the repository; make sure you"
    echo "   haven't accidentally deleted it."
    exit 1
fi

echo ""
echo "iOS build complete."
echo ""
echo "Static libraries are ready for Kotlin/Native cinterop."
echo "C header: $HEADER"
echo ""
echo "Next steps:"
echo "  cd $SDK_DIR && ./gradlew :lib:compileKotlinIosArm64"
echo "  cd $SDK_DIR && ./gradlew :lib:compileKotlinIosSimulatorArm64"
