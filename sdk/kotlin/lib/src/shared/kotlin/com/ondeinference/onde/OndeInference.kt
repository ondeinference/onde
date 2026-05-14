// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// OndeInference.kt
//
// Multiplatform wrapper around the UniFFI-generated OndeChatEngine.
// This file is compiled for both Android and JVM targets via shared srcDir.

package com.ondeinference.onde

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.withContext
import java.io.File

// ── Public re-exports ─────────────────────────────────────────────────────────

typealias ChatMessage     = uniffi.onde.ChatMessage
typealias ChatRole        = uniffi.onde.ChatRole
typealias SamplingConfig  = uniffi.onde.SamplingConfig
typealias GgufModelConfig = uniffi.onde.GgufModelConfig
typealias InferenceResult = uniffi.onde.InferenceResult
typealias StreamChunk     = uniffi.onde.StreamChunk
typealias EngineInfo      = uniffi.onde.EngineInfo
typealias EngineStatus    = uniffi.onde.EngineStatus
typealias InferenceError  = uniffi.onde.InferenceException

// ── OndeInference ─────────────────────────────────────────────────────────────

/**
 * On-device LLM inference engine.
 *
 * Wraps the Rust/UniFFI [uniffi.onde.OndeChatEngine] with platform-specific
 * filesystem initialisation and idiomatic Kotlin coroutine APIs.
 *
 * The engine is thread-safe. All suspending functions dispatch to
 * [Dispatchers.IO] internally — call them from any coroutine scope.
 *
 * Create instances via the platform-specific factory functions:
 * - **Android:** `OndeInference(context)` or `OndeInference(context, dataDir)`
 * - **JVM:** `OndeInference()` or `OndeInference(dataDir)`
 *
 * @param dataDir Root directory for the HuggingFace model cache and temp files.
 * @param platform Platform-specific operations (env vars, native lib loading).
 */
