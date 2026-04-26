#!/usr/bin/env bash
# build-jvm.sh
#
# Builds the Onde Rust crate as a shared library for the host JVM platform
# and copies the output into lib/src/jvmMain/resources/native/<os-arch>/ for
# bundling in the JVM JAR.
#
# This script detects the host OS and architecture automatically. For
# cross-compilation, set ONDE_TARGET_TRIPLE explicitly.
#
# Prerequisites
# -------------
# 1. Rust stable toolchain installed
# 2. On macOS: Xcode command line tools (for Metal backend)
# 3. On Linux: standard build tools (gcc, etc.)
#
# Usage
# -----
#   ./scripts/build-jvm.sh                  # host platform, release
#   ./scripts/build-jvm.sh --debug          # host platform, debug
#   ONDE_TARGET_TRIPLE=x86_64-apple-darwin ./scripts/build-jvm.sh  # cross-compile

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SDK_DIR/../.." && pwd)"
RESOURCES_DIR="$SDK_DIR/lib/src/jvmMain/resources/native"

PROFILE="release"
for arg in "$@"; do
    if [[ "$arg" == "--debug" ]]; then
        PROFILE="debug"
    fi
done

# ── Detect host platform ──────────────────────────────────────────────────────

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Darwin) os="macos" ;;
        Linux)  os="linux" ;;
        *)      echo "Error: unsupported OS: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        arm64|aarch64) arch="aarch64" ;;
        x86_64|amd64)  arch="x86_64" ;;
        *)             echo "Error: unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    echo "$os-$arch"
}

# Map our platform label to a Rust target triple
platform_to_triple() {
    case "$1" in
        macos-aarch64) echo "aarch64-apple-darwin" ;;
        macos-x86_64)  echo "x86_64-apple-darwin" ;;
        linux-aarch64) echo "aarch64-unknown-linux-gnu" ;;
        linux-x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        *)             echo "Error: unknown platform: $1"; exit 1 ;;
    esac
}

# Map our platform label to the shared library extension
platform_ext() {
    case "$1" in
        macos-*) echo "dylib" ;;
        linux-*) echo "so" ;;
        *)       echo "Error: unknown platform: $1"; exit 1 ;;
    esac
}

PLATFORM="$(detect_platform)"
TARGET_TRIPLE="${ONDE_TARGET_TRIPLE:-$(platform_to_triple "$PLATFORM")}"
LIB_EXT="$(platform_ext "$PLATFORM")"

echo "=== Building Onde for JVM (profile: $PROFILE) ==="
echo "Platform:       $PLATFORM"
echo "Rust target:    $TARGET_TRIPLE"
echo "Repository root: $REPO_ROOT"

# ── Build ──────────────────────────────────────────────────────────────────────

cd "$REPO_ROOT"

CARGO_ARGS=(build --manifest-path Cargo.toml --target "$TARGET_TRIPLE")
if [[ "$PROFILE" == "release" ]]; then
    CARGO_ARGS+=(--release)
fi

echo "→ cargo ${CARGO_ARGS[*]}"
cargo "${CARGO_ARGS[@]}"

# ── Copy to resources ──────────────────────────────────────────────────────────

SRC="$REPO_ROOT/target/$TARGET_TRIPLE/$PROFILE/libonde.$LIB_EXT"
DST_DIR="$RESOURCES_DIR/$PLATFORM"
DST="$DST_DIR/libonde.$LIB_EXT"

if [[ ! -f "$SRC" ]]; then
    echo "Error: expected output not found: $SRC"
    exit 1
fi

mkdir -p "$DST_DIR"
cp "$SRC" "$DST"
echo "✓ $PLATFORM -> $DST"

echo
echo "JVM build complete."
echo "Native library copied to: $DST"
echo
echo "To build the JVM JAR with the bundled native library:"
echo "  cd $SDK_DIR && ./gradlew :lib:jvmJar"
