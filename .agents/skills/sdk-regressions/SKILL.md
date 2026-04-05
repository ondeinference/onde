---
name: sdk-regressions
description: Cross-compilation regression checks for the onde crate. Covers macOS, iOS, tvOS (nightly), Android, Windows, and Linux target verification. Run before merging any change that touches platform-gated code, Cargo.toml, .cargo/config.toml, or build.rs.
allowed-tools: Read, Edit, Glob, Grep, Terminal
user-invocable: true
---

# SKILL: SDK Cross-Compilation Regression Checks

> Always run these checks before merging changes that touch `onde/src/`,
> `onde/Cargo.toml`, `onde/.cargo/config.toml`, `onde/build.rs`, or any
> platform-gated dependency.

---

## Why This Exists

The `onde` crate compiles for **6 target OSes** with heavy `#[cfg(target_os)]`
branching (106+ occurrences in `engine.rs` alone). A change that compiles on
macOS can silently break iOS, tvOS, or Android because:

- Different `mistralrs` feature sets per platform (`["metal"]` vs `[]`)
- Platform-conditional dependencies (`hf-hub` only on Android)
- Tier-3 targets (tvOS) require nightly + `-Z build-std`
- Assembly stubs (`tvos_chkstk.s`) and `build.rs` gating
- `.cargo/config.toml` sets `+fp16` rustflags on 5 Apple targets

---

## Tier 1 — Must Pass on Every PR (stable toolchain)

These use the default stable toolchain and must always succeed:

```bash
# macOS — host platform, Metal backend
cargo check

# iOS device — Metal backend, +fp16
cargo check --target aarch64-apple-ios

# iOS simulator — Metal backend, +fp16
cargo check --target aarch64-apple-ios-sim
```

### Quick one-liner

```bash
cargo check && \
cargo check --target aarch64-apple-ios && \
cargo check --target aarch64-apple-ios-sim
```

---

## Tier 2 — tvOS (nightly toolchain, `-Z build-std`)

tvOS targets are tier-3 in Rust — they are **not** in `rustup target list` and
require nightly with `-Z build-std` and the `rust-src` component.

```bash
# tvOS device
cargo +nightly check -Z build-std --target aarch64-apple-tvos

# tvOS simulator
cargo +nightly check -Z build-std --target aarch64-apple-tvos-sim
```

### tvOS-specific regression risks

| Risk | What breaks | How to detect |
|------|-------------|---------------|
| `scripts/tvos_chkstk.s` deleted | `Undefined symbol: ___chkstk_darwin` at link time | `test -f scripts/tvos_chkstk.s` |
| `cc` build-dep removed from `Cargo.toml` | `build.rs` can't compile the assembly stub | `grep 'cc' Cargo.toml` |
| `build.rs` tvOS block removed | Same linker error | `grep 'tvos' build.rs` |
| nightly `rust-src` not installed | `-Z build-std` fails immediately | `rustup +nightly component list --installed \| grep rust-src` |
| `.cargo/config.toml` `+fp16` removed | Silent precision loss or candle kernel panics | `grep 'fp16' .cargo/config.toml` |

---

## Tier 3 — Android, Windows, Linux (stable, may need cross-linker)

These check that CPU-only code paths compile. They may fail on linker resolution
if cross-linkers aren't installed, but `cargo check` (no linking) should pass:

```bash
# Android arm64 — CPU, requires hf-hub sandbox workaround
cargo check --target aarch64-linux-android

# Windows — CPU
cargo check --target x86_64-pc-windows-msvc

# Linux — CPU
cargo check --target x86_64-unknown-linux-gnu
```

---

## Full Regression Suite (copy-paste)

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "═══ Tier 1: macOS + iOS (stable) ═══"
cargo check
cargo check --target aarch64-apple-ios
cargo check --target aarch64-apple-ios-sim

echo "═══ Tier 2: tvOS (nightly + build-std) ═══"
cargo +nightly check -Z build-std --target aarch64-apple-tvos
cargo +nightly check -Z build-std --target aarch64-apple-tvos-sim

echo "═══ Tier 3: Android / Windows / Linux ═══"
cargo check --target aarch64-linux-android
cargo check --target x86_64-pc-windows-msvc
cargo check --target x86_64-unknown-linux-gnu

echo "✅ All cross-compilation checks passed"
```

---

## Required Rustup Targets

Install these once on a development machine:

```bash
# Tier 1 (stable)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# Tier 3 (stable)
rustup target add aarch64-linux-android armv7-linux-androideabi
rustup target add x86_64-linux-android i686-linux-android
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu

# Tier 2 — tvOS (nightly, no `rustup target add` — uses -Z build-std)
rustup toolchain install nightly
rustup +nightly component add rust-src
```

---

## `.cargo/config.toml` — Critical Settings

These rustflags are set at the workspace level and **must not be removed**:

```toml
[target.aarch64-apple-ios]
rustflags = ["-C", "target-feature=+fp16"]

[target.aarch64-apple-ios-sim]
rustflags = ["-C", "target-feature=+fp16"]

[target.aarch64-apple-tvos]
rustflags = ["-C", "target-feature=+fp16"]

[target.aarch64-apple-tvos-sim]
rustflags = ["-C", "target-feature=+fp16"]

