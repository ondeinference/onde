<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference</h1>

<p align="center">
  <strong>On-device LLM inference for Flutter & Dart — Metal on iOS and macOS, CPU everywhere else.</strong>
</p>

<p align="center">
  <a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
  <a href="https://crates.io/crates/onde"><img src="https://img.shields.io/crates/v/onde?style=flat-square&color=235843&labelColor=17211D&label=crates.io" alt="crates.io"></a>
  <a href="https://swiftpackageindex.com/ondeinference/onde-swift"><img src="https://img.shields.io/badge/Swift%20Package%20Index-onde--swift-235843?style=flat-square&labelColor=17211D" alt="Swift Package Index"></a>
  <a href="https://www.npmjs.com/package/@ondeinference/react-native"><img src="https://img.shields.io/npm/v/@ondeinference/react-native?style=flat-square&color=235843&labelColor=17211D&label=npm" alt="npm"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://apps.apple.com/se/developer/splitfire-ab/id1831430993"><img src="https://img.shields.io/badge/App%20Store-live-235843?style=flat-square&labelColor=17211D" alt="App Store"></a>
</p>

<p align="center">
  <a href="https://github.com/ondeinference/onde">Rust SDK</a> · <a href="https://swiftpackageindex.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://www.npmjs.com/package/@ondeinference/react-native">React Native SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

Run Qwen 2.5 models inside your Flutter app. The model downloads from HuggingFace on first launch, then everything runs locally — no server, no API key, nothing leaves the device. Metal gives you ~15 tok/s on an iPhone 15 Pro; Android and desktop run on CPU, slower but it works.

Multi-turn chat, streaming, one-shot generation, configurable sampling — the full API is one import away.

## Platform support

| Platform | Backend | Default model | Notes |
|----------|---------|---------------|-------|
| iOS 13+ | Metal | Qwen 2.5 1.5B (~941 MB) | Simulator uses `aarch64-apple-ios-sim` |
| macOS 10.15+ | Metal | Qwen 2.5 3B (~1.93 GB) | Apple silicon and Intel |
| Android API 21+ | CPU | Qwen 2.5 1.5B (~941 MB) | arm64-v8a, armeabi-v7a, x86_64, x86 |
| Linux x86_64 | CPU | Qwen 2.5 3B (~1.93 GB) | CUDA possible, see docs |
| Windows x86_64 | CPU | Qwen 2.5 3B (~1.93 GB) | CUDA possible, see docs |

Web is not supported. On-device inference needs native system access that browsers don't expose.

---

## Quick start

```yaml
dependencies:
  onde_inference: ^0.1.0
```

The inference engine is Rust compiled via [flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge). You need a working [Rust toolchain](https://rustup.rs). First build is slow (~5–10 min, compiling the full dep tree); incremental builds are under a minute.

### Initialize

Call once at startup, before anything else:

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
final engine = await OndeChatEngine.create();

// Picks the right model for the device:
//   iOS / Android → Qwen 2.5 1.5B (~941 MB)
//   macOS / Linux / Windows → Qwen 2.5 3B (~1.93 GB)
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
  setState(() => _displayText = buffer.toString());
  if (chunk.done) break;
}
```

### Engine status

```dart
final info = await engine.info();

print(info.status);        // EngineStatus.ready
print(info.modelName);     // "Qwen 2.5 3B"
print(info.approxMemory);  // "~1.93 GB"
print(info.historyLength); // number of turns so far
```

### History

```dart
final history = await engine.history();
for (final msg in history) {
  print('${msg.role}: ${msg.content}');
}

// Clear history but keep the model loaded.
final removed = await engine.clearHistory();
print('Cleared $removed messages.');

