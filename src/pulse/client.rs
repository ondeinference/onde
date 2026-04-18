use smbcloud_gresiq_sdk::{Environment, GresiqClient, GresiqCredentials};

use super::events::{InferenceEvent, ModelLoadedEvent};

/// GresIQ API key embedded at SDK build time.
/// Consumer apps never set this — it's Onde Inference's own credential.
const EMBEDDED_API_KEY: Option<&str> = option_env!("GRESIQ_API_KEY");

/// GresIQ API secret embedded at SDK build time.
const EMBEDDED_API_SECRET: Option<&str> = option_env!("GRESIQ_API_SECRET");

/// Onde telemetry client.  Wraps GresiqClient so pulse events land in the
/// right GresIQ-managed tables without consumer apps knowing anything about
/// the GresIQ auth layer underneath.
///
/// GresIQ credentials are embedded in the SDK at build time.
/// Consumer apps only provide an `edge_id` (stable device identifier).
///
/// Cheap to clone: the inner GresiqClient holds an Arc-backed reqwest::Client,
/// so cloning is a pointer bump, not a new TCP connection.
#[derive(Debug, Clone)]
pub struct PulseClient {
    inner: GresiqClient,
    edge_id: String,
}

impl PulseClient {
    /// Build a pulse client using the GresIQ credentials embedded in the SDK.
    ///
    /// Returns `None` if the SDK was compiled without `GRESIQ_API_KEY` /
    /// `GRESIQ_API_SECRET` (e.g. a local dev build of onde without `.env`).
    /// In that case telemetry is silently disabled — no panic, no partial state.
    ///
    /// `edge_id` is a stable device identifier (installation UUID).
    /// Pass an empty string to default to `"onde-unknown"`.
    pub fn new(environment: Environment, edge_id: String) -> Option<Self> {
        let api_key = EMBEDDED_API_KEY?;
        let api_secret = EMBEDDED_API_SECRET?;

        let edge_id = if edge_id.is_empty() {
            "onde-unknown".to_string()
        } else {
            edge_id
        };

        let credentials = GresiqCredentials {
            api_key,
            api_secret,
        };

        let inner = GresiqClient::from_credentials(environment, credentials);

        Some(PulseClient { inner, edge_id })
    }

    /// Spawns a background task that writes the model-load event to the
    /// pulse/model_loaded table, then returns immediately.  Slow network
    /// responses don't block the first inference request.
    ///
    /// Failed writes emit a warn! log line — no retry, no queue,
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
    /// completions.  Writes to pulse/inference_event.  Logs on failure, no retry.
    pub fn record_inference(
        &self,
        model_id: String,
        request_id: String,
        duration_ms: u64,
        status: String,
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
