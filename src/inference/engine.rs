//! On-device LLM chat inference engine powered by [mistral.rs](https://github.com/EricLBuehler/mistral.rs).
//!
//! `ChatEngine` provides a high-level, framework-agnostic API for:
//!
//! - Loading GGUF-quantized models (the primary format for mobile/desktop)
//! - Multi-turn chat with conversation history management
//! - Both blocking (non-streaming) and streaming inference
//! - Model lifecycle management (load / unload / status)
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │  App / UniFFI binding / test             │
//! └──────────────┬───────────────────────────┘
//!                │  (framework-agnostic API)
//!                ▼
//! ┌──────────────────────────────────────────┐
//! │            ChatEngine                    │
//! │  ┌────────────────────────────────────┐  │
//! │  │ Mutex<Option<LoadedModel>>         │  │
//! │  │  · Arc<mistralrs::Model>           │  │
//! │  │  · config (GgufModelConfig)        │  │
//! │  │  · history (Vec<ChatMessage>)      │  │
//! │  │  · sampling (SamplingConfig)       │  │
//! │  └────────────────────────────────────┘  │
//! └──────────────┬───────────────────────────┘
//!                │  (delegates to)
//!                ▼
//! ┌──────────────────────────────────────────┐
//! │  mistralrs::Model                        │
//! │  (wraps Arc<MistralRs> engine thread)    │
//! └──────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use onde::inference::engine::ChatEngine;
//! use onde::inference::types::*;
//!
//! let engine = ChatEngine::new();
//!
//! let config = GgufModelConfig {
//!     model_id: "bartowski/Qwen2.5-1.5B-Instruct-GGUF".into(),
//!     files: vec!["Qwen2.5-1.5B-Instruct-Q4_K_M.gguf".into()],
//!     tok_model_id: None,
//!     display_name: "Qwen 2.5 1.5B".into(),
//!     approx_memory: "~941 MB".into(),
//! };
//!
//! engine.load_gguf_model(config, None).await?;
//! engine.set_system_prompt("You are a helpful assistant.");
//!
//! let result = engine.send_message("Hello!").await?;
//! println!("{}", result.text);
//! ```

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
use std::sync::Arc;

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
use tokio::sync::Mutex;

use super::types::*;

// ── Platform-gated mistralrs imports ─────────────────────────────────────────

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
use mistralrs::{GgufModelBuilder, Model, RequestBuilder, TextMessageRole};

// ISQ types are only used by load_isq_model, which is macOS-only.
#[cfg(target_os = "macos")]
use mistralrs::{IsqBits, TextModelBuilder};

// ── Internals ────────────────────────────────────────────────────────────────

/// How a model was loaded — carries the data needed for status reporting.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
enum LoadedModelConfig {
    Gguf(GgufModelConfig),
    #[cfg(target_os = "macos")]
    Isq(IsqModelConfig),
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
impl LoadedModelConfig {
    fn display_name(&self) -> &str {
        match self {
            LoadedModelConfig::Gguf(c) => &c.display_name,
            #[cfg(target_os = "macos")]
            LoadedModelConfig::Isq(c) => &c.display_name,
        }
    }

    fn approx_memory(&self) -> &str {
        match self {
            LoadedModelConfig::Gguf(c) => &c.approx_memory,
            #[cfg(target_os = "macos")]
            LoadedModelConfig::Isq(c) => &c.approx_memory,
        }
    }
}

/// State of a loaded model held inside the engine's mutex.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
struct LoadedModel {
    /// The mistral.rs model handle. `Model` wraps `Arc<MistralRs>` so
    /// cloning is a cheap pointer copy — safe to snapshot before inference
    /// while releasing the mutex.
    model: Arc<Model>,
    /// Configuration used to load this model (kept for status reporting).
    config: LoadedModelConfig,
    /// Conversation history (system prompt is stored separately).
    history: Vec<ChatMessage>,
    /// System prompt prepended to every request.
    system_prompt: Option<String>,
    /// Sampling parameters for generation.
    sampling: SamplingConfig,
}

// ═════════════════════════════════════════════════════════════════════════════
// ChatEngine — the public API
// ═════════════════════════════════════════════════════════════════════════════

/// A reusable on-device LLM chat inference engine.
///
/// Thread-safe (`Send + Sync`) — safe to store in a `once_cell::sync::Lazy`,
/// `tokio::sync::OnceCell`, or shared application state.
///
/// All mutating operations acquire an internal `tokio::sync::Mutex`.  The
/// mutex is released *before* the actual (potentially slow) model inference
/// runs, so other tasks can still query status or history while generation
/// is in progress.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
pub struct ChatEngine {
    inner: Mutex<Option<LoadedModel>>,
    pulse: Option<crate::pulse::PulseClient>,
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
impl ChatEngine {
    // ── Construction ─────────────────────────────────────────────────────

