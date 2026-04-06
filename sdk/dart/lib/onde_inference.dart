// Copyright 2024 Onde Inference. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

/// On-device LLM inference SDK for Flutter & Dart.
///
/// Runs Qwen 2.5 models locally with Metal (Apple silicon) and CPU
/// acceleration — no cloud, no data leaving the device.
/// Powered by the Onde Rust engine and [mistral.rs](https://github.com/EricLBuehler/mistral.rs).
///
/// ## Quick start
///
/// ```dart
/// import 'package:onde_inference/onde_inference.dart';
///
/// Future<void> main() async {
///   // 1. Initialise the Rust library once at startup.
///   await OndeInference.init();
///
///   // 2. Create an engine and load the platform-appropriate default model.
///   final engine = await OndeChatEngine.create();
///   await engine.loadDefaultModel(
///     systemPrompt: 'You are a helpful assistant.',
///   );
///
///   // 3. Single-turn completion.
///   final result = await engine.sendMessage('Hello!');
///   print(result.text);
///
///   // 4. Streaming completion.
///   final buffer = StringBuffer();
///   await for (final chunk in engine.streamMessage('Tell me a short story.')) {
///     buffer.write(chunk.delta);
///     if (chunk.done) break;
///   }
///   print(buffer.toString());
///
///   // 5. Release device memory when done.
///   await engine.unloadModel();
/// }
/// ```
///
/// ## Selecting a model
///
/// ```dart
/// // Platform-aware default (1.5B on mobile, 3B on desktop).
/// final config = OndeInference.defaultModelConfig();
///
/// // Or pick a specific model.
/// final coderConfig = OndeInference.qwen25Coder3bConfig();
///
/// await engine.loadGgufModel(coderConfig);
/// ```
///
/// ## Customising sampling
///
/// ```dart
/// // Deterministic output for coding / fact-retrieval tasks.
/// await engine.setSampling(OndeInference.deterministicSamplingConfig());
///
/// // Or pass sampling directly when loading a model.
/// await engine.loadDefaultModel(
///   sampling: SamplingConfig(temperature: 0.5, maxTokens: 256),
/// );
/// ```
///
/// ## Error handling
///
/// All methods throw [OndeException] on failure:
///
/// ```dart
/// try {
///   await engine.sendMessage('...');
/// } on OndeException catch (e) {
///   debugPrint('Inference error: ${e.message}');
/// }
/// ```
///
/// ## Code generation (native bridge)
///
/// The package ships with a compilation stub so it can be imported before the
/// Rust binary is built. To generate the real FFI bridge, run from the package
/// root:
///
/// ```sh
/// dart run flutter_rust_bridge_codegen generate
/// ```
///
/// See the [README](https://github.com/ondeinference/onde) for full build
/// instructions.
library;

// All core data types (ChatMessage, SamplingConfig, GgufModelConfig, etc.)
// re-exported from the FRB-generated api.dart via types.dart.
export 'src/types.dart';

// Engine API — OndeChatEngineX extension, OndeInference static helpers,
// OndeChatEngine opaque type, OndeError sealed class, and RustLib.
export 'src/engine.dart';
