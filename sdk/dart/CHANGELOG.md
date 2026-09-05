## 1.2.4

- Dependency updates across the Rust tree
- onde-tqdm is retired: the mistral.rs fork now depends on upstream tqdm 0.8.1 directly, which carries the merged crossterm 0.29 bump
- Release workflow fixes for the Dart and Kotlin SDK jobs (Flutter pub get no longer fails the publish on a dirty tree; the Kotlin job installs the pinned Rust toolchain before adding Android targets)

## 1.2.3

- UQFF model support: `OndeInference.uqffModelConfig(...)` builds a config for a pre-quantised UQFF export, and `OndeChatEngine.loadUqffModel(...)` loads it. UQFF weights are quantised ahead of time, so loading skips the memory spike of in-situ quantisation
- Android native libraries are now built with cargo-ndk instead of the hand-rolled CMake path, which fixes NDK version comparison and the ordering between the native-lib build and the Gradle tasks that consume it
- Dependency updates

## 1.2.2

- Moved onto the published onde-mistralrs 0.9.3 crates
- Dropped the mio visionOS patch — the cfg gating it worked around now ships upstream, so the Apple XCFramework builds without it

## 1.2.1

- Fixed the Android build: gated the 10 GiB UQFF chunk-size constant so it no longer overflows `usize` on 32-bit targets (armv7)
- Moved onto the published onde-mistralrs 0.9.2 crates (fork synced with upstream 0.9.x)

## 1.2.0

- Full Qwen 3 model family support: 0.6B, 1.7B, 4B, 8B, 14B, 32B, plus the 4B Instruct/Thinking 2507 refreshes and the 30B-A3B MoE variant, each with a matching config free function
- Synced the mistral.rs fork with upstream 0.8.23, pulling in Metal compile fixes
- Refreshed branding and legal wording across generated docs
- Dependency updates

## 1.1.4

- Bump smbCloud GresIQ SDK to 0.4.7 and dual-write pulse telemetry to the GresIQ document gateway (ADR-0003 M2); telemetry events now carry an optional `onde_app_id`

## 1.1.3

- Bump smbCloud GresIQ SDK to 0.4.6 (telemetry transport now tracks the latest smbCloud platform SDK line)

## 1.1.2

- tvOS support: 0.5B model default, memory-optimized model builder, snapshot cache repair
- New `configure_cache_dir` FFI function for sandboxed platforms
- New `qwen25_0_5b_config()` free function

## 1.1.1

* **Apple SDK compatibility:** Pulls in the Onde Rust core `1.1.1` Apple deployment-target build fixes so downstream macOS consumers do not inherit newer-than-expected minimum OS metadata from the packaged native binaries.
* **Release alignment:** Aligns the Flutter/Dart package version with the Rust, Swift, Kotlin, and React Native SDKs.

## 1.1.0

* **Swift / UniFFI stability:** Pulls in the Onde Rust core `1.1.0` runtime fixes for Swift/Kotlin/Apple-hosted SDKs, including Tokio runtime annotations and panic-safe pulse telemetry initialization.
* **Telemetry:** Pulse telemetry now gracefully disables itself if a Tokio reactor is unavailable and can be explicitly disabled with `ONDE_DISABLE_PULSE=1` during local validation.
* **Release alignment:** Aligns the Flutter/Dart package version with the Rust, Swift, Kotlin, and React Native SDKs.

## 1.0.2

* **pub.dev:** Added Apple plugin Swift Package Manager manifests at `ios/onde_inference/Package.swift` and `macos/onde_inference/Package.swift` so pub.dev recognizes modern iOS and macOS plugin toolchain support.
* **pub.dev:** Removed redundant analyzer ignore directives from generated Dart bridge files to recover the remaining static-analysis points.
* **Tooling:** Ignore Swift Package Manager build directories (`.build/`, `.swiftpm/`) in both Git and pub publishing.

## 1.0.1

* **pub.dev:** Fixed stale README and API examples so the published package matches the current API (`OndeChatEngine()`, named `message:` arguments, current sampling helpers, and `OndeError` handling).
* **pub.dev:** Added explicit package metadata for documentation, topics, and supported platforms to improve pub.dev platform detection and package discoverability.
* **pub.dev:** Verified lower-bound compatibility again after the FRB-generated `toolCalls` and `loadAssignedModel` changes that previously hurt the package score on 1.0.0.

## 1.0.0

This is the first stable release. Onde has already been running in real App Store apps for months, so keeping it on `0.x` no longer felt right.

### New: assigned model loading

`loadAssignedModel(appId:appSecret:)` fetches the model you've assigned to your app in the [ondeinference.com](https://ondeinference.com) dashboard. If nothing has been assigned yet, it falls back to the platform default. This is the path we recommend for production apps. `loadDefaultModel` is still there if you just want to prototype quickly.

The example app reads credentials from `--dart-define=ONDE_APP_ID=...` and `--dart-define=ONDE_APP_SECRET=...`. If you do not pass them, it falls back to the default model like before.

### New models

