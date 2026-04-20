use serde::Serialize;

/// What gets sent to GresIQ when a model finishes loading.
#[derive(Debug, Clone, Serialize)]
pub struct ModelLoadedEvent {
    /// Which machine this is. Keep it stable across restarts — change it
    /// and you'll get a duplicate edge row in the dashboard.
    pub edge_id: String,
    /// The HuggingFace repo ID, e.g. `bartowski/Qwen2.5-3B-Instruct-GGUF`.
    pub model_id: String,
    /// What shows up in the UI, e.g. `Qwen 2.5 3B`.
    pub model_name: String,
    /// How long the load took. This is `elapsed.as_millis() as u64`,
    /// where `elapsed` is what `load_gguf_model` returns.
    pub load_duration_ms: u64,
}

/// What gets sent to GresIQ after each `send_message` or `generate`.
#[derive(Debug, Clone, Serialize)]
pub struct InferenceEvent {
    /// Same edge as in `ModelLoadedEvent`.
    pub edge_id: String,
    /// Same model as in `ModelLoadedEvent`.
    pub model_id: String,
    /// Auto-generated per request. Looks like `onde-1720000000000-42`.
    pub request_id: String,
    /// Wall-clock time for the whole response, in milliseconds.
    pub duration_ms: u64,
    /// `"success"`, `"cancelled"`, or `"error"`. Currently always
    /// `"success"` here — inference errors throw before we reach this point.
    pub status: String,
}
