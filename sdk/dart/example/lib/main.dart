// Copyright 2024 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license found in LICENSE.
//
// example/lib/main.dart
//
// Complete Material 3 Flutter chat app demonstrating the onde_inference SDK:
//   * Synchronous OndeChatEngine() factory constructor (no await, no null)
//   * Platform-aware default model loading via loadDefaultModel() extension
//   * Multi-turn streaming chat via streamMessage(message:)
//   * EngineInfo display using EngineInfoX.historyLengthInt extension
//   * OndeError sealed-class error handling (not OndeException)
//   * Sampling preset selector (creative / precise / fast)
//   * Unload / reload model flow with live status bar

import 'dart:async';

import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:onde_inference/onde_inference.dart';
import 'package:path_provider/path_provider.dart' show getApplicationSupportDirectory;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await OndeInference.init();

  // Resolve the model cache directory for sandboxed platforms.
  //
  // On iOS/macOS this tries the App Group shared container first
  // (group.com.ondeinference.apps) so all Onde-powered apps share
  // downloaded models.  If the App Group is unavailable it falls back
  // to the app's private Application Support directory.
  //
  // On Android there is no App Group — the fallback is always used.
  //
  // On desktop Linux/Windows this is a no-op (default ~/.cache works).
  String? fallback;
  if (Platform.isIOS || Platform.isAndroid) {
    final dir = await getApplicationSupportDirectory();
    fallback = dir.path;
  }
  await OndeInference.setupCacheDir(fallbackDir: fallback);

  runApp(const OndeInferenceApp());
}

// ---------------------------------------------------------------------------
// Root app
// ---------------------------------------------------------------------------

class OndeInferenceApp extends StatelessWidget {
  const OndeInferenceApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Onde Inference',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF6750A4),
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF6750A4),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: const ChatScreen(),
    );
  }
}

// ---------------------------------------------------------------------------
// Chat message model
// ---------------------------------------------------------------------------

enum _Role { user, assistant }

class _Message {
  final _Role role;
  final String text;
  final bool isStreaming;

  const _Message({
    required this.role,
    required this.text,
    this.isStreaming = false,
  });

  _Message copyWith({String? text, bool? isStreaming}) => _Message(
        role: role,
        text: text ?? this.text,
        isStreaming: isStreaming ?? this.isStreaming,
      );
}

// ---------------------------------------------------------------------------
// Sampling preset enum
// ---------------------------------------------------------------------------

enum _SamplingPreset { creative, precise, fast }

extension _SamplingPresetExt on _SamplingPreset {
  String get label => switch (this) {
        _SamplingPreset.creative => 'Creative',
        _SamplingPreset.precise => 'Precise',
        _SamplingPreset.fast => 'Fast',
      };

  SamplingConfig get config => switch (this) {
        _SamplingPreset.creative => OndeInference.defaultSamplingConfig(),
        _SamplingPreset.precise => OndeInference.deterministicSamplingConfig(),
        _SamplingPreset.fast => OndeInference.mobileSamplingConfig(),
      };
}

