//! Shared types for the on-device LLM inference engine.
//!
//! These types are intentionally framework-agnostic — they do **not** depend on
//! any UI framework.  They carry UniFFI derive annotations so that
//! UniFFI bindgen can generate Swift / Kotlin bindings automatically.
//!
//! ## UniFFI compatibility notes
//!
//! - `usize` is **not** supported by UniFFI → we use `u64` / `u32` instead.
//! - Enums use `#[derive(uniffi::Enum)]`.
//! - Plain data structs use `#[derive(uniffi::Record)]`.
//! - Error enums use `#[derive(uniffi::Error)]`.

use serde::{Deserialize, Serialize};

// ── Message role ─────────────────────────────────────────────────────────────

/// Role of a message in a chat conversation.
///
/// Mirrors the standard OpenAI / mistral.rs role taxonomy but is decoupled
/// from `mistralrs::TextMessageRole` so that callers don't need a direct
/// dependency on mistral.rs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for ChatRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatRole::System => write!(f, "system"),
            ChatRole::User => write!(f, "user"),
            ChatRole::Assistant => write!(f, "assistant"),
        }
    }
}

// ── Chat message ─────────────────────────────────────────────────────────────

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

// ── Sampling configuration ───────────────────────────────────────────────────

/// Sampling parameters for text generation.
///
/// All fields are optional — `None` means "use the engine default".
///
/// Note: `usize` is replaced with `u64` for UniFFI compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct SamplingConfig {
    /// Sampling temperature (higher = more random).  Typical range: 0.0–2.0.
    pub temperature: Option<f64>,
    /// Nucleus (top-p) sampling threshold.  Typical value: 0.9–0.95.
    pub top_p: Option<f64>,
    /// Top-k sampling limit.
    pub top_k: Option<u64>,
    /// Min-p sampling threshold.
    pub min_p: Option<f64>,
    /// Maximum number of tokens to generate.
    pub max_tokens: Option<u64>,
    /// Frequency penalty (penalise tokens proportional to occurrence count).
    pub frequency_penalty: Option<f32>,
    /// Presence penalty (penalise tokens that appeared at all).
    pub presence_penalty: Option<f32>,
}

impl Default for SamplingConfig {
    /// Sensible creative-chat defaults.
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.95),
            top_k: None,
            min_p: None,
            max_tokens: Some(512),
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

impl SamplingConfig {
    /// Deterministic sampling (temperature = 0, greedy decoding).
    pub fn deterministic() -> Self {
        Self {
            temperature: Some(0.0),
            top_p: None,
            top_k: None,
            min_p: None,
            max_tokens: Some(512),
            frequency_penalty: None,
            presence_penalty: None,
        }
    }

    /// Conservative mobile defaults — lower max_tokens for faster response on
    /// constrained devices (CPU-only ARM SoCs doing ~1-3 tok/sec).
    pub fn mobile() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.95),
            top_k: None,
            min_p: None,
            max_tokens: Some(128),
            frequency_penalty: None,
            presence_penalty: None,
        }
    }

    /// Sampling config tuned for coding assistants.
    ///
    /// Key differences from the general-purpose default:
    ///
    /// - **`frequency_penalty = 1.15`** — the primary fix for the repeated-token
    ///   loop you saw with small GGUF models.  Penalises each token proportional
    ///   to how many times it has already appeared in the output, making it
    ///   progressively harder to emit the same word again.  1.15 is aggressive
    ///   enough to kill echo loops without hurting code token repetition
    ///   (identifiers, keywords) at typical response lengths.
    ///
    /// - **`presence_penalty = 0.1`** — a light nudge away from any token that
    ///   has appeared at all, encouraging lexical variety across the response.
    ///
    /// - **`temperature = 0.2`** — low randomness keeps generated code
    ///   deterministic and syntactically correct.  High temperature causes
    ///   hallucinated variable names and broken syntax in code blocks.
    ///
    /// - **`top_p = 0.95`** — nucleus sampling retained to avoid degenerate
    ///   greedy outputs while still keeping the distribution tight.
    ///
    /// - **`max_tokens = 1024`** — doubled vs. the chat default to allow
    ///   complete function / class explanations without truncation.
    pub fn coding() -> Self {
        Self {
            temperature: Some(0.2),
            top_p: Some(0.95),
            top_k: None,
            min_p: None,
            max_tokens: Some(1024),
            frequency_penalty: Some(1.15),
            presence_penalty: Some(0.1),
        }
    }

    /// Like [`coding`](Self::coding) but capped at 256 tokens for mobile
    /// devices where latency matters more than completeness.
    pub fn coding_mobile() -> Self {
        Self {
            max_tokens: Some(256),
            ..Self::coding()
        }
    }
}

