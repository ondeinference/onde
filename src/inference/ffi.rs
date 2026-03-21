//! UniFFI-exported wrapper around [`ChatEngine`] for Swift / Kotlin consumers.
//!
//! [`ChatEngine`] uses Rust idioms (generics, `impl Into<String>`,
//! `tokio::sync::mpsc::Receiver`) that cannot cross the FFI boundary.
//! This module provides [`OndeChatEngine`], a `#[derive(uniffi::Object)]`
//! wrapper with concrete, FFI-safe method signatures.
//!
//! ## Swift usage
//!
//! ```swift
//! let engine = OndeChatEngine()
//!
//! // Load the platform-appropriate default model.
//! let elapsed = try await engine.loadDefaultModel(
//!     systemPrompt: "You are a helpful assistant.",
//!     sampling: nil
//! )
//! print("Model loaded in \(elapsed)s")
//!
//! // Multi-turn chat (history managed automatically).
//! let result = try await engine.sendMessage(message: "Hello!")
//! print(result.text)
//!
//! // Streaming (callback-based for FFI compatibility).
//! try await streamChatMessage(engine: engine, message: "Tell me a story.", listener: myListener)
//!
//! // One-shot generation (does NOT modify conversation history).
//! let enhanced = try await engine.generate(
//!     messages: [ChatMessage(role: .user, content: "Expand: a cat in space")],
//!     sampling: SamplingConfig(temperature: 0.0, topP: nil, topK: nil, minP: nil, maxTokens: 512, frequencyPenalty: nil, presencePenalty: nil)
//! )
//! print(enhanced.text)
//!
//! // Status & history.
//! let info = await engine.info()
//! let history = await engine.history()
//!
//! // Cleanup.
//! await engine.unloadModel()
//! ```

use std::sync::Arc;

use super::engine::ChatEngine;
use super::types::*;

// ═══════════════════════════════════════════════════════════════════════════
// Callback interface for streaming (used by free function, not Object method)
// ═══════════════════════════════════════════════════════════════════════════

/// Callback interface for receiving streaming token chunks.
///
/// Implement this in Swift / Kotlin and pass it to the free function
/// [`stream_chat_message`].
///
/// ```swift
/// class MyStreamHandler: StreamChunkListener {
///     func onChunk(chunk: StreamChunk) -> Bool {
///         print(chunk.delta, terminator: "")
///         return !chunk.done  // return false to stop early
///     }
/// }
/// ```
#[uniffi::export(callback_interface)]
pub trait StreamChunkListener: Send + Sync {
    /// Called for each token chunk during streaming inference.
    ///
    /// Return `true` to continue receiving chunks, or `false` to cancel
    /// the stream early (the engine will still persist partial history).
    fn on_chunk(&self, chunk: StreamChunk) -> bool;
}

// ═══════════════════════════════════════════════════════════════════════════
// OndeChatEngine — UniFFI Object
// ═══════════════════════════════════════════════════════════════════════════

/// On-device LLM chat inference engine — UniFFI-exported wrapper.
///
/// This is the primary entry point for Swift / Kotlin consumers.  It wraps
/// [`ChatEngine`] with FFI-safe method signatures (no generics, no Rust
/// channels, concrete `String` parameters).
///
/// Construct with [`OndeChatEngine::new()`].  The engine starts with no
/// model loaded — call [`load_gguf_model`] or [`load_default_model`] first.
#[derive(uniffi::Object)]
pub struct OndeChatEngine {
    inner: ChatEngine,
}

#[uniffi::export]
impl OndeChatEngine {
    // ── Construction ─────────────────────────────────────────────────────

