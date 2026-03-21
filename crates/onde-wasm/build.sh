#!/usr/bin/env bash
# =============================================================================
# onde/crates/onde-wasm/build.sh
#
# One-command wasm-pack build for the onde WASM Whisper engine.
#
# Prerequisites
# -------------
#   • Rust with the wasm32-unknown-unknown target:
#       rustup target add wasm32-unknown-unknown
#
#   • wasm-pack:
#       cargo install wasm-pack
#
#   • (Optional) wasm-opt for smaller binaries — installed automatically by
#     wasm-pack if missing, but a pre-installed version is faster.
#
# Usage
# -----
#   # Standard release build
#   ./build.sh
#
#   # Debug build (faster compile, larger .wasm, source-mapped)
#   BUILD_TYPE=debug ./build.sh
#
#   # Override the output directory (default: onde/crates/onde-wasm/pkg)
#   OUT_DIR=/path/to/output ./build.sh
#
# Output
# ------
#   pkg/
#     onde_wasm.js          — ES module glue
#     onde_wasm_bg.wasm     — compiled WASM binary
#     onde_wasm.d.ts        — TypeScript declarations for WhisperDecoder
#     onde_wasm_bg.wasm.d.ts
#     package.json
#
# Model assets (download separately — see section below)
# ------------------------------------------------------
#   The WhisperDecoder constructor requires four byte arrays that are NOT
#   bundled into the .wasm binary.  The JS consumer must fetch them from
#   HuggingFace and pass them as Uint8Arrays.
#
#   Recommended model: whisper-tiny.en (non-quantized, ~75 MB total)
#
#     WEIGHTS   https://huggingface.co/openai/whisper-tiny.en/resolve/main/model.safetensors
#     TOKENIZER https://huggingface.co/openai/whisper-tiny.en/resolve/main/tokenizer.json
#     CONFIG    https://huggingface.co/openai/whisper-tiny.en/resolve/main/config.json
#     MEL       https://huggingface.co/spaces/lmz/candle-whisper/resolve/main/mel_filters.safetensors
#
#   Recommended model: whisper-tiny (multilingual, ~75 MB total)
#
#     WEIGHTS   https://huggingface.co/openai/whisper-tiny/resolve/main/model.safetensors
#     TOKENIZER https://huggingface.co/openai/whisper-tiny/resolve/main/tokenizer.json
#     CONFIG    https://huggingface.co/openai/whisper-tiny/resolve/main/config.json
#     MEL       https://huggingface.co/spaces/lmz/candle-whisper/resolve/main/mel_filters.safetensors
#
#   Quantized model: whisper-tiny.en Q8_0 (~42 MB, faster load)
#
#     WEIGHTS   https://huggingface.co/lmz/candle-whisper/resolve/main/model-tiny-en-q80.gguf
#     TOKENIZER https://huggingface.co/lmz/candle-whisper/resolve/main/tokenizer-tiny-en.json
#     CONFIG    https://huggingface.co/lmz/candle-whisper/resolve/main/config-tiny-en.json
#     MEL       https://huggingface.co/spaces/lmz/candle-whisper/resolve/main/mel_filters.safetensors
#
#   Run the download helper:
#     ./build.sh --download-assets
#
# Vite integration (karokowe-connected-devices)
# ---------------------------------------------
#   1. Copy pkg/ into your project, e.g.:
#        cp -r pkg/ frontend/karokowe-connected-devices/src/wasm/onde/
#
#   2. Add to vite.config.ts:
#        import { defineConfig } from "vite";
#        export default defineConfig({
#          optimizeDeps: { exclude: ["onde-wasm"] },
#          server: {
#            headers: {
#              "Cross-Origin-Opener-Policy":   "same-origin",
#              "Cross-Origin-Embedder-Policy": "require-corp",
#            },
#          },
#        });
#
#      NOTE: The COOP/COEP headers are NOT required by this pure-Rust WASM
#      build (no SharedArrayBuffer / pthreads are used — inference runs
#      synchronously on the JS thread).  They are shown here for completeness
#      if you later add a Web Worker wrapper.
#
#   3. In your Worker or module:
#        import init, { WhisperDecoder } from "./onde_wasm.js";
#        await init();
#        const decoder = new WhisperDecoder(
#          weightsBytes, tokenizerBytes, melFiltersBytes, configBytes,
#          false,   // quantized
#          false,   // is_multilingual
#          true,    // timestamps
#          null,    // task
#          null,    // language
#        );
#        const json = decoder.decode(wavBytes);
#        const result = JSON.parse(json);
#        // result.text       — full transcript
#        // result.segments[] — [{ start, end, text }]
#
# =============================================================================

set -euo pipefail

# ── Script location ───────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Parameters ─────────────────────────────────────────────────────────────────
BUILD_TYPE="${BUILD_TYPE:-release}"  # "release" or "debug"
OUT_DIR="${OUT_DIR:-${SCRIPT_DIR}/pkg}"

# ── Colours ───────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; CYAN=''; BOLD=''; RESET=''
fi

