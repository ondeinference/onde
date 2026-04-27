// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.withContext
import java.io.File

actual class OndeInference internal constructor(
    private val dataDir: File,
    private val platform: PlatformSupport,
) : AutoCloseable {

    init {
        platform.ensureNativeLoaded()
    }

    private val engine = uniffi.onde.OndeChatEngine()

    @Volatile private var configured = false

    actual fun setup() {
        if (configured) return
        val hfHome     = File(dataDir, "models").also { it.mkdirs() }
        val hfHubCache = File(hfHome,  "hub").also   { it.mkdirs() }
        val tmpDir     = File(dataDir, "tmp").also   { it.mkdirs() }
        platform.setEnv("HF_HOME",               hfHome.absolutePath)
        platform.setEnv("HF_HUB_CACHE",          hfHubCache.absolutePath)
        platform.setEnv("HUGGINGFACE_HUB_CACHE", hfHubCache.absolutePath)
        platform.setEnv("TMPDIR",                tmpDir.absolutePath)
        configured = true
    }

    actual val modelCacheDir: String
        get() = File(File(dataDir, "models"), "hub").absolutePath

    // Also keep the File-based accessor for binary compatibility
    val modelCacheDirFile: File
        get() = File(File(dataDir, "models"), "hub")

    private fun ensureConfigured() { if (!configured) setup() }

    // ── Type conversion helpers ──────────────────────────────────────────

    private fun SamplingConfig.toUni(): uniffi.onde.SamplingConfig =
        uniffi.onde.SamplingConfig(
            temperature = temperature,
            topP = topP,
            topK = topK,
            minP = minP,
            maxTokens = maxTokens,
            frequencyPenalty = frequencyPenalty,
            presencePenalty = presencePenalty,
        )

    private fun GgufModelConfig.toUni(): uniffi.onde.GgufModelConfig =
        uniffi.onde.GgufModelConfig(
            modelId = modelId,
            files = files,
            tokModelId = tokModelId,
            displayName = displayName,
            approxMemory = approxMemory,
            chatTemplate = chatTemplate,
        )

    private fun ChatMessage.toUni(): uniffi.onde.ChatMessage =
        uniffi.onde.ChatMessage(
            role = when (role) {
                ChatRole.System -> uniffi.onde.ChatRole.SYSTEM
                ChatRole.User -> uniffi.onde.ChatRole.USER
                ChatRole.Assistant -> uniffi.onde.ChatRole.ASSISTANT
            },
            content = content,
        )

    private fun uniffi.onde.ChatMessage.toCommon(): ChatMessage =
        ChatMessage(
            role = when (role) {
                uniffi.onde.ChatRole.SYSTEM -> ChatRole.System
                uniffi.onde.ChatRole.USER -> ChatRole.User
                uniffi.onde.ChatRole.ASSISTANT -> ChatRole.Assistant
            },
            content = content,
        )

    private fun uniffi.onde.InferenceResult.toCommon(): InferenceResult =
        InferenceResult(
            text = text,
            durationSecs = durationSecs,
            durationDisplay = durationDisplay,
            finishReason = finishReason,
            toolCalls = toolCalls.map {
                ToolCallInfo(
                    id = it.id,
                    functionName = it.functionName,
                    arguments = it.arguments,
                )
            },
        )

    private fun uniffi.onde.StreamChunk.toCommon(): StreamChunk =
        StreamChunk(delta = delta, done = done, finishReason = finishReason)

    private fun uniffi.onde.EngineInfo.toCommon(): EngineInfo =
        EngineInfo(
            status = when (status) {
                uniffi.onde.EngineStatus.UNLOADED -> EngineStatus.Unloaded
                uniffi.onde.EngineStatus.LOADING -> EngineStatus.Loading
                uniffi.onde.EngineStatus.READY -> EngineStatus.Ready
                uniffi.onde.EngineStatus.GENERATING -> EngineStatus.Generating
                uniffi.onde.EngineStatus.ERROR -> EngineStatus.Error
            },
            modelName = modelName,
            approxMemory = approxMemory,
            historyLength = historyLength,
        )

    private inline fun <T> wrapError(block: () -> T): T {
        try {
            return block()
        } catch (e: uniffi.onde.InferenceException) {
            throw when (e) {
                is uniffi.onde.InferenceException.NoModelLoaded -> InferenceError.NoModelLoaded
                is uniffi.onde.InferenceException.AlreadyLoaded -> InferenceError.AlreadyLoaded(e.modelName)
                is uniffi.onde.InferenceException.ModelBuild -> InferenceError.ModelBuild(e.reason)
                is uniffi.onde.InferenceException.Inference -> InferenceError.Inference(e.reason)
                is uniffi.onde.InferenceException.Cancelled -> InferenceError.Cancelled
                is uniffi.onde.InferenceException.Other -> InferenceError.Other(e.reason)
            }
        }
    }

    // ── Model lifecycle ──────────────────────────────────────────────────

    actual suspend fun loadDefaultModel(
        systemPrompt: String?,
        sampling: SamplingConfig?,
    ): Double = withContext(Dispatchers.IO) {
        ensureConfigured()
        wrapError {
            engine.loadDefaultModel(
                systemPrompt = systemPrompt,
                sampling = sampling?.toUni(),
            )
        }
    }

    actual suspend fun loadModel(
        config: GgufModelConfig,
        systemPrompt: String?,
        sampling: SamplingConfig?,
    ): Double = withContext(Dispatchers.IO) {
        ensureConfigured()
        wrapError {
            engine.loadGgufModel(
                config = config.toUni(),
                systemPrompt = systemPrompt,
                sampling = sampling?.toUni(),
            )
        }
    }

    actual suspend fun unload(): String? = withContext(Dispatchers.IO) {
        engine.unloadModel()
    }

    actual suspend fun isLoaded(): Boolean = withContext(Dispatchers.IO) {
        engine.isLoaded()
    }

    // ── Inference ────────────────────────────────────────────────────────

    actual suspend fun chat(message: String): InferenceResult =
        withContext(Dispatchers.IO) {
            wrapError { engine.sendMessage(message).toCommon() }
        }

    actual fun stream(message: String): Flow<StreamChunk> = callbackFlow {
        val listener = object : uniffi.onde.StreamChunkListener {
            override fun onChunk(chunk: uniffi.onde.StreamChunk): Boolean {
                val result = trySend(chunk.toCommon())
                return result.isSuccess && !chunk.done
            }
        }
        withContext(Dispatchers.IO) {
            wrapError {
                uniffi.onde.streamChatMessage(
                    engine = engine,
                    message = message,
                    listener = listener,
                )
            }
        }
        awaitClose()
    }

    actual suspend fun generate(
        messages: List<ChatMessage>,
        sampling: SamplingConfig?,
    ): InferenceResult = withContext(Dispatchers.IO) {
        wrapError {
            engine.generate(
                messages = messages.map { it.toUni() },
                sampling = sampling?.toUni(),
            ).toCommon()
        }
    }

    // ── Engine state ─────────────────────────────────────────────────────

    actual suspend fun info(): EngineInfo = withContext(Dispatchers.IO) {
        engine.info().toCommon()
    }

    actual suspend fun setSystemPrompt(prompt: String) = withContext(Dispatchers.IO) {
        engine.setSystemPrompt(prompt)
    }

    actual suspend fun clearSystemPrompt() = withContext(Dispatchers.IO) {
        engine.clearSystemPrompt()
    }

    actual suspend fun setSampling(sampling: SamplingConfig) = withContext(Dispatchers.IO) {
        engine.setSampling(sampling.toUni())
    }

    // ── History ──────────────────────────────────────────────────────────

    actual suspend fun history(): List<ChatMessage> = withContext(Dispatchers.IO) {
        engine.history().map { it.toCommon() }
    }

    actual suspend fun clearHistory(): ULong = withContext(Dispatchers.IO) {
        engine.clearHistory().toULong()
    }

    actual suspend fun pushHistory(message: ChatMessage) = withContext(Dispatchers.IO) {
        engine.pushHistory(message.toUni())
    }

    // ── AutoCloseable ────────────────────────────────────────────────────

    actual override fun close() {
        engine.destroy()
    }
}
