---
name: sdk-swift-package
description: Build the Onde Swift package (XCFramework) from Rust source using UniFFI and distribute it via a remote-binary Package.swift for Swift Package Index. Covers xcframework assembly, App Group shared container, models/hub cache convention, and the onde-swift release workflow.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: true
---

# Skill: Swift Package (XCFramework Distribution)

## What this skill covers

Building the `Onde` Swift package from Rust source using UniFFI, assembling an
`XCFramework`, and distributing it via a remote-binary `Package.swift` for
Swift Package Index.

---

## Repository layout

```
onde/
├── src/                         # Rust source (lib.rs, inference/, hf_cache.rs)
├── sdk/Onde/                    # Output: generated Swift package (git-ignored locally)
│   ├── Package.swift
│   ├── Sources/Onde/            # UniFFI-generated onde.swift glue + header
│   └── OndeFramework.xcframework
├── uniffi-bindgen/              # onde's own uniffi-bindgen CLI binary
│   ├── Cargo.toml
│   └── uniffi-bindgen.rs        # fn main() { uniffi::uniffi_bindgen_main() }
├── uniffi.toml                  # UniFFI binding config (package_name, cdylib_name)
├── build.rs                     # Provides tvOS ___chkstk_darwin stub via cc
└── Cargo.toml                   # uniffi = "=0.31.0" — version MUST stay pinned
```

---

## Key constraints

| Constraint | Detail |
|---|---|
| UniFFI version | Pinned to **`=0.31.0`** in `Cargo.toml` and `uniffi-bindgen/Cargo.toml`. Never bump without coordinating with the Kotlin and Python SDKs. |
| tvOS targets | Tier-3 — require `cargo +nightly -Z build-std`. Stable toolchain is used for iOS/macOS. |
| tvOS linker | `___chkstk_darwin` is missing from tvOS libSystem. `build.rs` compiles `tvos_chkstk.s` as a no-op stub. This is automatic — do not remove it. |
| `cargo-swift` | **Not required.** The project owns its own `uniffi-bindgen` binary. Use it directly (same as `build-kotlin.sh` does for Android). |
| `.xcframework` in `Package.swift` | Published `Package.swift` must use **remote** `url:` + `checksum:` — never a local `path:`. |
| Swift package name | `Onde` (PascalCase). Git repo slug: `onde-swift` under the `ondeinference` org. |
| App Group ID | `group.com.ondeinference.apps` — shared across all Onde-powered apps (Flutter, Tauri, native Swift). |
| HF cache subdirectory | `<container>/models/hub/` — NOT `huggingface/hub/`. Must match the Tauri and Dart SDK convention. |

---

## Apple target triples

| Slice | Triple | Toolchain |
|---|---|---|
| iOS device | `aarch64-apple-ios` | stable |
| iOS simulator (Intel) | `x86_64-apple-ios` | stable |
| iOS simulator (Apple Silicon) | `aarch64-apple-ios-sim` | stable |
| tvOS device | `aarch64-apple-tvos` | nightly + `-Z build-std` |
| tvOS simulator (Intel) | `x86_64-apple-tvos` | nightly + `-Z build-std` |
| tvOS simulator (Apple Silicon) | `aarch64-apple-tvos-sim` | nightly + `-Z build-std` |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | stable |
| macOS (Intel) | `x86_64-apple-darwin` | stable |

---

## Build sequence (manual / CI)

### 1. Build the `uniffi-bindgen` CLI (host)

```bash
cargo build --manifest-path uniffi-bindgen/Cargo.toml --release
BINDGEN=uniffi-bindgen/target/release/uniffi-bindgen   # or workspace target/
```

### 2. Compile Rust staticlibs per Apple target

```bash
# Stable targets (iOS, macOS)
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release
cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release

# Nightly targets (tvOS — tier 3)
cargo +nightly build -Z build-std \
    --target aarch64-apple-tvos --release
cargo +nightly build -Z build-std \
    --target x86_64-apple-tvos --release
cargo +nightly build -Z build-std \
    --target aarch64-apple-tvos-sim --release
```

### 3. lipo simulator slices

```bash
# iOS simulator fat lib
lipo -create \
    target/x86_64-apple-ios/release/libonde.a \
    target/aarch64-apple-ios-sim/release/libonde.a \
    -output /tmp/libonde-ios-sim.a

# tvOS simulator fat lib
lipo -create \
    target/x86_64-apple-tvos/release/libonde.a \
    target/aarch64-apple-tvos-sim/release/libonde.a \
    -output /tmp/libonde-tvos-sim.a

# macOS universal fat lib
lipo -create \
    target/aarch64-apple-darwin/release/libonde.a \
    target/x86_64-apple-darwin/release/libonde.a \
    -output /tmp/libonde-macos.a
```

