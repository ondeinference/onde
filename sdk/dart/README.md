# onde_inference

[![pub.dev](https://img.shields.io/pub/v/onde_inference.svg)](https://pub.dev/packages/onde_inference)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ondeinference/onde/blob/main/sdk/dart/LICENSE)
[![Platform](https://img.shields.io/badge/platform-iOS%20%7C%20macOS%20%7C%20Android%20%7C%20Linux%20%7C%20Windows-lightgrey)](https://pub.dev/packages/onde_inference)

**On-device LLM inference for Flutter & Dart.**

Run [Qwen 2.5](https://huggingface.co/bartowski/Qwen2.5-3B-Instruct-GGUF) language models locally — no cloud, no API keys, no data leaving the device. Powered by the [Onde](https://ondeinference.com) Rust engine and [mistral.rs](https://github.com/EricLBuehler/mistral.rs), bridged to Flutter via [flutter_rust_bridge v2](https://pub.dev/packages/flutter_rust_bridge).

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

## Getting started

### 1. Add the dependency

```yaml
dependencies:
  onde_inference: ^0.1.0
```

### 2. Install Rust (required to build the native bridge)

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Add the targets for your platform(s):

```sh
# iOS
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# macOS
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Android (requires NDK r25+)
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Linux / Windows — already covered by the host toolchain
```

### 3. Run the code generator

From your **Flutter project root** (not the package root):

```sh
dart pub get
dart run flutter_rust_bridge_codegen generate
```

This reads `onde_inference`'s `rust/src/api.rs` and writes the FFI glue into
`lib/src/rust/frb_generated.dart` inside the package. You only need to re-run
this when the package is updated.

### 4. Build the native library

The native Rust library is compiled automatically as part of the normal Flutter
build. On iOS and macOS it is driven by the CocoaPods script phase in the
podspec; on Android by the CMake step in `android/build.gradle`; on Linux and
Windows by the `add_custom_command` in the platform `CMakeLists.txt`.

For the very first build, allow extra time for Cargo to compile the dependency
tree (~5–10 minutes cold, <1 minute incremental).

---

## Usage

### Initialize the library

Call `OndeInference.init()` **once** at application startup, before creating
any `OndeChatEngine`:

```dart
import 'package:onde_inference/onde_inference.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();
  runApp(const MyApp());
}
```

### Create an engine and load the default model

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

### Send a message (non-streaming)

```dart
final result = await engine.sendMessage('What is Rust's ownership model?');
print(result.text);
print('Generated in ${result.durationDisplay}');
```

### Stream a response

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

## Sampling configuration

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

## Running codegen

The Dart bindings are generated from the Rust source using
`flutter_rust_bridge_codegen`. Run this command from the package root
whenever `rust/src/api.rs` changes:

```sh
# From onde/sdk/dart/
dart pub get
dart run flutter_rust_bridge_codegen generate
```

The generated output is committed to `lib/src/frb_generated.dart` (and
platform-specific siblings). A hand-written stub at
`lib/src/frb_generated_stub.dart` stands in for the generated code before
the first codegen run, allowing the package to be compiled and the type
system to be checked without a built Rust binary.

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