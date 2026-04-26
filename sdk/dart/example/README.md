<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">onde_inference — Example App</h1>

<p align="center">
  <strong>A Flutter chat app that runs LLM inference fully on-device.</strong><br>
  No server. No API key. No data leaving the device.
</p>

<p align="center">
  <a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://apps.apple.com/se/developer/splitfire-ab/id1831430993"><img src="https://img.shields.io/badge/App%20Store-live-235843?style=flat-square&labelColor=17211D" alt="App Store"></a>
</p>

---

## What this example shows

This example is a full Material 3 chat app built with the `onde_inference` Flutter SDK. It puts the main SDK features in one place, inside a single file, [`lib/main.dart`](lib/main.dart):

| Feature | Where |
|---|---|
| `OndeInference.init()` + sandbox cache setup | `main()` |
| Synchronous `OndeChatEngine()` factory (no `await`, no nulls) | `_ChatScreenState` |
| Platform-aware default model loading (`loadDefaultModel`) | `_loadModel()` |
| Multi-turn streaming chat via `streamMessage()` | `_sendMessage()` |
| Live `EngineInfo` status bar (model name, memory, history length) | `_EngineStatusBar` |
| `OndeError` sealed-class error handling | `_loadModel()`, `_sendMessage()` |
| Sampling preset switcher (creative / precise / fast) | `_SamplingPreset` |
| Unload and reload model at runtime | `_unloadModel()`, `_loadModel()` |
| Blinking cursor animation during streaming | `_BlinkingCursor` |

---

## Running the example

### Prerequisites

- [Flutter SDK](https://docs.flutter.dev/get-started/install) ≥ 3.10
- [Rust toolchain](https://rustup.rs) (stable), required to compile the native inference engine
- A physical device or simulator/emulator for your target platform

> **The first build usually takes 5 to 10 minutes** because it has to compile the full Rust dependency tree (`mistral.rs` + `candle`). After that, builds are much faster.

### Steps

```sh
# From the repo root
cd onde/sdk/dart/example

# Install Dart dependencies
flutter pub get

# Run on your connected device or simulator
flutter run
```

On **iOS / macOS**, the Rust engine is built through the CocoaPods podspec and linked automatically by Xcode. On **Android**, it is built through CMake and Gradle. On **Linux / Windows**, it is linked as a shared library.

### Platform notes

| Platform | GPU backend | Default model loaded |
|---|---|---|
| iOS (device) | Metal | Qwen 2.5 1.5B (~941 MB) |
| macOS | Metal | Qwen 2.5 3B (~1.93 GB) |
| Android (arm64) | CPU | Qwen 2.5 1.5B (~941 MB) |
| Linux / Windows | CPU | Qwen 2.5 3B (~1.93 GB) |

> The model is downloaded from Hugging Face Hub on first launch and cached locally, so you only need an internet connection once.

---

## SDK quick reference

```dart
import 'package:onde_inference/onde_inference.dart';

// 1. Initialise once at startup
await OndeInference.init();

// 2. Create the engine and load the platform-appropriate model
final engine = OndeChatEngine();
await engine.loadDefaultModel(systemPrompt: 'You are a helpful assistant.');

// 3. Stream a reply token-by-token
await for (final chunk in engine.streamMessage('Hello!')) {
  stdout.write(chunk.delta);
  if (chunk.done) break;
}

// 4. Release memory when done
await engine.unloadModel();
```

---

## Project structure

```
example/
├── lib/
│   └── main.dart        # Complete chat UI + SDK integration
├── android/             # Android Gradle + CMake build
├── ios/                 # iOS Xcode project + CocoaPods
├── macos/               # macOS Xcode project + CocoaPods
├── linux/               # Linux CMake build
└── windows/             # Windows CMake build
```

---

## Learn more

- **[onde_inference on pub.dev](https://pub.dev/packages/onde_inference)**, for the full API reference
- **[Onde Inference docs](https://ondeinference.com)**, for guides, the model catalog, and platform setup
- **[onde on GitHub](https://github.com/ondeinference/onde)**, for the Rust engine source, Swift SDK, and issue tracker

---

<p align="center">
  <sub>© 2026 <a href="https://ondeinference.com">Onde Inference</a>, MIT License</sub>
</p>