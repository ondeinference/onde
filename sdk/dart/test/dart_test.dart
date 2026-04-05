import 'package:flutter_test/flutter_test.dart';
import 'package:onde_inference/onde_inference.dart';

void main() {
  group('OndeException', () {
    test('toString includes message', () {
      const e = OndeException('No model loaded');
      expect(e.toString(), contains('No model loaded'));
    });

    test('equality', () {
      const a = OndeException('foo');
      const b = OndeException('foo');
      const c = OndeException('bar');
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('ChatRole', () {
    test('toString values', () {
      expect(ChatRole.system.toString(), 'system');
      expect(ChatRole.user.toString(), 'user');
      expect(ChatRole.assistant.toString(), 'assistant');
    });
  });

  group('ChatMessage', () {
    test('factory constructors set correct roles', () {
      expect(ChatMessage.system('Hi').role, ChatRole.system);
      expect(ChatMessage.user('Hi').role, ChatRole.user);
      expect(ChatMessage.assistant('Hi').role, ChatRole.assistant);
    });

    test('equality', () {
      final a = ChatMessage.user('Hello');
      final b = ChatMessage.user('Hello');
      final c = ChatMessage.user('World');
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('SamplingConfig', () {
    test('defaultConfig preset', () {
      final s = SamplingConfig.defaultConfig();
      expect(s.temperature, 0.7);
      expect(s.topP, 0.95);
      expect(s.maxTokens, 512);
    });

    test('deterministic preset', () {
      final s = SamplingConfig.deterministic();
      expect(s.temperature, 0.0);
      expect(s.maxTokens, 512);
      expect(s.topP, isNull);
    });

    test('mobile preset', () {
      final s = SamplingConfig.mobile();
      expect(s.temperature, 0.7);
      expect(s.topP, 0.95);
      expect(s.maxTokens, 128);
    });

    test('copyWith overrides single field', () {
      final base = SamplingConfig.defaultConfig();
      final copy = base.copyWith(temperature: 0.3);
      expect(copy.temperature, 0.3);
      expect(copy.topP, base.topP);
      expect(copy.maxTokens, base.maxTokens);
    });

    test('equality', () {
      expect(
        SamplingConfig.deterministic(),
        equals(SamplingConfig.deterministic()),
      );
      expect(
        SamplingConfig.deterministic(),
        isNot(equals(SamplingConfig.mobile())),
      );
    });
  });

  group('GgufModelConfig', () {
    test('stub factory functions return non-empty configs', () {
      final config1_5b = OndeInference.qwen251_5bConfig();
      expect(config1_5b.modelId, contains('1.5B'));
      expect(config1_5b.files, isNotEmpty);
      expect(config1_5b.displayName, isNotEmpty);
      expect(config1_5b.approxMemory, isNotEmpty);

      final config3b = OndeInference.qwen253bConfig();
      expect(config3b.modelId, contains('3B'));
      expect(config3b.files, isNotEmpty);

      final configCoder1_5b = OndeInference.qwen25Coder1_5bConfig();
      expect(configCoder1_5b.modelId, contains('Coder'));

      final configCoder3b = OndeInference.qwen25Coder3bConfig();
      expect(configCoder3b.modelId, contains('Coder'));

      final defaultConfig = OndeInference.defaultModelConfig();
      expect(defaultConfig.modelId, isNotEmpty);
    });

    test('equality', () {
      expect(
        OndeInference.qwen251_5bConfig(),
        equals(OndeInference.qwen251_5bConfig()),
      );
      expect(
        OndeInference.qwen251_5bConfig(),
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

    test('toString values', () {
      expect(EngineStatus.unloaded.toString(), 'unloaded');
      expect(EngineStatus.loading.toString(), 'loading');
      expect(EngineStatus.ready.toString(), 'ready');
      expect(EngineStatus.generating.toString(), 'generating');
      expect(EngineStatus.error.toString(), 'error');
    });
  });

  group('EngineInfo', () {
    test('equality', () {
      const a = EngineInfo(status: EngineStatus.unloaded, historyLength: 0);
      const b = EngineInfo(status: EngineStatus.unloaded, historyLength: 0);
      const c = EngineInfo(
        status: EngineStatus.ready,
        modelName: 'Qwen 2.5 3B',
        approxMemory: '~1.93 GB',
        historyLength: 3,
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
      );
      const b = InferenceResult(
        text: 'Hi there!',
        durationSecs: 1.5,
        durationDisplay: '1.5s',
      );
      expect(a, equals(b));
    });
  });

  group('RustLib stub', () {
    test('init() is a safe no-op', () async {
      expect(RustLib.isInitialized, isFalse);
      await RustLib.init();
      expect(RustLib.isInitialized, isTrue);
      // Second call is also safe.
      await RustLib.init();
      expect(RustLib.isInitialized, isTrue);
    });
  });

  group('OndeInference helpers', () {
    test('init() delegates to RustLib.init()', () async {
      await expectLater(OndeInference.init(), completes);
    });

    test('config factories return valid configs', () {
      expect(OndeInference.defaultModelConfig().modelId, isNotEmpty);
      expect(OndeInference.qwen251_5bConfig().modelId, contains('1.5B'));
      expect(OndeInference.qwen253bConfig().modelId, contains('3B'));
      expect(OndeInference.qwen25Coder1_5bConfig().modelId, contains('Coder'));
      expect(OndeInference.qwen25Coder3bConfig().modelId, contains('Coder'));
    });

    test('sampling factories return correct presets', () {
      expect(OndeInference.defaultSamplingConfig().temperature, 0.7);
      expect(OndeInference.deterministicSamplingConfig().temperature, 0.0);
      expect(OndeInference.mobileSamplingConfig().maxTokens, 128);
    });
  });

  group('OndeChatEngine stub', () {
    test('create() throws UnimplementedError before codegen', () async {
      // RustLib.init() is already called above; init guard is met.
      await expectLater(
        OndeChatEngine.create(),
        throwsA(isA<UnimplementedError>()),
      );
    });
  });
}
