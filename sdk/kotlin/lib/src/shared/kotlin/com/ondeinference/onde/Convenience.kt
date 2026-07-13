// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//

package com.ondeinference.onde

/**
 * Construct preconfigured [SamplingConfig] instances.
 */
object OndeSampling {
    /** Creative chat defaults — temperature 0.7, top_p 0.95, max 512 tokens. */
    fun default(): SamplingConfig     = uniffi.onde.defaultSamplingConfig()
    /** Greedy/deterministic — temperature 0.0, max 512 tokens. */
    fun deterministic(): SamplingConfig = uniffi.onde.deterministicSamplingConfig()
    /** Conservative mobile — temperature 0.7, max 128 tokens. */
    fun mobile(): SamplingConfig      = uniffi.onde.mobileSamplingConfig()
}

/**
 * Construct [GgufModelConfig] instances for supported Onde models.
 */
object OndeModels {
    /** Platform-appropriate default — Qwen 2.5 1.5B on Android, 3B on macOS. */
    fun default(): GgufModelConfig  = uniffi.onde.defaultModelConfig()
    /** Qwen 2.5 1.5B Instruct GGUF Q4_K_M (~941 MB). */
    fun qwen25_1_5b(): GgufModelConfig = uniffi.onde.qwen2515bConfig()
    /** Qwen 2.5 3B Instruct GGUF Q4_K_M (~1.93 GB). */
    fun qwen25_3b(): GgufModelConfig   = uniffi.onde.qwen253bConfig()

    // ── Qwen 3 family ────────────────────────────────────────────────────────
    /** Qwen 3 0.6B GGUF Q4_K_M (~0.5 GB) — smallest Qwen 3 variant. */
    fun qwen3_0_6b(): GgufModelConfig = uniffi.onde.qwen306bConfig()
    /** Qwen 3 1.7B GGUF Q4_K_M (~1.3 GB). */
    fun qwen3_1_7b(): GgufModelConfig = uniffi.onde.qwen317bConfig()
    /** Qwen 3 4B GGUF Q4_K_M (~2.7 GB). */
    fun qwen3_4b(): GgufModelConfig   = uniffi.onde.qwen34bConfig()
    /** Qwen 3 8B GGUF Q4_K_M (~5 GB). */
    fun qwen3_8b(): GgufModelConfig   = uniffi.onde.qwen38bConfig()
    /** Qwen 3 14B GGUF Q4_K_M (~8.4 GB). */
    fun qwen3_14b(): GgufModelConfig  = uniffi.onde.qwen314bConfig()
    /** Qwen 3 32B GGUF Q4_K_M (~19.8 GB) — largest dense Qwen 3. */
    fun qwen3_32b(): GgufModelConfig  = uniffi.onde.qwen332bConfig()
    /** Qwen 3 4B Instruct 2507 GGUF Q4_K_M (~2.5 GB) — latest non-thinking 4B. */
    fun qwen3_4b_instruct_2507(): GgufModelConfig = uniffi.onde.qwen34bInstruct2507Config()
    /** Qwen 3 4B Thinking 2507 (~2.5 GB); nil sampling automatically uses 4096 tokens. */
    fun qwen3_4b_thinking_2507(): GgufModelConfig = uniffi.onde.qwen34bThinking2507Config()
    /** Qwen 3 30B-A3B Instruct 2507 GGUF Q4_K_M (~18.6 GB) — flagship MoE. */
    fun qwen3_30b_a3b_instruct_2507(): GgufModelConfig = uniffi.onde.qwen330bA3bInstruct2507Config()
}

/**
 * Construct [ChatMessage] values.
 */
object OndeMessage {
    fun system(content: String): ChatMessage    = uniffi.onde.systemMessage(content)
    fun user(content: String): ChatMessage      = uniffi.onde.userMessage(content)
    fun assistant(content: String): ChatMessage = uniffi.onde.assistantMessage(content)
}
