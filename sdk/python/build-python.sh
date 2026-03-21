#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# build-python.sh — Build the onde Python package using Maturin
# ──────────────────────────────────────────────────────────────────────────────
#
# This script:
#   1. Checks that `maturin` is available (or installs it).
#   2. Builds the Python wheel using `maturin build --release` from the
#      python/ directory.
#   3. Optionally runs `maturin develop` for development installs.
#   4. Shows a summary with the output wheel location.
#
# Prerequisites:
#   - Rust toolchain (cargo, rustc on PATH)
#   - Python 3.9+ with pip or uv
#   - uniffi-bindgen ==0.31.0 on PATH (auto-installed if missing)
#
# Usage:
#   ./build-python.sh              # Build release wheel
#   ./build-python.sh --develop    # Install into current venv (editable)
#   ./build-python.sh --release    # Build release wheel (explicit, default)
#   ./build-python.sh --help       # Show usage
#
# Environment variables:
#   ONDE_FEATURES     — Extra Cargo features to enable (e.g. "whisper")
#   CARGO             — Path to cargo binary (default: cargo)
#   PYTHON            — Path to python binary (default: python3)
#   MATURIN           — Path to maturin binary (default: auto-detected)
#   UNIFFI_BINDGEN    — Path to uniffi-bindgen binary (default: auto-detected)
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ── Constants ─────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PYTHON_DIR="$SCRIPT_DIR"

CARGO="${CARGO:-cargo}"
PYTHON="${PYTHON:-python3}"

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

Build the onde Python package using Maturin.

