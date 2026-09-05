---
name: sdk-regressions
description: Cross-compilation regression checks for the onde crate. Covers macOS, iOS, tvOS (nightly), Android, Windows, and Linux target verification. Run before merging any change that touches platform-gated code, Cargo.toml, .cargo/config.toml, or build.rs. Also documents the onde-mistralrs fork publishing strategy and cargo publish workflow.
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

See also: **[`cargo publish` workflow](#cargo-publish-workflow)** at the bottom
of this file — must be followed when releasing a new `onde` version.

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

## Tier 3 — Android, Windows, Linux (stable, needs cross toolchains)

These check that CPU-only code paths compile. For this crate, `cargo check`
still runs C build scripts from dependencies such as `aws-lc-sys` and `ring`,
so absent cross GCC toolchains are real blockers, not harmless link-only
failures.

```bash
# Android arm64 — CPU, requires hf-hub sandbox workaround
cargo check --target aarch64-linux-android

# Windows MSVC — CPU, exact CI target; run on Windows or a Windows CI runner
cargo check --target x86_64-pc-windows-msvc

# Linux — CPU
cargo check --target x86_64-unknown-linux-gnu
```

On macOS hosts without cross GCC installed, expect Linux checks to fail looking
for `x86_64-linux-gnu-gcc`, and Windows GNU checks to fail looking for
`x86_64-w64-mingw32-gcc`. Use the Colima/Docker flow below instead of treating
those as code regressions.

### macOS host fallback: Colima/Docker

Use this when the goal is to run the Linux and Windows cross-checks on a macOS
host and the host does not have cross GCC toolchains. On Apple Silicon, run a
native `linux/arm64` container. Avoid `linux/amd64` emulation for this crate;
the dependency build can crash under QEMU before reaching `onde` code.

Give Colima enough memory before running these checks:

```bash
colima status
colima stop
colima start --memory 8
```

Run Linux and the local Windows GNU proxy check in the container:

```bash
docker run --rm --platform linux/arm64 \
  -v "$PWD:/workspace" \
  -v onde-cargo-registry:/usr/local/cargo/registry \
  -v onde-cargo-git:/usr/local/cargo/git \
  -w /workspace \
  rust:1.92-bookworm \
  bash -lc '
    export PATH=/usr/local/cargo/bin:$PATH
    apt-get update >/dev/null
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc-x86-64-linux-gnu \
      libc6-dev-amd64-cross \
      linux-libc-dev-amd64-cross \
      gcc-mingw-w64-x86-64 \
      binutils-mingw-w64-x86-64 >/dev/null
    rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu >/dev/null
    CARGO_BUILD_JOBS=1 cargo check --target x86_64-unknown-linux-gnu
    CARGO_BUILD_JOBS=1 cargo check --target x86_64-pc-windows-gnu
  '
```

`x86_64-pc-windows-gnu` exercises `#[cfg(target_os = "windows")]` and the
CPU-only Windows path locally, but it is not a substitute for the CI
`x86_64-pc-windows-msvc` check. The MSVC ABI check needs a Windows/MSVC
toolchain, normally GitHub Actions `windows-latest`.

If the container reaches `Checking onde ...` and then reports Rust errors, the
cross toolchain is working. Fix the crate error and rerun the check.

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
rustup target add x86_64-pc-windows-gnu
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

---

## Publishing to crates.io

### Why we don't use upstream `mistralrs` directly

The short version: our PRs aren't merged yet.

`onde` depends on a personal fork of Eric Buehler's `mistral.rs`, kept at
`github.com/setoelkahfi/mistral.rs` on the `fix/all-platform-fixes` branch.
The fork is ahead of upstream by ~41 commits carrying fixes that only matter
for Apple and Android targets — things upstream hasn't needed to care about:

| Fix | Status |
|-----|--------|
| iOS Metal 3.0 support | PR open |
| Android 32-bit memory limit constants | PR open |
| HF_HOME propagation in diffusion/flux pipelines | PR open |
| `metallib` link step `--sdk` / `-std` flags | PR open |

Dropping the fork and pointing at upstream `mistralrs 0.8.1` would silently
break iOS on entry-level devices and Android on 32-bit targets. Don't do it
until those PRs land.

**Legal note:** The fork is a derivative of Eric Buehler's MIT-licensed
`mistral.rs`. His `LICENSE` file — including `Copyright (c) 2024 Eric Buehler`
— must remain intact and unmodified in the fork at all times. See
[`legal-and-trademarks/SKILL.md`](../legal-and-trademarks/SKILL.md) for full
attribution obligations.

### The `onde-mistralrs` workaround

> For the full inventory of every fork and `[patch.crates-io]` override in the
> stack (mistral.rs, candle, sysctl, mio, tqdm, core2) and how the
> git-vs-published tiers resolve differently, see the **forked-dependencies**
> skill. This section covers only the mistral.rs publish mechanics.

`cargo publish` strips every `git =` field before uploading to crates.io.
That means a git-only dep is useless to anyone who installs `onde` from the
registry — they can't resolve it.

The solution: publish the fork to crates.io under the name `onde-mistralrs`
(and `onde-mistralrs-core`, etc.), then reference it with `package =` so the
rest of the code still compiles unchanged with `use mistralrs::...`:

```toml
# In onde/Cargo.toml — each platform target section looks like this.
# Cargo resolves it as "onde-mistralrs" from crates.io,
# but the Rust code sees it as `mistralrs`.
mistralrs = { version = "0.8.2", package = "onde-mistralrs", features = ["metal"] }
```

This is the only supported publish path until the upstream PRs merge. Don't
swap it for plain `mistralrs` from crates.io without checking every platform
target first.

---

### Step 0 — check if the candle snapshot needs an `onde-candle-*` publish

Since 0.9.x, upstream mistral.rs pins `candle` to a git rev past the latest
released candle (e.g. rev `27f20fea`, 19 commits past `0.11.0`, adding
`QTensor::indexed_gemv`, `gemv_fused_shared_lhs`, `BarrierPool::execute_chunked`).
`cargo publish` strips git pins, so publish verification resolves candle from
crates.io and fails with E0599 "method not found" if the pinned rev has APIs the
released candle lacks.

When that happens, publish the pinned rev under `onde-candle-*` names from the
local candle checkout (`~/Repositories/candle`, branch pattern
`onde/candle-<version>-<rev>`):

```bash
# Which candle crates actually diverged from the released version?
git diff --stat <release-bump-commit> <pinned-rev> -- candle-core candle-nn candle-metal-kernels

# Rename ONLY the diverged crates (0.11.0 era: candle-core + candle-nn; the
# rest are byte-identical, official crates serve them). candle-nn must be
# republished whenever candle-core is, even if unchanged, or the official
# candle-nn drags the official candle-core into the graph -> two candle-core
# crates -> type mismatch.
cargo publish -p onde-candle-core
cargo publish -p onde-candle-nn
```

Then in the mistral.rs fork workspace `Cargo.toml`:

```toml
candle-core = { version = "0.11.0", package = "onde-candle-core" }
candle-nn = { version = "0.11.0", package = "onde-candle-nn" }
candle-flash-attn-v3 = { version = "0.11.0" }   # unchanged at rev -> official
candle-metal-kernels = { version = "0.11.0" }   # unchanged at rev -> official
```

### Step 1 — check if `onde-mistralrs` needs a new publish

First, see how far the fork has drifted from upstream:

```bash
curl -s "https://api.github.com/repos/setoelkahfi/mistral.rs/compare/EricLBuehler:mistral.rs:master...setoelkahfi:mistral.rs:fix%2Fall-platform-fixes" \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print('ahead:', d['ahead_by'], 'behind:', d['behind_by'])"
```

If there are new commits since the last `onde-mistralrs` publish, or if
upstream cut a new release that the fork has rebased onto, bump the workspace
version in the fork and publish. As of 0.9.1 the set is **12 crates** (upstream
split out flash-attn/metal-compile/code-exec/sandbox, and vision/audio/mcp must
be onde-published too because crates.io only has upstream's old versions).
The rename lives in the workspace dep table (`package = "onde-..."` on each
entry, local dep keys unchanged so imports stay `mistralrs_*`).

```bash
# 1. Edit the root Cargo.toml in setoelkahfi/mistral.rs
#    bump version, e.g. 0.9.0 → 0.9.1

# 2. Publish in dependency order (leaves first):
cargo publish -p onde-mistralrs-macros
cargo publish -p onde-mistralrs-metal-compile
cargo publish -p onde-mistralrs-sandbox
cargo publish -p onde-mistralrs-flash-attn
cargo publish -p onde-mistralrs-vision
cargo publish -p onde-mistralrs-audio
cargo publish -p onde-mistralrs-mcp
cargo publish -p onde-mistralrs-paged-attn
cargo publish -p onde-mistralrs-quant
cargo publish -p onde-mistralrs-code-exec
cargo publish -p onde-mistralrs-core
cargo publish -p onde-mistralrs

# 3. cargo waits for index propagation automatically after each publish.
```

Known publish-verification snags (hit during 0.9.1):

| Error | Fix |
|-------|-----|
| E0599 on candle APIs (`indexed_gemv`, ...) | Step 0 — publish `onde-candle-*` and repoint |
| `dependency tqdm does not specify a version` | tqdm must be a registry dep, not git — it is plain `tqdm = "0.8.1"` since the crossterm bump landed upstream |
| `core2 0.4.0 is yanked` | `cargo update -p bitstream-io` (4.10.0 dropped core2) |

---

### Step 2 — update `onde/Cargo.toml`

Every platform target section has its own `onde-mistralrs` line. They all need
to move to the new version together — leaving one target on an older version
causes a dependency conflict.

```bash
# All occurrences should print the same version number:
grep 'onde-mistralrs' onde/Cargo.toml
```

---

### Step 3 — dry-run first

```bash
cd onde
cargo publish --dry-run --allow-dirty 2>&1 | tail -20
```

A clean run ends with `Finished` and no errors. Common things that go wrong:

| Error | What to do |
|-------|------------|
| `all dependencies must have a version` | add `version = "..."` to the dep |
| `no matching package named onde-mistralrs` | publish the fork first (step 1) |
| `git specification will be removed` | expected — cargo is just telling you the git field gets stripped |
| `files in working directory contain changes` | add `--allow-dirty` |

---

### Step 4 — publish `onde`, then update `sigit`

```bash
cargo publish
```

Then bump the version in `sigit/Cargo.toml`:

```toml
# git = is used for local dev; crates.io consumers resolve via version =
onde = { version = "0.1.9", git = "https://github.com/ondeinference/onde", branch = "development" }
```

Keep both fields. The `git =` field takes precedence during local development;
crates.io strips it and falls back to `version =` for anyone installing from
the registry.

---

### GresIQ credentials and `HF_TOKEN`

People building `onde` from crates.io don't need to do anything about secrets.
Here's why.

`build.rs` passes credentials to the compiler via `option_env!()`. That macro
resolves at compile time — if the env var isn't set, it compiles to `None` and
the feature that needs it is silently disabled. No crash, no partial state, no
mystery error at runtime.

```rust
// pulse/client.rs — GresIQ telemetry (Onde Inference internal infrastructure)
const EMBEDDED_API_KEY_DEV: Option<&str> = option_env!("GRESIQ_API_KEY_DEV");
const EMBEDDED_API_SECRET_DEV: Option<&str> = option_env!("GRESIQ_API_SECRET_DEV");

// token.rs — HuggingFace Hub authentication
const BUILD_TIME_HF_TOKEN: Option<&str> = option_env!("HF_TOKEN");
```

Who gets what:

| Who is building | Env vars present? | What happens |
|-----------------|-------------------|--------------|
| Our CI (official builds) | yes, via GitHub secrets | telemetry on; HF token baked into binary |
| Someone who ran `cargo add onde` | no | telemetry off; HF token read from `~/.cache/huggingface/token` at runtime |
| iOS/tvOS builds via XCFramework | yes, via `.env` file | HF token baked in — required because there's no writable filesystem on device |

The GresIQ credentials are Onde Inference's own infrastructure secrets — they
exist so the SDK can phone home for usage telemetry on official builds. External
consumers don't have them, don't need them, and won't be affected by not having
them.

Never put `GRESIQ_API_KEY_*` or `HF_TOKEN` in `README.md`, `.env.example`, or
any committed file. The `.env` in the crate root is git-ignored for a reason:

```bash
# onde/.env — git-ignored, never commit
GRESIQ_API_KEY_DEV=...
GRESIQ_API_SECRET_DEV=...
GRESIQ_API_KEY_PRODUCTION=...
GRESIQ_API_SECRET_PRODUCTION=...
HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

### Working against the git fork locally

If you're making changes to the fork and want `onde` to pick them up without
publishing, uncomment the patch block in `onde/Cargo.toml`:

```toml
[patch.crates-io]
onde-mistralrs      = { git = "https://github.com/setoelkahfi/mistral.rs", branch = "fix/all-platform-fixes", package = "mistralrs" }
onde-mistralrs-core = { git = "https://github.com/setoelkahfi/mistral.rs", branch = "fix/all-platform-fixes", package = "mistralrs-core" }
```

Re-comment it before running `cargo publish`. Cargo strips `[patch.crates-io]`
from the published manifest automatically, so it won't affect registry
consumers either way — but leaving it active makes the dry-run confusing.

---

### Pre-publish checklist

- [ ] all `onde-mistralrs` version numbers in `onde/Cargo.toml` are the same
- [ ] `onde/Cargo.toml` `[package] version` is bumped
- [ ] `sigit/Cargo.toml` `onde` version matches
- [ ] `[patch.crates-io]` git override is commented out
- [ ] `cargo publish --dry-run --allow-dirty` passes clean
- [ ] `.env` is not listed in the dry-run's packaged file output
- [ ] Eric Buehler's `LICENSE` in the `mistral.rs` fork is unmodified

---

*Last updated: July 2026 — added Colima/Docker Linux and Windows GNU
cross-check guidance for macOS hosts without cross GCC, plus the MSVC CI
caveat.*