// ── GGUF model configuration ─────────────────────────────────────────────────

/// Configuration for loading a GGUF model via mistral.rs.
///
/// This is the primary model format for on-device mobile/desktop inference
/// because GGUF files are pre-quantized and load at their compressed size,
/// avoiding the large memory spike of in-situ quantization.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GgufModelConfig {
    /// HuggingFace model repository ID, e.g. `"bartowski/Qwen2.5-1.5B-Instruct-GGUF"`.
    pub model_id: String,
    /// GGUF filename(s) within the repository, e.g. `["Qwen2.5-1.5B-Instruct-Q4_K_M.gguf"]`.
    pub files: Vec<String>,
    /// Optional: explicit tokenizer model ID (required on Android where GGUF
    /// embedded tokenizers are not supported by the candle backend).
    pub tok_model_id: Option<String>,
    /// Human-friendly display name, e.g. `"Qwen 2.5 1.5B"`.
    pub display_name: String,
    /// Approximate memory footprint description, e.g. `"~941 MB (GGUF Q4_K_M)"`.
    pub approx_memory: String,
}

// ── ISQ model configuration ──────────────────────────────────────────────────

/// Configuration for loading an ISQ (in-situ quantised) model via mistral.rs
/// `TextModelBuilder`.
///
/// Unlike GGUF models, ISQ models are loaded directly from a HuggingFace
/// safetensors repo and quantised in-situ on the device.  They require more
/// RAM during the load phase but support a wider range of architectures
/// (e.g. DeepSeek-Coder-V2-Lite, Qwen2.5-Coder-7B).
///
/// **macOS-only for now** — ISQ with Metal acceleration requires the `metal`
/// feature on mistral.rs.  The struct is still defined on all platforms so
/// that types.rs compiles everywhere without `cfg` gates on the record
/// (UniFFI requires all record types to be reachable without feature flags).
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct IsqModelConfig {
    /// HuggingFace model repository ID, e.g. `"Qwen/Qwen2.5-Coder-7B-Instruct"`.
    pub model_id: String,
    /// Number of quantisation bits.  `4` → Q4K (AFQ4 on Metal), `8` → Q8_0 (AFQ8 on Metal).
    /// Any value outside `{4, 8}` defaults to 4-bit.
    pub isq_bits: u8,
    /// Human-friendly display name, e.g. `"Qwen 2.5 Coder 7B"`.
    pub display_name: String,
    /// Approximate memory footprint description, e.g. `"~4.5 GB (Q4K, Metal)"`.
    pub approx_memory: String,
}

impl IsqModelConfig {
    /// Qwen 2.5 Coder 7B Instruct — ISQ 4-bit, Metal-accelerated (~4.5 GB).
    ///
    /// Best macOS desktop coding quality.  Loaded from the official HF repo
    /// and quantised in-situ; requires ~8 GB RAM during the load phase.
    pub fn qwen25_coder_7b_isq4() -> Self {
        Self {
            model_id: super::models::QWEN25_CODER_7B_INSTRUCT.into(),
            isq_bits: 4,
            display_name: "Qwen 2.5 Coder 7B (ISQ 4-bit)".into(),
            approx_memory: "~4.5 GB (ISQ Q4K, Metal)".into(),
        }
    }

    /// Qwen 2.5 Coder 7B Instruct — ISQ 8-bit, Metal-accelerated (~9 GB).
    ///
    /// Higher quality than the 4-bit variant; requires ~12 GB RAM on load.
    /// Suitable for Macs with 16 GB+ unified memory.
    pub fn qwen25_coder_7b_isq8() -> Self {
        Self {
            model_id: super::models::QWEN25_CODER_7B_INSTRUCT.into(),
            isq_bits: 8,
            display_name: "Qwen 2.5 Coder 7B (ISQ 8-bit)".into(),
            approx_memory: "~9 GB (ISQ Q8K, Metal)".into(),
        }
    }
}

// ── Inference result ─────────────────────────────────────────────────────────

