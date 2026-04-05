<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference</h1>

<p align="center">
  <strong>On-device LLM inference for Flutter & Dart — optimized for <a href="https://en.wikipedia.org/wiki/Apple_silicon">Apple silicon</a>.</strong>
</p>

<p align="center">
  <a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://apps.apple.com/se/developer/splitfire-ab/id1831430993"><img src="https://img.shields.io/badge/App%20Store-live-235843?style=flat-square&labelColor=17211D" alt="App Store"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/onde">Rust SDK</a> · <a href="https://github.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

## Features

- 🚀 **On-device inference** — models run entirely on the local CPU or GPU; no network request is ever made during inference
- ⚡ **Metal acceleration** on iOS and macOS (Apple silicon) for fast token generation
- 💬 **Multi-turn chat** with automatic conversation history management
- 🌊 **Streaming token delivery** via Dart `Stream<StreamChunk>` — display tokens as they are generated
- 🤖 **Qwen 2.5 1.5B and 3B** GGUF Q4\_K\_M models, downloaded from HuggingFace Hub on first use and cached locally
- 🎛️ **Configurable sampling** — temperature, top-p, top-k, min-p, max tokens, frequency/presence penalties
- 📱 **Platform-aware defaults** — automatically selects the 1.5B model on mobile and the 3B model on desktop
- 🦀 **Rust core** — the inference engine is written in Rust for safety, performance, and zero-overhead FFI

---

## Platform support

| Platform | GPU backend | Default model | Notes |
|----------|-------------|---------------|-------|
| iOS 13+ | Metal | Qwen 2.5 1.5B (~941 MB) | Simulator uses `aarch64-apple-ios-sim` |
| macOS 10.15+ | Metal | Qwen 2.5 3B (~1.93 GB) | Apple silicon & Intel supported |
| Android (API 21+) | CPU | Qwen 2.5 1.5B (~941 MB) | arm64-v8a, armeabi-v7a, x86\_64, x86 |
| Linux (x86\_64) | CPU | Qwen 2.5 3B (~1.93 GB) | CUDA builds possible — see docs |
| Windows (x86\_64) | CPU | Qwen 2.5 3B (~1.93 GB) | CUDA builds possible — see docs |

> **Web is not supported.** On-device LLM inference requires native system access that is not available in a browser sandbox.

---

## Quick start

### Add the dependency

```yaml
dependencies:
  onde_inference: ^0.1.0
```

