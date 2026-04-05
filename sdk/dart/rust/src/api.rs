//! Flutter Rust Bridge v2 API for the Onde Inference Dart SDK.
//!
//! This module declares mirror types (re-declared plain Rust structs/enums
//! without UniFFI derives) and an `OndeChatEngine` opaque wrapper so that
//! `flutter_rust_bridge_codegen generate` can produce clean Dart bindings.
//!
//! All `onde::inference` types are imported with an `Onde*` alias to avoid
//! name collisions with the bridge mirrors defined below.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use onde::inference::{
    ChatEngine, ChatMessage as OndeChatMessage, ChatRole as OndeChatRole,
    EngineInfo as OndeEngineInfo, EngineStatus as OndeEngineStatus,
    GgufModelConfig as OndeGgufModelConfig, InferenceError, InferenceResult as OndeInferenceResult,
    SamplingConfig as OndeSamplingConfig, StreamChunk as OndeStreamChunk,
};

// ── Mirror: ChatRole ──────────────────────────────────────────────────────────

/// Role of a participant in a chat conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

impl From<OndeChatRole> for ChatRole {
    fn from(r: OndeChatRole) -> Self {
        match r {
            OndeChatRole::System => ChatRole::System,
            OndeChatRole::User => ChatRole::User,
            OndeChatRole::Assistant => ChatRole::Assistant,
        }
    }
}

impl From<ChatRole> for OndeChatRole {
    fn from(r: ChatRole) -> Self {
        match r {
            ChatRole::System => OndeChatRole::System,
            ChatRole::User => OndeChatRole::User,
            ChatRole::Assistant => OndeChatRole::Assistant,
        }
    }
}

// ── Mirror: ChatMessage ───────────────────────────────────────────────────────

/// A single message in a conversation.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl From<OndeChatMessage> for ChatMessage {
    fn from(m: OndeChatMessage) -> Self {
        Self {
            role: m.role.into(),
            content: m.content,
        }
    }
}

impl From<ChatMessage> for OndeChatMessage {
    fn from(m: ChatMessage) -> Self {
        Self {
            role: m.role.into(),
            content: m.content,
        }
    }
}

// ── Mirror: SamplingConfig ────────────────────────────────────────────────────

/// Sampling parameters for text generation.  All fields are optional — `None`
/// means "use the engine default".
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u64>,
    pub min_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

impl From<OndeSamplingConfig> for SamplingConfig {
    fn from(s: OndeSamplingConfig) -> Self {
        Self {
            temperature: s.temperature,
            top_p: s.top_p,
            top_k: s.top_k,
            min_p: s.min_p,
            max_tokens: s.max_tokens,
            frequency_penalty: s.frequency_penalty,
            presence_penalty: s.presence_penalty,
        }
    }
}

impl From<SamplingConfig> for OndeSamplingConfig {
    fn from(s: SamplingConfig) -> Self {
        Self {
            temperature: s.temperature,
            top_p: s.top_p,
            top_k: s.top_k,
            min_p: s.min_p,
            max_tokens: s.max_tokens,
            frequency_penalty: s.frequency_penalty,
            presence_penalty: s.presence_penalty,
        }
    }
}

// ── Mirror: GgufModelConfig ───────────────────────────────────────────────────

/// Configuration for loading a pre-quantised GGUF model.
#[derive(Debug, Clone)]
pub struct GgufModelConfig {
    /// HuggingFace repository ID, e.g. `"bartowski/Qwen2.5-1.5B-Instruct-GGUF"`.
    pub model_id: String,
    /// GGUF filename(s) within the repository.
    pub files: Vec<String>,
    /// Optional explicit tokenizer model ID (required on Android).
    pub tok_model_id: Option<String>,
    /// Human-friendly display name, e.g. `"Qwen 2.5 1.5B"`.
    pub display_name: String,
    /// Approximate memory footprint, e.g. `"~941 MB (GGUF Q4_K_M)"`.
    pub approx_memory: String,
}

