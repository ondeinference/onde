cmake_minimum_required(VERSION 3.18)
project(onde_inference_dart)

# ── Rust source directory ─────────────────────────────────────────────────────
# CMakeLists.txt lives in android/, so ../rust resolves to the sibling rust/ dir.
set(RUST_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../rust")

# ── Map Android ABI → Cargo target triple ────────────────────────────────────
if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(CARGO_TARGET "aarch64-linux-android")
elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
    set(CARGO_TARGET "armv7-linux-androideabi")
elseif(ANDROID_ABI STREQUAL "x86_64")
    set(CARGO_TARGET "x86_64-linux-android")
elseif(ANDROID_ABI STREQUAL "x86")
    set(CARGO_TARGET "i686-linux-android")
else()
    message(FATAL_ERROR "Unsupported Android ABI: ${ANDROID_ABI}")
endif()

# ── Output path for the compiled Rust shared library ─────────────────────────
set(LIB_OUTPUT
    "${RUST_DIR}/target/${CARGO_TARGET}/release/libonde_inference_dart.so"
)

# ── Custom build step: invoke cargo ──────────────────────────────────────────
# Requires the Android NDK cross-compilation toolchain to be on PATH, or
# a .cargo/config.toml in the Rust crate with the correct linker overrides
# for each Android target.
add_custom_command(
    OUTPUT  ${LIB_OUTPUT}
    COMMAND cargo build
                --release
                --target ${CARGO_TARGET}
                --manifest-path "${RUST_DIR}/Cargo.toml"
    WORKING_DIRECTORY ${RUST_DIR}
    COMMENT "Building onde_inference_dart for ${CARGO_TARGET}…"
    VERBATIM
)

# Dummy target that depends on the cargo output so CMake tracks it correctly.
add_custom_target(onde_inference_dart_cargo_build ALL
    DEPENDS ${LIB_OUTPUT}
)

# ── Import the built .so as a pre-built shared library ───────────────────────
add_library(onde_inference_dart SHARED IMPORTED GLOBAL)
set_target_properties(onde_inference_dart PROPERTIES
    IMPORTED_LOCATION ${LIB_OUTPUT}
)
add_dependencies(onde_inference_dart onde_inference_dart_cargo_build)
