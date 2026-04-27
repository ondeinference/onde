// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

// iOS actual implementations. Same preset values as the JVM side.
// TODO: call C scaffolding functions once cinterop is fully wired.

actual object OndeSampling {
    actual fun default(): SamplingConfig = SamplingConfig(
        temperature = 0.7,
        topP = 0.95,
        maxTokens = 512,
    )

    actual fun deterministic(): SamplingConfig = SamplingConfig(
        temperature = 0.0,
        maxTokens = 512,
    )

    actual fun mobile(): SamplingConfig = SamplingConfig(
        temperature = 0.7,
        topP = 0.95,
        maxTokens = 128,
    )
}

actual object OndeModels {
    actual fun default(): GgufModelConfig = qwen25_1_5b() // iOS defaults to 1.5B

    actual fun qwen25_1_5b(): GgufModelConfig = GgufModelConfig(
        modelId = "bartowski/Qwen2.5-1.5B-Instruct-GGUF",
        files = listOf("Qwen2.5-1.5B-Instruct-Q4_K_M.gguf"),
        tokModelId = null, // GGUF embeds tokenizer on iOS
        displayName = "Qwen 2.5 1.5B",
        approxMemory = "~941 MB",
    )

    actual fun qwen25_3b(): GgufModelConfig = GgufModelConfig(
        modelId = "bartowski/Qwen2.5-3B-Instruct-GGUF",
        files = listOf("Qwen2.5-3B-Instruct-Q4_K_M.gguf"),
        tokModelId = null,
        displayName = "Qwen 2.5 3B",
        approxMemory = "~1.93 GB",
    )
}

actual object OndeMessage {
    actual fun system(content: String): ChatMessage = ChatMessage(ChatRole.System, content)
    actual fun user(content: String): ChatMessage = ChatMessage(ChatRole.User, content)
    actual fun assistant(content: String): ChatMessage = ChatMessage(ChatRole.Assistant, content)
}