### 4. Generate Swift bindings

```bash
mkdir -p sdk/Onde/Sources/Onde

$BINDGEN generate \
    --library target/aarch64-apple-ios/release/libonde.a \
    --language swift \
    --out-dir sdk/Onde/Sources/Onde \
    --config uniffi.toml
```

This produces `onde.swift` and `ondeFFI.h` (+ `ondeFFI.modulemap`).

### 5. Create the `.xcframework`

Each slice needs a headers directory alongside its `.a`:

```bash
HEADERS=sdk/Onde/Sources/Onde   # contains ondeFFI.h + ondeFFI.modulemap

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libonde.a      -headers $HEADERS \
    -library /tmp/libonde-ios-sim.a                          -headers $HEADERS \
    -library target/aarch64-apple-tvos/release/libonde.a     -headers $HEADERS \
    -library /tmp/libonde-tvos-sim.a                         -headers $HEADERS \
    -library /tmp/libonde-macos.a                            -headers $HEADERS \
    -output sdk/Onde/OndeFramework.xcframework
```

### 6. Zip and compute checksum

```bash
cd sdk/Onde
zip -r OndeFramework.xcframework.zip OndeFramework.xcframework
CHECKSUM=$(shasum -a 256 OndeFramework.xcframework.zip | cut -d ' ' -f1)
echo $CHECKSUM
```

### 7. Publish (two-repo strategy)

1. Upload `OndeFramework.xcframework.zip` as a GitHub Release asset on the
   Rust source repo.
2. In the `ondeinference/onde-kit` Swift-only repo, update `Package.swift`:
   - `url:` → the GitHub Release asset download URL
   - `checksum:` → the SHA-256 from step 6
3. Commit, tag the version (`0.1.0`, etc.), and push to `onde-kit`.
4. Swift Package Index indexes `onde-kit` on every new tag.

---

## `Package.swift` (published form)

```swift
// swift-tools-version:5.5
import PackageDescription

let package = Package(
    name: "Onde",
    platforms: [
        .iOS(.v14),
        .macOS(.v12),
        .tvOS(.v13),
    ],
    products: [
        .library(name: "Onde", targets: ["Onde"])
    ],
    targets: [
        .binaryTarget(
            name: "OndeFramework",
            url: "https://github.com/ondeinference/onde-kit/releases/download/0.1.0/OndeFramework.xcframework.zip",
            checksum: "<sha256>"
        ),
        .target(
            name: "Onde",
            dependencies: [.target(name: "OndeFramework")]
        ),
    ]
)
```

---

## UniFFI type map (Rust → Swift)

| Rust type | UniFFI derive | Swift type |
|---|---|---|
| `ChatRole` | `uniffi::Enum` | `enum ChatRole` |
| `ChatMessage` | `uniffi::Record` | `struct ChatMessage` |
| `SamplingConfig` | `uniffi::Record` | `struct SamplingConfig` |
| `GgufModelConfig` | `uniffi::Record` | `struct GgufModelConfig` |
| `InferenceResult` | `uniffi::Record` | `struct InferenceResult` |
| `StreamChunk` | `uniffi::Record` | `struct StreamChunk` |
| `EngineStatus` | `uniffi::Enum` | `enum EngineStatus` |
| `EngineInfo` | `uniffi::Record` | `struct EngineInfo` |
| `InferenceError` | `uniffi::Error` | `enum OndeError : Error` |
| `OndeChatEngine` | `uniffi::Object` | `class OndeChatEngine` |
| `StreamChunkListener` | `callback_interface` | `protocol StreamChunkListener` |

**UniFFI compatibility rules:**
- Use `u64`/`u32` instead of `usize` (UniFFI does not support `usize`).
- Streaming is exposed as a **free function** (`streamChatMessage`) not an
  Object method — UniFFI 0.31 does not support `callback_interface` in Object
  method parameters.
- Use concrete `String` parameters in `OndeChatEngine` methods — `impl Into<String>`
  does not cross the FFI boundary.

---

## Swift API quick reference

```swift
import Onde

// Engine lifecycle
let engine = OndeChatEngine()
try await engine.loadDefaultModel(systemPrompt: "You are helpful.", sampling: nil)
try await engine.loadGgufModel(config: qwen251_5bConfig(), systemPrompt: nil, sampling: nil)
await engine.unloadModel()
let loaded: Bool = await engine.isLoaded()
let info: EngineInfo = await engine.info()

// Chat
let result: InferenceResult = try await engine.sendMessage(message: "Hello!")
let oneShot: InferenceResult = try await engine.generate(messages: [...], sampling: nil)

// History
let history: [ChatMessage] = await engine.history()
let removed: UInt64 = await engine.clearHistory()
await engine.pushHistory(message: userMessage(content: "..."))

// Streaming (free function, callback-based)
try await streamChatMessage(engine: engine, message: "Tell me a story.", listener: myHandler)

// Config free functions
defaultModelConfig()           // platform-appropriate default
qwen251_5bConfig()             // ~941 MB, iOS/tvOS/Android
qwen253bConfig()               // ~1.93 GB, macOS/desktop

// Sampling presets
defaultSamplingConfig()        // temp=0.7, top_p=0.95, max=512
deterministicSamplingConfig()  // temp=0.0
mobileSamplingConfig()         // temp=0.7, max=128

// Message constructors
systemMessage(content:)
userMessage(content:)
assistantMessage(content:)
```

