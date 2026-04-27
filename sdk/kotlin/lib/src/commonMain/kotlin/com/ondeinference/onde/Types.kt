// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

enum class ChatRole { System, User, Assistant }

data class ChatMessage(val role: ChatRole, val content: String)

data class SamplingConfig(
    val temperature: Double? = null,
    val topP: Double? = null,
    val topK: Long? = null,
    val minP: Double? = null,
    val maxTokens: Long? = null,
    val frequencyPenalty: Float? = null,
    val presencePenalty: Float? = null,
)

data class GgufModelConfig(
    val modelId: String,
    val files: List<String>,
    val tokModelId: String? = null,
    val displayName: String,
    val approxMemory: String,
    val chatTemplate: String? = null,
)

data class InferenceResult(
    val text: String,
    val durationSecs: Double,
    val durationDisplay: String,
    val finishReason: String?,
    val toolCalls: List<ToolCallInfo>,
)

data class ToolCallInfo(
    val id: String,
    val functionName: String,
    val arguments: String,
)

data class StreamChunk(
    val delta: String,
    val done: Boolean,
    val finishReason: String?,
)

enum class EngineStatus { Unloaded, Loading, Ready, Generating, Error }

data class EngineInfo(
    val status: EngineStatus,
    val modelName: String?,
    val approxMemory: String?,
    val historyLength: ULong,
)

sealed class InferenceError : Exception() {
    data object NoModelLoaded : InferenceError()
    data class AlreadyLoaded(val modelName: String) : InferenceError()
    data class ModelBuild(val reason: String) : InferenceError()
    data class Inference(val reason: String) : InferenceError()
    data object Cancelled : InferenceError()
    data class Other(val reason: String) : InferenceError()

    override val message: String
        get() = when (this) {
            is NoModelLoaded -> "No model loaded"
            is AlreadyLoaded -> "Model already loaded: $modelName"
            is ModelBuild -> "Model build error: $reason"
            is Inference -> "Inference error: $reason"
            is Cancelled -> "Cancelled"
            is Other -> reason
        }
}
