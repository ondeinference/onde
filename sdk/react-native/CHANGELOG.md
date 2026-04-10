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