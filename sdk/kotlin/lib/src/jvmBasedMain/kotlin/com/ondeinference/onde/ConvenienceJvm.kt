// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

actual object OndeSampling {
    actual fun default(): SamplingConfig {
        val u = uniffi.onde.defaultSamplingConfig()
        return SamplingConfig(u.temperature, u.topP, u.topK, u.minP, u.maxTokens, u.frequencyPenalty, u.presencePenalty)
    }
    actual fun deterministic(): SamplingConfig {
        val u = uniffi.onde.deterministicSamplingConfig()
        return SamplingConfig(u.temperature, u.topP, u.topK, u.minP, u.maxTokens, u.frequencyPenalty, u.presencePenalty)
    }
    actual fun mobile(): SamplingConfig {
        val u = uniffi.onde.mobileSamplingConfig()
        return SamplingConfig(u.temperature, u.topP, u.topK, u.minP, u.maxTokens, u.frequencyPenalty, u.presencePenalty)
    }
}

actual object OndeModels {
    private fun uniffi.onde.GgufModelConfig.toCommon() = GgufModelConfig(
        modelId = modelId, files = files, tokModelId = tokModelId,
        displayName = displayName, approxMemory = approxMemory, chatTemplate = chatTemplate,
    )
    actual fun default(): GgufModelConfig = uniffi.onde.defaultModelConfig().toCommon()
    actual fun qwen25_1_5b(): GgufModelConfig = uniffi.onde.qwen2515bConfig().toCommon()
    actual fun qwen25_3b(): GgufModelConfig = uniffi.onde.qwen253bConfig().toCommon()
}

actual object OndeMessage {
    actual fun system(content: String): ChatMessage = ChatMessage(ChatRole.System, content)
    actual fun user(content: String): ChatMessage = ChatMessage(ChatRole.User, content)
    actual fun assistant(content: String): ChatMessage = ChatMessage(ChatRole.Assistant, content)
}
