// Copyright 2026 Splitfire AB (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

/// On-device LLM inference SDK for Flutter & Dart.
///
/// Runs Qwen models locally with Metal on Apple platforms and CPU inference on
/// Android, Linux, and Windows. No cloud hop and no user data leaving the
/// device. Powered by the Onde Rust engine and
/// [mistral.rs](https://github.com/EricLBuehler/mistral.rs).
///
/// ## Quick start
///
/// ```dart
/// import 'package:flutter/widgets.dart';
/// import 'package:onde_inference/onde_inference.dart';
///
/// Future<void> main() async {
///   WidgetsFlutterBinding.ensureInitialized();
///   await OndeInference.init();
///
///   final engine = OndeChatEngine();
///   await engine.loadDefaultModel(
///     systemPrompt: 'You are a helpful assistant.',
///   );
///
///   final result = await engine.sendMessage(message: 'Hello!');
///   print(result.text);
///
///   final buffer = StringBuffer();
///   await for (final chunk in engine.streamMessage(message: 'Tell me a short story.')) {
///     buffer.write(chunk.delta);
///     if (chunk.done) break;
///   }
///   print(buffer.toString());
///
///   await engine.unloadModel();
/// }
/// ```
///
/// ## Selecting a model
///
/// ```dart
/// final config = OndeInference.defaultModelConfig();
/// final coderConfig = OndeInference.qwen25Coder3bConfig();
///
/// await engine.loadGgufModel(config: coderConfig);
/// ```
///
/// ## Customising sampling
///
/// ```dart
/// await engine.setSampling(
///   sampling: OndeInference.deterministicSamplingConfig(),
/// );
///
/// await engine.loadDefaultModel(
///   sampling: SamplingConfig(
///     temperature: 0.5,
///     maxTokens: BigInt.from(256),
///   ),
/// );
/// ```
///
/// ## Error handling
///
/// The generated bridge throws [OndeError] values directly:
///
/// ```dart
/// try {
///   await engine.sendMessage(message: '...');
/// } on OndeError catch (e) {
///   debugPrint('Inference error: $e');
/// }
/// ```
///
/// See the package README and the example app for platform-specific setup,
/// cache configuration, and end-to-end Flutter integration.
library;

// Re-export the core data types (ChatMessage, SamplingConfig,
// GgufModelConfig, and friends) from the FRB-generated api.dart via types.dart.
export 'src/types.dart';

// Engine API: OndeChatEngineX helpers, OndeInference static helpers,
// the OndeChatEngine opaque type, the OndeError sealed class, and RustLib.
export 'src/engine.dart';
