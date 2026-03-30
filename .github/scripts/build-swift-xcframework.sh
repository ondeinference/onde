#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT_DIR"

DIST_DIR="$ROOT_DIR/dist/swift"
PACKAGE_DIR="$DIST_DIR/Package"
HEADERS_DIR="$DIST_DIR/Headers"
FRAMEWORK_DIR="$DIST_DIR/OndeFramework.xcframework"
ZIP_PATH="$DIST_DIR/OndeFramework.xcframework.zip"
CHECKSUM_PATH="$DIST_DIR/OndeFramework.checksum.txt"
VERSION_PATH="$DIST_DIR/version.txt"
BINDGEN="$ROOT_DIR/uniffi-bindgen/target/release/uniffi-bindgen"

mkdir -p "$DIST_DIR" "$PACKAGE_DIR/Sources/Onde" "$HEADERS_DIR"

cargo +1.92.0 build --manifest-path uniffi-bindgen/Cargo.toml --release

# Build staticlibs only. Avoid the cdylib link step; the XCFramework consumes .a slices.
cargo +1.92.0 rustc --target aarch64-apple-ios --release --lib --crate-type staticlib
cargo +1.92.0 rustc --target aarch64-apple-ios-sim --release --lib --crate-type staticlib
cargo +1.92.0 rustc --target aarch64-apple-darwin --release --lib --crate-type staticlib
cargo +nightly rustc -Z build-std --target aarch64-apple-tvos --release --lib --crate-type staticlib
cargo +nightly rustc -Z build-std --target aarch64-apple-tvos-sim --release --lib --crate-type staticlib

"$BINDGEN" generate   "$ROOT_DIR/target/aarch64-apple-ios/release/libonde.a"   --language swift   --out-dir "$PACKAGE_DIR/Sources/Onde"

cp "$PACKAGE_DIR/Sources/Onde/ondeFFI.h" "$HEADERS_DIR/ondeFFI.h"
cp "$PACKAGE_DIR/Sources/Onde/ondeFFI.modulemap" "$HEADERS_DIR/module.modulemap"

cp "$ROOT_DIR/target/aarch64-apple-ios-sim/release/libonde.a" "$DIST_DIR/libonde-ios-sim.a"

cp "$ROOT_DIR/target/aarch64-apple-tvos-sim/release/libonde.a" "$DIST_DIR/libonde-tvos-sim.a"

cp "$ROOT_DIR/target/aarch64-apple-darwin/release/libonde.a" "$DIST_DIR/libonde-macos.a"

xcodebuild -create-xcframework   -library "$ROOT_DIR/target/aarch64-apple-ios/release/libonde.a" -headers "$HEADERS_DIR"   -library "$DIST_DIR/libonde-ios-sim.a" -headers "$HEADERS_DIR"   -library "$ROOT_DIR/target/aarch64-apple-tvos/release/libonde.a" -headers "$HEADERS_DIR"   -library "$DIST_DIR/libonde-tvos-sim.a" -headers "$HEADERS_DIR"   -library "$DIST_DIR/libonde-macos.a" -headers "$HEADERS_DIR"   -output "$FRAMEWORK_DIR"

export FRAMEWORK_DIR ZIP_PATH
python3 - <<'ZIP'
import os
import zipfile
from pathlib import Path
root = Path(os.environ['FRAMEWORK_DIR'])
out = Path(os.environ['ZIP_PATH'])
with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
    for folder, _, files in os.walk(root):
        for name in files:
            path = Path(folder) / name
            zf.write(path, path.relative_to(root.parent))
ZIP

CHECKSUM=$(swift package compute-checksum "$ZIP_PATH")
printf '%s
' "$CHECKSUM" > "$CHECKSUM_PATH"

VERSION=$(python3 - <<'VER'
from pathlib import Path
import tomllib
cargo = Path('Cargo.toml').read_text().encode()
print(tomllib.loads(cargo.decode())['package']['version'])
VER
)
printf '%s
' "$VERSION" > "$VERSION_PATH"

echo "XCFramework: $FRAMEWORK_DIR"
echo "Zip: $ZIP_PATH"
echo "Checksum: $CHECKSUM"
echo "Version: $VERSION"
