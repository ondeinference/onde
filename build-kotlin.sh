#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# build-kotlin.sh — Cross-compile libonde for Android and generate Kotlin bindings
# ──────────────────────────────────────────────────────────────────────────────
#
# This script:
#   1. Builds the `uniffi-bindgen` CLI tool (host binary).
#   2. Cross-compiles the `onde` cdylib (libonde.so) for each Android target.
#   3. Runs `uniffi-bindgen generate` to produce the Kotlin source from the
#      compiled library.
#   4. Copies the .so files into the Android library's jniLibs directories.
#
# Prerequisites:
#   - Rust toolchain with Android targets installed:
#       rustup target add aarch64-linux-android armv7-linux-androideabi \
#                         x86_64-linux-android i686-linux-android
#   - Android NDK installed (set ANDROID_NDK_HOME or auto-detected from ANDROID_HOME)
#   - cargo, rustc on PATH
#
# Usage:
#   ./build-kotlin.sh              # Build all targets (release)
#   ./build-kotlin.sh --debug      # Build all targets (debug)
#   ./build-kotlin.sh --target aarch64-linux-android   # Single target
#   ./build-kotlin.sh --help       # Show usage
#
# Environment variables:
#   ANDROID_NDK_HOME  — Path to the Android NDK (e.g. ~/Library/Android/sdk/ndk/27.2.12479018)
#   ANDROID_HOME      — Path to the Android SDK (NDK auto-detected under ndk/)
#   ONDE_FEATURES     — Extra Cargo features to enable (comma-separated, default: none)
#   CARGO             — Path to cargo binary (default: cargo)
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ── Constants ─────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR"
KOTLIN_PROJECT_DIR="$CRATE_DIR/kotlin"
KOTLIN_LIB_DIR="$KOTLIN_PROJECT_DIR/onde"
JNILIBS_DIR="$KOTLIN_LIB_DIR/src/main/jniLibs"
# UniFFI's --out-dir receives the Kotlin source root; it creates the
# package directory structure (com/ondeinference/onde/) automatically
# based on the package_name in uniffi.toml.
KOTLIN_SRC_ROOT="$KOTLIN_LIB_DIR/src/main/kotlin"
GENERATED_KT_DIR="$KOTLIN_SRC_ROOT/com/ondeinference/onde"
BINDGEN_CRATE_DIR="$CRATE_DIR/uniffi-bindgen"

CARGO="${CARGO:-cargo}"
PROFILE="release"
PROFILE_DIR="release"

# Map Rust target triples to Android ABI folder names and NDK toolchain prefixes.
declare -A TARGET_TO_ABI=(
    ["aarch64-linux-android"]="arm64-v8a"
    ["armv7-linux-androideabi"]="armeabi-v7a"
    ["x86_64-linux-android"]="x86_64"
    ["i686-linux-android"]="x86"
)

declare -A TARGET_TO_CC_PREFIX=(
    ["aarch64-linux-android"]="aarch64-linux-android"
    ["armv7-linux-androideabi"]="armv7a-linux-androideabi"
    ["x86_64-linux-android"]="x86_64-linux-android"
    ["i686-linux-android"]="i686-linux-android"
)

ALL_TARGETS=("aarch64-linux-android" "armv7-linux-androideabi" "x86_64-linux-android" "i686-linux-android")
SELECTED_TARGETS=()

# Minimum Android API level — must match the library's minSdk (24).
MIN_API=24