log_info()    { echo -e "${CYAN}[onde-wasm]${RESET} $*"; }
log_ok()      { echo -e "${GREEN}[onde-wasm]${RESET} $*"; }
log_warn()    { echo -e "${YELLOW}[onde-wasm]${RESET} $*"; }
log_error()   { echo -e "${RED}[onde-wasm] ERROR:${RESET} $*" >&2; }
log_section() { echo -e "\n${BOLD}── $* ─────────────────────────────────${RESET}"; }

# ── --download-assets helper ───────────────────────────────────────────────────
download_assets() {
    log_section "Downloading model assets"

    ASSETS_DIR="${SCRIPT_DIR}/assets"
    mkdir -p "${ASSETS_DIR}"

    declare -A URLS=(
        ["model.safetensors"]="https://huggingface.co/openai/whisper-tiny.en/resolve/main/model.safetensors"
        ["tokenizer.json"]="https://huggingface.co/openai/whisper-tiny.en/resolve/main/tokenizer.json"
        ["config.json"]="https://huggingface.co/openai/whisper-tiny.en/resolve/main/config.json"
        ["mel_filters.safetensors"]="https://huggingface.co/spaces/lmz/candle-whisper/resolve/main/mel_filters.safetensors"
    )

    for fname in "${!URLS[@]}"; do
        dest="${ASSETS_DIR}/${fname}"
        if [ -f "${dest}" ]; then
            log_info "  ${fname} — already present, skipping"
            continue
        fi
        log_info "  Downloading ${fname}…"
        curl -fL --progress-bar -o "${dest}" "${URLS[$fname]}"
        log_ok "  ${fname} saved ($(du -sh "${dest}" | cut -f1))"
    done

    log_ok "All assets in ${ASSETS_DIR}/"
    echo
    echo "  Pass these to WhisperDecoder in JS:"
    echo "    weights:     ${ASSETS_DIR}/model.safetensors"
    echo "    tokenizer:   ${ASSETS_DIR}/tokenizer.json"
    echo "    config:      ${ASSETS_DIR}/config.json"
    echo "    mel_filters: ${ASSETS_DIR}/mel_filters.safetensors"
    exit 0
}

if [[ "${1:-}" == "--download-assets" ]]; then
    download_assets
fi

# ── Preflight ──────────────────────────────────────────────────────────────────
log_section "Preflight"

if ! command -v wasm-pack &>/dev/null; then
    log_error "wasm-pack not found."
    echo
    echo "  Install with:"
    echo "    cargo install wasm-pack"
    echo
    exit 1
fi

WASM_PACK_VERSION=$(wasm-pack --version)
log_info "wasm-pack: ${WASM_PACK_VERSION}"

if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    log_warn "wasm32-unknown-unknown target not installed — installing now…"
    rustup target add wasm32-unknown-unknown
fi

log_info "Build type : ${BUILD_TYPE}"
log_info "Output dir : ${OUT_DIR}"

# ── Build ──────────────────────────────────────────────────────────────────────
log_section "Build"

WASM_PACK_PROFILE_FLAG="--release"
if [[ "${BUILD_TYPE}" == "debug" ]]; then
    WASM_PACK_PROFILE_FLAG="--dev"
fi

wasm-pack build \
    --target web \
    --out-dir "${OUT_DIR}" \
    ${WASM_PACK_PROFILE_FLAG} \
    "${SCRIPT_DIR}"

log_ok "wasm-pack build complete."

# ── Post-process: fix package.json name ───────────────────────────────────────
# wasm-pack emits the crate name as the package name; rename to onde-wasm
# so the Vite import path is predictable.
PKG_JSON="${OUT_DIR}/package.json"
if [ -f "${PKG_JSON}" ]; then
    # Use sed for portable in-place replacement (BSD and GNU sed both support this).
    sed -i.bak 's/"name": "onde_wasm"/"name": "onde-wasm"/' "${PKG_JSON}"
    rm -f "${PKG_JSON}.bak"
    log_info "package.json name → onde-wasm"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
log_section "Output"

echo
log_ok "onde-wasm build succeeded!"
echo
echo "  Artefacts:"
ls -lh "${OUT_DIR}/"*.{js,wasm,ts} 2>/dev/null \
    | grep -v '^total' \
    | awk '{printf "    %-40s %s\n", $9, $5}' \
    || true
echo
echo "  Next steps:"
echo
echo "  1. Download model assets (first time only):"
echo "       ${BASH_SOURCE[0]} --download-assets"
echo
echo "  2. Copy pkg/ into your Vite project:"
echo "       cp -r ${OUT_DIR}/. frontend/karokowe-connected-devices/src/wasm/onde/"
echo
echo "  3. Import in a Web Worker:"
echo "       import init, { WhisperDecoder } from \"./onde_wasm.js\";"
echo "       await init();"
echo "       const decoder = new WhisperDecoder("
echo "         weightsBytes, tokenizerBytes, melFiltersBytes, configBytes,"
echo "         false, false, true, null, null"
echo "       );"
echo "       const result = JSON.parse(decoder.decode(wavBytes));"
echo