[target.x86_64-apple-tvos]
rustflags = ["-C", "target-feature=+fp16"]
```

Removing `+fp16` causes silent precision degradation or runtime panics in
candle's Metal/NEON kernels.

---

## Platform Dependency Matrix

| `target_os` | `mistralrs` features | Extra deps | GPU | Default model |
|-------------|---------------------|------------|-----|---------------|
| `macos` | `["metal"]` | — | Metal | Qwen 2.5 3B |
| `ios` | `["metal"]` | `hf-hub`, `mistralrs-core` | Metal | Qwen 2.5 1.5B |
| `tvos` | `["metal"]` | `hf-hub`, `mistralrs-core` | Metal | Qwen 2.5 1.5B |
| `android` | `[]` | `hf-hub`, `mistralrs-core` | CPU | Qwen 2.5 1.5B |
| `windows` | `[]` | — | CPU | Qwen 2.5 3B |
| `linux` | `[]` | — | CPU | Qwen 2.5 3B |

`hf-hub` and `mistralrs-core` are required on all **sandboxed** platforms
(iOS, tvOS, Android) for the `GLOBAL_HF_CACHE` workaround — `~/.cache` is
outside the app container on iOS/tvOS, and `dirs::home_dir()` panics on Android.
The re-exports in `src/lib.rs` (`pub use hf_hub; pub use mistralrs_core;`) are
gated to `#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]`.

---

## Common Regression Patterns

### 1. New model added but not to all 6 platform `cfg` blocks

`engine.rs` has separate `#[cfg(target_os = "...")]` blocks for model builder
logic. A new model constructor added only under `macos` will fail on `ios`.

**Detection:** Tier 1 checks catch iOS; Tier 2 catches tvOS; Tier 3 catches
Android/Windows/Linux.

### 2. Dependency added without platform gating

Adding a dep that uses `SystemConfiguration.framework` (e.g., `hyper_util`
proxy detection) works on macOS but causes linker errors on iOS/tvOS unless the
podspec declares `s.frameworks = 'SystemConfiguration'`.

**Detection:** Tier 1 `cargo check` passes (no linking), but `flutter build ios`
fails. Must also test a full `flutter build` periodically.

### 3. Fork branch divergence

All `mistralrs` deps point to a fork:
`git = "https://github.com/setoelkahfi/mistral.rs"`, branch
`fix/all-platform-fixes`. If that branch is rebased, deleted, or diverges from
upstream, **all** platforms break.

**Detection:** Any `cargo check` fails with "could not find branch".

### 4. UniFFI version mismatch

`uniffi = "=0.31.0"` must be pinned identically in three places:
- `onde/Cargo.toml` dependencies
- `onde/Cargo.toml` build-dependencies
- `onde/uniffi-bindgen/Cargo.toml`

Mixing versions causes bindgen panics at codegen time.

**Detection:** `cargo build` in `uniffi-bindgen/` or `build.rs` panics.

### 5. Android `home_dir()` sandbox panic

`dirs::home_dir()` panics in the Android sandbox. The `hf-hub` dep is added
explicitly on Android so `HF_HOME` can be set programmatically. Never call
`dirs::home_dir()` or `home::home_dir()` in code paths reachable on Android.

**Detection:** Tier 3 `cargo check --target aarch64-linux-android` catches
compile-time issues; runtime panics need an Android emulator test.

---

## Dart SDK Cross-Build Checks

When changes touch `sdk/dart/`, also verify the Flutter plugin builds:

```bash
cd sdk/dart/example

# macOS (requires Xcode)
flutter build macos --debug

# iOS (requires Xcode + iOS SDK)
flutter build ios --debug --no-codesign
```

### Known environment pitfall

**Never export Android NDK toolchain binaries globally in `.zshrc`:**

```bash
# ❌ BREAKS Xcode builds — NDK's ld/ar/strip shadow Apple's tools
export LD=$TOOLCHAIN/bin/ld
export AR=$TOOLCHAIN/bin/llvm-ar

# ✅ Safe — env vars only, no PATH/tool overrides
export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/29.0.14206865"
```

If Xcode invokes the NDK's `ld` instead of Apple's, you get:
`ld: error: unknown argument '-Xlinker'`

The fix: comment out the tool exports, use Cargo's
`sdk/dart/rust/.cargo/config.toml` for Android linker configuration instead.

---

## Cache Path Consistency (Cross-SDK)

All Onde-powered apps share downloaded models via the App Group
`group.com.ondeinference.apps`.  The cache layout inside the shared container
**must be identical** across every SDK and app type:

```
<group container>/
├── models/          ← HF_HOME
│   └── hub/         ← HF_HUB_CACHE (GGUF files live here)
└── tmp/             ← TMPDIR (iOS sandbox restricts system TMPDIR)
```

**The subdirectory is `models/`, NOT `huggingface/`.**

Files that set this path (all must agree):

| File | Variable | Expected value |
|------|----------|----------------|
| `onde/sdk/dart/rust/src/api.rs` (`configure_cache_dir`) | `hf_home` | `data_dir.join("models")` |
| `onde/src/hf_cache.rs` (`download_model` — general) | `hf_home` | `data_dir.join("models")` |
| `onde/src/hf_cache.rs` (`download_model` — Android) | `hf_home` | `resolved_app_data.join("models")` |
| Tauri apps (`setup_application_filesystem.rs`) | `models_home` | `container_dir.join("models")` |
| `onde/src/hf_cache.rs` (`hf_cache_dir` — fallback) | default | `~/.cache/huggingface/hub` (non-sandboxed only, OK to differ) |

**Regression check:** `grep -rn 'join("huggingface")' src/hf_cache.rs sdk/dart/rust/src/api.rs`
should return **only** the non-sandboxed fallback in `hf_cache_dir()` (line ~164).
Any other match means a sandboxed path is using the wrong subdirectory.

---

*Last updated: July 2025 — added hf-hub/mistralrs-core iOS/tvOS deps,
cache path consistency checks, App Group convention.*

