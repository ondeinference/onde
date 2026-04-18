use std::collections::HashMap;

use smbcloud_gresiq_sdk::{Environment, GresiqClient, GresiqCredentials};

use super::events::{InferenceEvent, ModelLoadedEvent};

/// Onde telemetry client. Wraps GresiqClient so pulse events land in the
/// right GresIQ-managed tables without this crate knowing anything about
/// the HTTP auth layer underneath.
///
/// Cheap to clone: the inner GresiqClient holds an Arc-backed reqwest::Client,
/// so cloning is a pointer bump, not a new TCP connection.
#[derive(Debug, Clone)]
pub struct PulseClient {
    inner:   GresiqClient,
    edge_id: String,
}

impl PulseClient {
    /// Reads credentials from the environment and resolves the GresIQ
    /// gateway URL from the given `Environment`.
    ///
    /// GresIQ layer (shared across all SDK clients):
    ///   `GRESIQ_API_KEY`, `GRESIQ_API_SECRET`
    ///
    /// SDK client layer (this app’s own credentials):
    ///   `ONDE_CLIENT_KEY`, `ONDE_CLIENT_SECRET`
    ///   `ONDE_EDGE_ID`  (optional, defaults to `"onde-unknown"`)
    ///
    /// Returns `None` if any required var is absent. Nothing blows up —
    /// the engine just skips telemetry for the whole run.
    pub fn from_env(environment: Environment) -> Option<Self> {
        // GresIQ credentials
        let gresiq_api_key    = std::env::var("GRESIQ_API_KEY").ok()?;
        let gresiq_api_secret = std::env::var("GRESIQ_API_SECRET").ok()?;

        // SDK client credentials
        let client_key    = std::env::var("ONDE_CLIENT_KEY").ok()?;
        let client_secret = std::env::var("ONDE_CLIENT_SECRET").ok()?;
        let edge_id       = std::env::var("ONDE_EDGE_ID")
            .unwrap_or_else(|_| "onde-unknown".to_string());

        let mut extra = HashMap::new();
        extra.insert("X-Onde-Client-Key".to_string(),    client_key);
        extra.insert("X-Onde-Client-Secret".to_string(), client_secret);

        let credentials = GresiqCredentials {
            api_key:    &gresiq_api_key,
            api_secret: &gresiq_api_secret,
        };

        let inner = GresiqClient::from_credentials(environment, credentials)
            .with_extra_headers(extra);

        Some(PulseClient { inner, edge_id })
    }

    /// Spawns a background task that writes the model-load event to the
    /// pulse/model_loaded table, then returns immediately. Slow network
    /// responses don't block the first inference request.
    ///
    /// Failed writes emit a warn! log line -- no retry, no queue,
    /// and no effect on the caller.
    pub fn record_model_loaded(&self, model_id: String, model_name: String, load_duration_ms: u64) {
        let client = self.clone();
        tokio::spawn(async move {
            let event = ModelLoadedEvent {
                edge_id: client.edge_id.clone(),
                model_id,
                model_name,
                load_duration_ms,
            };
            if let Err(error) = client.inner.insert("pulse/model_loaded", &event).await {
                log::warn!("pulse: model_loaded failed: {}", error);
            }
        });
    }

    /// Same fire-and-forget pattern as record_model_loaded but for inference
    /// completions. Writes to pulse/inference_event. Logs on failure, no retry.
    pub fn record_inference(
        &self,
        model_id:    String,
        request_id:  String,
        duration_ms: u64,
        status:      String,
    ) {
        let client = self.clone();
        tokio::spawn(async move {
            let event = InferenceEvent {
                edge_id: client.edge_id.clone(),
                model_id,
                request_id,
                duration_ms,
                status,
            };
            if let Err(error) = client.inner.insert("pulse/inference_event", &event).await {
                log::warn!("pulse: inference_event failed: {}", error);
            }
        });
    }
}