// Seed from a saved session — no inference runs.
await engine.pushHistory(ChatMessage.user('Hello from last session!'));
await engine.pushHistory(ChatMessage.assistant('Hi! How can I help today?'));
```

### One-shot generation

Runs inference without touching conversation history. Good for prompt enhancement, classification, formatting.

```dart
final result = await engine.generate(
  [
    ChatMessage.system('You are a JSON formatter. Output only valid JSON.'),
    ChatMessage.user('Name: Alice, Age: 30, City: Stockholm'),
  ],
  sampling: SamplingConfig.deterministic(),
);
print(result.text);
```

### Unload

```dart
await engine.unloadModel();
```

---

## Model selection

```dart
// Platform-aware default (recommended).
final config = OndeInference.defaultModelConfig();

// Force a specific model.
final small  = OndeInference.qwen251_5bConfig();   // ~941 MB
final medium = OndeInference.qwen253bConfig();      // ~1.93 GB
final coder  = OndeInference.qwen25Coder3bConfig(); // ~1.93 GB, code-tuned

await engine.loadGgufModel(
  medium,
  systemPrompt: 'You are an expert software engineer.',
);
```

| Model | Size | Good for |
|-------|------|----------|
| Qwen 2.5 1.5B Instruct Q4_K_M | ~941 MB | iOS, tvOS, Android |
| Qwen 2.5 3B Instruct Q4_K_M | ~1.93 GB | macOS, Linux, Windows |
| Qwen 2.5 Coder 1.5B Instruct Q4_K_M | ~941 MB | Code on mobile |
| Qwen 2.5 Coder 3B Instruct Q4_K_M | ~1.93 GB | Code on desktop |

---

## Sampling

All fields are optional. `null` means "use the engine default".

```dart
final sampling = SamplingConfig(
  temperature: 0.7,
  topP: 0.95,
  topK: 40,
  maxTokens: 256,
);

await engine.setSampling(sampling);
```

Presets:

```dart
SamplingConfig.defaultConfig()   // temp=0.7, max 512 tokens
SamplingConfig.deterministic()   // greedy, temp=0.0
SamplingConfig.mobile()          // temp=0.7, max 128 tokens
```

---

## Error handling

All engine methods throw `OndeException` on failure:

```dart
try {
  await engine.loadDefaultModel();
} on OndeException catch (e) {
  debugPrint('Inference error: ${e.message}');
}
```

Common causes: calling `sendMessage` before loading a model, no internet on first run (the model needs to download), or out of memory (the 3B model needs ~2 GB free — use 1.5B on constrained devices).

---

## Sandboxed app setup (iOS / macOS)

On iOS and sandboxed macOS, the default HuggingFace cache path is outside the app container. Call `setupCacheDir()` once at startup to point it somewhere accessible:

```dart
import 'package:onde_inference/onde_inference.dart';
import 'package:path_provider/path_provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();

  String? fallback;
  if (Platform.isIOS || Platform.isAndroid) {
    final dir = await getApplicationSupportDirectory();
    fallback = dir.path;
  }
  await OndeInference.setupCacheDir(fallbackDir: fallback);

  runApp(const MyApp());
}
```

This first tries the App Group shared container (`group.com.ondeinference.apps`) so all Onde-powered apps share downloaded models. Falls back to the app's private directory if the App Group isn't configured.

---

## Contributing

Source lives at [github.com/ondeinference/onde](https://github.com/ondeinference/onde):

- Rust core: `src/`
- Dart bridge crate: `sdk/dart/rust/`
- Dart library: `sdk/dart/lib/`
- Example app: `sdk/dart/example/`

Open an issue before sending large PRs.

## License

Dual-licensed under **MIT** and **Apache 2.0**. Pick whichever works for you.

- [MIT License](https://github.com/ondeinference/onde/blob/main/LICENSE-MIT)
- [Apache License 2.0](https://github.com/ondeinference/onde/blob/main/LICENSE-APACHE)

© 2026 [Splitfire AB](https://splitfire.se)

---

<p align="center">
  <sub>© 2026 <a href="https://ondeinference.com">Onde Inference</a></sub>
</p>