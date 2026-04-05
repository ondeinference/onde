## 0.1.0

* Initial MVP release.
* Multi-turn chat inference with Qwen 2.5 1.5B and 3B GGUF Q4_K_M models.
* Streaming token delivery via Dart `Stream<StreamChunk>` — display tokens as they are generated.
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