* Qwen 3 8B, 14B, and 1.7B (GGUF Q4_K_M)
* Qwen 2.5 Coder 7B (GGUF Q4_K_M)
* DeepSeek Coder 6.7B (GGUF Q4_K_M) with bundled chat template

### Type changes

* `GgufModelConfig` now has an optional `chatTemplate` field for models that need a custom chat template, like DeepSeek Coder.
* `InferenceResult` now carries a `toolCalls` list (`List<ToolCallInfo>`). Most responses will still return an empty list, but if the model asks for a tool call, you now get structured data instead of raw markup in `text`. The `InferenceResultToolsX` extension also adds a `hasToolCalls` convenience getter.

### Engine

* Old model weights are now dropped outside the lock when loading a new model. Before this, the drop happened while the write lock was still held, which meant status queries could stall while memory was being released.

### Dependencies

* Switched from the git-based `mistralrs` dependency to the published `onde-mistralrs 0.8.2` crates on crates.io. Builds are faster, and `cargo publish` no longer needs `[patch.crates-io]` gymnastics.

### Cross-platform

* Linux and Windows builds with CPU inference now work out of the box, thanks to a `TokenSource` fix for non-Darwin platforms.

## 0.1.7

* **Fix:** Removed `example/android/app/src/main/java/io/flutter/plugins/GeneratedPluginRegistrant.java` from git tracking and added it to `.gitignore`. Flutter regenerates this file on every CI run, which kept leaving the working tree dirty and blocked `pub publish`. Added a CI restore step as an extra safety net.

## 0.1.6

* **Fix:** Replaced the composite `LICENSE` file with the standard MIT license text so pub.dev's `pana` tool correctly recognises the OSI-approved license and gives the package full license credit.

## 0.1.5

* **Engine:** Added `load_assigned_model()`. It fetches the model config assigned to your app from the Onde SDK backend using app credentials, with no user JWT required. If no model has been assigned yet, it falls back to the platform default.
* **Telemetry:** Added the GresIQ pulse telemetry client. The engine now reports usage events to the GresIQ dashboard. Configure it with the `GRESIQ_ENVIRONMENT` and `ONDE_EDGE_ID` environment variables before the engine starts.
* **Build:** GresIQ API credentials (`GRESIQ_API_KEY`, `GRESIQ_API_SECRET`, `GRESIQ_APP_ID`) are now embedded at build time through `dotenvy`, so CI can inject secrets through environment variables without changing source files.

## 0.1.4

* Added the Qwen 3 4B GGUF model (`bartowski/Qwen_Qwen3-4B-GGUF`) with full OpenAI-compatible tool calling support.
* Added the `GgufModelConfig.qwen3_4b()` constructor and registered it in the supported model list.

## 0.1.3

* **Platform:** Added support for watchOS and visionOS.

## 0.1.2

* **Engine:** Switched all platform-specific `mistralrs` and `mistralrs-core` dependencies to the `setoelkahfi/mistral.rs` fork (branch `fix/all-platform-fixes`) to pick up cross-platform stability fixes before they landed upstream.
* **License:** Moved to dual licensing under MIT OR Apache-2.0, and added `LICENSE-APACHE` alongside the existing `LICENSE-MIT` for pub.dev compliance.
* **Dependencies:** Upgraded `freezed_annotation` to `^3.1.0` and `freezed` to `^3.2.5`.
* Removed a stale `ignore_for_file` directive from the generated Flutter Rust Bridge glue code.

## 0.1.1

* CI/CD: `release-sdk-dart.yml` now publishes `onde_inference` to pub.dev on tag push.
* Added copyright headers to all hand-written source files (`engine.dart`, `types.dart`, `dart_test.dart`, and the iOS and macOS Swift plugin classes).
* Rewrote the example app README with updated branding, platform notes, and an SDK quick reference.
* Added `android/local.properties` to `.gitignore`, so local SDK paths no longer show up in diffs.

## 0.1.0

* Initial MVP release.
* Multi-turn chat inference with Qwen 2.5 1.5B and 3B GGUF Q4_K_M models.
* Streaming token delivery via Dart `Stream<StreamChunk>`, so you can display tokens as they are generated.
* Metal acceleration on iOS and macOS (Apple silicon and Intel).
* CPU inference on Android, Linux, and Windows.
* Platform-aware default model selection (1.5B on iOS / Android, 3B on macOS / Linux / Windows).
* Conversation history management: `history()`, `clearHistory()`, `pushHistory()`.
* One-shot `generate()` API that does not affect the conversation history.
* Configurable sampling: temperature, top-p, top-k, min-p, max tokens, frequency and presence penalties.
* Built-in sampling presets: `SamplingConfig.defaultConfig()`, `SamplingConfig.deterministic()`, `SamplingConfig.mobile()`.
* `EngineInfo` snapshot: status, loaded model name, approximate memory, and history length.
* `OndeInference` static helper namespace for library initialisation and model / sampling config factories.
* Compilation stub (`frb_generated_stub.dart`) so the package compiles before the native Rust bridge is built.
* Powered by [flutter_rust_bridge v2](https://pub.dev/packages/flutter_rust_bridge) and the [Onde](https://ondeinference.com) Rust engine.
