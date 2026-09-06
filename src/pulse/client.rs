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
    onde_app_id: Option<String>,
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

    /// ADR-0003 M2 dual-write toggle. The mirror is deliberately opt-in while
    /// document parity is being proven. When enabled, every attributed pulse
    /// event is *also* written to the GresIQ document gateway
    /// (`/gresiq/v1/collections/:collection`) in addition to the semantic
    /// `pulse/*` routes, which still serve reads until the M5 cut-over. Set
    /// `ONDE_PULSE_DUAL_WRITE=1` (or `true`/`yes`/`on`) to enable it.
    fn dual_write_enabled() -> bool {
        dual_write_enabled_value(std::env::var("ONDE_PULSE_DUAL_WRITE").ok().as_deref())
    }

    /// Build a pulse client using the GresIQ credentials embedded in the SDK.
    ///
    /// Returns `None` if the SDK was compiled without `GRESIQ_API_KEY` /
    /// `GRESIQ_API_SECRET` (e.g. a local dev build of onde without `.env`).
    /// In that case telemetry is silently disabled — no panic, no partial state.
    ///
    /// `edge_id` is a stable device identifier (installation UUID).
    /// Pass an empty string to default to `"onde-unknown"`.
    ///
    /// `onde_app_id` is the Onde app UUID (from the ondeinference.com
    /// dashboard) that owns this telemetry. `None` is fine — events then carry
    /// no app association (open-source/direct Rust consumers).
    pub fn new(
        environment: Environment,
        edge_id: String,
        onde_app_id: Option<String>,
    ) -> Option<Self> {
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

        Some(PulseClient {
            inner,
            edge_id,
            onde_app_id,
        })
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
                onde_app_id: client.onde_app_id.clone(),
            };
            if let Err(error) = client.inner.insert("pulse/model_loaded", &event).await {
                log::warn!("pulse: model_loaded failed: {}", error);
            }
            client.spawn_model_loaded_dual_write(event);
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
        ttft_ms: Option<u64>,
        status: String,
    ) {
        let client = self.clone();
        let send_event = async move {
            let event = InferenceEvent {
                edge_id: client.edge_id.clone(),
                model_id,
                request_id,
                duration_ms,
                ttft_ms,
                status,
                onde_app_id: client.onde_app_id.clone(),
            };
            if let Err(error) = client.inner.insert("pulse/inference_event", &event).await {
                log::warn!("pulse: inference_event failed: {}", error);
            }
            client.dual_write_inference(&event).await;
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

    // ── ADR-0003 M2 dual-write to the document gateway ───────────────────────
    //
    // These mirror each semantic pulse event into JSONB collections under the
    // Onde gresiq_app. The relational `pulse/*` routes still own reads until the
    // M5 cut-over, so failures are logged and never affect inference.

    /// Move the three model-load mirror requests off the model-loading critical
    /// path. A dedicated thread is acceptable here because this runs once per
    /// model activation, not once per inference, and also works for callers
    /// whose original async runtime is short-lived.
    fn spawn_model_loaded_dual_write(&self, event: ModelLoadedEvent) {
        if !Self::dual_write_enabled() || event.onde_app_id.is_none() {
            return;
        }

        let client = self.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match runtime {
                Ok(runtime) => runtime.block_on(client.dual_write_model_loaded(&event)),
                Err(error) => log::warn!(
                    "pulse: could not create runtime for model-loaded dual-write: {}",
                    error
                ),
            }
        });
    }

    /// Mirror a model-load into `pulse_edges`, `pulse_models`, and
    /// `pulse_deployments`. Each collection is a separate upsert (the
    /// FK-ordered chain of the semantic route becomes client-orchestrated
    /// multi-insert; the document store has no relationships to order).
    async fn dual_write_model_loaded(&self, event: &ModelLoadedEvent) {
        let Some(onde_app_id) = event.onde_app_id.as_deref() else {
            return;
        };

        let edge_key = scoped_document_key(onde_app_id, &[&event.edge_id]);

        let model_key = scoped_document_key(onde_app_id, &[&event.model_id]);
        let deployment_key = scoped_document_key(onde_app_id, &[&event.edge_id, &event.model_id]);
        let edge_document = edge_document(event, onde_app_id);
        let model_document = serde_json::json!({
            "slug": event.model_id,
            "model_id": event.model_id,
            "model_name": event.model_name,
            "onde_app_id": event.onde_app_id,
        });
        let deployment_document = serde_json::json!({
            "edge_id": event.edge_id,
            "model_id": event.model_id,
            "load_duration_ms": event.load_duration_ms,
            "onde_app_id": event.onde_app_id,
            "residency_state": "loaded",
        });

        tokio::join!(
            self.write_doc("pulse_edges", Some(&edge_key), &edge_document),
            self.write_doc("pulse_models", Some(&model_key), &model_document),
            self.write_doc(
                "pulse_deployments",
                Some(&deployment_key),
                &deployment_document,
            ),
        );
    }

    /// Mirror an inference event idempotently. Request IDs are already unique,
    /// so using an app-scoped natural key prevents a retry from double-counting
    /// the same inference during cutover validation.
    async fn dual_write_inference(&self, event: &InferenceEvent) {
        if !Self::dual_write_enabled() {
            return;
        }
        let Some(onde_app_id) = event.onde_app_id.as_deref() else {
            return;
        };
        let event_key = scoped_document_key(onde_app_id, &[&event.request_id]);
        let document = inference_document(event);

        self.write_doc("pulse_inference_events", Some(&event_key), &document)
            .await;
    }

    /// Upsert (or append, when `key` is `None`) one document, logging failures.
    async fn write_doc(&self, collection: &str, key: Option<&str>, doc: &serde_json::Value) {
        if let Err(error) = self.inner.upsert_document(collection, key, doc).await {
            log::warn!("pulse: dual-write to {} failed: {}", collection, error);
        }
    }
}

