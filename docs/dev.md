---
title: "Developer Guide"
description: "Build Onde from source, understand the architecture, and integrate it as a Rust dependency."
sidebarTitle: "Developer Guide"
---

# Onde — Developer Guide

> **See also:** [Swift Package](swift-package) · [Ruby Gem](ruby-gem) · [Distribution](distribution)

## Prerequisites

```bash
# Rust stable
rustup toolchain install stable

# Rust nightly (only needed for tvOS tier-3 targets with -Z build-std)
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly

# iOS targets (stable)
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

## Building

### Native (macOS — Metal accelerated)

```bash
# Check compilation
cargo check

# Run all tests
cargo test

# Clippy lint
cargo clippy
```

### Ruby Gem (magnus)

Build the Ruby native extension that exposes Onde's HuggingFace cache management and model metadata to Ruby via [magnus](https://github.com/matsadler/magnus):

```bash
cd sdk/gem

# Install Ruby dependencies + compile the Rust native extension
bin/setup

# Or step by step:
bundle install
bundle exec rake compile

# Interactive console
bin/console
```

Verify it works:

```bash
bundle exec ruby -e "require 'onde'; puts Onde::VERSION; puts Onde::SUPPORTED_MODELS"
```

For full API reference, usage examples, and architecture details, see [ruby-gem](ruby-gem).

## Architecture

```
onde/
├── src/
│   ├── lib.rs                    # Crate root — uniffi::setup_scaffolding!()
│   ├── hf_cache.rs               # HuggingFace Hub cache management
│   └── inference/
│       ├── mod.rs                # Module exports + re-exports
│       ├── types.rs              # UniFFI-annotated shared types
│       ├── engine.rs             # ChatEngine (Rust-native API)
│       ├── ffi.rs                # OndeChatEngine (UniFFI Object for Swift/Kotlin)
│       ├── models.rs             # Model ID constants + metadata
│       └── token.rs              # HF token resolution (build-time / cache)
├── sdk/
│   ├── Onde/                     # Generated Swift package (git-ignored)
│   ├── gem/                      # Ruby gem (magnus-based native extension)
│   ├── kotlin/                   # Kotlin bindings
│   └── python/                   # Python bindings
├── generated/                    # UniFFI-generated headers and sources
├── .cargo/
│   └── config.toml               # Target-specific rustflags (fp16)
├── Cargo.toml                    # Platform-conditional mistralrs deps
├── build.rs                      # UniFFI scaffolding + tvOS chkstk stub
├── uniffi.toml                   # UniFFI binding config
└── docs/
    ├── dev.md                    # This file
    ├── swift-package.md          # Swift package building & API guide
    └── ruby-gem.md               # Ruby gem API reference & guide
```

### Layer Diagram

```
┌──────────────────────────────────────────────────────┐
│  Swift (tvOS / iOS)          Rust app / CLI / server  │
│  import Onde                 use onde::inference::*    │
└──────────┬───────────────────────────┬───────────────┘
           │ UniFFI bindings           │ Direct Rust API
           ▼                           ▼
┌──────────────────────┐  ┌────────────────────────────┐
│  OndeChatEngine      │  │  ChatEngine                │
│  (ffi.rs — Object)   │──│  (engine.rs — native)      │
│  FFI-safe methods    │  │  impl Into<String>, mpsc   │
└──────────┬───────────┘  └────────────┬───────────────┘
           │                           │
           └───────────┬───────────────┘
                       ▼
           ┌───────────────────────┐
           │  mistralrs::Model     │
           │  (GgufModelBuilder)   │
           │  Metal / CUDA / CPU   │
           └───────────────────────┘
```

## Platform Support

| Platform | `target_os` | GPU Backend       | Default Model  | Dependency                       |
| -------- | ----------- | ----------------- | -------------- | -------------------------------- |
| macOS    | `macos`     | Metal             | Qwen 2.5 3B   | mistralrs `features = ["metal"]` |
| iOS      | `ios`       | Metal             | Qwen 2.5 1.5B | mistralrs `features = ["metal"]` |
| tvOS     | `tvos`      | Metal             | Qwen 2.5 1.5B | mistralrs `features = ["metal"]` |
| Windows  | `windows`   | CPU (CUDA via CI) | Qwen 2.5 3B   | mistralrs (no features)          |
| Linux    | `linux`     | CPU (CUDA via CI) | Qwen 2.5 3B   | mistralrs (no features)          |
| Android  | `android`   | CPU (candle)      | Qwen 2.5 1.5B | mistralrs + mistralrs-core       |

## Testing

```bash
# All tests
cargo test

# Just inference module tests
cargo test inference::

# Just FFI wrapper tests
cargo test inference::ffi::

# Just type tests
cargo test inference::types::

# Clippy
cargo clippy
```

## Using Onde as a Rust Dependency

Add onde as a dependency in your `Cargo.toml`:

```toml
[dependencies]
onde = { path = "../../onde" }       # adjust relative path to your layout
# or from git:
# onde = { git = "https://github.com/ondeinference/onde" }
```

The Rust API is accessed directly — no UniFFI needed:

```rust
use onde::mistralrs::{GgufModelBuilder, Model, RequestBuilder, TextMessageRole};
use onde::inference::token::hf_token_source;

// Load model
let model = GgufModelBuilder::new("bartowski/Qwen2.5-1.5B-Instruct-GGUF", vec!["Qwen2.5-1.5B-Instruct-Q4_K_M.gguf"])
    .with_token_source(hf_token_source())
    .with_logging()
    .build()
    .await?;

// Chat
let request = RequestBuilder::new()
    .add_message(TextMessageRole::System, "You are a helpful assistant.")
    .add_message(TextMessageRole::User, "Hello!")
    .set_sampler_temperature(0.7);

let response = model.send_chat_request(request).await?;
```

### Important Setup for Sandboxed Apps (iOS / macOS App Store)

Your app's setup function must configure `HF_HOME` and `TMPDIR` before any inference code runs. See the [pepakbasajawa](https://github.com/pfrfrfr/pepakbasajawa) or [fatapp](https://github.com/setoelkahfi/fatapp) repos for working examples of the `setup_hf_home()` and `setup_tmpdir()` pattern.

## Known Issues

### `___chkstk_darwin` linker error on tvOS

When building for `aarch64-apple-tvos`, `aws-lc-sys` (a transitive dependency via `rustls` → `reqwest` → `hf-hub`) references `___chkstk_darwin`, a stack probing symbol not available on tvOS. The onde `build.rs` provides a no-op assembly stub (`tvos_chkstk.s`) that resolves this automatically.

This does not affect macOS, iOS, Windows, Linux, or Android builds.

### Metal Toolchain must be installed (Xcode 26+)

On Xcode 26+, the Metal compiler is a separate downloadable component. If you see empty `.metallib` files (92 bytes) or `"Error while loading function: fused_glu_float"` at runtime, run:

```bash
xcodebuild -downloadComponent MetalToolchain
```

Then clean and rebuild:

```bash
cargo clean -p mistralrs-quant
cargo check
```
