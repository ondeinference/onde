## 1.0.0

This is the first stable release. Onde has already been running in real App Store apps for a while, so it felt like time to leave `0.x` behind.

### Assigned model loading

There is a new `loadAssignedModel(appId, appSecret)` method on `OndeChatEngine`. You register your app at [ondeinference.com](https://ondeinference.com), pick a model in the dashboard, and the SDK loads it at runtime. If nothing has been assigned yet, it falls back to the platform default. `loadDefaultModel` still works fine if you just want to get started quickly.

The example app includes `ONDE_APP_ID` and `ONDE_APP_SECRET` constants near the top. Fill those in and it will switch to the assigned-model flow automatically.

### New models

* Qwen 3 8B, 14B, and 1.7B (GGUF Q4_K_M)
* Qwen 2.5 Coder 7B (GGUF Q4_K_M)
* DeepSeek Coder 6.7B (GGUF Q4_K_M) with bundled chat template

### Type changes

* `GgufModelConfig` now has an optional `chatTemplate` field. This is for models that do not ship with a built-in template, like DeepSeek Coder.
* `InferenceResult` now includes a `toolCalls` array (`ToolCallInfo[]`). It will usually be empty, but when a model asks for a tool call, you now get structured data instead of raw markup in `text`.
* Added a `ToolCallInfo` type export.

### Engine

* Old model weights are now dropped outside the lock during model swaps, so status queries do not get stuck while memory is being freed.

### Dependencies

* Switched from the git-based `mistralrs` dependency to the published `onde-mistralrs 0.8.2` crates on crates.io.

### Cross-platform

* Linux and Windows CPU inference builds now work properly too, thanks to a `TokenSource` fix for non-Darwin platforms.

## 0.1.7

* **CI:** No user-facing changes here. This was an internal release to keep version numbers aligned across the SDKs.

## 0.1.6

* **Fix:** Replaced the composite `LICENSE` file with the standard MIT license text so package registries correctly recognise the OSI-approved license.

## 0.1.5

* **Engine:** Added `loadAssignedModel()`. It fetches the model config assigned to your app from the Onde SDK backend using app credentials, with no user JWT required. If no model has been assigned yet, it falls back to the platform default.
* **Telemetry:** Added the GresIQ pulse telemetry client. The engine now reports usage events to the GresIQ dashboard. Configure it with the `GRESIQ_ENVIRONMENT` and `ONDE_EDGE_ID` environment variables before the engine starts.
* **Build:** GresIQ API credentials (`GRESIQ_API_KEY`, `GRESIQ_API_SECRET`, `GRESIQ_APP_ID`) are now embedded at build time through `dotenvy`, so CI can inject secrets through environment variables without changing source files.

## 0.1.4

* Added the Qwen 3 4B GGUF model (`bartowski/Qwen_Qwen3-4B-GGUF`) with full OpenAI-compatible tool calling support.
* Added the `qwen3_4b()` model config and registered it in the supported model list.

## 0.1.3

* First release of the React Native Expo module.
* Multi-turn chat inference with Qwen 2.5 1.5B and 3B GGUF models.
* Metal acceleration on iOS (Apple silicon).
* CPU inference on Android.
* Platform-aware default model selection, with 1.5B on iOS and Android.
* Conversation history management through `history()`, `clearHistory()`, and `pushHistory()`.
* A one-shot `generate()` API that does not change conversation history.
* Configurable sampling, including temperature, top-p, top-k, min-p, max tokens, and penalties.
* Built-in sampling presets: `defaultSamplingConfig()`, `deterministicSamplingConfig()`, and `mobileSamplingConfig()`.
* Engine status through `info()`, including status, loaded model name, memory, and history length.
