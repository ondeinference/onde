## 1.5.1

- No changes to this SDK. The release adds `@ondeinference/web`, a new npm package for running Onde in the browser.

## 1.5.0

- No changes to this SDK. The release adds the Rust crate's browser engine for wasm32, which these bindings don't use.

## 1.4.0

- No changes to this SDK. The release carries the Rust crate's new opt-in `diffusion` feature, which these bindings do not enable.

## 1.3.5

- Fixes the 32-bit Android (`armv7`) build by moving to `onde-mistralrs-core` `0.9.6`. `0.9.5` overflowed a 32-bit `usize` in the memory plan and did not compile for `armv7-linux-androideabi`.
- Fixes `scripts/generate-bindings.sh`, which looked for `uniffi-bindgen` in `uniffi-bindgen/target/release/`. Since onde became a Cargo workspace the binary lands in the root `target/release/`.
- First release on Maven Central since `1.3.2`. `1.3.3` and `1.3.4` were tagged but never published there.

## 1.3.4

- The core now builds on `onde-mistralrs` `0.9.5`. Android and JVM behaviour is unchanged.
- Not published to Maven Central: the `armv7` build failed. These changes first shipped in `1.3.5`.

## 1.3.3

- No Kotlin API changes. `onde` became a Cargo workspace, which moved the `uniffi-bindgen` binary to the root `target/` directory.
- Not published to Maven Central.