class OndeInference internal constructor(
    private val dataDir: File,
    private val platform: PlatformSupport,
) : AutoCloseable {

    init {
        // Load the native library before any UniFFI type is accessed.
        // On Android this is a no-op (handled by System.loadLibrary in the APK).
        // On JVM this extracts libonde from JAR resources.
        platform.ensureNativeLoaded()
    }

    // The underlying UniFFI engine — thread-safe Arc<OndeChatEngine> under the hood.
    private val engine = uniffi.onde.OndeChatEngine()

    // Tracks whether the filesystem environment has been seeded.
    @Volatile private var configured = false

    // ── Filesystem setup ──────────────────────────────────────────────────────

    /**
     * Seed the HuggingFace cache and temp directory environment variables.
     *
     * Must be called before any model load. [loadDefaultModel] and [loadModel]
     * call this automatically, so explicit calls are only needed if you want
     * to control timing (e.g. at app startup).
     *
     * Sets:
     * - `HF_HOME`              → `<dataDir>/models`
     * - `HF_HUB_CACHE`         → `<dataDir>/models/hub`
     * - `HUGGINGFACE_HUB_CACHE`→ `<dataDir>/models/hub`
     * - `TMPDIR`               → `<dataDir>/tmp`
     *
     * This is idempotent — safe to call multiple times.
     */
    fun setup() {
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

    /**
     * Resolve the directory where HuggingFace model files are stored.
     */
    val modelCacheDir: File
        get() = File(File(dataDir, "models"), "hub")

    // ── Model lifecycle ───────────────────────────────────────────────────────

    /**
     * Load the platform-appropriate default model.
     *
     * On Android this is **Qwen 2.5 1.5B (GGUF Q4_K_M, ~941 MB)** on CPU.
     * On JVM/macOS this is **Qwen 2.5 3B (GGUF Q4_K_M, ~1.93 GB)** on Metal.
     *
     * If the model is not already cached locally it will be downloaded from
     * HuggingFace Hub on first call.
     *
     * @param systemPrompt Optional system prompt injected before the first turn.
     * @param sampling Optional sampling configuration.
     * @return Wall-clock model loading time in seconds.
     * @throws InferenceError if the model cannot be loaded.
     */
    suspend fun loadDefaultModel(
        systemPrompt: String?    = null,
        sampling: SamplingConfig? = null,
    ): Double = withContext(Dispatchers.IO) {
        ensureConfigured()
        engine.loadDefaultModel(
            systemPrompt = systemPrompt,
            sampling     = sampling,
        )
    }

    /**
     * Load a specific GGUF model.
     *
     * @param config Model repository and filename configuration.
     * @param systemPrompt Optional system prompt.
     * @param sampling Optional sampling configuration.
     * @return Wall-clock model loading time in seconds.
     */
    suspend fun loadModel(
        config: GgufModelConfig,
        systemPrompt: String?    = null,
        sampling: SamplingConfig? = null,
    ): Double = withContext(Dispatchers.IO) {
        ensureConfigured()
        engine.loadGgufModel(
            config       = config,
            systemPrompt = systemPrompt,
            sampling     = sampling,
        )
    }

    /** Unload the current model and free all associated memory. */
    suspend fun unload(): String? = withContext(Dispatchers.IO) {
        engine.unloadModel()
    }

    /** Check whether a model is currently loaded and ready. */
    suspend fun isLoaded(): Boolean = withContext(Dispatchers.IO) {
        engine.isLoaded()
    }

    // ── Inference ─────────────────────────────────────────────────────────────

    /**
     * Send a user message and receive a complete assistant reply.
     * The user turn and assistant reply are appended to conversation history.
     */
    suspend fun chat(message: String): InferenceResult =
        withContext(Dispatchers.IO) { engine.sendMessage(message) }

    /**
     * Stream a user message, emitting each token delta as it is generated.
     *
     * @param message The user's message text.
     * @return A cold [Flow] of [StreamChunk] values.
     */
    fun stream(message: String): Flow<StreamChunk> = callbackFlow {
        val listener = object : uniffi.onde.StreamChunkListener {
            override fun onChunk(chunk: StreamChunk): Boolean {
                val result = trySend(chunk)
                return result.isSuccess && !chunk.done
            }
        }
        withContext(Dispatchers.IO) {
            uniffi.onde.streamChatMessage(
                engine   = engine,
                message  = message,
                listener = listener,
            )
        }
        awaitClose()
    }

    /**
     * Run a one-shot inference WITHOUT modifying conversation history.
     */
    suspend fun generate(
        messages: List<ChatMessage>,
        sampling: SamplingConfig? = null,
    ): InferenceResult = withContext(Dispatchers.IO) {
        engine.generate(messages = messages, sampling = sampling)
    }

    // ── Engine state ──────────────────────────────────────────────────────────

    /** Get a snapshot of the engine's current state. */
    suspend fun info(): EngineInfo = withContext(Dispatchers.IO) { engine.info() }

    // ── System prompt ─────────────────────────────────────────────────────────

    /** Set or replace the system prompt without reloading the model. */
    suspend fun setSystemPrompt(prompt: String) = withContext(Dispatchers.IO) {
        engine.setSystemPrompt(prompt)
    }

    /** Clear the system prompt. */
    suspend fun clearSystemPrompt() = withContext(Dispatchers.IO) {
        engine.clearSystemPrompt()
    }

    // ── Sampling ──────────────────────────────────────────────────────────────

    /** Replace the sampling configuration without reloading the model. */
    suspend fun setSampling(sampling: SamplingConfig) = withContext(Dispatchers.IO) {
        engine.setSampling(sampling)
    }

    // ── Conversation history ──────────────────────────────────────────────────

    /** Get a copy of the full conversation history. */
    suspend fun history(): List<ChatMessage> = withContext(Dispatchers.IO) {
        engine.history()
    }

    /** Clear the conversation history. Returns the number of turns removed. */
    suspend fun clearHistory(): ULong = withContext(Dispatchers.IO) {
        engine.clearHistory()
    }

    /** Append a message to conversation history without running inference. */
    suspend fun pushHistory(message: ChatMessage) = withContext(Dispatchers.IO) {
        engine.pushHistory(message)
    }

    // ── AutoCloseable ─────────────────────────────────────────────────────────

    override fun close() {
        engine.destroy()
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    private fun ensureConfigured() {
        if (!configured) setup()
    }
}
