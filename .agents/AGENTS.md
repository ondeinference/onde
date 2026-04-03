# Onde Inference — AGENTS.md

> **AI agent reference for the `onde` repository and the `onde-swift` Swift SDK.**
> Keep this file accurate as the codebase evolves.

---

## What Is Onde?

Onde is an **on-device LLM inference SDK** targeting Apple silicon (iOS, tvOS, macOS) plus Android, Windows, and Linux. It wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) behind a clean, platform-aware API — automatic model selection, HuggingFace Hub downloads, cache management, and GPU acceleration via Metal.

- **Website:** https://ondeinference.com
- **Rust crate:** https://crates.io/crates/onde
- **Swift package:** https://github.com/ondeinference/onde-swift
- **In production:** [Splitfire AB apps on the Apple AppStore](https://apps.apple.com/se/developer/splitfire-ab/id1831430993)

---

## Repository Layout

```
onde/
├── src/
│   ├── lib.rs                    # Crate root — uniffi::setup_scaffolding!()
│   ├── hf_cache.rs               # HuggingFace Hub cache: list, download, delete, diagnose, repair
│   └── inference/
│       ├── mod.rs                # Module exports + top-level re-exports
│       ├── engine.rs             # ChatEngine — Rust-native API (generics, mpsc)
│       ├── ffi.rs                # OndeChatEngine — UniFFI Object (FFI-safe, Arc-wrapped)
│       ├── models.rs             # Model ID constants + SupportedModelInfo metadata
│       ├── token.rs              # HF token resolution: build-time literal vs cache file
│       └── types.rs              # All shared types: ChatMessage, SamplingConfig, InferenceResult, etc.
├── sdk/
│   ├── Onde/                     # Generated Swift package output (git-ignored; built by CI)
│   ├── gem/                      # Ruby native extension (Magnus)
│   ├── kotlin/                   # Kotlin/Android bindings
│   └── python/                   # Python bindings (maturin + uniffi)
├── generated/                    # UniFFI-generated headers and Swift glue (git-ignored)
├── uniffi-bindgen/               # Standalone bindgen binary crate (pinned uniffi =0.31.0)
├── .github/
│   ├── workflows/
│   │   └── build-swift-xcframework.yml   # CI: builds + uploads OndeFramework.xcframework
│   └── scripts/
│       └── build-swift-xcframework.sh    # Local/CI XCFramework assembly script
├── .cargo/config.toml            # Target-specific rustflags (fp16, linker overrides)
├── Cargo.toml                    # Platform-conditional mistralrs deps
├── build.rs                      # tvOS ___chkstk_darwin assembly stub
├── scripts/
│   └── tvos_chkstk.s             # No-op arm64 stub for missing tvOS symbol
├── uniffi.toml                   # UniFFI binding config
└── docs/
    ├── dev.md                    # Developer guide (build, architecture, platform table)
    ├── swift-package.md          # Swift API reference + XCFramework build steps
    ├── distribution.md           # Release process for all registries
    └── ruby-gem.md               # Ruby gem API reference
```

```
onde-swift/                       # Swift Package Manager wrapper repo
├── Package.swift                 # Declares OndeFramework.xcframework binary target
└── Sources/
    └── Onde/
        └── onde.swift            # UniFFI-generated Swift glue (do NOT edit manually)
```

---

## Architecture

### Layer Diagram

```
┌──────────────────────────────────────────────────────────┐
│  Swift (iOS / tvOS / macOS)    Rust app / CLI / server   │
│  import Onde                   use onde::inference::*    │
└──────────┬───────────────────────────────┬───────────────┘
           │ UniFFI FFI bindings           │ Direct Rust API
           ▼                               ▼
┌──────────────────────────┐  ┌─────────────────────────────┐
│  OndeChatEngine          │  │  ChatEngine                 │
│  (ffi.rs — uniffi::Object│──│  (engine.rs — Rust-native)  │
│  FFI-safe, Arc<Self>)    │  │  generics, mpsc channels    │
└──────────┬───────────────┘  └─────────────┬───────────────┘
           │                                │
           └────────────────┬───────────────┘
                            ▼
               ┌────────────────────────┐
               │  mistralrs::Model      │
               │  GgufModelBuilder      │
               │  Metal / CUDA / CPU    │
               └────────────────────────┘
```

### Key Design Rules

1. **`ChatEngine`** (`engine.rs`) owns all Rust-idiomatic logic: `impl Into<String>`, `tokio::sync::mpsc::Receiver`, etc. Never add UniFFI annotations here.
2. **`OndeChatEngine`** (`ffi.rs`) is a thin `Arc`-wrapped UniFFI `Object` with concrete, FFI-safe signatures. It delegates every call to the inner `ChatEngine`.
3. **`StreamChunkListener`** is a `#[uniffi::export(callback_interface)]` trait — it must be passed as a parameter to **free functions** (not Object methods) because UniFFI 0.31 does not support callback_interface on Object methods.
4. **Free functions** in `ffi.rs` (e.g. `stream_chat_message`, `default_model_config`, `user_message`) are exported with `#[uniffi::export]` and generate top-level Swift functions.
5. **`uniffi::setup_scaffolding!()`** lives in `lib.rs`. Never call `uniffi::generate_scaffolding!()` from `build.rs` for the proc-macro path.

---

## Platform Support

| Platform | `target_os` | GPU Backend       | Default Model         | mistralrs features |
| -------- | ----------- | ----------------- | --------------------- | -------------------|
| macOS    | `macos`     | Metal             | Qwen 2.5 3B (~1.93 GB) | `["metal"]`       |
| iOS      | `ios`       | Metal             | Qwen 2.5 1.5B (~941 MB)| `["metal"]`       |
| tvOS     | `tvos`      | Metal             | Qwen 2.5 1.5B (~941 MB)| `["metal"]`       |
| Android  | `android`   | CPU (candle)      | Qwen 2.5 1.5B (~941 MB)| `[]` + hf-hub     |
| Windows  | `windows`   | CPU (CUDA in CI)  | Qwen 2.5 3B (~1.93 GB) | `[]`              |
| Linux    | `linux`     | CPU (CUDA in CI)  | Qwen 2.5 3B (~1.93 GB) | `[]`              |

- **iOS / tvOS memory constraint:** iOS gives apps ~2–3 GB; the 3B model (~1.93 GB) causes OOM on iPhone 16e (8 GB RAM). Always default to the 1.5B on iOS/tvOS/Android.
- **tvOS tier-3 target:** requires `cargo +nightly -Z build-std`. Stable toolchain cannot build tvOS targets.
- **Android `hf_hub`:** `dirs::home_dir()` panics in the Android sandbox. `hf-hub` is added as an explicit dep on Android so `HF_HOME` can be seeded programmatically via `hf_hub::api::tokio::ApiBuilder`.

---

## Supported Models

All model constants live in `src/inference/models.rs`. When adding a new model:

1. Add `pub const` entries for the HF repo ID, GGUF filename, and (on Android) `TOK_MODEL_ID`.
2. Add the repo ID to `SUPPORTED_MODELS` so `list_local_hf_models` filters it.
3. Add a `SupportedModelInfo` entry to `SUPPORTED_MODEL_INFO` with accurate `expected_size_bytes` (from HF API `siblings[].size`).
4. Add a constructor to `GgufModelConfig` in `engine.rs` (see `qwen25_1_5b`, `qwen25_3b` for pattern).
5. Export a free function in `ffi.rs` (see `qwen25_1_5b_config`, `qwen25_3b_config`).

### Current Models

| Model | Repo | File | Size | Platforms |
|-------|------|------|------|-----------|
| Qwen 2.5 1.5B (GGUF Q4_K_M) | `bartowski/Qwen2.5-1.5B-Instruct-GGUF` | `Qwen2.5-1.5B-Instruct-Q4_K_M.gguf` | ~941 MB | iOS, tvOS, Android (default) |
| Qwen 2.5 3B (GGUF Q4_K_M) | `bartowski/Qwen2.5-3B-Instruct-GGUF` | `Qwen2.5-3B-Instruct-Q4_K_M.gguf` | ~1.93 GB | macOS, Windows, Linux (default) |
| Qwen 2.5 Coder 1.5B (GGUF) | `bartowski/Qwen2.5-Coder-1.5B-Instruct-GGUF` | `Qwen2.5-Coder-1.5B-Instruct-Q4_K_M.gguf` | ~941 MB | All platforms |
| Qwen 2.5 Coder 3B (GGUF) | `bartowski/Qwen2.5-Coder-3B-Instruct-GGUF` | `Qwen2.5-Coder-3B-Instruct-Q4_K_M.gguf` | ~1.93 GB | macOS |
| Qwen 2.5 Coder 7B (ISQ) | `Qwen/Qwen2.5-Coder-7B-Instruct` | safetensors (ISQ in-situ) | ~8 GB | macOS (ISQ pipeline) |

---

## Key Types (Rust ↔ Swift)

| Rust Type | Swift Type | Notes |
|-----------|------------|-------|
| `ChatRole` | `enum ChatRole` | `.system`, `.user`, `.assistant` |
| `ChatMessage` | `struct ChatMessage` | `role: ChatRole`, `content: String` |
| `SamplingConfig` | `struct SamplingConfig` | All fields `Optional` |
| `GgufModelConfig` | `struct GgufModelConfig` | `modelId`, `files`, `tokModelId?`, `displayName`, `approxMemory` |
| `IsqModelConfig` | `struct IsqModelConfig` | `modelId`, `isqBits: UInt8`, `displayName`, `approxMemory` |
| `InferenceResult` | `struct InferenceResult` | `text`, `durationSecs`, `durationDisplay`, `finishReason` |
| `StreamChunk` | `struct StreamChunk` | `delta`, `done`, `finishReason?` |
| `EngineStatus` | `enum EngineStatus` | `.unloaded`, `.loading`, `.ready`, `.generating`, `.error` |
| `EngineInfo` | `struct EngineInfo` | `status`, `modelName?`, `approxMemory?`, `historyLength: UInt64` |
| `InferenceError` | `enum InferenceError: Error` | `noModelLoaded`, `alreadyLoaded`, `modelBuild`, `inference`, `cancelled`, `other` |
| `OndeChatEngine` | `class OndeChatEngine` | Thread-safe; `Arc`-backed; constructed with `OndeChatEngine()` |
| `StreamChunkListener` | `protocol StreamChunkListener` | Implement `onChunk(chunk:) -> Bool` |

---

## HuggingFace Token

`src/inference/token.rs` resolves the HF token in priority order:

1. **Build-time literal** (`HF_TOKEN` env var baked in via `option_env!`) — required for iOS/tvOS (no filesystem token possible).
2. **On-disk cache** (`~/.cache/huggingface/token`) — works on macOS after `mistralrs login`.

Set `HF_TOKEN` before building:

```bash
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
cargo build --release
```

On sandboxed platforms, your app's setup function must also configure `HF_HOME`, `HF_HUB_CACHE`, and `TMPDIR` to point inside the app container before any `OndeChatEngine` method is called. See `docs/swift-package.md` for the full `setupInferenceEnvironment()` Swift snippet.

---

## Building (Rust)

### Prerequisites

```bash
# Stable toolchain (macOS, iOS, Android, Windows, Linux)
rustup toolchain install stable

# Nightly toolchain (tvOS tier-3 targets only)
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly

# Apple targets (stable)
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

### Commands

```bash
# Verify compilation (macOS host)
cargo check

# Run all tests
cargo test

# Run inference module tests only
cargo test inference::

# Run FFI wrapper tests only
cargo test inference::ffi::

# Clippy lint
cargo clippy

# Format
cargo fmt
```

### tvOS

tvOS targets require nightly and `-Z build-std`:

```bash
cargo +nightly rustc -Z build-std \
    --target aarch64-apple-tvos --release --lib --crate-type staticlib
```

---

## Building the Swift XCFramework

The XCFramework bundles staticlibs for iOS device, iOS simulator, tvOS device, tvOS simulator, and macOS into a single distributable that `Package.swift` references.

### Quick build (local or CI)

```bash
.github/scripts/build-swift-xcframework.sh
```

Output lands in `dist/swift/`:
- `OndeFramework.xcframework/` — the framework tree
- `OndeFramework.xcframework.zip` — zipped for SPM remote binary
- `OndeFramework.checksum.txt` — SHA-256 for `Package.swift`
- `version.txt` — semver from `Cargo.toml`
- `Package/Sources/Onde/onde.swift` — generated UniFFI Swift glue

### Manual steps

```bash
# 1. Build the uniffi-bindgen binary (pinned to uniffi =0.31.0)
cargo build --manifest-path uniffi-bindgen/Cargo.toml --release
BINDGEN=uniffi-bindgen/target/release/uniffi-bindgen

# 2. Compile staticlibs per target (use +1.92.0 or current stable)
cargo +stable rustc --target aarch64-apple-ios          --release --lib --crate-type staticlib
cargo +stable rustc --target aarch64-apple-ios-sim      --release --lib --crate-type staticlib
cargo +stable rustc --target aarch64-apple-darwin       --release --lib --crate-type staticlib
cargo +nightly rustc -Z build-std --target aarch64-apple-tvos     --release --lib --crate-type staticlib
cargo +nightly rustc -Z build-std --target aarch64-apple-tvos-sim --release --lib --crate-type staticlib

# 3. Generate Swift bindings from the iOS arm64 slice
$BINDGEN generate target/aarch64-apple-ios/release/libonde.a \
    --language swift \
    --out-dir sdk/Onde/Sources/Onde \
    --config uniffi.toml

# 4. Assemble the XCFramework (see build-swift-xcframework.sh for full xcodebuild invocation)
```

### Updating `onde-swift` after a new release

This is fully automated by CI. Publishing a GitHub Release on `onde` triggers the workflow, which:

1. Builds the XCFramework and attaches it to the release.
2. Rewrites `onde-swift/Package.swift` with the release `url:` + `checksum:`.
3. Copies the freshly generated `onde.swift` into `onde-swift/Sources/Onde/`.
4. Commits, tags, and pushes `onde-swift` — Swift Package Index picks it up automatically.

Manual intervention is only needed if `ONDE_SWIFT_PAT` has expired or the `onde-swift` push fails.

---

## Swift SDK (`onde-swift`)

### Package.swift structure

```
onde-swift/
├── Package.swift          # Declares OndeFramework binary target + Onde wrapper target
└── Sources/Onde/
    └── onde.swift         # UniFFI-generated glue (NEVER edit manually)
```

`Package.swift` has two targets:

- **`OndeFramework`** — `.binaryTarget` using `url:` + `checksum:` for distribution. For local development, swap to the `path:` form documented in the `Package.swift` header comment and run `build-swift-xcframework.sh` first.
- **`Onde`** — `.target` depending on `OndeFramework`, used as the public import name in Swift.

### Swift API Quick Reference

```swift
import Onde

// Create engine
let engine = OndeChatEngine()

// Load model (platform-aware default)
let elapsed = try await engine.loadDefaultModel(
    systemPrompt: "You are a helpful assistant.",
    sampling: nil
)

// Multi-turn chat
let result = try await engine.sendMessage(message: "Hello!")
print(result.text)

// Streaming (free function — callback_interface constraint in UniFFI 0.31)
class Handler: StreamChunkListener {
    func onChunk(chunk: StreamChunk) -> Bool {
        print(chunk.delta, terminator: "")
        return !chunk.done
    }
}
try await streamChatMessage(engine: engine, message: "Tell me a story.", listener: Handler())

// One-shot (does NOT modify conversation history)
let enhanced = try await engine.generate(
    messages: [userMessage(content: "Expand: a cat in space")],
    sampling: deterministicSamplingConfig()
)

// Status
let info = await engine.info()  // EngineInfo

// History management
let history  = await engine.history()
let removed  = await engine.clearHistory()
await engine.pushHistory(message: userMessage(content: "..."))

// Cleanup
await engine.unloadModel()
```

### Free Functions

| Function | Returns | Notes |
|----------|---------|-------|
| `defaultModelConfig()` | `GgufModelConfig` | Platform-aware (1.5B on iOS/tvOS, 3B on macOS) |
| `qwen251_5bConfig()` | `GgufModelConfig` | Forces 1.5B regardless of platform |
| `qwen253bConfig()` | `GgufModelConfig` | Forces 3B regardless of platform |
| `defaultSamplingConfig()` | `SamplingConfig` | temp=0.7, top_p=0.95, max_tokens=512 |
| `deterministicSamplingConfig()` | `SamplingConfig` | temp=0.0, greedy |
| `mobileSamplingConfig()` | `SamplingConfig` | temp=0.7, max_tokens=128 |
| `systemMessage(content:)` | `ChatMessage` | `.system` role |
| `userMessage(content:)` | `ChatMessage` | `.user` role |
| `assistantMessage(content:)` | `ChatMessage` | `.assistant` role |
| `streamChatMessage(engine:message:listener:)` | `async throws` | Streaming via callback |

---

## UniFFI Conventions

- **Version pin:** `uniffi = "=0.31.0"` everywhere — the `onde` crate, `uniffi-bindgen/`, and `[build-dependencies]` must all use the **same** version. Mixing versions causes bindgen panics.
- **`uniffi::setup_scaffolding!()`** in `lib.rs` — proc-macro approach, no UDL file needed.
- **Object methods** use `#[uniffi::export]` on the `impl OndeChatEngine` block.
- **Callback interfaces** (`StreamChunkListener`) must be parameters of **free functions** only, not Object methods.
- **`Arc<Self>`** is the return type for `#[uniffi::constructor]`. UniFFI automatically handles this.
- **Async:** all async exported methods use `tokio` runtime (enabled via `uniffi = { features = ["tokio"] }`).
- **`uniffi.toml`**: lives at the crate root. Adjust renaming or namespace settings there before regenerating.

---

## HuggingFace Cache (`hf_cache.rs`)

The `hf_cache` module manages the on-device model cache, exposed via UniFFI to Swift / Kotlin as-needed.

Key public functions:

| Function | Description |
|----------|-------------|
| `list_local_hf_models()` | List downloaded models that Onde supports |
| `list_supported_hf_models()` | All supported models with download status |
| `download_model(model_id, progress_callback)` | Download a model with progress reporting |
| `delete_local_hf_model(model_id)` | Remove a model from the local cache |
| `diagnose_hf_cache()` | Inspect the cache for corruption |
| `repair_hf_cache_symlinks()` | Fix broken symlinks in the HF cache layout |
| `model_cache_path(model_id)` | Resolve the filesystem path for a model |
| `clean_stale_lock_files()` | Remove leftover `.lock` files from interrupted downloads |

`ModelDownloadProgress` carries `downloaded_bytes`, `total_bytes`, `progress` (0.0–1.0), and `done`.

---

## Sampling Presets

| Preset | `temperature` | `top_p` | `max_tokens` | Use Case |
|--------|--------------|---------|--------------|----------|
| `SamplingConfig::default()` | 0.7 | 0.95 | 512 | General creative chat |
| `SamplingConfig::deterministic()` | 0.0 | — | 512 | Reproducible / coding |
| `SamplingConfig::mobile()` | 0.7 | 0.95 | 128 | Memory/latency constrained |
| `SamplingConfig::coding()` | 0.0 | — | 512 | Code generation |
| `SamplingConfig::coding_mobile()` | 0.0 | — | 128 | Code on mobile |

---

## Testing

```bash
# All tests (requires a macOS host with Metal)
cargo test

# Unit tests only (no model downloads)
cargo test inference::
cargo test inference::ffi::
cargo test inference::types::

# hf_cache module
cargo test hf_cache::

# Clippy (treat warnings as errors)
cargo clippy -- -D warnings
```

Tests that require model downloads are integration tests and not run by default in CI. Unit tests verify:
- Type constructors and `Display` implementations
- `SamplingConfig` presets
- `GgufModelConfig` constructors and `platform_default()` routing
- `OndeChatEngine` lifecycle: `new()` starts unloaded, `send_message` without model returns `InferenceError::NoModelLoaded`, `clear_history` on empty returns 0, `unload_model` on empty returns `nil`.

---

## Known Issues

### `___chkstk_darwin` linker error on tvOS

`aws-lc-sys` (transitive via `reqwest → rustls → aws-lc-rs`) references `___chkstk_darwin`, a stack probing symbol that tvOS does not export. `build.rs` compiles `scripts/tvos_chkstk.s` (a no-op `ret` stub) via the `cc` crate to satisfy the linker. **Do not delete `scripts/tvos_chkstk.s`.**

Affects: tvOS only. macOS, iOS, Android, Windows, Linux are unaffected.

### Metal Toolchain missing (Xcode 26+)

On Xcode 26+, the Metal compiler is a separate download. If you see empty `.metallib` files (92 bytes) or `"Error while loading function: fused_glu_float"` at runtime:

```bash
xcodebuild -downloadComponent MetalToolchain
cargo clean -p mistralrs-quant
cargo check
```

### Android `home_dir` sandbox panic

`dirs::home_dir()` panics in the Android sandbox. The crate-level `Cargo.toml` adds `hf-hub` as an explicit Android dependency so `HF_HOME` can be seeded via `ApiBuilder` before any hub request. Never call `home::home_dir()` or `dirs::home_dir()` on Android.

---

## Code Conventions

### Rust

- **Error handling:** `anyhow` for application-level errors inside `engine.rs`; `thiserror` for the `InferenceError` enum (exported via UniFFI). Never `.unwrap()` or `.expect()` in non-test code.
- **Async:** `tokio` runtime. All async functions in `OndeChatEngine` are `pub async fn`.
- **Logging:** `log` crate macros (`log::debug!`, `log::info!`, `log::warn!`, `log::error!`). No `println!` in library code.
- **Platform gating:** use `#[cfg(target_os = "...")]` blocks. Match `Cargo.toml`'s target-conditional dependency sections.
- **Re-exports:** `mistralrs`, `hf_hub`, and `mistralrs_core` are re-exported from `lib.rs` for downstream Rust consumers. Keep these re-exports in sync with what's actually available per platform.
- **No `mod.rs`:** use named files (`inference/engine.rs`) not `inference/mod.rs` — except that `inference/mod.rs` exists and is the intentional module root for the `inference` module.

### Swift / SDK

- **Never manually edit `onde-swift/Sources/Onde/onde.swift`** — it is generated by `uniffi-bindgen`. Regenerate by running the build script.
- **`Package.swift` binary target** uses `url:` + `checksum:` in the committed form. For local development, swap to `path:` (instructions are in the file header). Never commit the `path:` form — CI overwrites it on every release.
- **iOS/tvOS sandbox:** always call `setupInferenceEnvironment()` at app launch before any `OndeChatEngine` call.

---

## CI / Release Workflow

### `build-swift-xcframework.yml`

Triggered on `workflow_dispatch` or on published GitHub Releases. Runs on `macos-15`.

Steps:
1. Install stable Rust (`1.92.0`) with iOS/macOS targets.
2. Install nightly Rust with `rust-src` component and tvOS targets.
3. Run `build-swift-xcframework.sh`.
4. Read `version.txt` and `OndeFramework.checksum.txt` into step outputs (`steps.meta.outputs.version` / `steps.meta.outputs.checksum`).
5. Upload CI artifacts: zip, checksum, version, generated `onde.swift`.
6. On release: attach zip and checksum to the GitHub Release.
7. On release: check out `ondeinference/onde-swift` using `ONDE_SWIFT_PAT`.
8. On release: rewrite the `.binaryTarget` in `onde-swift/Package.swift` with the release download URL and computed checksum.
9. On release: copy the freshly generated `onde.swift` into `onde-swift/Sources/Onde/`.
10. On release: commit both files, tag the commit with the version, and push to `ondeinference/onde-swift`.

#### Required secret

`ONDE_SWIFT_PAT` — a GitHub Personal Access Token with **`contents: write`** scope on the `ondeinference/onde-swift` repository. Create it at <https://github.com/settings/tokens> and add it as a repository secret on the `onde` repo.

### Release Process (end-to-end)

1. Bump version in `onde/Cargo.toml`.
2. Create and publish a GitHub Release on the `onde` repo (tag = version, e.g. `0.2.0`).
3. CI builds the XCFramework, attaches `OndeFramework.xcframework.zip` + checksum to the release, then **automatically** rewrites `onde-swift/Package.swift` with the correct `url:` + `checksum:`, commits, tags, and pushes `onde-swift`.
4. Swift Package Index picks up the new `onde-swift` tag automatically — no manual step needed.
5. `cargo publish` to crates.io.

> **Note:** Steps 3–4 are fully automated. Manual edits to `onde-swift/Package.swift` are no longer needed after a release. Only touch it when doing local development (use the `path:` form documented in the file header).

---

## Distribution Registry Summary

| Registry | Name | Import |
|----------|------|--------|
| crates.io | `onde` | `onde = "0.x"` |
| Swift Package Index | `onde-swift` (org: `ondeinference`) | `import Onde` |
| PyPI | `onde-inference` | `import onde_inference` |
| RubyGems | `onde-inference` | `require 'onde'` |
| Maven Central | `com.ondeinference:onde-inference` | Gradle `implementation(...)` |

---

*This file is for AI agent and human developer reference. Update it when the architecture, API surface, supported models, or toolchain requirements change.*
```
