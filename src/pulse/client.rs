use smbcloud_gresiq_sdk::GresiqClient;

use super::events::{InferenceEvent, ModelLoadedEvent};

/// onde's thin wrapper around `GresiqClient`.
///
/// `GresiqClient` handles the HTTP transport and auth headers.
/// This layer adds the `edge_id` (which machine we're running on)
/// and the fire-and-forget spawning so telemetry never blocks inference.
#[derive(Debug, Clone)]
pub struct PulseClient {
    inner:   GresiqClient,
    edge_id: String,
}

impl PulseClient {
    /// Reads `GRESIQ_BASE_URL`, `GRESIQ_API_KEY`, and `GRESIQ_API_SECRET`
    /// from the environment (delegated to `GresiqClient::from_env`), plus
    /// `ONDE_EDGE_ID` for the node name. Returns `None` if any required
    /// GresIQ var is missing — telemetry stays off, nothing breaks.
    pub fn from_env() -> Option<Self> {
        let inner   = GresiqClient::from_env()?;
        let edge_id = std::env::var("ONDE_EDGE_ID")
            .unwrap_or_else(|_| "onde-unknown".to_string());
        Some(PulseClient { inner, edge_id })
    }

    /// Spawns a background task to POST the model-load event and returns
    /// immediately. Won't slow down the first message. If the request fails,
    /// you get a `warn!` log entry — that's the whole error budget.
    pub fn record_model_loaded(&self, model_id: String, model_name: String, load_duration_ms: u64) {
        let client  = self.clone();
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

    /// Same deal as `record_model_loaded` but for inference. Spawns, returns,
    /// logs on failure.
    pub fn record_inference(
        &self,
        model_id:   String,
        request_id: String,
        duration_ms: u64,
        status:     String,
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