impl From<OndeGgufModelConfig> for GgufModelConfig {
    fn from(c: OndeGgufModelConfig) -> Self {
        Self {
            model_id: c.model_id,
            files: c.files,
            tok_model_id: c.tok_model_id,
            display_name: c.display_name,
            approx_memory: c.approx_memory,
        }
    }
}

impl From<GgufModelConfig> for OndeGgufModelConfig {
    fn from(c: GgufModelConfig) -> Self {
        Self {
            model_id: c.model_id,
            files: c.files,
            tok_model_id: c.tok_model_id,
            display_name: c.display_name,
            approx_memory: c.approx_memory,
        }
    }
}

// ── Mirror: InferenceResult ───────────────────────────────────────────────────

/// The result of a completed inference request.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// The generated assistant reply text.
    pub text: String,
    /// Wall-clock inference duration in seconds.
    pub duration_secs: f64,
    /// Human-readable duration string, e.g. `"4.5s"` or `"2m 3.1s"`.
    pub duration_display: String,
    /// Finish reason reported by the model, e.g. `"stop"` or `"length"`.
    pub finish_reason: String,
}

impl From<OndeInferenceResult> for InferenceResult {
    fn from(r: OndeInferenceResult) -> Self {
        Self {
            text: r.text,
            duration_secs: r.duration_secs,
            duration_display: r.duration_display,
            finish_reason: r.finish_reason,
        }
    }
}

// ── Mirror: StreamChunk ───────────────────────────────────────────────────────

/// A single streaming token chunk emitted during inference.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// The token text delta (may be empty on the final chunk).
    pub delta: String,
    /// `true` when this is the last chunk in the stream.
    pub done: bool,
    /// Finish reason (set only on the final chunk).
    pub finish_reason: Option<String>,
}

impl From<OndeStreamChunk> for StreamChunk {
    fn from(c: OndeStreamChunk) -> Self {
        Self {
            delta: c.delta,
            done: c.done,
            finish_reason: c.finish_reason,
        }
    }
}

// ── Mirror: EngineStatus ──────────────────────────────────────────────────────

/// Lifecycle status of the inference engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    Unloaded,
    Loading,
    Ready,
    Generating,
    Error,
}

impl From<OndeEngineStatus> for EngineStatus {
    fn from(s: OndeEngineStatus) -> Self {
        match s {
            OndeEngineStatus::Unloaded => EngineStatus::Unloaded,
            OndeEngineStatus::Loading => EngineStatus::Loading,
            OndeEngineStatus::Ready => EngineStatus::Ready,
            OndeEngineStatus::Generating => EngineStatus::Generating,
            OndeEngineStatus::Error => EngineStatus::Error,
        }
    }
}

// ── Mirror: EngineInfo ────────────────────────────────────────────────────────

/// A point-in-time snapshot of the engine's state.
#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub status: EngineStatus,
    /// Display name of the currently loaded model, if any.
    pub model_name: Option<String>,
    /// Approximate memory footprint of the loaded model, if any.
    pub approx_memory: Option<String>,
    /// Number of conversation turns in the current history.
    pub history_length: u64,
}