    /// Create a new engine with no model loaded.
    pub fn new() -> Self {
        let environment = Self::pulse_environment();
        let pulse = crate::pulse::PulseClient::from_env(environment);

        match &pulse {
            Some(_) => log::info!(
                "ChatEngine: pulse telemetry enabled (environment={environment})"
            ),
            None => log::info!(
                "ChatEngine: pulse telemetry disabled \
                 (GRESIQ_API_KEY / GRESIQ_API_SECRET / ONDE_CLIENT_KEY / ONDE_CLIENT_SECRET not set)"
            ),
        }

        Self {
            inner: Mutex::new(None),
            pulse,
        }
    }

    /// Resolve the GresIQ environment from the `GRESIQ_ENVIRONMENT` env var.
    /// Defaults to `Production` when the var is absent or unrecognised.
    fn pulse_environment() -> smbcloud_gresiq_sdk::Environment {
        match std::env::var("GRESIQ_ENVIRONMENT").as_deref() {
            Ok("dev") => smbcloud_gresiq_sdk::Environment::Dev,
            _         => smbcloud_gresiq_sdk::Environment::Production,
        }
    }

    // ── Model lifecycle ──────────────────────────────────────────────────

    /// Load a GGUF model into the engine.
    ///
    /// If a model is already loaded it will be unloaded first (the previous
    /// `Model` is dropped, which terminates its engine thread).
    ///
    /// # Arguments
    ///
    /// * `config`         — Which model to load (repo ID, filename, etc.).
    /// * `system_prompt`  — Optional system prompt to prepend to every request.
    /// * `sampling`       — Sampling parameters; pass `None` for platform-aware
    ///   defaults (mobile gets [`SamplingConfig::mobile`],
    ///   desktop gets [`SamplingConfig::default`]).
    ///
    /// # Errors
    ///
    /// Returns [`InferenceError::ModelBuild`] if the model fails to download
    /// or load.
    pub async fn load_gguf_model(
        &self,
        config: GgufModelConfig,
        system_prompt: Option<String>,
        sampling: Option<SamplingConfig>,
    ) -> Result<std::time::Duration, InferenceError> {
        use log::info;
        use std::time::Instant;

        info!(
            "ChatEngine: loading GGUF model {} (files: {:?})",
            config.model_id, config.files
        );

        // ── Sandboxed platforms: seed GLOBAL_HF_CACHE ────────────────────
        //
        // On sandboxed platforms the default `~/.cache/huggingface/hub` path
        // is either inaccessible or non-existent:
        //   - Android: `dirs::home_dir()` returns `None` → `Cache::default()` panics.
        //   - iOS/tvOS: `~/.cache` is outside the app container → os error 1.
        //
        // The `get_paths_gguf!` macro in mistralrs falls back to
        // `Cache::default()` when `GLOBAL_HF_CACHE` (a OnceLock) is empty.
        //
        // `HF_HOME` must be set by the host app (via `configure_cache_dir`
        // or `download_model(app_data_dir:)`) before any model load.
        // `get_or_init` is a no-op if already seeded — safe to call repeatedly.
        #[cfg(any(
            target_os = "android",
            target_os = "ios",
            target_os = "tvos",
            target_os = "visionos",
            target_os = "watchos"
        ))]
        {
            let hf_home = std::env::var("HF_HOME")
                .map(std::path::PathBuf::from)
                .map_err(|_| InferenceError::ModelBuild {
                    reason: "HF_HOME is not set — cannot initialise HF cache. \
                             On iOS/tvOS/Android, call configure_cache_dir() or \
                             download_model(app_data_dir:) before load_gguf_model()."
                        .to_string(),
                })?;
            let hf_hub_cache = hf_home.join("hub");
            if let Err(e) = std::fs::create_dir_all(&hf_hub_cache) {
                log::warn!(
                    "ChatEngine: could not create HF hub cache dir {}: {}",
                    hf_hub_cache.display(),
                    e
                );
            }
            mistralrs_core::GLOBAL_HF_CACHE
                .get_or_init(|| hf_hub::Cache::new(hf_hub_cache.clone()));
            std::env::set_var("HF_HUB_CACHE", &hf_hub_cache);
            log::debug!(
                "ChatEngine: GLOBAL_HF_CACHE seeded at {}",
                hf_hub_cache.display()
            );
        }

        // Clean up stale HF cache artefacts before loading.
        crate::hf_cache::clean_stale_lock_files(&config.model_id);
        crate::hf_cache::repair_hf_cache_symlinks(&config.model_id);

        let start = Instant::now();

        let mut builder = GgufModelBuilder::new(&config.model_id, config.files.clone())
            .with_token_source(super::token::hf_token_source())
            .with_logging();

        // On Android the GGUF embedded tokenizer is not supported by the
        // candle backend — an explicit tok_model_id is required.
        if let Some(ref tok_id) = config.tok_model_id {
            builder = builder.with_tok_model_id(tok_id);
        }

