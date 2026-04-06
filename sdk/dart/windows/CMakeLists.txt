cmake_minimum_required(VERSION 3.14)
project(onde_inference_plugin VERSION 0.1.0 LANGUAGES CXX)

# ── Flutter plugin boilerplate ────────────────────────────────────────────────
set(PLUGIN_NAME "onde_inference_plugin")

# ── Rust source directory ─────────────────────────────────────────────────────
# CMakeLists.txt lives in windows/, so ../rust resolves to the sibling rust/ dir.
set(RUST_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../rust")

# ── Determine build profile ───────────────────────────────────────────────────
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
  set(CARGO_PROFILE "debug")
  set(CARGO_PROFILE_FLAG "")
else()
  set(CARGO_PROFILE "release")
  set(CARGO_PROFILE_FLAG "--release")
endif()

# ── Output paths for the compiled Rust DLL ───────────────────────────────────
# On Windows, cargo cdylib outputs are named without a "lib" prefix:
#   onde_inference_dart.dll  (not libonde_inference_dart.dll)
set(RUST_DLL
    "${RUST_DIR}/target/${CARGO_PROFILE}/onde_inference_dart.dll"
)
set(RUST_IMPLIB
    "${RUST_DIR}/target/${CARGO_PROFILE}/onde_inference_dart.dll.lib"
)

# ── Custom build step: invoke cargo ──────────────────────────────────────────
# Requires Rust + the current host toolchain (msvc or gnu) to be on PATH.
# For CUDA builds, append --features cuda to CARGO_PROFILE_FLAG.
add_custom_command(
  OUTPUT  "${RUST_DLL}" "${RUST_IMPLIB}"
  COMMAND cargo build
              ${CARGO_PROFILE_FLAG}
              --manifest-path "${RUST_DIR}/Cargo.toml"
  WORKING_DIRECTORY "${RUST_DIR}"
  COMMENT "Building onde_inference_dart Rust bridge (${CARGO_PROFILE})…"
  VERBATIM
)

add_custom_target(onde_inference_dart_cargo_build ALL
  DEPENDS "${RUST_DLL}"
)

# ── Import the Rust DLL as a pre-built shared library ───────────────────────
add_library(onde_inference_dart SHARED IMPORTED GLOBAL)
set_target_properties(onde_inference_dart PROPERTIES
  IMPORTED_LOCATION    "${RUST_DLL}"
  IMPORTED_IMPLIB      "${RUST_IMPLIB}"
)
add_dependencies(onde_inference_dart onde_inference_dart_cargo_build)

# ── Flutter plugin shared library ────────────────────────────────────────────
add_library(${PLUGIN_NAME} SHARED
  "onde_inference_plugin.cpp"
)

apply_standard_settings(${PLUGIN_NAME})

target_compile_definitions(${PLUGIN_NAME} PRIVATE FLUTTER_PLUGIN_IMPL)

target_include_directories(${PLUGIN_NAME} INTERFACE
  "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

target_link_libraries(${PLUGIN_NAME} PRIVATE
  flutter
  flutter_wrapper_plugin
  onde_inference_dart
)

add_dependencies(${PLUGIN_NAME} onde_inference_dart_cargo_build)

# ── Copy the Rust DLL next to the Flutter runner executable ──────────────────
# Flutter's build system picks up files installed to the install prefix, so
# we install the DLL there so it is packaged with the final app bundle.
install(FILES "${RUST_DLL}"
  DESTINATION "${CMAKE_INSTALL_PREFIX}"
)

# ── Bundled libraries declaration ─────────────────────────────────────────────
# Tells the Flutter tooling to bundle these DLLs with the distributed app.
set(onde_inference_plugin_bundled_libraries
  "${RUST_DLL}"
  PARENT_SCOPE
)
