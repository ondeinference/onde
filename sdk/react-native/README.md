<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference</h1>

<p align="center">
  <strong>On-device LLM inference for React Native — Metal on iOS, CPU on Android.</strong>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@ondeinference/react-native"><img src="https://img.shields.io/npm/v/@ondeinference/react-native?style=flat-square&color=235843&labelColor=17211D&label=npm" alt="npm"></a>
  <a href="https://crates.io/crates/onde"><img src="https://img.shields.io/crates/v/onde?style=flat-square&color=235843&labelColor=17211D&label=crates.io" alt="crates.io"></a>
  <a href="https://swiftpackageindex.com/ondeinference/onde-swift"><img src="https://img.shields.io/badge/Swift%20Package%20Index-onde--swift-235843?style=flat-square&labelColor=17211D" alt="Swift Package Index"></a>
  <a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
</p>

<p align="center">
  <a href="https://github.com/ondeinference/onde">Rust SDK</a> · <a href="https://swiftpackageindex.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://pub.dev/packages/onde_inference">Flutter SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

Run Qwen 2.5 models on the phone. No server, no API key, nothing leaves the device.

The model downloads from HuggingFace on first load, then runs locally. ~941 MB for the 1.5B variant. Metal gives you ~15 tok/s on an iPhone 15; Android runs on CPU, slower but works.

## Installation

```bash
npx expo install @ondeinference/react-native
```

## Quick start

```typescript
import { OndeChatEngine, userMessage } from "@ondeinference/react-native";

// Picks the right model for the device:
//   iOS     → Qwen 2.5 1.5B (~941 MB, Metal)
//   Android → Qwen 2.5 1.5B (~941 MB, CPU)
const seconds = await OndeChatEngine.loadDefaultModel(
  "You are a helpful assistant."
);

const reply = await OndeChatEngine.sendMessage("Hello!");
console.log(reply.text);

// One-shot — doesn't touch conversation history
const expanded = await OndeChatEngine.generate(
  [userMessage("Expand: a cat in space")],
  { temperature: 0.0 }
);

await OndeChatEngine.unloadModel();
```

## Platforms

| Platform | Backend | Default model |
|----------|---------|---------------|
| iOS      | Metal   | Qwen 2.5 1.5B (~941 MB) |
| Android  | CPU     | Qwen 2.5 1.5B (~941 MB) |

## API

### OndeChatEngine

| Method | Returns | What it does |
|--------|---------|--------------|
| `loadDefaultModel(systemPrompt?, sampling?)` | `Promise<number>` | Load the platform default. Returns load time in seconds. |
| `loadModel(config, systemPrompt?, sampling?)` | `Promise<number>` | Load a specific GGUF model. |
| `unloadModel()` | `Promise<string \| null>` | Drop the model, free memory. Returns the model name. |
| `isLoaded()` | `boolean` | Is anything loaded right now? |
| `info()` | `Promise<EngineInfo>` | Status, model name, memory, history length. |
| `sendMessage(message)` | `Promise<InferenceResult>` | Chat turn. Appends to history automatically. |
| `generate(messages, sampling?)` | `Promise<InferenceResult>` | One-shot. History stays untouched. |
| `setSystemPrompt(prompt)` | `void` | Replace the system prompt. |
| `clearSystemPrompt()` | `void` | Remove it. |
| `setSampling(config)` | `void` | Swap sampling params. |
| `history()` | `Promise<ChatMessage[]>` | Full conversation so far. |
| `clearHistory()` | `number` | Wipe it. Returns how many messages were removed. |
| `pushHistory(message)` | `void` | Inject a message without running inference. |

### Helpers

```typescript
import {
  defaultModelConfig,     // platform-aware (1.5B on mobile, 3B on desktop)
  qwen251_5bConfig,       // force 1.5B (~941 MB)
  qwen253bConfig,         // force 3B (~1.93 GB)
  defaultSamplingConfig,  // temp=0.7, top_p=0.95, max_tokens=512
  deterministicSamplingConfig,  // temp=0.0
  mobileSamplingConfig,   // temp=0.7, max_tokens=128
  systemMessage,
  userMessage,
  assistantMessage,
} from "@ondeinference/react-native";
```

## Example app

There's a working chat app in [`example/`](./example):

```bash
cd example
npm install
npx expo run:ios
```

Single file, ~290 lines. Shows loading, chat, status, history management, and error handling.

## Building from source

You need Rust and the right cross-compilation targets.

```bash
# iOS
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
./scripts/build-rust.sh ios

# Android (set ANDROID_NDK_HOME first)
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
./scripts/build-rust.sh android
```

The script compiles the Rust FFI bridge in `rust/`, then copies the static lib (iOS) or shared libs (Android) into the right places under `ios/` and `android/`.

## How it works

```
TypeScript  →  Expo Module (Swift / Kotlin)  →  Rust C FFI  →  onde crate  →  mistral.rs
                @_silgen_name (iOS)                               ↓
                JNI external (Android)                     Metal / CPU
```

The native module talks to Rust through `extern "C"` functions. Complex types cross the boundary as JSON strings — the TypeScript layer handles camelCase ↔ snake_case conversion. A global `tokio::Runtime` (created once) runs the async inference.

## License

Dual-licensed under **MIT** and **Apache 2.0**. Pick whichever works for you.

- [MIT License](https://github.com/ondeinference/onde/blob/main/LICENSE-MIT)
- [Apache License 2.0](https://github.com/ondeinference/onde/blob/main/LICENSE-APACHE)

© 2026 [Splitfire AB](https://splitfire.se)

---

## Copyright

© 2026 [Onde Inference](https://ondeinference.com) (Splitfire AB).