    /// Create a new engine with no model loaded.
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: ChatEngine::new(),
        })
    }

    // ── Model lifecycle ──────────────────────────────────────────────────

    /// Load a GGUF model into the engine.
    ///
    /// If a model is already loaded it will be unloaded first.
    ///
    /// Returns the wall-clock loading time in seconds.
    pub async fn load_gguf_model(
        &self,
        config: GgufModelConfig,
        system_prompt: Option<String>,
        sampling: Option<SamplingConfig>,
    ) -> Result<f64, InferenceError> {
        let elapsed = self
            .inner
            .load_gguf_model(config, system_prompt, sampling)
            .await?;
        Ok(elapsed.as_secs_f64())
    }

    /// Load the platform-appropriate default model.
    ///
    /// - tvOS / iOS / Android → Qwen 2.5 1.5B (~941 MB)
    /// - macOS / Windows / Linux → Qwen 2.5 3B (~1.93 GB)
    ///
    /// Returns the wall-clock loading time in seconds.
    pub async fn load_default_model(
        &self,
        system_prompt: Option<String>,
        sampling: Option<SamplingConfig>,
    ) -> Result<f64, InferenceError> {
        let config = GgufModelConfig::platform_default();
        let elapsed = self
            .inner
            .load_gguf_model(config, system_prompt, sampling)
            .await?;
        Ok(elapsed.as_secs_f64())
    }

    /// Unload the current model, freeing all memory.
    ///
    /// Returns the display name of the model that was unloaded, or `nil`
    /// if no model was loaded.
    pub async fn unload_model(&self) -> Option<String> {
        self.inner.unload_model().await
    }

    /// Check whether a model is currently loaded.
    pub async fn is_loaded(&self) -> bool {
        self.inner.is_loaded().await
    }

    /// Get a snapshot of the engine's current state.
    pub async fn info(&self) -> EngineInfo {
        self.inner.info().await
    }

    // ── System prompt ────────────────────────────────────────────────────

    /// Set or replace the system prompt.
    pub async fn set_system_prompt(&self, prompt: String) {
        self.inner.set_system_prompt(prompt).await;
    }

    /// Clear the system prompt.
    pub async fn clear_system_prompt(&self) {
        self.inner.clear_system_prompt().await;
    }

    // ── Sampling configuration ───────────────────────────────────────────

    /// Replace the sampling configuration.
    pub async fn set_sampling(&self, sampling: SamplingConfig) {
        self.inner.set_sampling(sampling).await;
    }

    // ── Conversation history ─────────────────────────────────────────────

    /// Get a copy of the full conversation history.
    pub async fn history(&self) -> Vec<ChatMessage> {
        self.inner.history().await
    }

    /// Clear the conversation history but keep the model loaded.
    ///
    /// Returns the number of turns that were removed.
    pub async fn clear_history(&self) -> u64 {
        self.inner.clear_history().await as u64
    }

    /// Append a message to history without running inference.
    pub async fn push_history(&self, message: ChatMessage) {
        self.inner.push_history(message).await;
    }

    // ── Non-streaming inference ──────────────────────────────────────────

    /// Send a user message and receive a complete assistant reply.
    ///
    /// The user message and assistant reply are automatically appended to
    /// the conversation history on success.
    pub async fn send_message(&self, message: String) -> Result<InferenceResult, InferenceError> {
        self.inner.send_message(message).await
    }

    /// Run inference on an explicit list of messages WITHOUT modifying the
    /// engine's internal history.
    ///
    /// Useful for one-shot prompts (e.g. prompt enhancement).
    pub async fn generate(
        &self,
        messages: Vec<ChatMessage>,
        sampling: Option<SamplingConfig>,
    ) -> Result<InferenceResult, InferenceError> {
        self.inner.generate(messages, sampling).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Free function for streaming — UniFFI callback_interface works in free fns
// ═══════════════════════════════════════════════════════════════════════════

/// Stream a chat message through the engine, delivering token chunks to
/// the `listener` callback.
///
/// This is a **free function** (not an Object method) because UniFFI 0.31
/// requires `callback_interface` parameters to be passed to free functions
/// rather than Object methods.
///
/// The `listener.on_chunk()` callback is called for each token delta.
/// Return `true` to continue, or `false` to cancel early.
///
/// The user message and assembled reply are automatically appended to the
/// engine's conversation history when streaming completes.
///
/// ```swift
/// class MyHandler: StreamChunkListener {
///     func onChunk(chunk: StreamChunk) -> Bool {
///         print(chunk.delta, terminator: "")
///         return !chunk.done
///     }
/// }
///
/// try await streamChatMessage(
///     engine: engine,
///     message: "Tell me a story.",
///     listener: MyHandler()
/// )
/// ```
#[uniffi::export]
pub async fn stream_chat_message(
    engine: Arc<OndeChatEngine>,
    message: String,
    listener: Box<dyn StreamChunkListener>,
) -> Result<(), InferenceError> {
    let mut rx = engine.inner.stream_message(message).await?;

    while let Some(chunk) = rx.recv().await {
        let done = chunk.done;
        let should_continue = listener.on_chunk(chunk);
        if done || !should_continue {
            break;
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Free functions — convenience exports for Swift / Kotlin
// ═══════════════════════════════════════════════════════════════════════════

/// Return the platform-appropriate default GGUF model configuration.
///
/// - tvOS / iOS / Android → Qwen 2.5 1.5B (~941 MB)
/// - macOS / Windows / Linux → Qwen 2.5 3B (~1.93 GB)
#[uniffi::export]
pub fn default_model_config() -> GgufModelConfig {
    GgufModelConfig::platform_default()
}

/// Return the Qwen 2.5 1.5B GGUF model configuration (~941 MB).
#[uniffi::export]
pub fn qwen25_1_5b_config() -> GgufModelConfig {
    GgufModelConfig::qwen25_1_5b()
}

/// Return the Qwen 2.5 3B GGUF model configuration (~1.93 GB).
#[uniffi::export]
pub fn qwen25_3b_config() -> GgufModelConfig {
    GgufModelConfig::qwen25_3b()
}

/// Return default sampling parameters for creative chat.
#[uniffi::export]
pub fn default_sampling_config() -> SamplingConfig {
    SamplingConfig::default()
}

/// Return deterministic (greedy) sampling parameters.
#[uniffi::export]
pub fn deterministic_sampling_config() -> SamplingConfig {
    SamplingConfig::deterministic()
}

/// Return conservative sampling parameters for mobile devices.
#[uniffi::export]
pub fn mobile_sampling_config() -> SamplingConfig {
    SamplingConfig::mobile()
}

/// Create a system message.
#[uniffi::export]
pub fn system_message(content: String) -> ChatMessage {
    ChatMessage::system(content)
}

/// Create a user message.
#[uniffi::export]
pub fn user_message(content: String) -> ChatMessage {
    ChatMessage::user(content)
}

/// Create an assistant message.
#[uniffi::export]
pub fn assistant_message(content: String) -> ChatMessage {
    ChatMessage::assistant(content)
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_functions_return_valid_configs() {
        let cfg = default_model_config();
        assert!(!cfg.model_id.is_empty());

        let cfg = qwen25_1_5b_config();
        assert!(cfg.model_id.contains("1.5B"));

        let cfg = qwen25_3b_config();
        assert!(cfg.model_id.contains("3B"));
    }

    #[test]
    fn free_functions_return_valid_sampling() {
        let s = default_sampling_config();
        assert_eq!(s.temperature, Some(0.7));

        let s = deterministic_sampling_config();
        assert_eq!(s.temperature, Some(0.0));

        let s = mobile_sampling_config();
        assert_eq!(s.max_tokens, Some(128));
    }

    #[test]
    fn message_helpers() {
        let m = system_message("You are helpful.".into());
        assert_eq!(m.role, ChatRole::System);
        assert_eq!(m.content, "You are helpful.");

        let m = user_message("Hello".into());
        assert_eq!(m.role, ChatRole::User);

        let m = assistant_message("Hi!".into());
        assert_eq!(m.role, ChatRole::Assistant);
    }

    #[tokio::test]
    async fn onde_chat_engine_new_is_unloaded() {
        let engine = OndeChatEngine::new();
        assert!(!engine.is_loaded().await);
        let info = engine.info().await;
        assert_eq!(info.status, EngineStatus::Unloaded);
        assert_eq!(info.history_length, 0);
    }

    #[tokio::test]
    async fn onde_chat_engine_send_without_model_errors() {
        let engine = OndeChatEngine::new();
        let result = engine.send_message("hello".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn onde_chat_engine_history_empty() {
        let engine = OndeChatEngine::new();
        assert!(engine.history().await.is_empty());
        assert_eq!(engine.clear_history().await, 0);
    }

    #[tokio::test]
    async fn onde_chat_engine_unload_when_none() {
        let engine = OndeChatEngine::new();
        assert!(engine.unload_model().await.is_none());
    }
}
