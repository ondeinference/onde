## 1.0.0

Onde has been running in production across multiple Splitfire AB apps on the App Store for months. The `0.x` versioning wasn't reflecting reality anymore.

### Assigned model loading (FFI)

`load_assigned_model` is now exposed through the UniFFI layer. Swift and Kotlin consumers can call it directly — the method hardcodes `Environment::Production` so SDK consumers don't need to know about the internal environment enum. Pass your `app_id` and `app_secret` from [ondeinference.com](https://ondeinference.com), and the SDK fetches whatever model you've assigned in the dashboard. Falls back to the platform default if nothing's assigned.

### New models

* Qwen 3 1.7B, 4B, 8B, and 14B (GGUF Q4_K_M)
* Qwen 2.5 Coder 1.5B, 3B, and 7B (GGUF Q4_K_M)
* DeepSeek Coder 6.7B (GGUF Q4_K_M) — ships with a bundled chat template since the GGUF doesn't include one

### Type changes

* `GgufModelConfig` has a new `chat_template: Option<String>` field for models that need a custom template.
* `InferenceResult` now carries `tool_calls: Vec<ToolCallInfo>`. Empty for most responses. When a model requests a tool call, you get structured data instead of raw markup in `text`.
* New types: `ToolCallInfo`, `ToolDefinition`, `ToolResult`. The last two are Rust-only for now — they'll hit the FFI surface when `send_message_with_tools` is wired up.

### Engine

* Model weights are dropped outside the write lock during model swaps. Status queries no longer block while the old model's memory is being freed.
* `platform_default()` now returns Coder variants (Qwen 2.5 Coder 1.5B on mobile, Coder 3B on desktop).

### Tool calling (Rust-native only)

* `send_message_with_tools`, `send_tool_results`, and `stream_tool_results` on `ChatEngine`. These are available to direct Rust consumers (like [siGit](https://github.com/smbcloud/sigit)) but not yet exposed through UniFFI. The types cross the FFI boundary on the output side (`InferenceResult.tool_calls`), just not the input side yet.

### Dependencies

* Switched from git-based `mistralrs` to published `onde-mistralrs 0.8.2` crates on crates.io. No more `[patch.crates-io]` gymnastics for `cargo publish`.

### Cross-platform

* Linux and Windows CPU builds work properly now — fixed `TokenSource` for non-Darwin platforms.
* Added cross-compile CI for all supported targets.

## 0.1.8

* Published to crates.io with `onde-mistralrs 0.8.2` registry dependencies (replacing git refs that `cargo publish` strips).

## 0.1.7

* Qwen 3 4B GGUF model with OpenAI-compatible tool calling.
* `GgufModelConfig::qwen3_4b()` constructor.

## 0.1.6

* License fix: canonical MIT text so crates.io recognizes the license correctly.

## 0.1.5

* `ChatEngine::load_assigned_model()` — Rust-native API for fetching operator-assigned models from the GresIQ backend.
* GresIQ pulse telemetry client (build-time credential embedding via `dotenvy`).
* watchOS and visionOS platform support.

## 0.1.4

* Switched all `mistralrs` dependencies to the `fix/all-platform-fixes` fork for cross-platform stability.
* Dual-licensed MIT OR Apache-2.0.

## 0.1.3

* Added watchOS and visionOS targets.

## 0.1.2

* HF cache sandbox workaround for iOS and Android (`GLOBAL_HF_CACHE` seeding).
* `configure_cache_dir` free function for sandboxed platforms.

## 0.1.1

* CI/CD: `release-sdk-swift.yml` builds the XCFramework and auto-updates `onde-swift` on tag push.

## 0.1.0

* Initial release.
* Multi-turn chat with Qwen 2.5 1.5B and 3B GGUF models.
* Metal on iOS/macOS, CPU on Android/Linux/Windows.
* Streaming inference via `tokio::sync::mpsc`.
* Conversation history management.
* One-shot `generate()` API.
* Configurable sampling with presets.
* UniFFI bindings for Swift/Kotlin.