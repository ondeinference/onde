import ExpoModulesCore
import Foundation

private let ondeAppGroupIdentifier = "group.com.ondeinference.apps"

// MARK: - Rust C FFI declarations

// Engine lifecycle
@_silgen_name("onde_engine_create")
func onde_engine_create() -> UnsafeMutableRawPointer?

@_silgen_name("onde_engine_destroy")
func onde_engine_destroy(_ engine: UnsafeMutableRawPointer)

// Model loading / unloading
@_silgen_name("onde_engine_load_default_model")
func onde_engine_load_default_model(
    _ engine: UnsafeMutableRawPointer,
    _ systemPrompt: UnsafePointer<CChar>?,
    _ samplingJson: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_engine_load_model")
func onde_engine_load_model(
    _ engine: UnsafeMutableRawPointer,
    _ configJson: UnsafePointer<CChar>,
    _ systemPrompt: UnsafePointer<CChar>?,
    _ samplingJson: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_engine_unload_model")
func onde_engine_unload_model(_ engine: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_engine_is_loaded")
func onde_engine_is_loaded(_ engine: UnsafeMutableRawPointer) -> Bool

// Engine info
@_silgen_name("onde_engine_info")
func onde_engine_info(_ engine: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>?

// System prompt
@_silgen_name("onde_engine_set_system_prompt")
func onde_engine_set_system_prompt(
    _ engine: UnsafeMutableRawPointer, _ prompt: UnsafePointer<CChar>)

@_silgen_name("onde_engine_clear_system_prompt")
func onde_engine_clear_system_prompt(_ engine: UnsafeMutableRawPointer)

// Sampling
@_silgen_name("onde_engine_set_sampling")
func onde_engine_set_sampling(
    _ engine: UnsafeMutableRawPointer, _ samplingJson: UnsafePointer<CChar>)

// History
@_silgen_name("onde_engine_history")
func onde_engine_history(_ engine: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_engine_clear_history")
func onde_engine_clear_history(_ engine: UnsafeMutableRawPointer) -> UInt64

@_silgen_name("onde_engine_push_history")
func onde_engine_push_history(
    _ engine: UnsafeMutableRawPointer, _ messageJson: UnsafePointer<CChar>)

// Inference
@_silgen_name("onde_engine_send_message")
func onde_engine_send_message(
    _ engine: UnsafeMutableRawPointer,
    _ message: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_engine_generate")
func onde_engine_generate(
    _ engine: UnsafeMutableRawPointer,
    _ messagesJson: UnsafePointer<CChar>,
    _ samplingJson: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>?

// Model config free functions
@_silgen_name("onde_default_model_config")
func onde_default_model_config() -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_qwen25_1_5b_config")
func onde_qwen25_1_5b_config() -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_qwen25_3b_config")
func onde_qwen25_3b_config() -> UnsafeMutablePointer<CChar>?

// Sampling config free functions
@_silgen_name("onde_default_sampling_config")
func onde_default_sampling_config() -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_deterministic_sampling_config")
func onde_deterministic_sampling_config() -> UnsafeMutablePointer<CChar>?

@_silgen_name("onde_mobile_sampling_config")
func onde_mobile_sampling_config() -> UnsafeMutablePointer<CChar>?

// Memory management
@_silgen_name("onde_free_string")
func onde_free_string(_ ptr: UnsafeMutablePointer<CChar>?)

// MARK: - Error types

enum OndeError: Error, CustomStringConvertible {
    case engineNotInitialized
    case invalidResponse
    case inferenceError(String)

    var description: String {
        switch self {
        case .engineNotInitialized:
            return "Onde engine is not initialized"
        case .invalidResponse:
            return "Invalid response from Onde engine"
        case .inferenceError(let message):
            return "Onde inference error: \(message)"
        }
    }
}

// MARK: - Expo Module

public class OndeInferenceModule: Module {
    private var enginePtr: UnsafeMutableRawPointer?

    // MARK: Helpers

    private func configureApplicationFilesystem() throws {
        let fileManager = FileManager.default

        let containerDirectory: URL
        if let appGroupDirectory = fileManager.containerURL(
            forSecurityApplicationGroupIdentifier: ondeAppGroupIdentifier
        ) {
            containerDirectory = appGroupDirectory
        } else {
            containerDirectory = try fileManager.url(
                for: .applicationSupportDirectory,
                in: .userDomainMask,
                appropriateFor: nil,
                create: true
            )
        }

        let hfHomeDirectory = containerDirectory.appendingPathComponent("models", isDirectory: true)
        let hfHubCacheDirectory = hfHomeDirectory.appendingPathComponent("hub", isDirectory: true)
        let tempDirectory = containerDirectory.appendingPathComponent("tmp", isDirectory: true)

        try fileManager.createDirectory(at: hfHomeDirectory, withIntermediateDirectories: true)
        try fileManager.createDirectory(at: hfHubCacheDirectory, withIntermediateDirectories: true)
        try fileManager.createDirectory(at: tempDirectory, withIntermediateDirectories: true)

        setenv("HF_HOME", hfHomeDirectory.path, 1)
        setenv("HF_HUB_CACHE", hfHubCacheDirectory.path, 1)
        setenv("HUGGINGFACE_HUB_CACHE", hfHubCacheDirectory.path, 1)
        setenv("TMPDIR", tempDirectory.path, 1)
    }

    /// Consume a Rust-allocated C string: copy it into a Swift `String`, then
    /// free the original allocation via `onde_free_string`.
    private func consumeRustString(_ ptr: UnsafeMutablePointer<CChar>?) -> String? {
        guard let ptr = ptr else { return nil }
        let str = String(cString: ptr)
        onde_free_string(ptr)
        return str
    }

    /// Validate a JSON string returned by the Rust FFI.
    /// If the JSON contains an `"error"` key, throw an `OndeError.inferenceError`.
    /// Otherwise return the original JSON string unchanged so the TypeScript
    /// wrapper can parse it consistently.
    private func validateJsonResult(_ json: String?) throws -> String {
        guard let json = json,
            let data = json.data(using: .utf8),
            let dict = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            throw OndeError.invalidResponse
        }
        if let error = dict["error"] as? String {
            throw OndeError.inferenceError(error)
        }
        return json
    }

    /// Validate that a JSON string is an array payload and return it unchanged.
    private func validateJsonArray(_ json: String?) throws -> String {
        guard let json = json,
            let data = json.data(using: .utf8),
            (try JSONSerialization.jsonObject(with: data) as? [[String: Any]]) != nil
        else {
            throw OndeError.invalidResponse
        }
        return json
    }

    // MARK: Module definition

    public func definition() -> ModuleDefinition {
        Name("OndeInference")

        OnCreate {
            do {
                try self.configureApplicationFilesystem()
                self.enginePtr = onde_engine_create()
            } catch {
                self.enginePtr = nil
            }
        }

        OnDestroy {
            if let ptr = self.enginePtr {
                onde_engine_destroy(ptr)
                self.enginePtr = nil
            }
        }

        // MARK: Sync functions

        Function("isLoaded") { () -> Bool in
            guard let ptr = self.enginePtr else { return false }
            return onde_engine_is_loaded(ptr)
        }

        Function("setSystemPrompt") { (prompt: String) in
            guard let ptr = self.enginePtr else { return }
            prompt.withCString { cPrompt in
                onde_engine_set_system_prompt(ptr, cPrompt)
            }
        }

        Function("clearSystemPrompt") { () in
            guard let ptr = self.enginePtr else { return }
            onde_engine_clear_system_prompt(ptr)
        }

        Function("setSampling") { (samplingJson: String) in
            guard let ptr = self.enginePtr else { return }
            samplingJson.withCString { cJson in
                onde_engine_set_sampling(ptr, cJson)
            }
        }

        Function("clearHistory") { () -> Int in
            guard let ptr = self.enginePtr else { return 0 }
            return Int(onde_engine_clear_history(ptr))
        }

        Function("pushHistory") { (messageJson: String) in
            guard let ptr = self.enginePtr else { return }
            messageJson.withCString { cJson in
                onde_engine_push_history(ptr, cJson)
            }
        }

        // MARK: Config free functions (no engine needed)

        Function("defaultModelConfig") { () -> String in
            let result = self.consumeRustString(onde_default_model_config())
            return try self.validateJsonResult(result)
        }

        Function("qwen251_5bConfig") { () -> String in
            let result = self.consumeRustString(onde_qwen25_1_5b_config())
            return try self.validateJsonResult(result)
        }

        Function("qwen253bConfig") { () -> String in
            let result = self.consumeRustString(onde_qwen25_3b_config())
            return try self.validateJsonResult(result)
        }

        Function("defaultSamplingConfig") { () -> String in
            let result = self.consumeRustString(onde_default_sampling_config())
            return try self.validateJsonResult(result)
        }

        Function("deterministicSamplingConfig") { () -> String in
            let result = self.consumeRustString(onde_deterministic_sampling_config())
            return try self.validateJsonResult(result)
        }

        Function("mobileSamplingConfig") { () -> String in
            let result = self.consumeRustString(onde_mobile_sampling_config())
            return try self.validateJsonResult(result)
        }

        // MARK: Async functions

        AsyncFunction("loadDefaultModel") {
            (systemPrompt: String?, samplingJson: String?) -> String in
            try self.configureApplicationFilesystem()
            guard let ptr = self.enginePtr else {
                throw OndeError.engineNotInitialized
            }
            let result: String?
            if let sp = systemPrompt, let sj = samplingJson {
                result = sp.withCString { cSp in
                    sj.withCString { cSj in
                        self.consumeRustString(onde_engine_load_default_model(ptr, cSp, cSj))
                    }
                }
            } else if let sp = systemPrompt {
                result = sp.withCString { cSp in
                    self.consumeRustString(onde_engine_load_default_model(ptr, cSp, nil))
                }
            } else if let sj = samplingJson {
                result = sj.withCString { cSj in
                    self.consumeRustString(onde_engine_load_default_model(ptr, nil, cSj))
                }
            } else {
                result = self.consumeRustString(onde_engine_load_default_model(ptr, nil, nil))
            }
            return try self.validateJsonResult(result)
        }

        AsyncFunction("loadModel") {
            (configJson: String, systemPrompt: String?, samplingJson: String?) -> String in
            try self.configureApplicationFilesystem()
            guard let ptr = self.enginePtr else {
                throw OndeError.engineNotInitialized
            }
            let result: String? = configJson.withCString { cConfig in
                if let sp = systemPrompt, let sj = samplingJson {
                    return sp.withCString { cSp in
                        sj.withCString { cSj in
                            self.consumeRustString(onde_engine_load_model(ptr, cConfig, cSp, cSj))
                        }
                    }
                } else if let sp = systemPrompt {
                    return sp.withCString { cSp in
                        self.consumeRustString(onde_engine_load_model(ptr, cConfig, cSp, nil))
                    }
                } else if let sj = samplingJson {
                    return sj.withCString { cSj in
                        self.consumeRustString(onde_engine_load_model(ptr, cConfig, nil, cSj))
                    }
                } else {
                    return self.consumeRustString(onde_engine_load_model(ptr, cConfig, nil, nil))
                }
            }
            return try self.validateJsonResult(result)
        }

        AsyncFunction("unloadModel") { () -> String in
            guard let ptr = self.enginePtr else {
                throw OndeError.engineNotInitialized
            }
            let result = self.consumeRustString(onde_engine_unload_model(ptr))
            return try self.validateJsonResult(result)
        }

        AsyncFunction("info") { () -> String in
            guard let ptr = self.enginePtr else {
                throw OndeError.engineNotInitialized
            }
            let result = self.consumeRustString(onde_engine_info(ptr))
            return try self.validateJsonResult(result)
        }

        AsyncFunction("history") { () -> String in
            guard let ptr = self.enginePtr else { return "[]" }
            let result = self.consumeRustString(onde_engine_history(ptr))
            return try self.validateJsonArray(result)
        }

        AsyncFunction("sendMessage") { (message: String) -> String in
            guard let ptr = self.enginePtr else {
                throw OndeError.engineNotInitialized
            }
            let result = message.withCString { cMessage in
                self.consumeRustString(onde_engine_send_message(ptr, cMessage))
            }
            return try self.validateJsonResult(result)
        }

        AsyncFunction("generate") {
            (messagesJson: String, samplingJson: String?) -> String in
            guard let ptr = self.enginePtr else {
                throw OndeError.engineNotInitialized
            }
            let result: String? = messagesJson.withCString { cMessages in
                if let sj = samplingJson {
                    return sj.withCString { cSj in
                        self.consumeRustString(onde_engine_generate(ptr, cMessages, cSj))
                    }
                } else {
                    return self.consumeRustString(onde_engine_generate(ptr, cMessages, nil))
                }
            }
            return try self.validateJsonResult(result)
        }
    }
}