        let model = builder
            .build()
            .await
            .map_err(|e| InferenceError::ModelBuild {
                reason: format!("Failed to build {} model: {}", config.display_name, e),
            })?;

        let elapsed = start.elapsed();

        let sampling = sampling.unwrap_or_else(|| {
            if cfg!(any(
                target_os = "ios",
                target_os = "tvos",
                target_os = "visionos",
                target_os = "watchos",
                target_os = "android"
            )) {
                SamplingConfig::mobile()
            } else {
                SamplingConfig::default()
            }
        });

        info!(
            "ChatEngine: model {} loaded in {} (sampling: temp={:?}, max_tokens={:?})",
            config.display_name,
            format_duration(elapsed),
            sampling.temperature,
            sampling.max_tokens,
        );

        // Capture before config is moved into LoadedModel.
        let pulse_model_id   = config.model_id.clone();
        let pulse_model_name = config.display_name.clone();

        let mut guard = self.inner.lock().await;
        *guard = Some(LoadedModel {
            model: Arc::new(model),
            config: LoadedModelConfig::Gguf(config),
            history: Vec::new(),
            system_prompt,
            sampling,
        });

        if let Some(ref pulse) = self.pulse {
            pulse.record_model_loaded(pulse_model_id, pulse_model_name, elapsed.as_millis() as u64);
        }

