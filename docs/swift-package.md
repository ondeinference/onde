---
title: "Swift Package"
description: "Build and distribute the Onde Swift package (XCFramework) for iOS, tvOS, and macOS using UniFFI bindings."
---

# Swift Package

The Onde Swift package distributes a pre-built `OndeFramework.xcframework` via Swift Package Manager. The framework is built from Rust source using UniFFI bindings — no `cargo-swift` required. The project owns its own `uniffi-bindgen` binary that generates the Swift glue code directly.

## Installation

### Swift Package Manager

Add the package in Xcode: **File → Add Package Dependencies** and enter:

```
https://github.com/ondeinference/onde-swift
```

Or add it to your `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/ondeinference/onde-swift", from: "0.1.0")
],
targets: [
    .target(name: "MyApp", dependencies: ["Onde"])
]
```

## Supported Platforms

| Platform | Minimum Version | GPU Backend |
| -------- | --------------- | ----------- |
| iOS | 14.0 | Metal |
| tvOS | 13.0 | Metal |
| macOS | 12.0 | Metal |

## Swift API

### Basic Chat

```swift
import Onde

// Create the engine
let engine = OndeChatEngine()

// Load the platform-appropriate default model:
//   iOS / tvOS  → Qwen 2.5 1.5B (~941 MB)
//   macOS       → Qwen 2.5 3B   (~1.93 GB)
let loadTimeSecs = try await engine.loadDefaultModel(
    systemPrompt: "You are a helpful assistant.",
    sampling: nil  // uses platform-aware defaults
)
print("Model loaded in \(loadTimeSecs)s")

// Multi-turn chat (history managed automatically)
let result = try await engine.sendMessage(message: "What can you help me with?")
print(result.text)
print(result.durationDisplay)  // e.g. "4.5s"

let followUp = try await engine.sendMessage(message: "Tell me more about that.")
print(followUp.text)

// Unload model to free memory
await engine.unloadModel()
```

### Streaming

```swift
import Onde

class StreamHandler: StreamChunkListener {
    func onChunk(chunk: StreamChunk) -> Bool {
        print(chunk.delta, terminator: "")
        if chunk.done { print() }
        return !chunk.done  // return false to cancel early
    }
}

let engine = OndeChatEngine()
try await engine.loadDefaultModel(systemPrompt: nil, sampling: nil)

try await streamChatMessage(
    engine: engine,
    message: "Tell me a story.",
    listener: StreamHandler()
)
```

Streaming is exposed as a free function rather than an `OndeChatEngine` method because UniFFI 0.31 does not support `callback_interface` parameters on Object methods.

### One-Shot Generation

`generate` runs inference on an explicit message list without modifying the engine's conversation history. Useful for prompt enhancement, summarisation, or any case where you don't want side effects on the chat history.

```swift
import Onde

let engine = OndeChatEngine()
try await engine.loadDefaultModel(systemPrompt: nil, sampling: nil)

let result = try await engine.generate(
    messages: [userMessage(content: "Expand into a detailed prompt: a cat in space")],
    sampling: deterministicSamplingConfig()
)
print(result.text)
```

### Custom Model and Sampling

```swift
import Onde

let engine = OndeChatEngine()

let sampling = SamplingConfig(
    temperature: 0.9,
    topP: 0.95,
    topK: nil,
    minP: nil,
    maxTokens: 256,
    frequencyPenalty: nil,
    presencePenalty: nil
)

try await engine.loadGgufModel(
    config: qwen253bConfig(),
    systemPrompt: "You are a music expert.",
    sampling: sampling
)
```

### History and Status

```swift
// Check engine status
let info = await engine.info()
print(info.status)          // .ready
print(info.historyLength)   // number of turns

// Retrieve full conversation history
let history = await engine.history()
for msg in history {
    print("\(msg.role): \(msg.content)")
}

// Clear history without unloading the model
let removed = await engine.clearHistory()
print("Removed \(removed) turns")

// Append a message without running inference
await engine.pushHistory(message: userMessage(content: "..."))
```

## Free Functions Reference

### Model Configurations

```swift
defaultModelConfig()    // platform-appropriate default (1.5B on iOS/tvOS, 3B on macOS)
qwen251_5bConfig()      // Qwen 2.5 1.5B Q4_K_M (~941 MB)
qwen253bConfig()        // Qwen 2.5 3B Q4_K_M (~1.93 GB)
```

### Sampling Presets

```swift
defaultSamplingConfig()         // temp=0.7, top_p=0.95, max_tokens=512
deterministicSamplingConfig()   // temp=0.0, greedy decoding
mobileSamplingConfig()          // temp=0.7, max_tokens=128
```

### Message Constructors

```swift
systemMessage(content:)     // ChatRole.system
userMessage(content:)        // ChatRole.user
assistantMessage(content:)   // ChatRole.assistant
```

## Type Reference

| Rust Type | Swift Type | Description |
| --------- | ---------- | ----------- |
| `ChatRole` | `enum ChatRole` | `.system`, `.user`, `.assistant` |
| `ChatMessage` | `struct ChatMessage` | `role` + `content` |
| `SamplingConfig` | `struct SamplingConfig` | Temperature, top-p, max tokens, etc. |
| `GgufModelConfig` | `struct GgufModelConfig` | Model repo, filename, display name |
| `InferenceResult` | `struct InferenceResult` | `text`, `durationSecs`, `durationDisplay`, `finishReason` |
| `StreamChunk` | `struct StreamChunk` | `delta`, `done`, `finishReason` |
| `EngineStatus` | `enum EngineStatus` | `.unloaded`, `.loading`, `.ready`, `.generating`, `.error` |
| `EngineInfo` | `struct EngineInfo` | `status`, `modelName`, `approxMemory`, `historyLength` |
| `InferenceError` | `enum InferenceError: Error` | Thrown by `loadGgufModel`, `sendMessage`, `generate`, `streamChatMessage` |
| `OndeChatEngine` | `class OndeChatEngine` | Main inference engine, thread-safe |
| `StreamChunkListener` | `protocol StreamChunkListener` | Implement `onChunk(chunk:) -> Bool` |