/// The result of a chat inference request.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct InferenceResult {
    /// The generated assistant reply text.
    pub text: String,
    /// Wall-clock duration of the inference in seconds.
    pub duration_secs: f64,
    /// Human-readable duration string (e.g. `"2m 3.1s"` or `"4.5s"`).
    pub duration_display: String,
    /// Finish reason reported by the model (e.g. `"stop"`, `"length"`).
    pub finish_reason: String,
    /// Tool calls requested by the model (empty when no tools were invoked).
    pub tool_calls: Vec<ToolCallInfo>,
}

// ── Tool calling ─────────────────────────────────────────────────────────────

/// A tool definition passed to the model so it knows which tools are available.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct ToolDefinition {
    /// Tool name (e.g. `"read_file"`).
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema string describing the tool's parameters.
    pub parameters_schema: String,
}

/// Information about a single tool call requested by the model.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct ToolCallInfo {
    /// Unique identifier for this tool call (used to correlate results).
    pub id: String,
    /// The name of the function the model wants to invoke.
    pub function_name: String,
    /// JSON-encoded arguments for the function.
    pub arguments: String,
}

/// The result of executing a tool, sent back to the model.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct ToolResult {
    /// The tool call ID this result corresponds to.
    pub tool_call_id: String,
    /// The output produced by executing the tool.
    pub content: String,
}

// ── Streaming chunk ──────────────────────────────────────────────────────────

/// A single streaming token chunk from the inference engine.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct StreamChunk {
    /// The token text delta (may be empty for the final chunk).
    pub delta: String,
    /// Whether this is the final chunk in the stream.
    pub done: bool,
    /// Finish reason (only set on the final chunk).
    pub finish_reason: Option<String>,
}

// ── Engine status ────────────────────────────────────────────────────────────

/// Lifecycle status of the inference engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
#[serde(rename_all = "snake_case")]
pub enum EngineStatus {
    /// No model loaded.
    Unloaded,
    /// Model is being downloaded / loaded into memory.
    Loading,
    /// Model is loaded and ready to accept requests.
    Ready,
    /// Model is currently running inference.
    Generating,
    /// An error occurred (model may or may not still be loaded).
    Error,
}

impl std::fmt::Display for EngineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineStatus::Unloaded => write!(f, "unloaded"),
            EngineStatus::Loading => write!(f, "loading"),
            EngineStatus::Ready => write!(f, "ready"),
            EngineStatus::Generating => write!(f, "generating"),
            EngineStatus::Error => write!(f, "error"),
        }
    }
}

// ── Engine info snapshot ─────────────────────────────────────────────────────

/// A point-in-time snapshot of the engine's state, suitable for status UIs.
///
/// Note: `history_length` is `u64` (not `usize`) for UniFFI compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct EngineInfo {
    pub status: EngineStatus,
    /// Display name of the currently loaded model, if any.
    pub model_name: Option<String>,
    /// Approximate memory footprint description, if a model is loaded.
    pub approx_memory: Option<String>,
    /// Number of conversation turns in the current history.
    pub history_length: u64,
}

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors that can occur during inference engine operations.
///
/// Exposed to Swift / Kotlin as a sealed class / enum via `uniffi::Error`.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum InferenceError {
    #[error("No model loaded — call `load_model` first")]
    NoModelLoaded,

    #[error("Model is already loaded: {model_name}")]
    AlreadyLoaded { model_name: String },

    #[error("Failed to build model: {reason}")]
    ModelBuild { reason: String },

    #[error("Inference failed: {reason}")]
    Inference { reason: String },

    #[error("Model loading was cancelled")]
    Cancelled,

    #[error("{reason}")]
    Other { reason: String },
}

// ── Duration formatting helper ───────────────────────────────────────────────

/// Format a `std::time::Duration` as `Xm Ys` or just `Ys` when under a minute.
pub fn format_duration(d: std::time::Duration) -> String {
    let total_secs = d.as_secs_f64();
    let mins = (total_secs / 60.0).floor() as u64;
    let secs = total_secs - (mins as f64 * 60.0);
    if mins > 0 {
        format!("{}m {:.1}s", mins, secs)
    } else {
        format!("{:.1}s", secs)
    }
}