        Ok(elapsed)
    }

    /// Load an ISQ (in-situ quantised) model into the engine.
    ///
    /// Unlike [`load_gguf_model`], this downloads the full-precision safetensors
    /// from HuggingFace and quantises the weights in-situ on Metal (macOS only).
    /// The quantisation happens once at load time; subsequent inference uses the
    /// compressed weights.
    ///
    /// # macOS only
    ///
    /// ISQ with Metal requires the `metal` feature on mistral.rs and is therefore
    /// restricted to macOS at the Rust level.  Other platforms should continue to
    /// use [`load_gguf_model`] with pre-quantised GGUF files.
    ///
    /// # Arguments
    ///
    /// * `config`        — Which model to load and with how many ISQ bits.
    /// * `system_prompt` — Optional system prompt prepended to every request.
    /// * `sampling`      — Sampling parameters; `None` uses [`SamplingConfig::default`].
    ///
    /// # Errors
    ///
    /// Returns [`InferenceError::ModelBuild`] if the model fails to download
    /// or quantise.
    #[cfg(target_os = "macos")]
    pub async fn load_isq_model(
        &self,
        config: IsqModelConfig,
        system_prompt: Option<String>,
        sampling: Option<SamplingConfig>,
    ) -> Result<std::time::Duration, InferenceError> {
        use log::info;
        use std::time::Instant;

        info!(
            "ChatEngine: loading ISQ model {} (bits={})",
            config.model_id, config.isq_bits
        );

        // Clean up stale HF cache artefacts before loading.
        crate::hf_cache::clean_stale_lock_files(&config.model_id);
        crate::hf_cache::repair_hf_cache_symlinks(&config.model_id);

        let start = Instant::now();

        // Choose ISQ bit width.
        let isq_bits = match config.isq_bits {
            8 => IsqBits::Eight,
            _ => IsqBits::Four, // default to 4-bit
        };

        let model = TextModelBuilder::new(&config.model_id)
            .with_token_source(super::token::hf_token_source())
            .with_auto_isq(isq_bits)
            .with_logging()
            .build()
            .await
            .map_err(|e| InferenceError::ModelBuild {
                reason: format!("Failed to build ISQ model {}: {}", config.display_name, e),
            })?;

        let elapsed = start.elapsed();

        let sampling = sampling.unwrap_or_default();

        info!(
            "ChatEngine: ISQ model {} loaded in {} (sampling: temp={:?}, max_tokens={:?})",
            config.display_name,
            format_duration(elapsed),
            sampling.temperature,
            sampling.max_tokens,
        );

        let mut guard = self.inner.lock().await;
        *guard = Some(LoadedModel {
            model: Arc::new(model),
            config: LoadedModelConfig::Isq(config),
            history: Vec::new(),
            system_prompt,
            sampling,
        });

        Ok(elapsed)
    }

    /// Unload the current model, freeing all memory.
    ///
    /// Dropping the `Model` sends a `Terminate` message to the mistral.rs
    /// engine thread, which tears down the KV cache, activations, and model
    /// weights.
    ///
    /// Returns the display name of the model that was unloaded, or `None`
    /// if no model was loaded.
    pub async fn unload_model(&self) -> Option<String> {
        let mut guard = self.inner.lock().await;
        if let Some(loaded) = guard.take() {
            let name = loaded.config.display_name().to_string();
            log::info!("ChatEngine: unloading model {}", name);
            // `loaded` is dropped here → Model → Arc<MistralRs> (last ref)
            // → MistralRs::drop() → engine thread termination.
            Some(name)
        } else {
            log::debug!("ChatEngine: unload_model called but no model was loaded.");
            None
        }
    }

    /// Check whether a model is currently loaded.
    pub async fn is_loaded(&self) -> bool {
        self.inner.lock().await.is_some()
    }

    /// Get a snapshot of the engine's current state.
    pub async fn info(&self) -> EngineInfo {
        let guard = self.inner.lock().await;
        match guard.as_ref() {
            Some(loaded) => EngineInfo {
                status: EngineStatus::Ready,
                model_name: Some(loaded.config.display_name().to_string()),
                approx_memory: Some(loaded.config.approx_memory().to_string()),
                history_length: loaded.history.len() as u64,
            },
            None => EngineInfo {
                status: EngineStatus::Unloaded,
                model_name: None,
                approx_memory: None,
                history_length: 0u64,
            },
        }
    }

    // ── System prompt ────────────────────────────────────────────────────

    /// Set or replace the system prompt.
    ///
    /// The system prompt is prepended to every inference request (it is NOT
    /// stored in the conversation history).
    pub async fn set_system_prompt(&self, prompt: impl Into<String>) {
        if let Some(loaded) = self.inner.lock().await.as_mut() {
            loaded.system_prompt = Some(prompt.into());
        }
    }

    /// Clear the system prompt.
    pub async fn clear_system_prompt(&self) {
        if let Some(loaded) = self.inner.lock().await.as_mut() {
            loaded.system_prompt = None;
        }
    }

    // ── Sampling configuration ───────────────────────────────────────────

    /// Replace the sampling configuration.
    pub async fn set_sampling(&self, sampling: SamplingConfig) {
        if let Some(loaded) = self.inner.lock().await.as_mut() {
            loaded.sampling = sampling;
        }
    }

    // ── Conversation history ─────────────────────────────────────────────

    /// Get a clone of the full conversation history.
    pub async fn history(&self) -> Vec<ChatMessage> {
        let guard = self.inner.lock().await;
        match guard.as_ref() {
            Some(loaded) => loaded.history.clone(),
            None => Vec::new(),
        }
    }

    /// Clear the conversation history but keep the model loaded.
    ///
    /// Returns the number of turns that were removed.
    pub async fn clear_history(&self) -> usize {
        let mut guard = self.inner.lock().await;
        match guard.as_mut() {
            Some(loaded) => {
                let count = loaded.history.len();
                loaded.history.clear();
                log::info!("ChatEngine: cleared {} history turns.", count);
                count
            }
            None => 0,
        }
    }

    /// Append a message to history without running inference.
    ///
    /// Useful for restoring a saved conversation or injecting context.
    pub async fn push_history(&self, message: ChatMessage) {
        if let Some(loaded) = self.inner.lock().await.as_mut() {
            loaded.history.push(message);
        }
    }

    // ── Non-streaming inference ──────────────────────────────────────────

    /// Send a user message and receive a complete assistant reply.
    ///
    /// The user message and assistant reply are automatically appended to
    /// the conversation history on success.
    ///
    /// # Errors
    ///
    /// - [`InferenceError::NoModelLoaded`] if no model is loaded.
    /// - [`InferenceError::Inference`] if the model fails to generate.
    pub async fn send_message(
        &self,
        user_message: impl Into<String>,
    ) -> Result<InferenceResult, InferenceError> {
        let user_message = user_message.into();

        // ── 1. Snapshot model handle + build request, then release lock ──
        let (model, request, pulse_model_id) = {
            let guard = self.inner.lock().await;
            let loaded = guard.as_ref().ok_or(InferenceError::NoModelLoaded)?;
            let request = self::build_request(loaded, &user_message);
            let pulse_model_id = match &loaded.config {
                LoadedModelConfig::Gguf(c) => c.model_id.clone(),
                #[cfg(target_os = "macos")]
                LoadedModelConfig::Isq(c) => c.model_id.clone(),
            };
            (loaded.model.clone(), request, pulse_model_id)
        }; // ← mutex released before inference

        log::info!(
            "ChatEngine: inference START — message: \"{}\"",
            truncate_for_log(&user_message, 100)
        );

        // ── 2. Run inference (potentially slow — mutex is NOT held) ──────
        let start = std::time::Instant::now();
        let response =
            model
                .send_chat_request(request)
                .await
                .map_err(|e| InferenceError::Inference {
                    reason: e.to_string(),
                })?;
        let elapsed = start.elapsed();

        let reply = response.choices[0]
            .message
            .content
            .as_ref()
            .map(|c| c.trim().to_string())
            .unwrap_or_else(|| "(empty response)".to_string());

        let finish_reason = response.choices[0].finish_reason.clone();

        log::info!(
            "ChatEngine: inference END — {} — reply: \"{}\"",
            format_duration(elapsed),
            truncate_for_log(&reply, 100)
        );

        // ── 3. Persist turns to history (brief lock re-acquisition) ──────
        {
            let mut guard = self.inner.lock().await;
            if let Some(loaded) = guard.as_mut() {
                loaded.history.push(ChatMessage::user(user_message));
                loaded.history.push(ChatMessage::assistant(reply.clone()));
            }
        }

        if let Some(ref pulse) = self.pulse {
            pulse.record_inference(
                pulse_model_id,
                crate::pulse::next_request_id(),
                elapsed.as_millis() as u64,
                "success".to_string(),
            );
        }

        Ok(InferenceResult {
            text: reply,
            duration_secs: elapsed.as_secs_f64(),
            duration_display: format_duration(elapsed),
            finish_reason,
        })
    }

    /// Run inference on an explicit list of messages WITHOUT modifying the
    /// engine's internal history.
    ///
    /// This is useful for one-shot prompts (e.g. prompt enhancement) where
    /// you don't want side-effects on the main conversation.
    ///
    /// The system prompt set on the engine is still prepended.
    pub async fn generate(
        &self,
        messages: Vec<ChatMessage>,
        sampling: Option<SamplingConfig>,
    ) -> Result<InferenceResult, InferenceError> {
        let (model, request) = {
            let guard = self.inner.lock().await;
            let loaded = guard.as_ref().ok_or(InferenceError::NoModelLoaded)?;

            let sampling = sampling.as_ref().unwrap_or(&loaded.sampling);
            let mut req = RequestBuilder::new();

            // Apply sampling parameters.
            req = apply_sampling(req, sampling);

            // System prompt.
            if let Some(ref sp) = loaded.system_prompt {
                req = req.add_message(TextMessageRole::System, sp);
            }

            // Provided messages.
            for msg in &messages {
                req = req.add_message(chat_role_to_mistral(&msg.role), &msg.content);
            }

            (loaded.model.clone(), req)
        };

        let start = std::time::Instant::now();
        let response =
            model
                .send_chat_request(request)
                .await
                .map_err(|e| InferenceError::Inference {
                    reason: e.to_string(),
                })?;
        let elapsed = start.elapsed();

        let reply = response.choices[0]
            .message
            .content
            .as_ref()
            .map(|c| c.trim().to_string())
            .unwrap_or_else(|| "(empty response)".to_string());

        let finish_reason = response.choices[0].finish_reason.clone();

        Ok(InferenceResult {
            text: reply,
            duration_secs: elapsed.as_secs_f64(),
            duration_display: format_duration(elapsed),
            finish_reason,
        })
    }

    // ── Streaming inference ──────────────────────────────────────────────

    /// Send a user message and receive an async stream of token chunks.
    ///
    /// The user message and full assembled reply are automatically appended
    /// to the conversation history once the stream finishes.
    ///
    /// # Returns
    ///
    /// A `tokio::sync::mpsc::Receiver<StreamChunk>`.  The caller should
    /// receive from it until `chunk.done == true` or the channel closes.
    ///
    /// # Errors
    ///
    /// Returns immediately with [`InferenceError::NoModelLoaded`] if no
    /// model is loaded.  Inference errors are delivered as the final chunk
    /// with `done = true` and an empty delta.
    pub async fn stream_message(
        &self,
        user_message: impl Into<String>,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamChunk>, InferenceError> {
        let user_message = user_message.into();

        let (model, request) = {
            let guard = self.inner.lock().await;
            let loaded = guard.as_ref().ok_or(InferenceError::NoModelLoaded)?;
            let request = self::build_request(loaded, &user_message);
            (loaded.model.clone(), request)
        };

        let (tx, rx) = tokio::sync::mpsc::channel::<StreamChunk>(64);

        // We need a reference to `self` inside the spawned task so we can
        // update history when the stream completes.  Since ChatEngine is
        // behind a shared reference (callers hold &self or Arc<ChatEngine>),
        // we borrow the inner Mutex via a raw pointer.  This is safe because
        // ChatEngine's lifetime exceeds the spawned task (it's typically in
        // a Lazy static or shared application state).
        //
        // Alternative: require ChatEngine to always be wrapped in Arc and
        // accept `self: Arc<Self>`.  We avoid that to keep the API simple.
        let inner_ptr = &self.inner as *const Mutex<Option<LoadedModel>>;
        // SAFETY: ChatEngine is stored in a Lazy static or equivalent with
        // 'static lifetime.  The spawned task cannot outlive the engine.
        let inner_ref: &'static Mutex<Option<LoadedModel>> = unsafe { &*inner_ptr };

        let user_msg_clone = user_message.clone();

        tokio::task::spawn(async move {
            // `model` is an `Arc<Model>` moved into this task, so it's owned.
            // `stream_chat_request` borrows `&Model` — the borrow is scoped
            // to this async block and does NOT need to be `'static`.
            let stream_result = model.stream_chat_request(request).await;

            match stream_result {
                Ok(mut stream) => {
                    let mut assembled = String::new();
                    let mut last_finish_reason = None;

                    // `Stream::next()` is an inherent async method on
                    // mistralrs::Stream that returns `Option<Response>`.
                    // No `futures::StreamExt` import needed.
                    while let Some(response) = stream.next().await {
                        match response {
                            mistralrs::Response::Chunk(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(ref text) = choice.delta.content {
                                        assembled.push_str(text);
                                        let _ = tx
                                            .send(StreamChunk {
                                                delta: text.clone(),
                                                done: false,
                                                finish_reason: None,
                                            })
                                            .await;
                                    }
                                    if let Some(ref reason) = choice.finish_reason {
                                        last_finish_reason = Some(reason.clone());
                                    }
                                }
                            }
                            mistralrs::Response::Done(_) => {
                                // Non-streaming response arrived on a streaming
                                // channel — should not happen, but handle gracefully.
                                break;
                            }
                            mistralrs::Response::InternalError(e) => {
                                log::error!("ChatEngine stream internal error: {}", e);
                                break;
                            }
                            mistralrs::Response::ValidationError(e) => {
                                log::error!("ChatEngine stream validation error: {}", e);
                                break;
                            }
                            mistralrs::Response::ModelError(msg, _) => {
                                log::error!("ChatEngine stream model error: {}", msg);
                                break;
                            }
                            _ => {
                                // Completion / Image / Speech variants — not expected
                                // for chat streaming.
                                break;
                            }
                        }
                    }

                    // Persist turns to history.
                    {
                        let mut guard = inner_ref.lock().await;
                        if let Some(loaded) = guard.as_mut() {
                            loaded.history.push(ChatMessage::user(user_msg_clone));
                            loaded
                                .history
                                .push(ChatMessage::assistant(assembled.trim()));
                        }
                    }

                    // Send the final "done" chunk.
                    let _ = tx
                        .send(StreamChunk {
                            delta: String::new(),
                            done: true,
                            finish_reason: last_finish_reason,
                        })
                        .await;
                }
                Err(e) => {
                    log::error!("ChatEngine: stream_chat_request failed: {}", e);
                    let _ = tx
                        .send(StreamChunk {
                            delta: String::new(),
                            done: true,
                            finish_reason: Some(format!("error: {}", e)),
                        })
                        .await;
                }
            }
        });

        Ok(rx)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Default impl
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
impl Default for ChatEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Internal helpers (platform-gated)
// ═════════════════════════════════════════════════════════════════════════════

/// Build a `RequestBuilder` from the current engine state and a new user message.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
fn build_request(loaded: &LoadedModel, user_message: &str) -> RequestBuilder {
    let mut req = RequestBuilder::new();

    // Apply sampling.
    req = apply_sampling(req, &loaded.sampling);

    // System prompt.
    if let Some(ref sp) = loaded.system_prompt {
        req = req.add_message(TextMessageRole::System, sp);
    }

    // Replay conversation history for multi-turn context.
    for turn in &loaded.history {
        req = req.add_message(chat_role_to_mistral(&turn.role), &turn.content);
    }

    // Append the current user message.
    req = req.add_message(TextMessageRole::User, user_message);

    req
}

/// Apply a [`SamplingConfig`] to a [`RequestBuilder`].
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
fn apply_sampling(mut req: RequestBuilder, sampling: &SamplingConfig) -> RequestBuilder {
    if let Some(temp) = sampling.temperature {
        req = req.set_sampler_temperature(temp);
    }
    if let Some(top_p) = sampling.top_p {
        req = req.set_sampler_topp(top_p);
    }
    if let Some(top_k) = sampling.top_k {
        req = req.set_sampler_topk(top_k as usize);
    }
    if let Some(min_p) = sampling.min_p {
        req = req.set_sampler_minp(min_p);
    }
    if let Some(max_tokens) = sampling.max_tokens {
        req = req.set_sampler_max_len(max_tokens as usize);
    }
    if let Some(freq) = sampling.frequency_penalty {
        req = req.set_sampler_frequency_penalty(freq);
    }
    if let Some(pres) = sampling.presence_penalty {
        req = req.set_sampler_presence_penalty(pres);
    }
    req
}

/// Convert [`ChatRole`] → [`TextMessageRole`].
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
fn chat_role_to_mistral(role: &ChatRole) -> TextMessageRole {
    match role {
        ChatRole::System => TextMessageRole::System,
        ChatRole::User => TextMessageRole::User,
        ChatRole::Assistant => TextMessageRole::Assistant,
    }
}

/// Truncate a string for log output, appending `"..."` if truncated.
fn truncate_for_log(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Unsupported-platform stub
// ═════════════════════════════════════════════════════════════════════════════

/// Stub `ChatEngine` for platforms where mistral.rs is not available.
///
/// All methods return errors or empty values so that code using the engine
/// can compile on any target without `#[cfg]` at every call site.
#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
)))]
pub struct ChatEngine;

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
)))]
impl ChatEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_gguf_model(
        &self,
        _config: GgufModelConfig,
        _system_prompt: Option<String>,
        _sampling: Option<SamplingConfig>,
    ) -> Result<std::time::Duration, InferenceError> {
        Err(InferenceError::Other {
            reason: "LLM inference is not supported on this platform.".into(),
        })
    }

    pub async fn unload_model(&self) -> Option<String> {
        None
    }

    pub async fn is_loaded(&self) -> bool {
        false
    }

    pub async fn info(&self) -> EngineInfo {
        EngineInfo {
            status: EngineStatus::Unloaded,
            model_name: None,
            approx_memory: None,
            history_length: 0u64,
        }
    }

    pub async fn set_system_prompt(&self, _prompt: impl Into<String>) {}

    pub async fn clear_system_prompt(&self) {}

    pub async fn set_sampling(&self, _sampling: SamplingConfig) {}

    pub async fn history(&self) -> Vec<ChatMessage> {
        Vec::new()
    }

    pub async fn clear_history(&self) -> usize {
        0
    }

    pub async fn push_history(&self, _message: ChatMessage) {}

    pub async fn send_message(
        &self,
        _user_message: impl Into<String>,
    ) -> Result<InferenceResult, InferenceError> {
        Err(InferenceError::Other {
            reason: "LLM inference is not supported on this platform.".into(),
        })
    }

    pub async fn generate(
        &self,
        _messages: Vec<ChatMessage>,
        _sampling: Option<SamplingConfig>,
    ) -> Result<InferenceResult, InferenceError> {
        Err(InferenceError::Other {
            reason: "LLM inference is not supported on this platform.".into(),
        })
    }

    pub async fn stream_message(
        &self,
        _user_message: impl Into<String>,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamChunk>, InferenceError> {
        Err(InferenceError::Other {
            reason: "LLM inference is not supported on this platform.".into(),
        })
    }
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
)))]
impl Default for ChatEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Prebuilt model configs (convenience constructors)
// ═════════════════════════════════════════════════════════════════════════════

