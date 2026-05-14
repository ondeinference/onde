// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
import 'package:flutter_test/flutter_test.dart';
import 'package:onde_inference/onde_inference.dart';
import 'package:onde_inference/src/frb_generated.dart/frb_generated.dart'
    as frb
    show RustLibApi;

class _FakeOndeChatEngine implements OndeChatEngine {
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnimplementedError('Unexpected OndeChatEngine member: $invocation');
}

class _MockRustLibApi implements frb.RustLibApi {
  static const _qwen2515b = GgufModelConfig(
    modelId: 'bartowski/Qwen2.5-1.5B-Instruct-GGUF',
    files: ['Qwen2.5-1.5B-Instruct-Q4_K_M.gguf'],
    displayName: 'Qwen 2.5 1.5B',
    approxMemory: '~941 MB',
  );

  static const _qwen253b = GgufModelConfig(
    modelId: 'bartowski/Qwen2.5-3B-Instruct-GGUF',
    files: ['Qwen2.5-3B-Instruct-Q4_K_M.gguf'],
    displayName: 'Qwen 2.5 3B',
    approxMemory: '~1.93 GB',
  );

  static const _qwen25Coder15b = GgufModelConfig(
    modelId: 'bartowski/Qwen2.5-Coder-1.5B-Instruct-GGUF',
    files: ['Qwen2.5-Coder-1.5B-Instruct-Q4_K_M.gguf'],
    displayName: 'Qwen 2.5 Coder 1.5B',
    approxMemory: '~941 MB',
  );

  static const _qwen25Coder3b = GgufModelConfig(
    modelId: 'bartowski/Qwen2.5-Coder-3B-Instruct-GGUF',
    files: ['Qwen2.5-Coder-3B-Instruct-Q4_K_M.gguf'],
    displayName: 'Qwen 2.5 Coder 3B',
    approxMemory: '~1.93 GB',
  );

  static final _defaultSampling = SamplingConfig(
    temperature: 0.7,
    topP: 0.95,
    maxTokens: BigInt.from(512),
  );

  static final _deterministicSampling = SamplingConfig(
    temperature: 0.0,
    maxTokens: BigInt.from(512),
  );

  static final _mobileSampling = SamplingConfig(
    temperature: 0.7,
    topP: 0.95,
    maxTokens: BigInt.from(128),
  );

  @override
  GgufModelConfig crateApiDefaultModelConfig() => _qwen25Coder3b;

  @override
  SamplingConfig crateApiDefaultSamplingConfig() => _defaultSampling;

  @override
  SamplingConfig crateApiDeterministicSamplingConfig() =>
      _deterministicSampling;

  @override
  SamplingConfig crateApiMobileSamplingConfig() => _mobileSampling;

  @override
  OndeChatEngine crateApiOndeChatEngineNew() => _FakeOndeChatEngine();

  @override
  GgufModelConfig crateApiQwen2515BConfig() => _qwen2515b;

  @override
  GgufModelConfig crateApiQwen253BConfig() => _qwen253b;

  @override
  GgufModelConfig crateApiQwen25Coder15BConfig() => _qwen25Coder15b;

  @override
  GgufModelConfig crateApiQwen25Coder3BConfig() => _qwen25Coder3b;

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnimplementedError('Unexpected RustLibApi member: $invocation');
}