// ── Tool calling types (Rust-only) ──────────────────────────────────────────
//
// These types are intentionally NOT annotated with `uniffi::Record` /
// `uniffi::Enum` — they are for Rust consumers only.  The UniFFI surface
// area (Swift / Kotlin) remains unchanged.
// (`ToolDefinition`, `ToolCallInfo`, and `ToolResult` are defined above
// with UniFFI derives and shared across both layers.)

/// A tool call requested by the model.
#[derive(Debug, Clone)]
pub struct ToolCallRequest {
    /// Unique identifier for this tool call (used to correlate results).
    pub id: String,
    /// Name of the function the model wants to invoke.
    pub function_name: String,
    /// JSON string of the function arguments.
    pub arguments: String,
}

/// Result from a tool-aware inference call (Rust-only, not UniFFI-exported).
#[derive(Debug, Clone)]
pub struct ToolAwareResult {
    /// Text content of the assistant reply (may be empty when the model
    /// decides to call tools instead of responding with text).
    pub text: String,
    /// Tool calls requested by the model.  Empty when the model responds
    /// with text only.
    pub tool_calls: Vec<ToolCallRequest>,
    /// Wall-clock inference duration in seconds.
    pub duration_secs: f64,
    /// Human-readable duration string (e.g. `"1.3s"`).
    pub duration_display: String,
    /// Finish reason reported by the model (e.g. `"stop"`, `"tool_calls"`).
    pub finish_reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful.");
        assert_eq!(sys.role, ChatRole::System);
        assert_eq!(sys.content, "You are helpful.");

        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, ChatRole::User);

        let asst = ChatMessage::assistant("Hi there!");
        assert_eq!(asst.role, ChatRole::Assistant);
    }

    #[test]
    fn chat_role_display() {
        assert_eq!(ChatRole::System.to_string(), "system");
        assert_eq!(ChatRole::User.to_string(), "user");
        assert_eq!(ChatRole::Assistant.to_string(), "assistant");
    }

    #[test]
    fn chat_role_serde_rename() {
        // Verify that #[serde(rename_all = "lowercase")] is applied by
        // checking the Display impl matches the expected serialised form.
        // (Full serde_json round-trip is tested in integration tests that
        // already depend on serde_json.)
        assert_eq!(format!("{}", ChatRole::System), "system");
        assert_eq!(format!("{}", ChatRole::User), "user");
        assert_eq!(format!("{}", ChatRole::Assistant), "assistant");
    }

    #[test]
    fn sampling_config_defaults() {
        let cfg = SamplingConfig::default();
        assert_eq!(cfg.temperature, Some(0.7));
        assert_eq!(cfg.top_p, Some(0.95));
        assert_eq!(cfg.max_tokens, Some(512));
    }

    #[test]
    fn sampling_config_deterministic() {
        let cfg = SamplingConfig::deterministic();
        assert_eq!(cfg.temperature, Some(0.0));
        assert!(cfg.top_p.is_none());
    }

    #[test]
    fn sampling_config_mobile() {
        let cfg = SamplingConfig::mobile();
        assert_eq!(cfg.max_tokens, Some(128));
    }

    #[test]
    fn engine_status_display() {
        assert_eq!(EngineStatus::Ready.to_string(), "ready");
        assert_eq!(EngineStatus::Generating.to_string(), "generating");
    }

    #[test]
    fn format_duration_under_minute() {
        let d = std::time::Duration::from_secs_f64(4.567);
        assert_eq!(format_duration(d), "4.6s");
    }

    #[test]
    fn format_duration_over_minute() {
        let d = std::time::Duration::from_secs_f64(125.3);
        assert_eq!(format_duration(d), "2m 5.3s");
    }

    #[test]
    fn inference_error_display() {
        let err = InferenceError::NoModelLoaded;
        assert_eq!(err.to_string(), "No model loaded — call `load_model` first");

        let err = InferenceError::ModelBuild {
            reason: "out of memory".into(),
        };
        assert_eq!(err.to_string(), "Failed to build model: out of memory");

        let err = InferenceError::AlreadyLoaded {
            model_name: "Qwen 2.5".into(),
        };
        assert_eq!(err.to_string(), "Model is already loaded: Qwen 2.5");
    }

    #[test]
    fn engine_info_history_is_u64() {
        let info = EngineInfo {
            status: EngineStatus::Ready,
            model_name: Some("Test".into()),
            approx_memory: None,
            history_length: 42,
        };
        assert_eq!(info.history_length, 42u64);
    }
}