# ── Helpers ───────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log()   { echo -e "${CYAN}[onde]${NC} $*"; }
ok()    { echo -e "${GREEN}[onde]${NC} $*"; }
warn()  { echo -e "${YELLOW}[onde]${NC} $*"; }
err()   { echo -e "${RED}[onde]${NC} $*" >&2; }
die()   { err "$@"; exit 1; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Cross-compile libonde for Android and generate Kotlin (UniFFI) bindings.

Options:
  --debug               Build in debug mode (default: release)
  --target <triple>     Build only for the specified Rust target triple.
                        Can be repeated. If omitted, all four Android targets
                        are built: ${ALL_TARGETS[*]}
  --generate-only       Skip Rust compilation; only run uniffi-bindgen to
                        regenerate the Kotlin source from an existing .so.
  --ndk <path>          Explicit path to the Android NDK.
  --help, -h            Show this help message.

Environment:
  ANDROID_NDK_HOME      Android NDK path (auto-detected from ANDROID_HOME)
  ANDROID_HOME          Android SDK path (NDK searched under ndk/)
  ONDE_FEATURES         Extra Cargo features, comma-separated
  CARGO                 Path to cargo (default: cargo)

Examples:
  # Full build for all Android ABIs (release)
  ./build-kotlin.sh

  # Debug build, arm64 only
  ./build-kotlin.sh --debug --target aarch64-linux-android

  # Re-generate Kotlin source without recompiling Rust
  ./build-kotlin.sh --generate-only
EOF
    exit 0
}

# ── Argument parsing ──────────────────────────────────────────────────────────

GENERATE_ONLY=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)
            PROFILE="debug"
            PROFILE_DIR="debug"
            shift
            ;;
        --target)
            [[ -n "${2:-}" ]] || die "--target requires a Rust target triple"
            SELECTED_TARGETS+=("$2")
            shift 2
            ;;
        --generate-only)
            GENERATE_ONLY=true
            shift
            ;;
        --ndk)
            [[ -n "${2:-}" ]] || die "--ndk requires a path"
            ANDROID_NDK_HOME="$2"
            shift 2
            ;;
        --help|-h)
            usage
            ;;
        *)
            die "Unknown option: $1 (try --help)"
            ;;
    esac
done

