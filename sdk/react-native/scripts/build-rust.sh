#!/usr/bin/env bash
set -euo pipefail

# Build the Rust FFI bridge for React Native targets.
#
# Usage:
#   ./scripts/build-rust.sh ios       # Build for iOS device + simulator
#   ./scripts/build-rust.sh android   # Build for Android architectures
#   ./scripts/build-rust.sh all       # Build for all targets

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_DIR="$SCRIPT_DIR/../rust"
IOS_OUT="$SCRIPT_DIR/../ios/rust"
ANDROID_OUT="$SCRIPT_DIR/../android/src/main/jniLibs"

build_ios() {
    echo "=== Building Rust for iOS ==="

    # Device (arm64)
    echo "  → aarch64-apple-ios"
    cargo build --manifest-path "$RUST_DIR/Cargo.toml" \
        --target aarch64-apple-ios --release

    # Simulator (arm64 Apple Silicon)
    echo "  → aarch64-apple-ios-sim"
    cargo build --manifest-path "$RUST_DIR/Cargo.toml" \
        --target aarch64-apple-ios-sim --release

    # Copy both device and simulator staticlibs to ios/rust/{device,simulator}/
    mkdir -p "$IOS_OUT/device" "$IOS_OUT/simulator"
    cp "$RUST_DIR/target/aarch64-apple-ios/release/libonde_react_native.a" \
       "$IOS_OUT/device/libonde_react_native.a"
    cp "$RUST_DIR/target/aarch64-apple-ios-sim/release/libonde_react_native.a" \
       "$IOS_OUT/simulator/libonde_react_native.a"

    # Create an XCFramework that bundles both slices. CocoaPods picks the
    # correct one automatically based on the target SDK.
    echo "  → Creating XCFramework"
    rm -rf "$IOS_OUT/OndeReactNative.xcframework"
    xcodebuild -create-xcframework \
        -library "$IOS_OUT/device/libonde_react_native.a" \
        -library "$IOS_OUT/simulator/libonde_react_native.a" \
        -output "$IOS_OUT/OndeReactNative.xcframework"

    echo "  ✓ iOS XCFramework created at ios/rust/OndeReactNative.xcframework"
}

build_android() {
    echo "=== Building Rust for Android ==="

    # Verify NDK is available
    if [ -z "${ANDROID_NDK_HOME:-}" ]; then
        echo "Error: ANDROID_NDK_HOME is not set."
        echo "Set it to your Android NDK installation path, e.g.:"
        echo "  export ANDROID_NDK_HOME=\$HOME/Library/Android/sdk/ndk/<version>"
        exit 1
    fi

    TARGETS=(
        "aarch64-linux-android:arm64-v8a"
        "armv7-linux-androideabi:armeabi-v7a"
        "x86_64-linux-android:x86_64"
        "i686-linux-android:x86"
    )

    for entry in "${TARGETS[@]}"; do
        RUST_TARGET="${entry%%:*}"
        ABI="${entry##*:}"

        echo "  → $RUST_TARGET ($ABI)"
        cargo build --manifest-path "$RUST_DIR/Cargo.toml" \
            --target "$RUST_TARGET" --release

        mkdir -p "$ANDROID_OUT/$ABI"
        cp "$RUST_DIR/target/$RUST_TARGET/release/libonde_react_native.so" \
           "$ANDROID_OUT/$ABI/libonde_react_native.so"
    done

    echo "  ✓ Android shared libraries copied to android/src/main/jniLibs/"
}

case "${1:-all}" in
    ios)     build_ios ;;
    android) build_android ;;
    all)     build_ios; build_android ;;
    *)
        echo "Usage: $0 {ios|android|all}"
        exit 1
        ;;
esac

echo ""
echo "Build complete."
