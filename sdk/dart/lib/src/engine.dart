// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
// ignore_for_file: prefer_const_constructors

import 'dart:async';
import 'dart:io' show Platform;

import 'package:flutter/services.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show ExternalLibrary;

import 'frb_generated.dart/frb_generated.dart' as frb;
import 'frb_generated.dart/api.dart' as api;
import 'types.dart';

export 'frb_generated.dart/frb_generated.dart' show RustLib;
export 'frb_generated.dart/api.dart'
    show
        OndeChatEngine,
        OndeError,
        OndeError_NoModelLoaded,
        OndeError_AlreadyLoaded,
        OndeError_ModelBuild,
        OndeError_Inference,
        OndeError_Cancelled,
        OndeError_Other,
        configureCacheDir;

// ---------------------------------------------------------------------------
// OndeChatEngine convenience wrapper
// ---------------------------------------------------------------------------

/// High-level wrapper around the FRB-generated [api.OndeChatEngine] opaque type.
///
/// Provides:
/// - [OndeChatEngineX.loadDefaultModel] — convenience method not in generated code
/// - [OndeChatEngineX.clearHistoryCount] — returns `int` instead of `BigInt`
/// - [OndeInference] — static initialisation + config/sampling factory helpers
///
/// ## Lifecycle
///
/// ```dart
/// // 1. Initialise once at app startup (idempotent).
/// await OndeInference.init();
///
/// // 2. Create a native engine instance (sync — no Future).
/// final engine = OndeChatEngine();
///
/// // 3a. Load your assigned model (recommended for production).
/// await engine.loadAssignedModel(
///   appId: 'your-app-id',       // from ondeinference.com
///   appSecret: 'your-app-secret',
///   systemPrompt: 'You are a helpful assistant.',
/// );
///
/// // 3b. Or load the platform default (for prototyping).
/// // await engine.loadDefaultModel(
/// //   systemPrompt: 'You are a helpful assistant.',
/// // );
///
/// // 4. Chat.
/// final result = await engine.sendMessage(message: 'Hello!');
/// print(result.text);
///
/// // 5. Streaming.
/// await for (final chunk in engine.streamMessage(message: 'Tell me a story.')) {
///   stdout.write(chunk.delta);
///   if (chunk.done) break;
/// }
///
/// // 6. Clean up.
/// await engine.unloadModel();
/// ```
///
/// ## Thread safety
///
/// [OndeChatEngine] is backed by an `Arc`-wrapped Rust object and is safe to
/// reference from multiple Dart isolates. Concurrent inference calls are
/// serialised internally.
extension OndeChatEngineX on api.OndeChatEngine {
  // -------------------------------------------------------------------------
  // loadDefaultModel — implemented in Dart (not in generated bridge)
  // -------------------------------------------------------------------------

  /// Loads the platform-appropriate default model.
  ///
  /// On iOS, tvOS, and Android the Qwen 2.5 1.5B (Q4_K_M, ~941 MB) model is
  /// used.  On macOS, Windows, and Linux the Qwen 2.5 3B (Q4_K_M, ~1.93 GB)
  /// model is used.
  ///
  /// Delegates to [loadGgufModel] with [OndeInference.defaultModelConfig].
  ///
  /// [systemPrompt] replaces the engine's system prompt before loading.
  /// [sampling] overrides the default sampling parameters.
  ///
  /// Returns the wall-clock seconds taken to load the model.
  ///
  /// Throws [OndeError] if the model cannot be downloaded or loaded.
  Future<double> loadDefaultModel({
    String? systemPrompt,
    SamplingConfig? sampling,
  }) => loadGgufModel(
    config: api.defaultModelConfig(),
    systemPrompt: systemPrompt,
    sampling: sampling,
  );

  // -------------------------------------------------------------------------
  // clearHistoryCount — int-typed convenience over clearHistory() → BigInt
  // -------------------------------------------------------------------------

  /// Clears the conversation history and returns the count as a plain [int].
  ///
  /// The generated [clearHistory] returns `Future<BigInt>`; this helper
  /// downcasts it so callers never need to handle [BigInt] directly.
  Future<int> clearHistoryCount() async => (await clearHistory()).toInt();
}

// ---------------------------------------------------------------------------
// OndeInference — static SDK helpers
// ---------------------------------------------------------------------------

/// Static helper namespace for Onde SDK initialisation and configuration.
///
/// ```dart
/// // Initialise before creating any OndeChatEngine.
/// await OndeInference.init();
///
/// // Obtain a model config.
/// final config = OndeInference.defaultModelConfig();
///
/// // Obtain sampling parameters.
/// final sampling = OndeInference.deterministicSamplingConfig();
/// ```
abstract final class OndeInference {
  // -------------------------------------------------------------------------
  // Initialisation
  // -------------------------------------------------------------------------

