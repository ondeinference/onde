#!/usr/bin/env bash
# build-android.sh
#
# Builds the Onde Rust crate as an Android shared library (.so) for all ABIs
# and copies the outputs into lib/src/main/jniLibs/ for the Gradle build.
#
# Why this script uses plain indexed arrays instead of associative arrays:
# some environments still ship older Bash versions, and associative arrays can
# fail there. Indexed arrays are much more portable.
#
# Prerequisites
# -------------
# 1. Rust Android targets installed:
#      rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
#
# 2. Android NDK installed and ANDROID_NDK_HOME set:
#      export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125
#
# 3. cargo-ndk installed:
#      cargo install cargo-ndk
#
# Usage
# -----
#   ./scripts/build-android.sh
#   ./scripts/build-android.sh --debug
#   FILTER_ABI=arm64-v8a ./scripts/build-android.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SDK_DIR/../.." && pwd)"
JNILIBS_DIR="$SDK_DIR/lib/src/main/jniLibs"

PROFILE="release"
for arg in "$@"; do
    if [[ "$arg" == "--debug" ]]; then
        PROFILE="debug"
    fi
done

# Keep these arrays in the same order.
ABIS=(
    "arm64-v8a"
    "armeabi-v7a"
    "x86_64"
    "x86"
)

RUST_TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
    "i686-linux-android"
)

if [[ "${#ABIS[@]}" -ne "${#RUST_TARGETS[@]}" ]]; then
    echo "Error: ABI and Rust target arrays are out of sync."
    exit 1
fi

if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    echo "Error: ANDROID_NDK_HOME is not set."
    echo "Example: export ANDROID_NDK_HOME=\$HOME/Library/Android/sdk/ndk/26.1.10909125"
    exit 1
fi

if ! command -v cargo-ndk >/dev/null 2>&1; then
    echo "Error: cargo-ndk not found."
    echo "Install with: cargo install cargo-ndk"
    exit 1
fi

SELECTED_ABIS=()
SELECTED_TARGETS=()

if [[ -n "${FILTER_ABI:-}" ]]; then
    FOUND="false"
    for i in "${!ABIS[@]}"; do
        if [[ "${ABIS[$i]}" == "$FILTER_ABI" ]]; then
            SELECTED_ABIS+=("${ABIS[$i]}")
            SELECTED_TARGETS+=("${RUST_TARGETS[$i]}")
            FOUND="true"
            break
        fi
    done

    if [[ "$FOUND" != "true" ]]; then
        echo "Error: FILTER_ABI='$FILTER_ABI' is not supported."
        echo "Supported ABIs: ${ABIS[*]}"
        exit 1
    fi
else
    SELECTED_ABIS=("${ABIS[@]}")
    SELECTED_TARGETS=("${RUST_TARGETS[@]}")
fi

echo "=== Building Onde for Android (profile: $PROFILE) ==="
echo "Repository root: $REPO_ROOT"
echo "Output jniLibs:   $JNILIBS_DIR"

cd "$REPO_ROOT"

CARGO_NDK_ARGS=(--manifest-path Cargo.toml)
if [[ "$PROFILE" == "release" ]]; then
    CARGO_NDK_ARGS+=(--release)
fi

ABI_FLAGS=()
for abi in "${SELECTED_ABIS[@]}"; do
    ABI_FLAGS+=(-t "$abi")
done

echo "→ cargo ndk ${ABI_FLAGS[*]} build ${CARGO_NDK_ARGS[*]}"
cargo ndk "${ABI_FLAGS[@]}" build "${CARGO_NDK_ARGS[@]}"

for i in "${!SELECTED_ABIS[@]}"; do
    abi="${SELECTED_ABIS[$i]}"
    rust_target="${SELECTED_TARGETS[$i]}"
    src="$REPO_ROOT/target/$rust_target/$PROFILE/libonde.so"
    dst="$JNILIBS_DIR/$abi/libonde.so"

    if [[ ! -f "$src" ]]; then
        echo "Error: expected output not found: $src"
        exit 1
    fi

    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
    echo "✓ $abi -> $dst"
done

echo
echo "Android build complete."
echo "Shared libraries copied to: $JNILIBS_DIR"
