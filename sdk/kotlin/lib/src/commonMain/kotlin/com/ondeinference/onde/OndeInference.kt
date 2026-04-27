// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

import kotlinx.coroutines.flow.Flow

expect class OndeInference : AutoCloseable {
    fun setup()
    val modelCacheDir: String

    suspend fun loadDefaultModel(systemPrompt: String? = null, sampling: SamplingConfig? = null): Double
    suspend fun loadModel(config: GgufModelConfig, systemPrompt: String? = null, sampling: SamplingConfig? = null): Double
    suspend fun unload(): String?
    suspend fun isLoaded(): Boolean

    suspend fun chat(message: String): InferenceResult
    fun stream(message: String): Flow<StreamChunk>
    suspend fun generate(messages: List<ChatMessage>, sampling: SamplingConfig? = null): InferenceResult

    suspend fun info(): EngineInfo
    suspend fun setSystemPrompt(prompt: String)
    suspend fun clearSystemPrompt()
    suspend fun setSampling(sampling: SamplingConfig)

    suspend fun history(): List<ChatMessage>
    suspend fun clearHistory(): ULong
    suspend fun pushHistory(message: ChatMessage)

    override fun close()
}
