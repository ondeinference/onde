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