## Sandboxed App Setup (iOS / macOS App Store)

On sandboxed platforms, `~/.cache` is inaccessible. Before any inference code runs, configure `HF_HOME` and `TMPDIR` to point inside the app's container:

```swift
import Foundation

func setupInferenceEnvironment() {
    let appSupport = FileManager.default
        .urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]

    let hfHome = appSupport.appendingPathComponent("huggingface")
    let hfHub  = hfHome.appendingPathComponent("hub")
    try? FileManager.default.createDirectory(at: hfHub, withIntermediateDirectories: true)

    setenv("HF_HOME",      hfHome.path, 1)
    setenv("HF_HUB_CACHE", hfHub.path,  1)

    // mistral.rs uses TMPDIR for temporary files during model loading
    let tmp = appSupport.appendingPathComponent("tmp")
    try? FileManager.default.createDirectory(at: tmp, withIntermediateDirectories: true)
    setenv("TMPDIR", tmp.path, 1)
}
```

Call this once at app launch, before calling any `OndeChatEngine` method.

## Building the XCFramework Locally

If you are contributing to Onde or need a custom build, you can assemble the `OndeFramework.xcframework` from source.

### Prerequisites

```bash
# Stable toolchain for iOS and macOS targets
rustup toolchain install stable

# Nightly toolchain for tvOS (tier-3 targets require -Z build-std)
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly

# Apple targets
rustup target add \
  aarch64-apple-ios \
  x86_64-apple-ios \
  aarch64-apple-ios-sim \
  aarch64-apple-darwin \
  x86_64-apple-darwin
```

### Build

```bash
# 1. Build the uniffi-bindgen CLI (host)
cargo build --manifest-path uniffi-bindgen/Cargo.toml --release
BINDGEN=uniffi-bindgen/target/release/uniffi-bindgen

# 2. Compile staticlibs per target
cargo build --target aarch64-apple-ios          --release
cargo build --target x86_64-apple-ios           --release
cargo build --target aarch64-apple-ios-sim      --release
cargo build --target aarch64-apple-darwin       --release
cargo build --target x86_64-apple-darwin        --release
cargo +nightly build -Z build-std --target aarch64-apple-tvos     --release
cargo +nightly build -Z build-std --target x86_64-apple-tvos      --release
cargo +nightly build -Z build-std --target aarch64-apple-tvos-sim --release

# 3. lipo simulator and macOS slices
lipo -create \
    target/x86_64-apple-ios/release/libonde.a \
    target/aarch64-apple-ios-sim/release/libonde.a \
    -output /tmp/libonde-ios-sim.a

lipo -create \
    target/x86_64-apple-tvos/release/libonde.a \
    target/aarch64-apple-tvos-sim/release/libonde.a \
    -output /tmp/libonde-tvos-sim.a

lipo -create \
    target/aarch64-apple-darwin/release/libonde.a \
    target/x86_64-apple-darwin/release/libonde.a \
    -output /tmp/libonde-macos.a

# 4. Generate Swift bindings
mkdir -p sdk/Onde/Sources/Onde
$BINDGEN generate \
    --library target/aarch64-apple-ios/release/libonde.a \
    --language swift \
    --out-dir sdk/Onde/Sources/Onde \
    --config uniffi.toml

# 5. Assemble the XCFramework
HEADERS=sdk/Onde/Sources/Onde
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libonde.a  -headers $HEADERS \
    -library /tmp/libonde-ios-sim.a                      -headers $HEADERS \
    -library target/aarch64-apple-tvos/release/libonde.a -headers $HEADERS \
    -library /tmp/libonde-tvos-sim.a                     -headers $HEADERS \
    -library /tmp/libonde-macos.a                        -headers $HEADERS \
    -output sdk/Onde/OndeFramework.xcframework
```

### Local Xcode Integration

During development, use the local path form in `Package.swift`:

```swift
.binaryTarget(name: "OndeFramework", path: "./OndeFramework.xcframework")
```

In Xcode: **File → Add Package Dependencies → Add Local** → select `sdk/Onde/`.

## Known Issues

### `___chkstk_darwin` linker error on tvOS

`aws-lc-sys` (a transitive dependency via `hf-hub`) references `___chkstk_darwin`, a stack probing symbol that tvOS does not export. The `build.rs` in the onde crate provides a no-op assembly stub (`tvos_chkstk.s`) that resolves this automatically. No action is needed — ensure `tvos_chkstk.s` is present at the crate root.

This does not affect iOS, macOS, Android, Windows, or Linux builds.

### Metal Toolchain missing (Xcode 26+)

On Xcode 26+, the Metal compiler is a separate downloadable component. If you see empty `.metallib` files (92 bytes) or `"Error while loading function: fused_glu_float"` at runtime:

```bash
xcodebuild -downloadComponent MetalToolchain
cargo clean -p mistralrs-quant
cargo check
```