/// Convenience constructors for common GGUF model configurations.
///
/// These mirror the constants in [`super::models`] but return a ready-to-use
/// [`GgufModelConfig`].
impl GgufModelConfig {
    /// Qwen 2.5 1.5B Instruct (GGUF Q4_K_M) — ~941 MB.
    ///
    /// Lightest option, ideal for iOS and memory-constrained Android devices.
    pub fn qwen25_1_5b() -> Self {
        Self {
            model_id: super::models::BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF.into(),
            files: vec![super::models::QWEN25_1_5B_GGUF_FILE.into()],
            tok_model_id: if cfg!(target_os = "android") {
                Some(super::models::QWEN25_1_5B_TOK_MODEL_ID.into())
            } else {
                None
            },
            display_name: "Qwen 2.5 1.5B".into(),
            approx_memory: "~941 MB (GGUF Q4_K_M)".into(),
        }
    }

    /// Qwen 2.5 3B Instruct (GGUF Q4_K_M) — ~1.93 GB.
    ///
    /// Balanced quality/size for macOS desktops and high-RAM Android devices.
    /// Not recommended for iOS (OOM on 8 GB devices).
    pub fn qwen25_3b() -> Self {
        Self {
            model_id: super::models::BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF.into(),
            files: vec![super::models::QWEN25_3B_GGUF_FILE.into()],
            tok_model_id: if cfg!(target_os = "android") {
                Some(super::models::QWEN25_3B_TOK_MODEL_ID.into())
            } else {
                None
            },
            display_name: "Qwen 2.5 3B".into(),
            approx_memory: "~1.93 GB (GGUF Q4_K_M)".into(),
        }
    }

