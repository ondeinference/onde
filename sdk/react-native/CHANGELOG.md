## 1.0.0

First stable release. Onde has been in production across multiple App Store apps for a while now, so we're dropping the `0.x`.

### Assigned model loading

New `loadAssignedModel(appId, appSecret)` method on `OndeChatEngine`. Register your app at [ondeinference.com](https://ondeinference.com), assign a model in the dashboard, and the SDK fetches it at runtime. Falls back to the platform default if nothing's assigned yet. `loadDefaultModel` still works for quick prototyping.

The example app has `ONDE_APP_ID` and `ONDE_APP_SECRET` constants at the top — fill them in and it switches to the assigned model path automatically.

### New models

* Qwen 3 8B, 14B, and 1.7B (GGUF Q4_K_M)
* Qwen 2.5 Coder 7B (GGUF Q4_K_M)
* DeepSeek Coder 6.7B (GGUF Q4_K_M) with bundled chat template

### Type changes

* `GgufModelConfig` has a new optional `chatTemplate` field. Models that ship without a built-in template (like DeepSeek Coder) use this.
* `InferenceResult` now includes a `toolCalls` array (`ToolCallInfo[]`). Usually empty — but when a model decides to request a tool call, you get structured data instead of raw markup in `text`.
* Added `ToolCallInfo` type export.

### Engine

* Old model weights are dropped outside the lock during model swaps, so status queries don't block while memory is being freed.

### Dependencies

* Switched from git-based `mistralrs` to published `onde-mistralrs 0.8.2` crates on crates.io.

### Cross-platform

* Linux and Windows CPU inference builds work properly now (TokenSource fix for non-Darwin platforms).

## 0.1.7

* **CI:** No user-visible changes. Internal release to keep version numbers in sync across all SDK distributions.

## 0.1.6

* **Fix:** Replaced the composite `LICENSE` file with the canonical MIT license text so registries correctly recognise the OSI-approved license.

## 0.1.5

* **Engine:** Added `loadAssignedModel()` — fetches the operator-assigned model config from the Onde SDK backend using app credentials (no user JWT required); falls back gracefully to the platform default when no model is assigned yet.
* **Telemetry:** Added GresIQ pulse telemetry client. The engine now reports usage events to the GresIQ dashboard. Configure via `GRESIQ_ENVIRONMENT` and `ONDE_EDGE_ID` env vars before the engine initialises.
* **Build:** GresIQ API credentials (`GRESIQ_API_KEY`, `GRESIQ_API_SECRET`, `GRESIQ_APP_ID`) are now embedded at build time via `dotenvy`. CI can inject secrets via env vars without modifying source.

## 0.1.4

* Added Qwen 3 4B GGUF model (`bartowski/Qwen_Qwen3-4B-GGUF`) with full OpenAI-compatible tool calling support.
* Added `qwen3_4b()` model config and registered it in the supported model list.

## 0.1.3

* Initial release of the React Native Expo module.
* Multi-turn chat inference with Qwen 2.5 1.5B and 3B GGUF models.
* Metal acceleration on iOS (Apple silicon).
* CPU inference on Android.
* Platform-aware default model selection (1.5B on iOS / Android).
* Conversation history management: `history()`, `clearHistory()`, `pushHistory()`.
* One-shot `generate()` API that does not affect conversation history.
* Configurable sampling: temperature, top-p, top-k, min-p, max tokens, penalties.
* Built-in sampling presets: `defaultSamplingConfig()`, `deterministicSamplingConfig()`, `mobileSamplingConfig()`.
* Engine status via `info()`: status, loaded model name, memory, history length.
