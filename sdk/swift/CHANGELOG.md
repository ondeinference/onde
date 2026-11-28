## 1.6.0

- No changes to this SDK. The release moves the Rust crate's browser engine to vectoron and adds `@ondeinference/web`, neither of which these bindings use.

## 1.5.0

- No changes to this SDK. The release adds the Rust crate's browser engine for wasm32, which these bindings don't use.

## 1.4.0

- No changes to this SDK. The release carries the Rust crate's new opt-in `diffusion` feature, which the XCFramework is built without.

## 1.3.5

- Fixes the XCFramework build. `build-swift-xcframework.sh` looked for `uniffi-bindgen` in `uniffi-bindgen/target/release/`, but since onde became a Cargo workspace the binary lands in the root `target/release/`, so the script failed at the first `generate` call.
- First release on `onde-swift` since `1.3.2`. `1.3.3` and `1.3.4` were tagged but never published there.

## 1.3.4

- The core now builds on `onde-mistralrs` `0.9.5`, which restores Metal builds for tvOS, visionOS and watchOS.
- Paged attention is turned off on iOS, tvOS, visionOS and watchOS, where its KV block pool does not fit in memory. macOS is unchanged.
- On tvOS the auto device map override is replaced by a 512-token maximum model length.
- Not published to `onde-swift`. These changes first shipped in `1.3.5`.

## 1.3.3

- No Swift API changes. `onde` became a Cargo workspace, which moved the `uniffi-bindgen` binary to the root `target/` directory.
- Not published to `onde-swift`. The XCFramework script still used the old path and failed.