    /// Qwen 2.5 Coder 1.5B Instruct (GGUF Q4_K_M) — ~941 MB.
    ///
    /// Dedicated coding model fine-tuned on 5.5T tokens of code and math data.
    /// Same `qwen2` GGUF architecture as the general 1.5B — loads via the
    /// existing `quantized_qwen.rs` path without any mistral.rs changes.
    /// Same memory footprint as the general 1.5B but dramatically better at
    /// code generation, explanation, and debugging.
    ///
    /// Preferred default for iOS, Android, and any coding-focused deployment.
    pub fn qwen25_coder_1_5b() -> Self {
        Self {
            model_id: super::models::BARTOWSKI_QWEN25_CODER_1_5B_INSTRUCT_GGUF.into(),
            files: vec![super::models::QWEN25_CODER_1_5B_GGUF_FILE.into()],
            tok_model_id: if cfg!(target_os = "android") {
                Some(super::models::QWEN25_CODER_1_5B_TOK_MODEL_ID.into())
            } else {
                None
            },
            display_name: "Qwen 2.5 Coder 1.5B".into(),
            approx_memory: "~941 MB (GGUF Q4_K_M)".into(),
        }
    }

    /// Qwen 2.5 Coder 3B Instruct (GGUF Q4_K_M) — ~1.93 GB.
    ///
    /// Best on-device coding quality for macOS desktop. Fine-tuned on 5.5T
    /// tokens of code and math. Same `qwen2` architecture as the general 3B.
    /// Not recommended for iOS due to memory constraints.
    pub fn qwen25_coder_3b() -> Self {
        Self {
            model_id: super::models::BARTOWSKI_QWEN25_CODER_3B_INSTRUCT_GGUF.into(),
            files: vec![super::models::QWEN25_CODER_3B_GGUF_FILE.into()],
            tok_model_id: if cfg!(target_os = "android") {
                Some(super::models::QWEN25_CODER_3B_TOK_MODEL_ID.into())
            } else {
                None
            },
            display_name: "Qwen 2.5 Coder 3B".into(),
            approx_memory: "~1.93 GB (GGUF Q4_K_M)".into(),
        }
    }

