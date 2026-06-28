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
    /// The Onde app that loaded this model (UUID string from the
    /// ondeinference.com dashboard). `None` for SDK builds without app
    /// credentials or for direct Rust consumers. In the document store this
    /// is a plain field on the `pulse_models` doc, not a foreign key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onde_app_id: Option<String>,
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
    /// Same Onde app as the corresponding `ModelLoadedEvent`. `None` for SDK
    /// builds without app credentials or direct Rust consumers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onde_app_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_loaded_serializes_onde_app_id_when_present() {
        let event = ModelLoadedEvent {
            edge_id: "edge-1".into(),
            model_id: "org/repo".into(),
            model_name: "Repo 3B".into(),
            load_duration_ms: 1234,
            onde_app_id: Some("app-uuid".into()),
        };
        let value = serde_json::to_value(&event).expect("serialize");
        assert_eq!(value["onde_app_id"], "app-uuid");
        assert_eq!(value["load_duration_ms"], 1234);
    }

    #[test]
    fn onde_app_id_is_omitted_when_absent() {
        let event = InferenceEvent {
            edge_id: "edge-1".into(),
            model_id: "org/repo".into(),
            request_id: "onde-1-0".into(),
            duration_ms: 42,
            status: "success".into(),
            onde_app_id: None,
        };
        let value = serde_json::to_value(&event).expect("serialize");
        assert!(value.get("onde_app_id").is_none());
    }
}