fn dual_write_enabled_value(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

/// Keys are scoped inside Onde's shared GresIQ app so identical installation
/// ids belonging to different Onde customer apps cannot overwrite each other.
fn scoped_document_key(onde_app_id: &str, parts: &[&str]) -> String {
    std::iter::once(onde_app_id)
        .chain(parts.iter().copied())
        .collect::<Vec<_>>()
        .join(":")
}

/// Build the inference document. `ttft_ms` is omitted entirely when the path
/// could not measure it, so a reader can tell "not measured" from a real zero
/// — the relational route stores a hardcoded `0` and cannot make that
/// distinction.
fn inference_document(event: &InferenceEvent) -> serde_json::Value {
    let mut document = serde_json::json!({
        "edge_id": event.edge_id,
        "model_id": event.model_id,
        "request_id": event.request_id,
        "duration_ms": event.duration_ms,
        "status": event.status,
        "onde_app_id": event.onde_app_id,
    });

    if let (Some(map), Some(ttft_ms)) = (document.as_object_mut(), event.ttft_ms) {
        map.insert("ttft_ms".to_string(), serde_json::json!(ttft_ms));
    }

    document
}

/// `status` is always `"online"`: this document is only written when the edge
/// just loaded a model, and nothing reports a shutdown. `last_seen_ms` is what
/// makes that honest — a reader counts an edge as active by recency, the way
/// the relational dashboard windows on `last_seen_at`, rather than trusting a
/// status that is never revoked.
fn edge_document(event: &ModelLoadedEvent, onde_app_id: &str) -> serde_json::Value {
    serde_json::json!({
        "edge_id": event.edge_id,
        "onde_app_id": onde_app_id,
        "status": "online",
        "last_seen_ms": epoch_millis(),
    })
}

/// Milliseconds since the Unix epoch. The crate has no date dependency and
/// this is only ever read as a recency window, so an integer beats pulling in
/// a formatting library for an RFC 3339 string.
fn epoch_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_write_requires_an_explicit_positive_flag() {
        for disabled in [
            None,
            Some(""),
            Some("0"),
            Some("false"),
            Some("no"),
            Some("off"),
        ] {
            assert!(!dual_write_enabled_value(disabled));
        }
        for enabled in [Some("1"), Some("true"), Some("YES"), Some(" on ")] {
            assert!(dual_write_enabled_value(enabled));
        }
    }

    #[test]
    fn document_keys_include_the_onde_app_boundary() {
        assert_eq!(
            scoped_document_key("app-a", &["edge-1", "model-1"]),
            "app-a:edge-1:model-1"
        );
        assert_ne!(
            scoped_document_key("app-a", &["onde-unknown"]),
            scoped_document_key("app-b", &["onde-unknown"])
        );
        assert_eq!(
            scoped_document_key("app-a", &["onde-1720000000000-42"]),
            "app-a:onde-1720000000000-42"
        );
    }

    #[test]
    fn edge_document_carries_cutover_state_and_tenant_id() {
        let event = ModelLoadedEvent {
            edge_id: "edge-1".to_string(),
            model_id: "model-1".to_string(),
            model_name: "Model 1".to_string(),
            load_duration_ms: 42,
            onde_app_id: Some("app-a".to_string()),
        };
        let document = edge_document(&event, "app-a");
        assert_eq!(document["edge_id"], "edge-1");
        assert_eq!(document["onde_app_id"], "app-a");
        assert_eq!(document["status"], "online");
        // Liveness has to be derivable from the document itself: nothing ever
        // writes an offline status, so a reader windows on this instead.
        assert!(document["last_seen_ms"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn inference_document_omits_ttft_when_it_was_not_measured() {
        let mut event = InferenceEvent {
            edge_id: "edge-1".to_string(),
            model_id: "model-1".to_string(),
            request_id: "req-1".to_string(),
            duration_ms: 400,
            ttft_ms: None,
            status: "success".to_string(),
            onde_app_id: Some("app-a".to_string()),
        };
        assert!(inference_document(&event).get("ttft_ms").is_none());

        event.ttft_ms = Some(90);
        assert_eq!(inference_document(&event)["ttft_ms"], 90);
    }
}
