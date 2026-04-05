cmake_minimum_required(VERSION 3.14)
project(onde_inference LANGUAGES CXX)

set(PLUGIN_NAME "onde_inference_plugin")
set(RUST_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../rust")

# -- Rust build step ----------------------------------------------------------
set(RUST_LIB_OUTPUT "${RUST_DIR}/target/release/libonde_inference_dart.so")

add_custom_command(
    OUTPUT  "${RUST_LIB_OUTPUT}"
    COMMAND cargo build
                --release
                --manifest-path "${RUST_DIR}/Cargo.toml"
    WORKING_DIRECTORY "${RUST_DIR}"
    COMMENT "Building onde_inference_dart Rust bridge for Linux..."
    VERBATIM
)

add_custom_target(onde_inference_rust_build ALL
    DEPENDS "${RUST_LIB_OUTPUT}"
)

# -- Flutter plugin stub ------------------------------------------------------
add_library(${PLUGIN_NAME} SHARED
    "onde_inference_plugin.cc"
)

apply_standard_settings(${PLUGIN_NAME})

set_target_properties(${PLUGIN_NAME} PROPERTIES
    CXX_VISIBILITY_PRESET hidden
)

target_compile_definitions(${PLUGIN_NAME} PRIVATE FLUTTER_PLUGIN_IMPL)

target_include_directories(${PLUGIN_NAME} INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

target_link_libraries(${PLUGIN_NAME} PRIVATE flutter)
target_link_libraries(${PLUGIN_NAME} PRIVATE PkgConfig::GTK)

add_dependencies(${PLUGIN_NAME} onde_inference_rust_build)

# -- Installation -------------------------------------------------------------
install(FILES "${RUST_LIB_OUTPUT}"
    DESTINATION "${CMAKE_INSTALL_PREFIX}"
)

set(onde_inference_plugin_bundled_libraries
    "${RUST_LIB_OUTPUT}"
    PARENT_SCOPE
)