Options:
  --release             Build a release wheel (default).
  --develop             Install the package into the current virtualenv
                        using \`maturin develop\` (editable/debug build).
  --publish             Build and publish to PyPI (skips sdist).
  --help, -h            Show this help message.

Environment:
  ONDE_FEATURES         Extra Cargo features, e.g. "whisper"
  CARGO                 Path to cargo (default: cargo)
  PYTHON                Path to python3 (default: python3)
  MATURIN               Path to maturin (default: auto-detected)
  UNIFFI_BINDGEN        Path to uniffi-bindgen (default: auto-detected)

Examples:
  # Build a release wheel
  ./build-python.sh

  # Development install into current venv
  ./build-python.sh --develop

  # Build with whisper feature enabled
  ONDE_FEATURES=whisper ./build-python.sh

  # Publish to PyPI (requires MATURIN_PYPI_TOKEN or ~/.pypirc)
  ./build-python.sh --publish
EOF
    exit 0
}

# ── Argument parsing ──────────────────────────────────────────────────────────

MODE="release"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)
            MODE="release"
            shift
            ;;
        --develop)
            MODE="develop"
            shift
            ;;
        --publish)
            MODE="publish"
            shift
            ;;
        --help|-h)
            usage
            ;;
        *)
            die "Unknown option: $1 (try --help)"
            ;;
    esac
done

# ── Step 1: Ensure maturin is available ───────────────────────────────────────

find_or_install_maturin() {
    # 1. Explicit MATURIN env var.
    if [[ -n "${MATURIN:-}" ]]; then
        if command -v "$MATURIN" &>/dev/null; then
            echo "$MATURIN"
            return
        else
            die "MATURIN is set to '$MATURIN' but it is not executable."
        fi
    fi

    # 2. Already on PATH.
    if command -v maturin &>/dev/null; then
        echo "maturin"
        return
    fi

    # 3. Try installing via uv (preferred — fast, no venv needed).
    if command -v uv &>/dev/null; then
        warn "maturin not found. Installing via ${BOLD}uv tool install maturin${NC}..."
        uv tool install maturin >/dev/null 2>&1 || true
        if command -v maturin &>/dev/null; then
            echo "maturin"
            return
        fi
    fi

    # 4. Try installing via pip.
    if command -v pip3 &>/dev/null; then
        warn "maturin not found. Installing via ${BOLD}pip3 install maturin${NC}..."
        pip3 install maturin >/dev/null 2>&1 || true
        if command -v maturin &>/dev/null; then
            echo "maturin"
            return
        fi
    elif command -v pip &>/dev/null; then
        warn "maturin not found. Installing via ${BOLD}pip install maturin${NC}..."
        pip install maturin >/dev/null 2>&1 || true
        if command -v maturin &>/dev/null; then
            echo "maturin"
            return
        fi
    fi

    return 1
}

log "Checking for ${BOLD}maturin${NC}..."

MATURIN_BIN=$(find_or_install_maturin) || die \
    "maturin not found and could not be installed.\n" \
    "Install it manually: ${BOLD}pip install maturin${NC} or ${BOLD}uv tool install maturin${NC}"

MATURIN_VERSION=$($MATURIN_BIN --version 2>/dev/null || echo "unknown")
ok "maturin ready: ${BOLD}$MATURIN_BIN${NC} ($MATURIN_VERSION)"

# ── Step 2: Ensure uniffi-bindgen is available ────────────────────────────────
#
# Maturin with `bindings = "uniffi"` shells out to `uniffi-bindgen` to generate
# the Python bindings.  The version MUST match the `uniffi` crate version
# pinned in Cargo.toml (==0.31.0).  We check several locations:
#   1. Explicit UNIFFI_BINDGEN env var
#   2. Already on PATH
#   3. The project's local uniffi-bindgen binary (uniffi-bindgen/ crate)
#   4. pip install uniffi-bindgen==0.31.0

UNIFFI_VERSION="0.31.0"

find_or_install_uniffi_bindgen() {
    # 1. Explicit env var.
    if [[ -n "${UNIFFI_BINDGEN:-}" ]]; then
        if command -v "$UNIFFI_BINDGEN" &>/dev/null; then
            echo "$UNIFFI_BINDGEN"
            return
        elif [[ -x "$UNIFFI_BINDGEN" ]]; then
            echo "$UNIFFI_BINDGEN"
            return
        else
            die "UNIFFI_BINDGEN is set to '$UNIFFI_BINDGEN' but it is not executable."
        fi
    fi

    # 2. Already on PATH.
    if command -v uniffi-bindgen &>/dev/null; then
        echo "uniffi-bindgen"
        return
    fi

    # 3. Project-local binary — use it if already built, otherwise build it.
    local local_bin="$CRATE_DIR/uniffi-bindgen/target/release/uniffi-bindgen"
    if [[ -x "$local_bin" ]]; then
        echo "$local_bin"
        return
    fi

    # 3b. Binary not built yet — try to compile the local uniffi-bindgen crate.
    local local_crate="$CRATE_DIR/uniffi-bindgen"
    if [[ -f "$local_crate/Cargo.toml" ]]; then
        warn "Local uniffi-bindgen binary not found. Building from ${BOLD}$local_crate${NC}..."
        if $CARGO build --release --manifest-path "$local_crate/Cargo.toml" 2>&1; then
            if [[ -x "$local_bin" ]]; then
                echo "$local_bin"
                return
            fi
        else
            warn "Failed to build local uniffi-bindgen crate, trying other methods..."
        fi
    fi

    # 4. Check common pip --user bin directories.
    for candidate in \
        "$HOME/Library/Python/3.9/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.10/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.11/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.12/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.13/bin/uniffi-bindgen" \
        "$HOME/.local/bin/uniffi-bindgen"; do
        if [[ -x "$candidate" ]]; then
            echo "$candidate"
            return
        fi
    done

    # 5. Install via pip.
    warn "uniffi-bindgen not found. Installing ${BOLD}uniffi-bindgen==$UNIFFI_VERSION${NC} via pip..."
    if command -v pip3 &>/dev/null; then
        pip3 install "uniffi-bindgen==$UNIFFI_VERSION" >/dev/null 2>&1 || true
    elif command -v pip &>/dev/null; then
        pip install "uniffi-bindgen==$UNIFFI_VERSION" >/dev/null 2>&1 || true
    fi

    # Re-check PATH and common locations after install.
    if command -v uniffi-bindgen &>/dev/null; then
        echo "uniffi-bindgen"
        return
    fi
    for candidate in \
        "$HOME/Library/Python/3.9/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.10/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.11/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.12/bin/uniffi-bindgen" \
        "$HOME/Library/Python/3.13/bin/uniffi-bindgen" \
        "$HOME/.local/bin/uniffi-bindgen"; do
        if [[ -x "$candidate" ]]; then
            echo "$candidate"
            return
        fi
    done

    return 1
}

log "Checking for ${BOLD}uniffi-bindgen${NC} (==$UNIFFI_VERSION)..."

UNIFFI_BIN=$(find_or_install_uniffi_bindgen) || die \
    "uniffi-bindgen not found and could not be installed.\n" \
    "Install it manually: ${BOLD}pip install uniffi-bindgen==$UNIFFI_VERSION${NC}\n" \
    "The version MUST match the uniffi crate version in Cargo.toml."

# Ensure the discovered binary is on PATH so maturin can find it.
UNIFFI_BIN_DIR="$(dirname "$UNIFFI_BIN")"
case ":$PATH:" in
    *":$UNIFFI_BIN_DIR:"*) ;;
    *) export PATH="$UNIFFI_BIN_DIR:$PATH" ;;
esac

UNIFFI_DETECTED_VERSION=$($UNIFFI_BIN --version 2>/dev/null || echo "unknown")
ok "uniffi-bindgen ready: ${BOLD}$UNIFFI_BIN${NC} ($UNIFFI_DETECTED_VERSION)"

# ── Step 3: Verify Python & Rust toolchains ───────────────────────────────────

if ! command -v "$PYTHON" &>/dev/null; then
    die "Python not found at '$PYTHON'. Set the PYTHON env var or install Python 3.8+."
fi

PYTHON_VERSION=$($PYTHON --version 2>&1)
log "Python: ${BOLD}$PYTHON_VERSION${NC}"

if ! command -v "$CARGO" &>/dev/null; then
    die "Cargo not found at '$CARGO'. Install Rust: https://rustup.rs"
fi

CARGO_VERSION=$($CARGO --version 2>&1)
log "Cargo:  ${BOLD}$CARGO_VERSION${NC}"

# ── Step 4: Prepare feature flags ────────────────────────────────────────────

FEATURES_ARGS=()
if [[ -n "${ONDE_FEATURES:-}" ]]; then
    # Split comma-separated features and pass each via --features
    IFS=',' read -ra FEAT_LIST <<< "$ONDE_FEATURES"
    for feat in "${FEAT_LIST[@]}"; do
        feat=$(echo "$feat" | xargs) # trim whitespace
        if [[ -n "$feat" ]]; then
            FEATURES_ARGS+=("--cargo-extra-args=--features=$feat")
        fi
    done
    log "Features: ${BOLD}${ONDE_FEATURES}${NC}"
fi

# ── Step 5: Build / develop / publish ─────────────────────────────────────────

cd "$PYTHON_DIR"

if [[ "$MODE" == "develop" ]]; then
    # ── Development install ───────────────────────────────────────────────
    log "Running ${BOLD}maturin develop${NC} (editable install into current venv)..."
    echo ""

    $MATURIN_BIN develop \
        --manifest-path "$CRATE_DIR/Cargo.toml" \
        "${FEATURES_ARGS[@]+"${FEATURES_ARGS[@]}"}"

    echo ""
    ok "Development install complete."
    ok "The ${BOLD}onde_inference${NC} package is now available in your current Python environment."

elif [[ "$MODE" == "publish" ]]; then
    # ── Publish to PyPI ───────────────────────────────────────────────────
    # All Rust deps use git refs (no local path deps), and
    # uniffi-bindgen==0.31.0 is in [build-system] requires so PEP 517
    # isolated builds install it automatically.
    log "Running ${BOLD}maturin publish${NC}..."
    echo ""

    $MATURIN_BIN publish \
        --manifest-path "$CRATE_DIR/Cargo.toml" \
        --skip-existing \
        --compatibility pypi \
        "${FEATURES_ARGS[@]+"${FEATURES_ARGS[@]}"}"

    echo ""
    ok "Published to PyPI."

else
    # ── Release wheel build ───────────────────────────────────────────────
    log "Running ${BOLD}maturin build --release${NC}..."
    echo ""

    $MATURIN_BIN build \
        --release \
        --manifest-path "$CRATE_DIR/Cargo.toml" \
        --out "$PYTHON_DIR/dist" \
        "${FEATURES_ARGS[@]+"${FEATURES_ARGS[@]}"}"

    echo ""

    # Find the built wheel(s).
    WHEEL_DIR="$PYTHON_DIR/dist"
    if [[ -d "$WHEEL_DIR" ]]; then
        WHEEL_FILES=()
        while IFS= read -r -d '' whl; do
            WHEEL_FILES+=("$whl")
        done < <(find "$WHEEL_DIR" -name '*.whl' -newer "$PYTHON_DIR" -print0 2>/dev/null || true)

        # If the timestamp trick didn't find any, just list all wheels.
        if [[ ${#WHEEL_FILES[@]} -eq 0 ]]; then
            while IFS= read -r -d '' whl; do
                WHEEL_FILES+=("$whl")
            done < <(find "$WHEEL_DIR" -name '*.whl' -print0 2>/dev/null || true)
        fi
    fi
fi

# ── Step 6: Summary ──────────────────────────────────────────────────────────

echo ""
ok "════════════════════════════════════════════════════════════════════"
ok "  Onde Python package build complete!"
ok "════════════════════════════════════════════════════════════════════"
echo ""

if [[ "$MODE" == "develop" ]]; then
    log "Mode:     ${BOLD}develop${NC} (editable install)"
    log "Python:   ${BOLD}$PYTHON_VERSION${NC}"
    echo ""
    log "Verify the install:"
    log "  ${BOLD}$PYTHON -c \"from onde_inference import __version__; print(__version__)\"${NC}"

elif [[ "$MODE" == "publish" ]]; then
    log "Mode:     ${BOLD}publish${NC} (PyPI)"
    log "Python:   ${BOLD}$PYTHON_VERSION${NC}"
    echo ""
    log "Install from PyPI:"
    log "  ${BOLD}pip install onde-inference${NC}"

else
    log "Mode:     ${BOLD}release${NC} (wheel)"
    log "Python:   ${BOLD}$PYTHON_VERSION${NC}"

    if [[ ${#WHEEL_FILES[@]} -gt 0 ]]; then
        echo ""
        log "Built wheel(s):"
        for whl in "${WHEEL_FILES[@]}"; do
            SIZE=$(du -h "$whl" | cut -f1)
            ok "  $(basename "$whl") (${SIZE})"
            log "    ${BOLD}$whl${NC}"
        done
    else
        warn "No .whl files found in $WHEEL_DIR"
    fi

    echo ""
    log "Install the wheel:"
    if [[ ${#WHEEL_FILES[@]} -gt 0 ]]; then
        log "  ${BOLD}pip install ${WHEEL_FILES[-1]}${NC}"
    else
        log "  ${BOLD}pip install $WHEEL_DIR/<wheel-file>.whl${NC}"
    fi
fi

if [[ -n "${ONDE_FEATURES:-}" ]]; then
    echo ""
    log "Features: ${BOLD}${ONDE_FEATURES}${NC}"
fi

echo ""