impl From<OndeEngineInfo> for EngineInfo {
    fn from(i: OndeEngineInfo) -> Self {
        Self {
            status: i.status.into(),
            model_name: i.model_name,
            approx_memory: i.approx_memory,
            history_length: i.history_length,
        }
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Bridge error type mapping from [`onde::inference::InferenceError`].
#[derive(Debug, thiserror::Error)]
pub enum OndeError {
    #[error("No model loaded — call load_gguf_model first")]
    NoModelLoaded,

    #[error("Model already loaded: {model_name}")]
    AlreadyLoaded { model_name: String },

    #[error("Failed to build model: {reason}")]
    ModelBuild { reason: String },

    #[error("Inference failed: {reason}")]
    Inference { reason: String },

    #[error("Operation was cancelled")]
    Cancelled,

    #[error("{reason}")]
    Other { reason: String },
}

impl From<InferenceError> for OndeError {
    fn from(e: InferenceError) -> Self {
        match e {
            InferenceError::NoModelLoaded => OndeError::NoModelLoaded,
            InferenceError::AlreadyLoaded { model_name } => OndeError::AlreadyLoaded { model_name },
            InferenceError::ModelBuild { reason } => OndeError::ModelBuild { reason },
            InferenceError::Inference { reason } => OndeError::Inference { reason },
            InferenceError::Cancelled => OndeError::Cancelled,
            InferenceError::Other { reason } => OndeError::Other { reason },
        }
    }
}

// ── OndeChatEngine ────────────────────────────────────────────────────────────

/// Opaque wrapper around `onde::inference::ChatEngine`.
///
/// Dart/Flutter receives this as an opaque `RustOpaque<OndeChatEngine>` handle.
/// All methods delegate directly to the inner `ChatEngine`.
pub struct OndeChatEngine {
    inner: ChatEngine,
}

impl OndeChatEngine {
    // ── Construction ──────────────────────────────────────────────────────

    /// Create a new engine with no model loaded.
    #[frb(sync)]
    pub fn new() -> OndeChatEngine {
        OndeChatEngine {
            inner: ChatEngine::new(),
        }
    }

    // ── Model lifecycle ───────────────────────────────────────────────────

    /// Load a GGUF model into the engine.
    ///
    /// Returns the load duration in seconds on success.
    pub async fn load_gguf_model(
        &self,
        config: GgufModelConfig,
        system_prompt: Option<String>,
        sampling: Option<SamplingConfig>,
    ) -> Result<f64, OndeError> {
        let elapsed = self
            .inner
            .load_gguf_model(config.into(), system_prompt, sampling.map(Into::into))
            .await
            .map_err(OndeError::from)?;
        Ok(elapsed.as_secs_f64())
    }

    /// Unload the current model and release its memory.
    ///
    /// Returns the display name of the model that was unloaded, or `None`
    /// if no model was loaded.
    pub async fn unload_model(&self) -> Option<String> {
        self.inner.unload_model().await
    }

    /// Returns `true` if a model is currently loaded and ready.
    pub async fn is_loaded(&self) -> bool {
        self.inner.is_loaded().await
    }

    /// Returns a point-in-time snapshot of the engine's state.
    pub async fn info(&self) -> EngineInfo {
        self.inner.info().await.into()
    }

    // ── Configuration ─────────────────────────────────────────────────────

    /// Set or replace the system prompt.
    ///
    /// The prompt is prepended to every inference request and is **not**
    /// stored in the conversation history.
    pub async fn set_system_prompt(&self, prompt: String) {
        self.inner.set_system_prompt(prompt).await;
    }

    /// Clear the current system prompt.
    pub async fn clear_system_prompt(&self) {
        self.inner.clear_system_prompt().await;
    }

    /// Replace the sampling configuration used for all subsequent inference.
    pub async fn set_sampling(&self, sampling: SamplingConfig) {
        self.inner.set_sampling(sampling.into()).await;
    }

    // ── Conversation history ──────────────────────────────────────────────

    /// Returns a clone of the full conversation history.
    pub async fn history(&self) -> Vec<ChatMessage> {
        self.inner
            .history()
            .await
            .into_iter()
            .map(ChatMessage::from)
            .collect()
    }

    /// Clear the conversation history while keeping the model loaded.
    ///
    /// Returns the number of turns that were removed.
    pub async fn clear_history(&self) -> u64 {
        self.inner.clear_history().await as u64
    }

    /// Append a message to history without running inference.
    ///
    /// Useful for restoring a saved conversation or injecting context.
    pub async fn push_history(&self, message: ChatMessage) {
        self.inner.push_history(message.into()).await;
    }

    // ── Non-streaming inference ───────────────────────────────────────────

    /// Send a user message and receive a complete assistant reply.
    ///
    /// Both the user message and assistant reply are appended to the
    /// conversation history on success.
    pub async fn send_message(&self, message: String) -> Result<InferenceResult, OndeError> {
        self.inner
            .send_message(message)
            .await
            .map(InferenceResult::from)
            .map_err(OndeError::from)
    }

    /// One-shot generation from an explicit message list.
    ///
    /// Does **not** modify the conversation history.
    pub async fn generate(
        &self,
        messages: Vec<ChatMessage>,
        sampling: Option<SamplingConfig>,
    ) -> Result<InferenceResult, OndeError> {
        let onde_messages: Vec<OndeChatMessage> =
            messages.into_iter().map(OndeChatMessage::from).collect();
        self.inner
            .generate(onde_messages, sampling.map(Into::into))
            .await
            .map(InferenceResult::from)
            .map_err(OndeError::from)
    }

    // ── Streaming inference ───────────────────────────────────────────────

    /// Stream a reply token-by-token.
    ///
    /// Each [`StreamChunk`] carries a `delta` string.  The final chunk has
    /// `done == true`.  The `sink` is closed automatically when the stream
    /// ends or if the receiver is dropped.
    pub async fn stream_message(
        &self,
        message: String,
        sink: StreamSink<StreamChunk>,
    ) -> Result<(), OndeError> {
        let mut rx = self
            .inner
            .stream_message(message)
            .await
            .map_err(OndeError::from)?;

        while let Some(chunk) = rx.recv().await {
            let bridge_chunk = StreamChunk::from(chunk);
            let done = bridge_chunk.done;
            let _ = sink.add(bridge_chunk);
            if done {
                break;
            }
        }

        Ok(())
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Return the platform-appropriate default `GgufModelConfig`.
///
/// Selects the 1.5B model on iOS / tvOS / Android and the 3B model on
/// macOS / Windows / Linux.
#[frb(sync)]
pub fn default_model_config() -> GgufModelConfig {
    OndeGgufModelConfig::platform_default().into()
}

/// `GgufModelConfig` for Qwen 2.5 1.5B Instruct Q4_K_M (~941 MB).
#[frb(sync)]
pub fn qwen25_1_5b_config() -> GgufModelConfig {
    OndeGgufModelConfig::qwen25_1_5b().into()
}

/// `GgufModelConfig` for Qwen 2.5 3B Instruct Q4_K_M (~1.93 GB).
#[frb(sync)]
pub fn qwen25_3b_config() -> GgufModelConfig {
    OndeGgufModelConfig::qwen25_3b().into()
}

/// `GgufModelConfig` for Qwen 2.5 Coder 1.5B Instruct Q4_K_M (~941 MB).
#[frb(sync)]
pub fn qwen25_coder_1_5b_config() -> GgufModelConfig {
    OndeGgufModelConfig::qwen25_coder_1_5b().into()
}

/// `GgufModelConfig` for Qwen 2.5 Coder 3B Instruct Q4_K_M (~1.93 GB).
#[frb(sync)]
pub fn qwen25_coder_3b_config() -> GgufModelConfig {
    OndeGgufModelConfig::qwen25_coder_3b().into()
}

/// Default sampling config: `temperature=0.7`, `top_p=0.95`, `max_tokens=512`.
#[frb(sync)]
pub fn default_sampling_config() -> SamplingConfig {
    OndeSamplingConfig::default().into()
}

/// Deterministic sampling config: `temperature=0.0`, greedy, `max_tokens=512`.
#[frb(sync)]
pub fn deterministic_sampling_config() -> SamplingConfig {
    OndeSamplingConfig::deterministic().into()
}

/// Mobile sampling config: `temperature=0.7`, `top_p=0.95`, `max_tokens=128`.
#[frb(sync)]
pub fn mobile_sampling_config() -> SamplingConfig {
    OndeSamplingConfig::mobile().into()
}