  /// Initialises the Rust shared library.
  ///
  /// Must be called before creating any [OndeChatEngine].  Subsequent calls
  /// are safe no-ops — the library is only initialised once per process.
  ///
  /// Call this in `main()` or in a Flutter `initState` override before any
  /// user interaction that could trigger model loading.
  ///
  /// On macOS the Rust static library is force-loaded into the
  /// `onde_inference.framework` CocoaPods dynamic framework — we must
  /// `dlopen` that framework by name.
  ///
  /// On iOS the pod is statically linked into the Runner executable
  /// (no separate `.framework` file exists at runtime), so we use
  /// `DynamicLibrary.process()` (`RTLD_DEFAULT`) which searches the
  /// main executable and all loaded images.
  ///
  /// On Android / Linux / Windows the default FRB stem-based loader
  /// finds the `.so` / `.dll` automatically — no override needed.
  static Future<void> init() async {
    ExternalLibrary? lib;
    if (Platform.isMacOS) {
      // CocoaPods + use_frameworks! → pod is a dynamic framework.
      // DynamicLibrary.process() can't resolve symbols inside it.
      lib = ExternalLibrary.open('onde_inference.framework/onde_inference');
    } else if (Platform.isIOS) {
      // CocoaPods on iOS statically links the pod into the Runner
      // binary via -force_load.  Symbols live in the main executable.
      lib = ExternalLibrary.process(iKnowHowToUseIt: true);
    }
    await frb.RustLib.init(externalLibrary: lib);
  }

  // -------------------------------------------------------------------------
  // Sandbox / cache setup
  // -------------------------------------------------------------------------

  /// Method channel shared with the native iOS/macOS plugin.
  static const _channel = MethodChannel('com.ondeinference.onde_inference');

  /// Configures the HuggingFace model cache for sandboxed platforms.
  ///
  /// On **iOS and macOS** this first tries to resolve the App Group shared
  /// container (`group.com.ondeinference.apps`) via a native method channel
  /// so all Onde-powered apps share downloaded models.  If the App Group is
  /// unavailable (entitlement missing, group not registered), it falls back
  /// to [fallbackDir] if provided.
  ///
  /// On **Android** there is no App Group equivalent — pass [fallbackDir]
  /// (e.g. from `getApplicationSupportDirectory().path`).
  ///
  /// On **desktop Linux / Windows** this is a no-op — the default
  /// `~/.cache/huggingface` works without intervention.
  ///
  /// Call **once** at startup, after [init] and before any model load:
  ///
  /// ```dart
  /// await OndeInference.init();
  /// await OndeInference.setupCacheDir();
  /// // or with explicit fallback:
  /// final dir = await getApplicationSupportDirectory();
  /// await OndeInference.setupCacheDir(fallbackDir: dir.path);
  /// ```
  static Future<void> setupCacheDir({String? fallbackDir}) async {
    // 1. Apple platforms — try the shared App Group container.
    if (Platform.isIOS || Platform.isMacOS) {
      try {
        final String? groupPath = await _channel.invokeMethod<String>(
          'getAppGroupContainerPath',
        );
        if (groupPath != null && groupPath.isNotEmpty) {
          api.configureCacheDir(appDataDir: groupPath);
          return;
        }
      } catch (_) {
        // Method channel not available (e.g. plugin not registered).
        // Fall through to fallbackDir.
      }
    }

    // 2. Fallback — use the caller-provided directory (app sandbox).
    if (fallbackDir != null && fallbackDir.isNotEmpty) {
      api.configureCacheDir(appDataDir: fallbackDir);
    }
  }

  // -------------------------------------------------------------------------
  // Model config factories
  // -------------------------------------------------------------------------

  /// Platform-appropriate default model config.
  ///
  /// Selects Qwen 2.5 1.5B on iOS / tvOS / Android and Qwen 2.5 3B on
  /// macOS / Windows / Linux.  Delegates to the Rust `default_model_config`
  /// free function so the platform check runs natively.
  static GgufModelConfig defaultModelConfig() => api.defaultModelConfig();

  /// Qwen 2.5 1.5B Instruct (GGUF Q4_K_M, ~941 MB).
  ///
  /// Suitable for iOS, tvOS, and Android where available memory is limited.
  static GgufModelConfig qwen2515bConfig() => api.qwen2515BConfig();

  /// Qwen 2.5 3B Instruct (GGUF Q4_K_M, ~1.93 GB).
  ///
  /// Suitable for macOS, Windows, and Linux desktop deployments.
  static GgufModelConfig qwen253bConfig() => api.qwen253BConfig();

  /// Qwen 2.5 Coder 1.5B Instruct (GGUF Q4_K_M, ~941 MB).
  ///
  /// Optimised for code-generation tasks on memory-constrained devices.
  static GgufModelConfig qwen25Coder15bConfig() => api.qwen25Coder15BConfig();

  /// Qwen 2.5 Coder 3B Instruct (GGUF Q4_K_M, ~1.93 GB).
  ///
  /// Optimised for code-generation tasks on macOS and Linux desktop.
  static GgufModelConfig qwen25Coder3bConfig() => api.qwen25Coder3BConfig();

  // -------------------------------------------------------------------------
  // Sampling config factories
  // -------------------------------------------------------------------------

  /// Balanced defaults: `temperature=0.7`, `topP=0.95`, `maxTokens=512`.
  ///
  /// Good for general-purpose conversational chat.
  static SamplingConfig defaultSamplingConfig() => api.defaultSamplingConfig();

  /// Greedy / fully deterministic decoding: `temperature=0.0`, `maxTokens=512`.
  ///
  /// Use for coding, fact-retrieval, or reproducibility-sensitive tasks.
  static SamplingConfig deterministicSamplingConfig() =>
      api.deterministicSamplingConfig();

  /// Low-latency preset for memory-constrained mobile:
  /// `temperature=0.7`, `topP=0.95`, `maxTokens=128`.
  static SamplingConfig mobileSamplingConfig() => api.mobileSamplingConfig();
}