---

## App Group Shared Container & Cache Convention

All Onde-powered apps use the App Group `group.com.ondeinference.apps` to share
downloaded models.  The cache layout inside the shared container is:

```
<group container>/
├── models/          ← HF_HOME
│   └── hub/         ← HF_HUB_CACHE (GGUF files live here)
└── tmp/             ← TMPDIR (iOS sandbox restricts system TMPDIR)
```

**Important:** The subdirectory is `models/`, NOT `huggingface/`.  This was
chosen to be user-friendly when browsing the container in Finder/Files.  All
SDKs (Rust, Swift, Dart) and all app types (Tauri, Flutter, native) must use
the same path or models won't be shared.

### Swift setup (called once at app launch)

```swift
import Foundation

func setupInferenceEnvironment() {
    guard let container = FileManager.default.containerURL(
        forSecurityApplicationGroupIdentifier: "group.com.ondeinference.apps"
    ) else {
        // Fall back to app-private Application Support if App Group is unavailable.
        let appSupport = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        setupHfCache(at: appSupport)
        return
    }
    setupHfCache(at: container)
}

private func setupHfCache(at base: URL) {
    let hfHome = base.appendingPathComponent("models")
    let hfHub  = hfHome.appendingPathComponent("hub")
    try? FileManager.default.createDirectory(at: hfHub, withIntermediateDirectories: true)

    setenv("HF_HOME",      hfHome.path, 1)
    setenv("HF_HUB_CACHE", hfHub.path,  1)

    let tmp = base.appendingPathComponent("tmp")
    try? FileManager.default.createDirectory(at: tmp, withIntermediateDirectories: true)
    setenv("TMPDIR", tmp.path, 1)
}
```

### Entitlements required

Both iOS and macOS targets need the App Group entitlement:

```xml
<key>com.apple.security.application-groups</key>
<array>
    <string>group.com.ondeinference.apps</string>
</array>
```

The App Group must also be registered in the Apple Developer Portal under
**Identifiers → App Groups** before Xcode can provision it.

---

## Local development (Xcode)

Use the **local path** form during development — the remote binary form is only
needed for distribution:

```swift
// Package.swift (local dev only — never commit to onde-kit)
.binaryTarget(name: "OndeFramework", path: "./OndeFramework.xcframework")
```

In Xcode: **File → Add Package Dependencies → Add Local** → select `sdk/Onde/`.

---

## Common pitfalls

| Pitfall | Fix |
|---|---|
| Local `path:` in published `Package.swift` | Always `url:` + `checksum:` in `onde-swift` |
| Stale checksum after rebuild | Always recompute `shasum -a 256` — never hardcode |
| Missing simulator slice | lipo `x86_64` + `aarch64-sim` before passing to `xcodebuild` |
| UniFFI version drift | Keep `uniffi = "=0.31.0"` in both `Cargo.toml` and `uniffi-bindgen/Cargo.toml` |
| tvOS build with stable toolchain | tvOS targets are tier-3; always use `cargo +nightly -Z build-std` |
| Empty `Sources/` in `onde-swift` | Copy the generated `onde.swift` (and header) into the repo on every release |
| `___chkstk_darwin` linker error | Ensure `tvos_chkstk.s` exists at crate root; `build.rs` compiles it automatically |
| Missing Metal toolchain (Xcode 26+) | Run `xcodebuild -downloadComponent MetalToolchain`, then `cargo clean -p mistralrs-quant` |
| Cache path mismatch (`huggingface/` vs `models/`) | All SDKs MUST use `<container>/models/hub/` — using `huggingface/hub/` means models are not shared across apps |
| App Group not working | Register `group.com.ondeinference.apps` in Apple Developer Portal → Identifiers → App Groups; enable App Groups capability on the App ID; add entitlement to the target |
| `CODE_SIGN_IDENTITY = "-"` blocks App Groups | Remove ad-hoc signing from project-level build settings; App Groups require a real development certificate |
| Android NDK `LD` on PATH breaks Xcode | Never `export LD=$TOOLCHAIN/bin/ld` globally — it hijacks Xcode's linker. Use Cargo's `.cargo/config.toml` for Android linker config |