> **Note:** The native inference engine is written in Rust and compiled automatically during the Flutter build. A working [Rust toolchain](https://rustup.rs) is required. The first build compiles the full dependency tree (~5–10 minutes cold, <1 minute incremental).

### Initialize

Call `OndeInference.init()` **once** at application startup, before creating any `OndeChatEngine`:

```dart
import 'package:onde_inference/onde_inference.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();
  runApp(const MyApp());
}
```

### Load a model

```dart
// Create the engine (synchronous — no model is loaded yet).
final engine = await OndeChatEngine.create();

// Load the platform-appropriate default model.
// On iOS / Android → Qwen 2.5 1.5B (~941 MB)
// On macOS / Linux / Windows → Qwen 2.5 3B (~1.93 GB)
final elapsed = await engine.loadDefaultModel(
  systemPrompt: 'You are a helpful assistant.',
);
print('Model loaded in ${elapsed.toStringAsFixed(1)} s');
```

### Chat

```dart
final result = await engine.sendMessage('What is Rust's ownership model?');
print(result.text);
print('Generated in ${result.durationDisplay}');
```

### Stream

```dart
final buffer = StringBuffer();

await for (final chunk in engine.streamMessage('Tell me a short story.')) {
  buffer.write(chunk.delta);

  // Update your UI with the partial text on each chunk.
  setState(() => _displayText = buffer.toString());

  if (chunk.done) break;
}
```

### Check engine status

```dart
final info = await engine.info();

print(info.status);        // EngineStatus.ready
print(info.modelName);     // "Qwen 2.5 3B"
print(info.approxMemory);  // "~1.93 GB"
print(info.historyLength); // number of turns in the conversation
```

### Manage conversation history

```dart
// Retrieve the full history.
final history = await engine.history();
for (final msg in history) {
  print('${msg.role}: ${msg.content}');
}

// Clear history (keeps the model loaded).
final removed = await engine.clearHistory();
print('Cleared $removed messages.');

// Seed history from a saved session without running inference.
await engine.pushHistory(ChatMessage.user('Hello from last session!'));
await engine.pushHistory(ChatMessage.assistant('Hi! How can I help today?'));
```

### One-shot generation (does not affect history)

```dart
// Useful for prompt enhancement, classification, summarisation, etc.
final result = await engine.generate(
  [
    ChatMessage.system('You are a JSON formatter. Output only valid JSON.'),
    ChatMessage.user('Name: Alice, Age: 30, City: Stockholm'),
  ],
  sampling: SamplingConfig.deterministic(),
);
print(result.text);
```

### Unload the model

```dart
// Release GPU / CPU memory when inference is no longer needed.
await engine.unloadModel();
```

---

## Model selection

Use `OndeInference` static helpers to pick a specific model:

```dart
// Platform-aware default (recommended).
final config = OndeInference.defaultModelConfig();

// Force a specific model regardless of platform.
final small  = OndeInference.qwen251_5bConfig();   // ~941 MB
final medium = OndeInference.qwen253bConfig();      // ~1.93 GB
final coder  = OndeInference.qwen25Coder3bConfig(); // ~1.93 GB, code-tuned

await engine.loadGgufModel(
  medium,
  systemPrompt: 'You are an expert software engineer.',
);
```

### Supported models

| Model | Size | Best for |
|-------|------|----------|
| Qwen 2.5 1.5B Instruct Q4\_K\_M | ~941 MB | iOS, tvOS, Android |
| Qwen 2.5 3B Instruct Q4\_K\_M | ~1.93 GB | macOS, Linux, Windows |
| Qwen 2.5 Coder 1.5B Instruct Q4\_K\_M | ~941 MB | Code generation on mobile |
| Qwen 2.5 Coder 3B Instruct Q4\_K\_M | ~1.93 GB | Code generation on desktop |

---

## Sampling

```dart
// All fields are optional — null means "use the engine default".
final sampling = SamplingConfig(
  temperature: 0.7,    // Higher = more creative, lower = more focused
  topP: 0.95,          // Nucleus sampling cutoff
  topK: 40,            // Top-k token limit
  maxTokens: 256,      // Maximum reply length in tokens
);

await engine.setSampling(sampling);

// Or use a preset:
await engine.setSampling(SamplingConfig.deterministic()); // greedy, temp=0.0
await engine.setSampling(SamplingConfig.mobile());        // temp=0.7, max 128 tokens
await engine.setSampling(SamplingConfig.defaultConfig()); // temp=0.7, max 512 tokens
```

---

## Error handling

All `OndeChatEngine` methods throw `OndeException` on failure:

```dart
try {
  await engine.loadDefaultModel();
} on OndeException catch (e) {
  debugPrint('Inference error: ${e.message}');
}
```

Common causes:
- **No model loaded** — calling `sendMessage` before `loadDefaultModel` / `loadGgufModel`
- **Download failure** — check internet connectivity on first run (model files are fetched from HuggingFace Hub)
- **Out of memory** — the 3B model requires ~2 GB of free RAM; use the 1.5B model on constrained devices

---

## Sandboxed app setup (iOS / macOS)

```dart
import 'package:onde_inference/onde_inference.dart';
import 'package:path_provider/path_provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();

  // Resolve shared App Group container (iOS/macOS) or private sandbox (Android).
  String? fallback;
  if (Platform.isIOS || Platform.isAndroid) {
    final dir = await getApplicationSupportDirectory();
    fallback = dir.path;
  }
  await OndeInference.setupCacheDir(fallbackDir: fallback);

  runApp(const MyApp());
}
```

> On iOS and macOS, `setupCacheDir()` first tries the App Group shared container (`group.com.ondeinference.apps`) so all Onde-powered apps share downloaded models. If unavailable, it falls back to the app's private directory.

---

## Contributing

Contributions are welcome! The project is hosted at
[github.com/ondeinference/onde](https://github.com/ondeinference/onde).

- Rust source: `onde/src/`
- Dart bridge Rust crate: `onde/sdk/dart/rust/`
- Dart library: `onde/sdk/dart/lib/`
- Example app: `onde/sdk/dart/example/`

Please open an issue before submitting a pull request for significant changes.

---

## License

MIT © [Splitfire AB](https://splitfire.se) — see [LICENSE](LICENSE).

<p align="center">
  <sub>© 2026 <a href="https://ondeinference.com">Onde Inference</a></sub>
</p>