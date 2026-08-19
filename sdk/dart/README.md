<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference</h1>

<p align="center">
  <strong>Run LLMs on-device from Flutter and Dart with <a href="https://ondeinference.com/">Onde Inference</a>. Metal on iOS and macOS, CPU everywhere else.</strong>
</p>

<p align="center">
  <a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
</p>

<p align="center">
  <a href="https://github.com/ondeinference/onde">Rust SDK</a> · <a href="https://swiftpackageindex.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference">Kotlin Multiplatform SDK</a> · <a href="https://www.npmjs.com/package/@ondeinference/react-native">React Native SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

Run Qwen 2.5 models directly inside your Flutter app. The model downloads from Hugging Face on first launch, then everything runs locally. No server, no API key, and no user data leaves the device. On an iPhone 15 Pro, Metal reaches around 15 tok/s. Android, Linux, and Windows run on CPU, so they are slower but still useful for fully local inference.

You get multi-turn chat, streaming, one-shot generation, configurable sampling, and structured tool call metadata in one package.

## Platform support

| Platform | Backend | Default model | Notes |
|----------|---------|---------------|-------|
| iOS 13+ | Metal | Qwen 2.5 Coder 1.5B (~941 MB) | CocoaPods and Swift Package Manager plugin manifests are included |
| macOS 10.15+ | Metal | Qwen 2.5 Coder 3B (~1.93 GB) | CocoaPods and Swift Package Manager plugin manifests are included |
| Android API 21+ | CPU | Qwen 2.5 Coder 1.5B (~941 MB) | arm64-v8a, armeabi-v7a, x86_64 by default; see [Android ABIs](#android-abis) |
| Linux x86_64 | CPU | Qwen 2.5 Coder 3B (~1.93 GB) | CUDA possible, see docs |
| Windows x86_64 | CPU | Qwen 2.5 Coder 3B (~1.93 GB) | CUDA possible, see docs |

Web is not supported. On-device inference needs native system access that browsers do not expose.

---

## Quick start

```yaml
dependencies:
  onde_inference: ^1.0.2
```

The inference engine is written in Rust and connected to Dart through [flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge). You need a working [Rust toolchain](https://rustup.rs). The first build is usually slow because it compiles the full native dependency tree.

### Initialize

Call this once at startup before creating any `OndeChatEngine`:

```dart
import 'package:flutter/widgets.dart';
import 'package:onde_inference/onde_inference.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();
  runApp(const MyApp());
}
```

### Load a model

```dart
final engine = OndeChatEngine();

final elapsed = await engine.loadDefaultModel(
  systemPrompt: 'You are a helpful assistant.',
);

print('Model loaded in ${elapsed.toStringAsFixed(1)} s');
```

For production, you can load the model assigned to your Onde app from the dashboard:

```dart
final assignedElapsed = await engine.loadAssignedModel(
  appId: 'your-app-id',
  appSecret: 'your-app-secret',
  systemPrompt: 'You are a helpful assistant.',
);

print('Assigned model loaded in ${assignedElapsed.toStringAsFixed(1)} s');
```

### Chat

```dart
final result = await engine.sendMessage(
  message: 'What is Rust ownership?',
);

print(result.text);
print(result.durationDisplay);
print(result.toolCalls);
```

### Stream

```dart
final buffer = StringBuffer();

await for (final chunk in engine.streamMessage(message: 'Tell me a short story.')) {
  buffer.write(chunk.delta);
  if (chunk.done) break;
}

print(buffer.toString());
```

### Status and history

```dart
final info = await engine.info();
print(info.status);
print(info.modelName);
print(info.approxMemory);
print(info.historyLength);

final history = await engine.history();
for (final msg in history) {
  print('${msg.role}: ${msg.content}');
}

final removed = await engine.clearHistoryCount();
print('Cleared $removed messages.');
```

### One-shot generation

This runs inference without modifying conversation history.

```dart
final result = await engine.generate(
  messages: [
    ChatMessage(role: ChatRole.system, content: 'Output valid JSON only.'),
    ChatMessage(role: ChatRole.user, content: 'Name: Alice, Age: 30'),
  ],
  sampling: OndeInference.deterministicSamplingConfig(),
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
final config = OndeInference.defaultModelConfig();
final small = OndeInference.qwen2515bConfig();
final medium = OndeInference.qwen253bConfig();
final coder = OndeInference.qwen25Coder3bConfig();

await engine.loadGgufModel(
  config: coder,
  systemPrompt: 'You are an expert software engineer.',
);
```

Pre-quantized UQFF models use the base repository (or local model directory)
for tokenizer/configuration resolution and the first UQFF shard as the file:

```dart
final uqff = OndeInference.uqffModelConfig(
  modelId: 'google/gemma-4-E4B-it',
  files: ['q4k-0.uqff'],
  displayName: 'Gemma 4 E4B (UQFF Q4K)',
  approxMemory: '~2.5 GB (UQFF Q4K)',
);

await engine.loadUqffModel(
  config: uqff,
  systemPrompt: 'You are an expert software engineer.',
);
```

For sharded UQFF models, pass the first shard; mistral.rs discovers sibling
shards with the same prefix.

| Model | Size | Good for |
|-------|------|----------|
| Qwen 2.5 1.5B Instruct Q4_K_M | ~941 MB | iOS, tvOS, Android |
| Qwen 2.5 3B Instruct Q4_K_M | ~1.93 GB | macOS, Linux, Windows |
| Qwen 2.5 Coder 1.5B Instruct Q4_K_M | ~941 MB | Code on mobile |
| Qwen 2.5 Coder 3B Instruct Q4_K_M | ~1.93 GB | Code on desktop |

---

## Sampling

All sampling fields are optional. `null` means "use the engine default".

```dart
final sampling = SamplingConfig(
  temperature: 0.7,
  topP: 0.95,
  topK: BigInt.from(40),
  maxTokens: BigInt.from(256),
);

await engine.setSampling(sampling: sampling);
```

Presets:

```dart
OndeInference.defaultSamplingConfig();
OndeInference.deterministicSamplingConfig();
OndeInference.mobileSamplingConfig();
```

---

## Error handling

The generated bridge throws `OndeError` values directly:

```dart
try {
  await engine.loadDefaultModel();
} on OndeError catch (e) {
  debugPrint('Inference error: $e');
}
```

Common causes include calling `sendMessage` before loading a model, having no internet on first run while the model still needs to download, or running out of memory.

---

## Sandboxed app setup (iOS / macOS / Android)

On iOS, macOS, and Android, configure the Hugging Face cache directory before loading a model. On Apple platforms, Onde first tries the shared App Group container (`group.com.ondeinference.apps`) and falls back to your provided directory.

```dart
import 'dart:io' show Platform;

import 'package:flutter/widgets.dart';
import 'package:onde_inference/onde_inference.dart';
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();

  String? fallbackDir;
  if (Platform.isIOS || Platform.isAndroid) {
    final dir = await getApplicationSupportDirectory();
    fallbackDir = dir.path;
  }

  await OndeInference.setupCacheDir(fallbackDir: fallbackDir);
  runApp(const MyApp());
}
```

---

## Android ABIs

The Rust engine is cross-compiled during `flutter build apk` by the plugin's
Gradle module, which shells out to [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk).
You need it once per machine, along with the Rust targets:

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

The NDK comes from whichever version Gradle already resolved, so no
`ANDROID_NDK_HOME` export is needed for Gradle-driven builds.

Unless you say otherwise, all three default ABIs are built: arm64-v8a,
armeabi-v7a, and x86_64. Each is a full release build of a large crate, so this
is slow. While iterating, build only the one your device needs:

```bash
flutter run -d <device> --android-project-arg=onde.androidAbis=arm64-v8a
```

## Example app

A full Flutter example lives in `example/`. It demonstrates:

- `OndeChatEngine()` lifecycle
- assigned-model loading with dashboard credentials
- streaming chat UI
- sampling preset switching
- cache directory setup for sandboxed platforms

Run it locally from `sdk/dart/example/`.

## Contributing

The source lives at [github.com/ondeinference/onde](https://github.com/ondeinference/onde):

- Rust core: `src/`
- Dart bridge crate: `sdk/dart/rust/`
- Dart library: `sdk/dart/lib/`
- Example app: `sdk/dart/example/`

Open an issue before sending large PRs.

## License

Onde is dual-licensed under **MIT** and **Apache 2.0**. You can use either one.

- [MIT License](https://github.com/ondeinference/onde/blob/main/LICENSE-MIT)
- [Apache License 2.0](https://github.com/ondeinference/onde/blob/main/LICENSE-APACHE)

---

## Copyright

© 2026 [Splitfire AB](https://5mb.app) ([Onde Inference](https://ondeinference.com)).
