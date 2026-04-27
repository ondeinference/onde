// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

import kotlinx.cinterop.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.withContext
import onde.native.*
import platform.Foundation.NSHomeDirectory
import kotlin.native.concurrent.freeze

actual class OndeInference internal constructor(
    private val dataDir: String,
) : AutoCloseable {

    private var handle: COpaquePointer? = null

    @Volatile private var configured = false

    init {
        handle = onde_engine_new()
            ?: throw IllegalStateException("Failed to create engine: ${lastError()}")
    }

    actual fun setup() {
        if (configured) return
        memScoped {
            onde_engine_setup(dataDir.cstr.ptr)
        }
        configured = true
    }

    actual val modelCacheDir: String
        get() = "$dataDir/models/hub"

    private fun ensureConfigured() { if (!configured) setup() }

    private fun lastError(): String {
        val errPtr = onde_last_error()
        return errPtr?.toKString() ?: "unknown error"
    }

    private fun readAndFreeString(ptr: CPointer<ByteVar>?): String? {
        if (ptr == null) return null
        val str = ptr.toKString()
        onde_string_free(ptr)
        return str
    }

    // ── Model lifecycle ──────────────────────────────────────────────────

    actual suspend fun loadDefaultModel(
        systemPrompt: String?,
        sampling: SamplingConfig?,
    ): Double = withContext(Dispatchers.IO) {
        ensureConfigured()
        val result = memScoped {
            onde_engine_load_default_model(
                handle,
                systemPrompt?.cstr?.ptr,
            )
        }
        if (result < 0.0) {
            throw InferenceError.ModelBuild(lastError())
        }
        result
    }

    actual suspend fun loadModel(
        config: GgufModelConfig,
        systemPrompt: String?,
        sampling: SamplingConfig?,
    ): Double = withContext(Dispatchers.IO) {
        // The C API currently only supports loading the default model.
        // For a specific model, we'd need to extend the C API.
        // For now, fall back to default model.
        ensureConfigured()
        val result = memScoped {
            onde_engine_load_default_model(
                handle,
                systemPrompt?.cstr?.ptr,
            )
        }
        if (result < 0.0) {
            throw InferenceError.ModelBuild(lastError())
        }
        result
    }

    actual suspend fun unload(): String? = withContext(Dispatchers.IO) {
        readAndFreeString(onde_engine_unload(handle))
    }

    actual suspend fun isLoaded(): Boolean = withContext(Dispatchers.IO) {
        onde_engine_is_loaded(handle)
    }

    // ── Inference ────────────────────────────────────────────────────────

    actual suspend fun chat(message: String): InferenceResult = withContext(Dispatchers.IO) {
        memScoped {
            val durationVar = alloc<DoubleVar>()
            val resultPtr = onde_engine_chat(
                handle,
                message.cstr.ptr,
                durationVar.ptr,
            )

            if (resultPtr == null) {
                throw InferenceError.Inference(lastError())
            }

            val text = resultPtr.toKString()
            val duration = durationVar.value
            onde_string_free(resultPtr)

            InferenceResult(
                text = text,
                durationSecs = duration,
                durationDisplay = formatDuration(duration),
                finishReason = null,
                toolCalls = emptyList(),
            )
        }
    }

    actual fun stream(message: String): Flow<StreamChunk> = callbackFlow {
        withContext(Dispatchers.IO) {
            memScoped {
                // We use a StableRef to pass the ProducerScope through the C callback
                val channelRef = StableRef.create(this@callbackFlow)

                val result = onde_engine_stream(
                    handle,
                    message.cstr.ptr,
                    staticCFunction { delta, done, userData ->
                        val ref = userData!!.asStableRef<kotlinx.coroutines.channels.ProducerScope<StreamChunk>>()
                        val scope = ref.get()
                        val deltaStr = delta?.toKString() ?: ""
                        val chunk = StreamChunk(delta = deltaStr, done = done, finishReason = null)
                        val sendResult = scope.trySend(chunk)
                        sendResult.isSuccess && !done
                    },
                    channelRef.asCPointer(),
                )

                channelRef.dispose()

                if (result != 0) {
                    throw InferenceError.Inference(lastError())
                }
            }
        }
        awaitClose()
    }

    actual suspend fun generate(
        messages: List<ChatMessage>,
        sampling: SamplingConfig?,
    ): InferenceResult = withContext(Dispatchers.IO) {
        // One-shot generation: for the minimum viable iOS implementation,
        // we concatenate messages and use chat(). A proper implementation
        // would extend the C API with a generate function.
        val combinedPrompt = messages.joinToString("\n") { "${it.role}: ${it.content}" }
        chat(combinedPrompt)
    }

    // ── Engine state ─────────────────────────────────────────────────────

    actual suspend fun info(): EngineInfo = withContext(Dispatchers.IO) {
        val statusStr = readAndFreeString(onde_engine_status(handle)) ?: "unloaded"
        val status = when (statusStr.lowercase()) {
            "unloaded" -> EngineStatus.Unloaded
            "loading" -> EngineStatus.Loading
            "ready" -> EngineStatus.Ready
            "generating" -> EngineStatus.Generating
            "error" -> EngineStatus.Error
            else -> EngineStatus.Unloaded
        }
        EngineInfo(
            status = status,
            modelName = readAndFreeString(onde_engine_model_name(handle)),
            approxMemory = readAndFreeString(onde_engine_approx_memory(handle)),
            historyLength = onde_engine_history_length(handle).toULong(),
        )
    }

    actual suspend fun setSystemPrompt(prompt: String) = withContext(Dispatchers.IO) {
        memScoped {
            onde_engine_set_system_prompt(handle, prompt.cstr.ptr)
        }
        Unit
    }

    actual suspend fun clearSystemPrompt() = withContext(Dispatchers.IO) {
        onde_engine_set_system_prompt(handle, null)
        Unit
    }

    actual suspend fun setSampling(sampling: SamplingConfig) {
        // TODO: Extend C API to support setSampling
    }

    // ── History ──────────────────────────────────────────────────────────

    actual suspend fun history(): List<ChatMessage> = withContext(Dispatchers.IO) {
        // TODO: Extend C API to return history
        emptyList()
    }

    actual suspend fun clearHistory(): ULong = withContext(Dispatchers.IO) {
        onde_engine_clear_history(handle).toULong()
    }

    actual suspend fun pushHistory(message: ChatMessage) {
        // TODO: Extend C API to support pushHistory
    }

    // ── AutoCloseable ────────────────────────────────────────────────────

    actual override fun close() {
        handle?.let {
            onde_engine_destroy(it)
            handle = null
        }
    }
}

private fun formatDuration(secs: Double): String {
    return if (secs < 60.0) {
        String.format("%.1fs", secs)
    } else {
        val minutes = (secs / 60.0).toInt()
        val remaining = secs - minutes * 60.0
        "${minutes}m ${String.format("%.1f", remaining)}s"
    }
}

/**
 * Create an [OndeInference] engine for iOS.
 *
 * The default data directory is the app's Documents directory.
 * Model files are cached in `<dataDir>/models/hub/`.
 */
fun OndeInference(
    dataDir: String = "${NSHomeDirectory()}/Documents/.onde",
): OndeInference = OndeInference(dataDir = dataDir)
