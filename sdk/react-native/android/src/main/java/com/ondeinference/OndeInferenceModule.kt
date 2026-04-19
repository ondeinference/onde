package com.ondeinference

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class OndeInferenceModule : Module() {
    companion object {
        init {
            System.loadLibrary("onde_react_native")
        }
    }

    private var enginePtr: Long = 0L

    // -- JNI native methods (implemented in Rust via #[cfg(target_os = "android")]) --

    private external fun nativeEngineCreate(): Long
    private external fun nativeEngineDestroy(engine: Long)
    private external fun nativeEngineLoadDefaultModel(
        engine: Long,
        systemPrompt: String?,
        samplingJson: String?
    ): String

    private external fun nativeEngineLoadModel(
        engine: Long,
        configJson: String,
        systemPrompt: String?,
        samplingJson: String?
    ): String

    private external fun nativeEngineUnloadModel(engine: Long): String
    private external fun nativeEngineIsLoaded(engine: Long): Boolean
    private external fun nativeEngineInfo(engine: Long): String
    private external fun nativeEngineSetSystemPrompt(engine: Long, prompt: String)
    private external fun nativeEngineClearSystemPrompt(engine: Long)
    private external fun nativeEngineSetSampling(engine: Long, samplingJson: String)
    private external fun nativeEngineHistory(engine: Long): String
    private external fun nativeEngineClearHistory(engine: Long): Long
    private external fun nativeEnginePushHistory(engine: Long, messageJson: String)
    private external fun nativeEngineSendMessage(engine: Long, message: String): String
    private external fun nativeEngineGenerate(
        engine: Long,
        messagesJson: String,
        samplingJson: String?
    ): String

    private external fun nativeDefaultModelConfig(): String
    private external fun nativeQwen251_5bConfig(): String
    private external fun nativeQwen253bConfig(): String
    private external fun nativeDefaultSamplingConfig(): String
    private external fun nativeDeterministicSamplingConfig(): String
    private external fun nativeMobileSamplingConfig(): String

    // -- Module definition --

    override fun definition() = ModuleDefinition {
        Name("OndeInference")

        OnCreate {
            enginePtr = nativeEngineCreate()
        }

        OnDestroy {
            if (enginePtr != 0L) {
                nativeEngineDestroy(enginePtr)
                enginePtr = 0L
            }
        }

        // -- Sync functions --

        Function("isLoaded") {
            if (enginePtr == 0L) return@Function false
            nativeEngineIsLoaded(enginePtr)
        }

        Function("setSystemPrompt") { prompt: String ->
            if (enginePtr != 0L) nativeEngineSetSystemPrompt(enginePtr, prompt)
        }

        Function("clearSystemPrompt") {
            if (enginePtr != 0L) nativeEngineClearSystemPrompt(enginePtr)
        }

        Function("setSampling") { samplingJson: String ->
            if (enginePtr != 0L) nativeEngineSetSampling(enginePtr, samplingJson)
        }

        Function("clearHistory") {
            if (enginePtr == 0L) return@Function 0L
            nativeEngineClearHistory(enginePtr)
        }

        Function("pushHistory") { messageJson: String ->
            if (enginePtr != 0L) nativeEnginePushHistory(enginePtr, messageJson)
        }

        // -- Config free functions (no engine needed) --

        Function("defaultModelConfig") { nativeDefaultModelConfig() }
        Function("qwen251_5bConfig") { nativeQwen251_5bConfig() }
        Function("qwen253bConfig") { nativeQwen253bConfig() }
        Function("defaultSamplingConfig") { nativeDefaultSamplingConfig() }
        Function("deterministicSamplingConfig") { nativeDeterministicSamplingConfig() }
        Function("mobileSamplingConfig") { nativeMobileSamplingConfig() }

        // -- Async functions --

        AsyncFunction("loadDefaultModel") { systemPrompt: String?, samplingJson: String? ->
            check(enginePtr != 0L) { "Engine not initialized" }
            nativeEngineLoadDefaultModel(enginePtr, systemPrompt, samplingJson)
        }

        AsyncFunction("loadModel") { configJson: String, systemPrompt: String?, samplingJson: String? ->
            check(enginePtr != 0L) { "Engine not initialized" }
            nativeEngineLoadModel(enginePtr, configJson, systemPrompt, samplingJson)
        }

        AsyncFunction("unloadModel") {
            check(enginePtr != 0L) { "Engine not initialized" }
            nativeEngineUnloadModel(enginePtr)
        }

        AsyncFunction("info") {
            check(enginePtr != 0L) { "Engine not initialized" }
            nativeEngineInfo(enginePtr)
        }

        AsyncFunction("history") {
            if (enginePtr == 0L) return@AsyncFunction "[]"
            nativeEngineHistory(enginePtr)
        }

        AsyncFunction("sendMessage") { message: String ->
            check(enginePtr != 0L) { "Engine not initialized" }
            nativeEngineSendMessage(enginePtr, message)
        }

        AsyncFunction("generate") { messagesJson: String, samplingJson: String? ->
            check(enginePtr != 0L) { "Engine not initialized" }
            nativeEngineGenerate(enginePtr, messagesJson, samplingJson)
        }
    }
}