    /// Qwen 3 4B Instruct (GGUF Q4_K_M) — ~2.7 GB.
    ///
    /// Full OpenAI-compatible tool calling with extended thinking mode.
    /// Always load with `max_tokens ≥ 4096`; the `<think>…</think>` block can
    /// consume 300–400 tokens before the real response begins.
    pub fn qwen3_4b() -> Self {
        Self {
            model_id: super::models::BARTOWSKI_QWEN3_4B_GGUF.into(),
            files: vec![super::models::QWEN3_4B_GGUF_FILE.into()],
            tok_model_id: None,
            display_name: "Qwen 3 4B (Q4_K_M)".into(),
            approx_memory: "~2.7 GB".into(),
        }
    }

    /// Return the platform-appropriate default **coding** model config.
    ///
    /// - iOS / Android → Qwen 2.5 Coder 1.5B (~941 MB, fits mobile memory budgets)
    /// - macOS / Windows / Linux → Qwen 2.5 Coder 3B (~1.93 GB, best desktop quality)
    ///
    /// Both are dedicated coding models (trained on 5.5T code + math tokens)
    /// and use the `qwen2` GGUF architecture supported by mistral.rs.
    pub fn platform_default() -> Self {
        if cfg!(any(
            target_os = "ios",
            target_os = "tvos",
            target_os = "visionos",
            target_os = "watchos",
            target_os = "android"
        )) {
            Self::qwen25_coder_1_5b()
        } else {
            Self::qwen25_coder_3b()
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_for_log_short() {
        assert_eq!(truncate_for_log("hello", 10), "hello");
    }

    #[test]
    fn truncate_for_log_long() {
        let long = "a".repeat(200);
        let result = truncate_for_log(&long, 50);
        assert!(result.ends_with("..."));
        assert_eq!(result.len(), 53); // 50 chars + "..."
    }

    #[test]
    fn gguf_model_config_qwen25_1_5b() {
        let cfg = GgufModelConfig::qwen25_1_5b();
        assert!(cfg.model_id.contains("1.5B"));
        assert_eq!(cfg.files.len(), 1);
    }

    #[test]
    fn gguf_model_config_qwen25_3b() {
        let cfg = GgufModelConfig::qwen25_3b();
        assert!(cfg.model_id.contains("3B"));
        assert_eq!(cfg.files.len(), 1);
    }

    #[test]
    fn gguf_model_config_platform_default() {
        let cfg = GgufModelConfig::platform_default();
        // On the test host (macOS or Linux), this should be the 3B model.
        // On iOS/Android CI it would be 1.5B.  We just check it's valid.
        assert!(!cfg.model_id.is_empty());
        assert!(!cfg.files.is_empty());
    }

    #[tokio::test]
    async fn engine_new_is_unloaded() {
        let engine = ChatEngine::new();
        assert!(!engine.is_loaded().await);
        let info = engine.info().await;
        assert_eq!(info.status, EngineStatus::Unloaded);
        assert_eq!(info.history_length, 0);
    }

    #[tokio::test]
    async fn engine_send_without_model_errors() {
        let engine = ChatEngine::new();
        let result = engine.send_message("hello").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            InferenceError::NoModelLoaded => {} // expected
            other => panic!("Expected NoModelLoaded, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_history_empty_when_no_model() {
        let engine = ChatEngine::new();
        assert!(engine.history().await.is_empty());
        assert_eq!(engine.clear_history().await, 0);
    }

    #[tokio::test]
    async fn engine_unload_when_none() {
        let engine = ChatEngine::new();
        assert!(engine.unload_model().await.is_none());
    }
}