void main() {
  setUpAll(() {
    RustLib.initMock(api: _MockRustLibApi());
  });

  tearDownAll(() {
    RustLib.dispose();
  });
  group('OndeException', () {
    test('toString includes error info', () {
      const e = OndeException(OndeError_NoModelLoaded());
      expect(e.toString(), contains('OndeException'));
    });

    test('equality', () {
      const a = OndeException(OndeError_NoModelLoaded());
      const b = OndeException(OndeError_NoModelLoaded());
      const c = OndeException(OndeError_Cancelled());
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('ChatRole', () {
    test('name values', () {
      expect(ChatRole.system.name, 'system');
      expect(ChatRole.user.name, 'user');
      expect(ChatRole.assistant.name, 'assistant');
    });
  });

  group('ChatMessage', () {
    test('factory constructors set correct roles', () {
      expect(ChatMessageX.system('Hi').role, ChatRole.system);
      expect(ChatMessageX.user('Hi').role, ChatRole.user);
      expect(ChatMessageX.assistant('Hi').role, ChatRole.assistant);
    });

    test('equality', () {
      final a = ChatMessageX.user('Hello');
      final b = ChatMessageX.user('Hello');
      final c = ChatMessageX.user('World');
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('SamplingConfig', () {
    test('defaultConfig preset', () {
      final s = SamplingConfigX.defaultConfig();
      expect(s.temperature, 0.7);
      expect(s.topP, 0.95);
      expect(s.maxTokens, BigInt.from(512));
    });

    test('deterministic preset', () {
      final s = SamplingConfigX.deterministic();
      expect(s.temperature, 0.0);
      expect(s.maxTokens, BigInt.from(512));
      expect(s.topP, isNull);
    });

    test('mobile preset', () {
      final s = SamplingConfigX.mobile();
      expect(s.temperature, 0.7);
      expect(s.topP, 0.95);
      expect(s.maxTokens, BigInt.from(128));
    });

    test('copyWith overrides single field', () {
      final base = SamplingConfigX.defaultConfig();
      final copy = base.copyWith(temperature: 0.3);
      expect(copy.temperature, 0.3);
      expect(copy.topP, base.topP);
      expect(copy.maxTokens, base.maxTokens);
    });

    test('equality', () {
      expect(
        SamplingConfigX.deterministic(),
        equals(SamplingConfigX.deterministic()),
      );
      expect(
        SamplingConfigX.deterministic(),
        isNot(equals(SamplingConfigX.mobile())),
      );
    });
  });

  group('GgufModelConfig', () {
    test('factory functions return non-empty configs', () {
      final config1_5b = OndeInference.qwen2515bConfig();
      expect(config1_5b.modelId, contains('1.5B'));
      expect(config1_5b.files, isNotEmpty);
      expect(config1_5b.displayName, isNotEmpty);
      expect(config1_5b.approxMemory, isNotEmpty);

      final config3b = OndeInference.qwen253bConfig();
      expect(config3b.modelId, contains('3B'));
      expect(config3b.files, isNotEmpty);

      final configCoder1_5b = OndeInference.qwen25Coder15bConfig();
      expect(configCoder1_5b.modelId, contains('Coder'));

      final configCoder3b = OndeInference.qwen25Coder3bConfig();
      expect(configCoder3b.modelId, contains('Coder'));

      final defaultConfig = OndeInference.defaultModelConfig();
      expect(defaultConfig.modelId, isNotEmpty);
    });

    test('equality', () {
      expect(
        OndeInference.qwen2515bConfig(),
        equals(OndeInference.qwen2515bConfig()),
      );
      expect(
        OndeInference.qwen2515bConfig(),
        isNot(equals(OndeInference.qwen253bConfig())),
      );
    });
  });

  group('EngineStatus', () {
    test('isReady only true for ready', () {
      expect(EngineStatus.ready.isReady, isTrue);
      expect(EngineStatus.unloaded.isReady, isFalse);
      expect(EngineStatus.loading.isReady, isFalse);
      expect(EngineStatus.generating.isReady, isFalse);
      expect(EngineStatus.error.isReady, isFalse);
    });

    test('name values', () {
      expect(EngineStatus.unloaded.name, 'unloaded');
      expect(EngineStatus.loading.name, 'loading');
      expect(EngineStatus.ready.name, 'ready');
      expect(EngineStatus.generating.name, 'generating');
      expect(EngineStatus.error.name, 'error');
    });
  });

  group('EngineInfo', () {
    test('equality', () {
      final a = EngineInfo(
        status: EngineStatus.unloaded,
        historyLength: BigInt.zero,
      );
      final b = EngineInfo(
        status: EngineStatus.unloaded,
        historyLength: BigInt.zero,
      );
      final c = EngineInfo(
        status: EngineStatus.ready,
        modelName: 'Qwen 2.5 3B',
        approxMemory: '~1.93 GB',
        historyLength: BigInt.from(3),
      );
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('StreamChunk', () {
    test('equality', () {
      const a = StreamChunk(delta: 'hello', done: false);
      const b = StreamChunk(delta: 'hello', done: false);
      const c = StreamChunk(delta: 'bye', done: true, finishReason: 'stop');
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('InferenceResult', () {
    test('equality', () {
      const a = InferenceResult(
        text: 'Hi there!',
        durationSecs: 1.5,
        durationDisplay: '1.5s',
        finishReason: 'stop',
        toolCalls: [],
      );
      const b = InferenceResult(
        text: 'Hi there!',
        durationSecs: 1.5,
        durationDisplay: '1.5s',
        finishReason: 'stop',
        toolCalls: [],
      );
      expect(a, equals(b));
    });
  });

  group('RustLib', () {
    test('instance is accessible before init', () {
      // instance is always non-null; initialization requires the native
      // library to be compiled first.
      expect(RustLib.instance, isNotNull);
    });
  });

  group('OndeInference helpers', () {
    test('config factories return valid configs', () {
      expect(OndeInference.defaultModelConfig().modelId, isNotEmpty);
      expect(OndeInference.qwen2515bConfig().modelId, contains('1.5B'));
      expect(OndeInference.qwen253bConfig().modelId, contains('3B'));
      expect(OndeInference.qwen25Coder15bConfig().modelId, contains('Coder'));
      expect(OndeInference.qwen25Coder3bConfig().modelId, contains('Coder'));
    });

    test('sampling factories return correct presets', () {
      expect(OndeInference.defaultSamplingConfig().temperature, 0.7);
      expect(OndeInference.deterministicSamplingConfig().temperature, 0.0);
      expect(OndeInference.mobileSamplingConfig().maxTokens, BigInt.from(128));
    });
  });

  group('OndeChatEngine', () {
    test('constructor succeeds when mock API is initialized', () {
      expect(OndeChatEngine(), isA<OndeChatEngine>());
    });
  });
}
