use smbcloud_gresiq_sdk::{Environment, GresiqClient, GresiqCredentials};

use super::events::{InferenceEvent, ModelLoadedEvent};

/// GresIQ credentials embedded at SDK build time — one pair per environment.
/// Consumer apps never set these — they're Onde Inference's own credentials.
const EMBEDDED_API_KEY_DEV: Option<&str> = option_env!("GRESIQ_API_KEY_DEV");
const EMBEDDED_API_SECRET_DEV: Option<&str> = option_env!("GRESIQ_API_SECRET_DEV");
const EMBEDDED_API_KEY_PRODUCTION: Option<&str> = option_env!("GRESIQ_API_KEY_PRODUCTION");
const EMBEDDED_API_SECRET_SECRET_PRODUCTION: Option<&str> =
    option_env!("GRESIQ_API_SECRET_PRODUCTION");

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
    /// Returns true when pulse telemetry is disabled explicitly by the host app.
    pub fn disabled_by_env() -> bool {
        matches!(
            std::env::var("ONDE_DISABLE_PULSE")
                .ok()
                .as_deref()
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("1") | Some("true") | Some("yes") | Some("on")
        )
    }

    /// Build a pulse client using the GresIQ credentials embedded in the SDK.
    ///
    /// Returns `None` if the SDK was compiled without `GRESIQ_API_KEY` /
    /// `GRESIQ_API_SECRET` (e.g. a local dev build of onde without `.env`).
    /// In that case telemetry is silently disabled — no panic, no partial state.
    ///
    /// `edge_id` is a stable device identifier (installation UUID).
    /// Pass an empty string to default to `"onde-unknown"`.
    pub fn new(environment: Environment, edge_id: String) -> Option<Self> {
        if Self::disabled_by_env() {
            return None;
        }

        let (api_key, api_secret) = match environment {
            Environment::Dev => (EMBEDDED_API_KEY_DEV?, EMBEDDED_API_SECRET_DEV?),
            Environment::Production => (
                EMBEDDED_API_KEY_PRODUCTION?,
                EMBEDDED_API_SECRET_SECRET_PRODUCTION?,
            ),
        };

        let edge_id = if edge_id.is_empty() {
            "onde-unknown".to_string()
        } else {
            edge_id
        };

        // reqwest 0.12.x requires a live Tokio runtime (with I/O reactor)
        // to construct a Client.  GresiqClient::from_credentials calls
        // reqwest::Client::new() internally.  Guard against panics when
        // called from a thread/context that lacks a Tokio reactor.
        if tokio::runtime::Handle::try_current().is_err() {
            log::warn!(
                "pulse: no Tokio runtime available — \
                 deferring PulseClient creation"
            );
            return None;
        }

        let credentials = GresiqCredentials {
            api_key,
            api_secret,
        };

        let inner = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            GresiqClient::from_credentials(environment, credentials)
        })) {
            Ok(client) => client,
            Err(_) => {
                log::warn!(
                    "pulse: GresiqClient::from_credentials panicked \
                     (likely missing Tokio reactor) — telemetry disabled"
                );
                return None;
            }
        };

        Some(PulseClient { inner, edge_id })
    }

    /// Writes the model-load event to the pulse/model_loaded table and
    /// **awaits** the result before returning.
    ///
    /// This must complete before any `record_inference` call for the same
    /// edge + model is made, because the API enforces a foreign-key
    /// constraint: an `inference_event` row is rejected with HTTP 404
    /// ("Call model_loaded first") if no matching `model_loaded` row exists.
    ///
    /// Keeping this synchronous with respect to the model-load path means
    /// the row is always present by the time the first inference event fires.
    /// A failed write emits a warn! log line — no retry, no queue, and no
    /// effect on the caller.
    pub async fn record_model_loaded(
        &self,
        model_id: String,
        model_name: String,
        load_duration_ms: u64,
    ) {
        let client = self.clone();
        let send_event = async move {
            let event = ModelLoadedEvent {
                edge_id: client.edge_id.clone(),
                model_id,
                model_name,
                load_duration_ms,
            };
            if let Err(error) = client.inner.insert("pulse/model_loaded", &event).await {
                log::warn!("pulse: model_loaded failed: {}", error);
            }
        };

        if tokio::runtime::Handle::try_current().is_ok() {
            send_event.await;
            return;
        }

        let join = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            match runtime {
                Ok(runtime) => {
                    runtime.block_on(send_event);
                }
                Err(error) => {
                    log::warn!(
                        "pulse: could not create fallback runtime for model_loaded: {}",
                        error
                    );
                }
            }
        });

        if join.join().is_err() {
            log::warn!("pulse: fallback model_loaded thread panicked");
        }
    }

    /// Same fire-and-forget pattern as record_model_loaded but for inference
    /// completions.  Writes to pulse/inference_event.  Logs on failure, no retry.
    ///
    /// Swift / UniFFI consumers may call into the SDK from contexts where
    /// `tokio::spawn` does not have a current runtime handle even though the
    /// outer API is async.  To avoid panicking on Apple platforms, we prefer
    /// the current Tokio runtime when available and otherwise fall back to a
    /// short-lived current-thread runtime on a native background thread.
    pub fn record_inference(
        &self,
        model_id: String,
        request_id: String,
        duration_ms: u64,
        status: String,
    ) {
        let client = self.clone();
        let send_event = async move {
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
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(send_event);
            return;
        }

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            match runtime {
                Ok(runtime) => {
                    runtime.block_on(send_event);
                }
                Err(error) => {
                    log::warn!(
                        "pulse: could not create fallback runtime for inference_event: {}",
                        error
                    );
                }
            }
        });
    }
}