# Default to all targets if none specified.
if [[ ${#SELECTED_TARGETS[@]} -eq 0 ]]; then
    SELECTED_TARGETS=("${ALL_TARGETS[@]}")
fi

# Validate selected targets.
for t in "${SELECTED_TARGETS[@]}"; do
    if [[ -z "${TARGET_TO_ABI[$t]:-}" ]]; then
        die "Unknown Android target: $t\nValid targets: ${ALL_TARGETS[*]}"
    fi
done

# ── Locate the Android NDK ────────────────────────────────────────────────────

find_ndk() {
    # 1. Explicit ANDROID_NDK_HOME
    if [[ -n "${ANDROID_NDK_HOME:-}" && -d "$ANDROID_NDK_HOME" ]]; then
        echo "$ANDROID_NDK_HOME"
        return
    fi

    # 2. Search inside ANDROID_HOME/ndk/ — pick the newest version.
    if [[ -n "${ANDROID_HOME:-}" && -d "$ANDROID_HOME/ndk" ]]; then
        local latest
        latest=$(find "$ANDROID_HOME/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -1)
        if [[ -n "$latest" ]]; then
            echo "$latest"
            return
        fi
    fi

    # 3. Common macOS SDK location.
    local default_sdk="$HOME/Library/Android/sdk"
    if [[ -d "$default_sdk/ndk" ]]; then
        local latest
        latest=$(find "$default_sdk/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -1)
        if [[ -n "$latest" ]]; then
            echo "$latest"
            return
        fi
    fi

    # 4. Linux default.
    local linux_sdk="$HOME/Android/Sdk"
    if [[ -d "$linux_sdk/ndk" ]]; then
        local latest
        latest=$(find "$linux_sdk/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -1)
        if [[ -n "$latest" ]]; then
            echo "$latest"
            return
        fi
    fi

    return 1
}

if ! $GENERATE_ONLY; then
    NDK_HOME=$(find_ndk) || die \
        "Android NDK not found.\n" \
        "Set ANDROID_NDK_HOME or ANDROID_HOME, or pass --ndk <path>."

    log "Using Android NDK: ${BOLD}$NDK_HOME${NC}"

    # Determine the host OS tag for the NDK toolchain.
    case "$(uname -s)" in
        Darwin*) HOST_TAG="darwin-x86_64" ;;
        Linux*)  HOST_TAG="linux-x86_64" ;;
        MINGW*|MSYS*|CYGWIN*) HOST_TAG="windows-x86_64" ;;
        *)       die "Unsupported host OS: $(uname -s)" ;;
    esac

    TOOLCHAIN_BIN="$NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"
    [[ -d "$TOOLCHAIN_BIN" ]] || die "NDK toolchain not found at: $TOOLCHAIN_BIN"
fi

# ── Step 1: Build the uniffi-bindgen CLI (host) ──────────────────────────────

log "Building ${BOLD}uniffi-bindgen${NC} CLI (host)..."

$CARGO build \
    --manifest-path "$BINDGEN_CRATE_DIR/Cargo.toml" \
    --release \
    --quiet

UNIFFI_BINDGEN="$BINDGEN_CRATE_DIR/target/release/uniffi-bindgen"
# If the binary lands in the workspace target/ dir instead, try that too.
if [[ ! -x "$UNIFFI_BINDGEN" ]]; then
    # When the crate shares a workspace target dir (or no workspace), the
    # binary may be under the onde crate's target/ or the workspace root.
    for candidate in \
        "$CRATE_DIR/target/release/uniffi-bindgen" \
        "$CRATE_DIR/../../../target/release/uniffi-bindgen"; do
        if [[ -x "$candidate" ]]; then
            UNIFFI_BINDGEN="$candidate"
            break
        fi
    done
fi

[[ -x "$UNIFFI_BINDGEN" ]] || die "uniffi-bindgen binary not found after build."
ok "uniffi-bindgen ready: $UNIFFI_BINDGEN"

# ── Step 2: Cross-compile the Rust cdylib for each Android target ─────────────

# We'll remember the path to the first successfully built .so so we can use
# it for bindgen (any ABI's .so contains the same UniFFI metadata).
FIRST_SO=""

if ! $GENERATE_ONLY; then
    FEATURES_FLAG=""
    if [[ -n "${ONDE_FEATURES:-}" ]]; then
        FEATURES_FLAG="--features $ONDE_FEATURES"
    fi

    for TARGET in "${SELECTED_TARGETS[@]}"; do
        ABI="${TARGET_TO_ABI[$TARGET]}"
        CC_PREFIX="${TARGET_TO_CC_PREFIX[$TARGET]}"

        log "Compiling ${BOLD}libonde.so${NC} for ${BOLD}$TARGET${NC} ($ABI)..."

        # Set up the CC/CXX/AR/RANLIB environment for the NDK clang toolchain
        # so that cc-rs, cmake, and any -sys crates find the right compiler.
        export CC="${TOOLCHAIN_BIN}/${CC_PREFIX}${MIN_API}-clang"
        export CXX="${TOOLCHAIN_BIN}/${CC_PREFIX}${MIN_API}-clang++"
        export AR="${TOOLCHAIN_BIN}/llvm-ar"
        export RANLIB="${TOOLCHAIN_BIN}/llvm-ranlib"

        # Cargo uses the uppercase-dashed form of the target for env overrides.
        TARGET_UPPER=$(echo "$TARGET" | tr '[:lower:]-' '[:upper:]_')
        export "CARGO_TARGET_${TARGET_UPPER}_LINKER=$CC"

        if [[ "$PROFILE" == "release" ]]; then
            PROFILE_FLAG="--release"
        else
            PROFILE_FLAG=""
        fi

        $CARGO build \
            --manifest-path "$CRATE_DIR/Cargo.toml" \
            --lib \
            --target "$TARGET" \
            $PROFILE_FLAG \
            $FEATURES_FLAG

        # Locate the built .so
        SO_PATH="$CRATE_DIR/target/$TARGET/$PROFILE_DIR/libonde.so"
        if [[ ! -f "$SO_PATH" ]]; then
            # Some workspace layouts put target/ at the workspace root.
            for candidate in \
                "$CRATE_DIR/../../../target/$TARGET/$PROFILE_DIR/libonde.so" \
                "$CRATE_DIR/../../target/$TARGET/$PROFILE_DIR/libonde.so"; do
                if [[ -f "$candidate" ]]; then
                    SO_PATH="$(cd "$(dirname "$candidate")" && pwd)/$(basename "$candidate")"
                    break
                fi
            done
        fi

        [[ -f "$SO_PATH" ]] || die "libonde.so not found for $TARGET at expected paths."

        # Copy into jniLibs/
        mkdir -p "$JNILIBS_DIR/$ABI"
        cp "$SO_PATH" "$JNILIBS_DIR/$ABI/libonde.so"
        ok "  → $JNILIBS_DIR/$ABI/libonde.so ($(du -h "$JNILIBS_DIR/$ABI/libonde.so" | cut -f1))"

        if [[ -z "$FIRST_SO" ]]; then
            FIRST_SO="$SO_PATH"
        fi
    done
else
    # Generate-only: find an existing .so to extract metadata from.
    for TARGET in "${SELECTED_TARGETS[@]}"; do
        for candidate in \
            "$CRATE_DIR/target/$TARGET/$PROFILE_DIR/libonde.so" \
            "$CRATE_DIR/../../../target/$TARGET/$PROFILE_DIR/libonde.so" \
            "$JNILIBS_DIR/${TARGET_TO_ABI[$TARGET]}/libonde.so"; do
            if [[ -f "$candidate" ]]; then
                FIRST_SO="$candidate"
                break 2
            fi
        done
    done

    # Also try the host dylib (macOS .dylib, Linux .so) — uniffi-bindgen
    # only needs the metadata, not a runnable Android binary.
    if [[ -z "$FIRST_SO" ]]; then
        for ext in dylib so; do
            candidate="$CRATE_DIR/target/$PROFILE_DIR/libonde.$ext"
            if [[ -f "$candidate" ]]; then
                FIRST_SO="$candidate"
                break
            fi
        done
    fi

    [[ -n "$FIRST_SO" ]] || die "No compiled libonde found. Run without --generate-only first."
    log "Using existing library for bindgen: $FIRST_SO"
fi

# ── Step 3: Generate Kotlin bindings via uniffi-bindgen ───────────────────────

log "Generating Kotlin bindings..."

mkdir -p "$KOTLIN_SRC_ROOT"

$UNIFFI_BINDGEN generate \
    --library "$FIRST_SO" \
    --language kotlin \
    --out-dir "$KOTLIN_SRC_ROOT" \
    --config "$CRATE_DIR/uniffi.toml"

# Count generated files.
KT_COUNT=$(find "$GENERATED_KT_DIR" -name '*.kt' 2>/dev/null | wc -l | tr -d ' ')
ok "Generated ${BOLD}$KT_COUNT${NC} Kotlin file(s) in $GENERATED_KT_DIR"

# List them for visibility.
find "$GENERATED_KT_DIR" -name '*.kt' -exec basename {} \; | sort | while read -r f; do
    log "  • $f"
done

# ── Step 4: Summary ──────────────────────────────────────────────────────────

echo ""
ok "════════════════════════════════════════════════════════════════════"
ok "  Onde Kotlin library build complete!"
ok "════════════════════════════════════════════════════════════════════"
echo ""

log "Native libraries:"
for TARGET in "${SELECTED_TARGETS[@]}"; do
    ABI="${TARGET_TO_ABI[$TARGET]}"
    SO="$JNILIBS_DIR/$ABI/libonde.so"
    if [[ -f "$SO" ]]; then
        SIZE=$(du -h "$SO" | cut -f1)
        ok "  $ABI → $SIZE"
    else
        warn "  $ABI → (not built)"
    fi
done

echo ""
log "Kotlin source:  $GENERATED_KT_DIR"
log "Android lib:    $KOTLIN_LIB_DIR"
echo ""
log "Next steps:"
log "  1. Open ${BOLD}$KOTLIN_PROJECT_DIR${NC} in Android Studio, or"
log "  2. Add the library as a dependency in your app's build.gradle.kts:"
log "       ${BOLD}implementation(project(\":onde\"))${NC}"
log "  3. Or publish the AAR: ${BOLD}cd $KOTLIN_PROJECT_DIR && ./gradlew :onde:assembleRelease${NC}"
echo ""