// ---------------------------------------------------------------------------
// ChatScreen
// ---------------------------------------------------------------------------

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key});

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  // OndeChatEngine() is a synchronous factory constructor -- no Future, no null.
  final OndeChatEngine _engine = OndeChatEngine();

  EngineInfo _engineInfo = EngineInfo(
    status: EngineStatus.unloaded,
    historyLength: BigInt.zero,
  );

  final List<_Message> _messages = [];
  final TextEditingController _inputController = TextEditingController();
  final ScrollController _scrollController = ScrollController();

  bool _isModelLoading = false;
  String _loadingStatus = 'Tap "Load model" to begin.';
  String? _errorBanner;
  bool _isGenerating = false;
  _SamplingPreset _samplingPreset = _SamplingPreset.creative;

  @override
  void initState() {
    super.initState();
    _loadModel();
  }

  @override
  void dispose() {
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  // --------------------------------------------------------------------------
  // Scroll helper
  // --------------------------------------------------------------------------

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  // --------------------------------------------------------------------------
  // Engine management
  // --------------------------------------------------------------------------

  Future<void> _loadModel() async {
    setState(() {
      _isModelLoading = true;
      _loadingStatus = 'Downloading / loading model…';
      _errorBanner = null;
    });
    try {
      final elapsed = await _engine.loadDefaultModel(
        systemPrompt: 'You are a helpful, concise assistant.',
        sampling: _samplingPreset.config,
      );
      final info = await _engine.info();
      setState(() {
        _isModelLoading = false;
        _engineInfo = info;
        _loadingStatus =
            'Loaded ${info.modelName ?? "model"} in ${elapsed.toStringAsFixed(1)}s';
      });
    } on OndeError catch (e) {
      setState(() {
        _isModelLoading = false;
        _loadingStatus = 'Load failed.';
        _errorBanner = e.toString();
      });
    }
  }

  Future<void> _unloadModel() async {
    await _engine.unloadModel();
    final info = await _engine.info();
    setState(() {
      _engineInfo = info;
      _messages.clear();
      _loadingStatus = 'Model unloaded.';
    });
  }

  Future<void> _clearHistory() async {
    final removed = await _engine.clearHistoryCount();
    final info = await _engine.info();
    setState(() {
      _messages.clear();
      _engineInfo = info;
    });
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Cleared $removed messages.')),
      );
    }
  }

  Future<void> _sendMessage() async {
    final text = _inputController.text.trim();
    if (text.isEmpty ||
        _isGenerating ||
        _engineInfo.status != EngineStatus.ready) {
      return;
    }
    _inputController.clear();
    setState(() {
      _messages.add(_Message(role: _Role.user, text: text));
      _messages.add(const _Message(
        role: _Role.assistant,
        text: '',
        isStreaming: true,
      ));
      _isGenerating = true;
      _errorBanner = null;
    });
    _scrollToBottom();

    final buffer = StringBuffer();
    try {
      await for (final chunk in _engine.streamMessage(message: text)) {
        if (!mounted) break;
        buffer.write(chunk.delta);
        setState(() {
          _messages[_messages.length - 1] =
              _messages.last.copyWith(text: buffer.toString());
        });
        _scrollToBottom();
        if (chunk.done) break;
      }
      final info = await _engine.info();
      setState(() {
        _messages[_messages.length - 1] =
            _messages.last.copyWith(isStreaming: false);
        _isGenerating = false;
        _engineInfo = info;
      });
    } on OndeError catch (e) {
      setState(() {
        _messages[_messages.length - 1] = _messages.last.copyWith(
          text: '⚠ ${e.toString()}',
          isStreaming: false,
        );
        _isGenerating = false;
        _errorBanner = e.toString();
      });
    }
    _scrollToBottom();
  }

  // --------------------------------------------------------------------------
  // Build
  // --------------------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    final isReady = _engineInfo.status == EngineStatus.ready;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Onde Inference'),
        centerTitle: false,
        actions: [
          if (isReady)
            IconButton(
              icon: const Icon(Icons.delete_sweep_outlined),
              tooltip: 'Clear history',
              onPressed: _clearHistory,
            ),
          PopupMenuButton<_SamplingPreset>(
            tooltip: 'Sampling preset',
            icon: const Icon(Icons.tune),
            initialValue: _samplingPreset,
            onSelected: (preset) => setState(() => _samplingPreset = preset),
            itemBuilder: (context) => _SamplingPreset.values
                .map(
                  (p) => PopupMenuItem(
                    value: p,
                    child: Text(p.label),
                  ),
                )
                .toList(),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Column(
        children: [
          _EngineStatusBar(
            info: _engineInfo,
            isLoading: _isModelLoading,
            loadingStatus: _loadingStatus,
            onLoad: (!_isModelLoading &&
                    _engineInfo.status == EngineStatus.unloaded)
                ? _loadModel
                : null,
            onUnload: isReady ? _unloadModel : null,
          ),
          if (_errorBanner != null)
            _ErrorBanner(
              message: _errorBanner!,
              onDismiss: () => setState(() => _errorBanner = null),
            ),
          Expanded(
            child: _messages.isEmpty
                ? const _EmptyState()
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 12,
                      vertical: 8,
                    ),
                    itemCount: _messages.length,
                    itemBuilder: (context, index) =>
                        _MessageBubble(message: _messages[index]),
                  ),
          ),
          _InputBar(
            controller: _inputController,
            isEnabled: isReady && !_isGenerating,
            onSend: _sendMessage,
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _EngineStatusBar
// ---------------------------------------------------------------------------

class _EngineStatusBar extends StatelessWidget {
  final EngineInfo info;
  final bool isLoading;
  final String loadingStatus;
  final VoidCallback? onLoad;
  final VoidCallback? onUnload;

  const _EngineStatusBar({
    required this.info,
    required this.isLoading,
    required this.loadingStatus,
    this.onLoad,
    this.onUnload,
  });

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final tt = Theme.of(context).textTheme;
    return Material(
      color: cs.surfaceContainerHighest,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Row(
          children: [
            if (isLoading)
              Padding(
                padding: const EdgeInsets.only(right: 10),
                child: SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    color: cs.primary,
                  ),
                ),
              ),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    loadingStatus,
                    style: tt.bodySmall?.copyWith(color: cs.onSurfaceVariant),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  if (info.status == EngineStatus.ready)
                    Text(
                      '${info.status.label}'
                      ' · ${info.historyLengthInt} msgs'
                      '${info.approxMemory != null ? " · ${info.approxMemory}" : ""}',
                      style: tt.labelSmall?.copyWith(color: cs.outline),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                ],
              ),
            ),
            const SizedBox(width: 8),
            if (onLoad != null)
              FilledButton.tonal(
                onPressed: onLoad,
                child: const Text('Load model'),
              ),
            if (onUnload != null)
              TextButton(
                onPressed: onUnload,
                child: const Text('Unload'),
              ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _ErrorBanner
// ---------------------------------------------------------------------------

class _ErrorBanner extends StatelessWidget {
  final String message;
  final VoidCallback onDismiss;

  const _ErrorBanner({required this.message, required this.onDismiss});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return MaterialBanner(
      backgroundColor: cs.errorContainer,
      leading: Icon(Icons.error_outline, color: cs.onErrorContainer),
      content: Text(
        message,
        style: TextStyle(color: cs.onErrorContainer),
        maxLines: 3,
        overflow: TextOverflow.ellipsis,
      ),
      actions: [
        TextButton(
          onPressed: onDismiss,
          child: Text(
            'Dismiss',
            style: TextStyle(color: cs.onErrorContainer),
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _MessageBubble
// ---------------------------------------------------------------------------

class _MessageBubble extends StatelessWidget {
  final _Message message;

  const _MessageBubble({required this.message});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final tt = Theme.of(context).textTheme;
    final isUser = message.role == _Role.user;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment:
            isUser ? MainAxisAlignment.end : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          if (!isUser)
            CircleAvatar(
              radius: 14,
              backgroundColor: cs.primaryContainer,
              child: Icon(
                Icons.auto_awesome,
                size: 14,
                color: cs.onPrimaryContainer,
              ),
            ),
          if (!isUser) const SizedBox(width: 6),
          Flexible(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
              decoration: BoxDecoration(
                color: isUser ? cs.primary : cs.surfaceContainerHigh,
                borderRadius: BorderRadius.only(
                  topLeft: const Radius.circular(18),
                  topRight: const Radius.circular(18),
                  bottomLeft: Radius.circular(isUser ? 18 : 4),
                  bottomRight: Radius.circular(isUser ? 4 : 18),
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Flexible(
                    child: Text(
                      message.text.isEmpty && message.isStreaming
                          ? ' '
                          : message.text,
                      style: tt.bodyMedium?.copyWith(
                        color: isUser ? cs.onPrimary : cs.onSurface,
                      ),
                    ),
                  ),
                  if (message.isStreaming) ...[
                    const SizedBox(width: 4),
                    _BlinkingCursor(
                      color: isUser ? cs.onPrimary : cs.primary,
                    ),
                  ],
                ],
              ),
            ),
          ),
          if (isUser) const SizedBox(width: 6),
          if (isUser)
            CircleAvatar(
              radius: 14,
              backgroundColor: cs.secondaryContainer,
              child: Icon(
                Icons.person,
                size: 14,
                color: cs.onSecondaryContainer,
              ),
            ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _EmptyState
// ---------------------------------------------------------------------------

class _EmptyState extends StatelessWidget {
  const _EmptyState();

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final tt = Theme.of(context).textTheme;
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.chat_bubble_outline_rounded,
            size: 64,
            color: cs.outlineVariant,
          ),
          const SizedBox(height: 16),
          Text(
            'No messages yet',
            style: tt.titleMedium?.copyWith(color: cs.onSurfaceVariant),
          ),
          const SizedBox(height: 6),
          Text(
            'Load the model, then say hello.',
            style: tt.bodySmall?.copyWith(color: cs.outline),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _InputBar
// ---------------------------------------------------------------------------

class _InputBar extends StatelessWidget {
  final TextEditingController controller;
  final bool isEnabled;
  final VoidCallback onSend;

  const _InputBar({
    required this.controller,
    required this.isEnabled,
    required this.onSend,
  });

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 8, 12, 12),
        child: Row(
          children: [
            Expanded(
              child: TextField(
                controller: controller,
                enabled: isEnabled,
                minLines: 1,
                maxLines: 5,
                textInputAction: TextInputAction.send,
                onSubmitted: isEnabled ? (_) => onSend() : null,
                decoration: InputDecoration(
                  hintText: isEnabled ? 'Message…' : 'Load a model first…',
                  border: const OutlineInputBorder(
                    borderRadius: BorderRadius.all(Radius.circular(24)),
                  ),
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 10,
                  ),
                  filled: true,
                  fillColor: cs.surfaceContainerHigh,
                ),
              ),
            ),
            const SizedBox(width: 8),
            FilledButton(
              onPressed: isEnabled ? onSend : null,
              style: FilledButton.styleFrom(
                padding: const EdgeInsets.all(14),
                shape: const CircleBorder(),
              ),
              child: const Icon(Icons.send_rounded, size: 20),
            ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _BlinkingCursor
// ---------------------------------------------------------------------------

class _BlinkingCursor extends StatefulWidget {
  final Color color;

  const _BlinkingCursor({required this.color});

  @override
  State<_BlinkingCursor> createState() => _BlinkingCursorState();
}

class _BlinkingCursorState extends State<_BlinkingCursor>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 530),
  )..repeat(reverse: true);

  late final Animation<double> _opacity = CurvedAnimation(
    parent: _controller,
    curve: Curves.easeInOut,
  );

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FadeTransition(
      opacity: _opacity,
      child: Container(
        width: 2,
        height: 14,
        decoration: BoxDecoration(
          color: widget.color,
          borderRadius: BorderRadius.circular(1),
        ),
      ),
    );
  }
}
