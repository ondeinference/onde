---
name: sdk-kotlin-lib
description: Build the Onde Kotlin Android library from Rust source using UniFFI bindings. Use when cross-compiling libonde.so for Android ABIs, generating Kotlin bindings, or modifying the build-kotlin.sh pipeline.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: false
---

# Skill: Onde Kotlin SDK (Android Library)

## What This Skill Covers

Building the `onde` Kotlin Android library from the Rust crate using UniFFI bindings. The output is a Kotlin/JVM library module (`sdk/kotlin/onde/`) with:

- Pre-compiled `.so` native libraries for all four Android ABIs (`arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`) under `jniLibs/`
- Auto-generated Kotlin source at `sdk/kotlin/onde/src/main/kotlin/com/ondeinference/onde/onde.kt`

## Crate & Tooling Layout

```
onde/
├── Cargo.toml                 # crate-type = ["lib", "cdylib", "staticlib"]
├── uniffi.toml                # [bindings.kotlin] package_name = "com.ondeinference.onde"
├── build-kotlin.sh            # main build script — cross-compiles + runs bindgen
├── uniffi-bindgen/
│   ├── Cargo.toml             # bin: uniffi-bindgen, dep: uniffi = "=0.31.0", features = ["cli"]
│   └── uniffi-bindgen.rs      # fn main() { uniffi::uniffi_bindgen_main() }
└── sdk/kotlin/onde/
    └── src/main/
        ├── jniLibs/<ABI>/libonde.so
        └── kotlin/com/ondeinference/onde/onde.kt
```

## Key Constraints

- **UniFFI is pinned to `=0.31.0`** in both `Cargo.toml` (lib) and `uniffi-bindgen/Cargo.toml`. Never change this independently — both must stay in sync.
- **`cdylib` crate-type** is required for the `.so` output consumed by the JVM at runtime.
- **`uniffi::setup_scaffolding!()`** in `src/lib.rs` is the proc-macro approach — no `.udl` file is needed.
- **Android NDK minimum API = 24** (`MIN_API=24` in `build-kotlin.sh`) to match the library's `minSdk`.
- **Package name is `com.ondeinference.onde`** — set in `uniffi.toml`, never override in bindgen flags.

## Build Command

```bash
# Full release build for all four Android ABIs
./build-kotlin.sh

# Debug build, arm64 only
./build-kotlin.sh --debug --target aarch64-linux-android

# Regenerate Kotlin source from an existing .so (skip Rust compilation)
./build-kotlin.sh --generate-only
```

The script:
1. Builds the `onde-uniffi-bindgen` CLI binary (host).
2. Cross-compiles `libonde.so` for each selected Android target using the NDK clang toolchain.
3. Copies `.so` files into `sdk/kotlin/onde/src/main/jniLibs/<ABI>/`.
4. Runs `uniffi-bindgen generate --library <.so> --language kotlin --out-dir <src_root> --config uniffi.toml`.

## NDK Setup

The script auto-detects the NDK in order:
1. `$ANDROID_NDK_HOME`
2. `$ANDROID_HOME/ndk/` (picks newest version)
3. `~/Library/Android/sdk/ndk/` (macOS default)
4. `~/Android/Sdk/ndk/` (Linux default)

Or pass explicitly: `./build-kotlin.sh --ndk /path/to/ndk`

Required Rust targets:
```bash
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  x86_64-linux-android \
  i686-linux-android
```

## UniFFI Binding Rules

All types that cross the FFI boundary must carry the appropriate UniFFI derive:

| Rust trait/derive         | Kotlin output             |
|---------------------------|---------------------------|
| `#[derive(uniffi::Record)]` | `data class`             |
| `#[derive(uniffi::Enum)]`   | `sealed class` / `enum`  |
| `#[derive(uniffi::Error)]`  | `sealed class` (throwable)|
| `#[derive(uniffi::Object)]` | `class` (Arc-backed)     |
| `#[uniffi::export(callback_interface)]` | `interface`  |

**Forbidden in UniFFI-exported types:**
- `usize` / `isize` — use `u64` / `i64` instead
- `impl Into<String>` — use concrete `String`
- `tokio::sync::mpsc::Receiver` — use `callback_interface` + free function
- Callback interfaces as parameters of `#[derive(uniffi::Object)]` methods — must be free functions

## Adding a New Exported Function or Type

1. Add the type/function in the relevant `src/` file with the correct UniFFI derive or `#[uniffi::export]`.
2. Verify `uniffi::setup_scaffolding!()` is still the only scaffolding call in `src/lib.rs`.
3. Re-run `./build-kotlin.sh --generate-only` (if a compiled `.so` already exists) or the full build.
4. The new class/function will appear in `sdk/kotlin/onde/src/main/kotlin/com/ondeinference/onde/onde.kt`.
5. Never hand-edit `onde.kt` — it is always fully regenerated.

## Common Pitfalls

- **Version mismatch between `uniffi` in `Cargo.toml` and `uniffi-bindgen/Cargo.toml`** — bindgen will refuse to process the library and emit a schema version error. Both must be `=0.31.0`.
- **Missing `cdylib` in `crate-type`** — `cargo build` will not produce a `.so`. The `Cargo.toml` must keep `["lib", "cdylib", "staticlib"]`.
- **`___chkstk_darwin` linker error** — only occurs on tvOS, not Android. Ignore on Android builds.
- **Android sandbox has no `HOME`** — `dirs::home_dir()` returns `None` inside the Android sandbox. On Android, `hf-hub` is a direct dependency (not optional) so that the HF cache path can be set explicitly via `GLOBAL_HF_CACHE` before any model download.
- **`libonde.so` not found after build** — the script searches multiple candidate paths including workspace `target/`. If the workspace `target/` is above the crate directory, the fallback path logic in `build-kotlin.sh` will find it.
