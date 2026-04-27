// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

/**
 * Construct preconfigured [SamplingConfig] instances.
 */
expect object OndeSampling {
    /** Creative chat defaults: temperature 0.7, top_p 0.95, max 512 tokens. */
    fun default(): SamplingConfig
    /** Greedy/deterministic: temperature 0.0, max 512 tokens. */
    fun deterministic(): SamplingConfig
    /** Conservative mobile: temperature 0.7, max 128 tokens. */
    fun mobile(): SamplingConfig
}

/**
 * Construct [GgufModelConfig] instances for supported Onde models.
 */
expect object OndeModels {
    /** Platform-appropriate default: Qwen 2.5 1.5B on Android, 3B on macOS. */
    fun default(): GgufModelConfig
    /** Qwen 2.5 1.5B Instruct GGUF Q4_K_M (~941 MB). */
    fun qwen25_1_5b(): GgufModelConfig
    /** Qwen 2.5 3B Instruct GGUF Q4_K_M (~1.93 GB). */
    fun qwen25_3b(): GgufModelConfig
}

/**
 * Construct [ChatMessage] values.
 */
expect object OndeMessage {
    fun system(content: String): ChatMessage
    fun user(content: String): ChatMessage
    fun assistant(content: String): ChatMessage
}
