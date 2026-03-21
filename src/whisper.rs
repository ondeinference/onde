//! Whisper speech-to-text transcription via `transcribe-rs`.
//!
//! This module re-exports the Whisper engine and associated types from
//! `transcribe-rs` so that callers can depend on `onde` alone for all
//! on-device inference — both LLM/diffusion (via mistral.rs) and
//! speech-to-text (via whisper.cpp / whisper-rs).
//!
//! It also provides [`find_or_download_whisper_model`] which resolves a
//! local Whisper GGML model file — downloading the platform-appropriate
//! default from HuggingFace (`ggerganov/whisper.cpp`) if none is found.
//! The download uses `hf-hub` so it shares the standard HuggingFace cache
//! directory with all other model downloads in the app.
//!
//! # Feature gate
//!
//! This module is only compiled when the `whisper` Cargo feature is enabled:
//!
//! ```toml
//! [dependencies]
//! onde = { path = "…", features = ["whisper"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use onde::whisper::{WhisperEngine, WhisperInferenceParams};
//! use onde::whisper::{TranscriptionEngine, TranscriptionSegment};
//!
//! let mut engine = WhisperEngine::new();
//! engine.load_model(&model_path)?;
//!
//! let params = WhisperInferenceParams {
//!     language: Some("en".to_string()),
//!     ..Default::default()
//! };
//!
//! let result = engine.transcribe_samples(samples, Some(params))?;
//! ```
//!
//! # Auto-download example
//!
//! ```rust,ignore
//! use onde::whisper::{find_or_download_whisper_model, WhisperModelDownloadProgress};
//! use std::path::PathBuf;
//!
//! let model_dir = PathBuf::from("/path/to/models/whisper");
//! let model_path = find_or_download_whisper_model(
//!     &model_dir,
//!     None, // use platform default model
//!     |progress| {
//!         println!(
//!             "Downloading {} — {}/{}",
//!             progress.model_name, progress.downloaded_display, progress.total_display
//!         );
//!     },
//! )?;
//! ```

use {
    log::{debug, info, warn},
    serde::{Deserialize, Serialize},
    std::path::{Path, PathBuf},
    std::sync::Arc,
    tsync::tsync,
};

// ---------------------------------------------------------------------------
// Re-exports from transcribe-rs
// ---------------------------------------------------------------------------

// Re-export the Whisper engine and its parameter types.
pub use transcribe_rs::whisper_cpp::{WhisperEngine, WhisperInferenceParams, WhisperLoadParams};

// Re-export the core transcription trait and result types so callers
// don't need a separate `use transcribe_rs::…` import.
pub use transcribe_rs::{SpeechModel, TranscribeError, TranscriptionResult, TranscriptionSegment};

// Re-export the audio utility for reading raw WAV samples.
pub use transcribe_rs::audio::read_wav_samples;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// HuggingFace repository that hosts pre-converted Whisper GGML models.
const HF_WHISPER_REPO: &str = "ggerganov/whisper.cpp";

/// Whisper model file names, in order of preference (best quality first).
/// [`find_or_download_whisper_model`] tries each in sequence and uses the
/// first one already present on disk before falling back to downloading.
pub const MODEL_CANDIDATES: &[&str] = &[
    "ggml-large-v3-turbo.bin",
    "ggml-large-v3.bin",
    "ggml-medium.bin",
    "ggml-medium.en.bin",
    "ggml-medium-q5_0.bin",
    "ggml-small.bin",
    "ggml-small.en.bin",
    "ggml-base.bin",
    "ggml-base.en.bin",
    "ggml-tiny.bin",
    "ggml-tiny.en.bin",
];

// ===========================================================================
// UniFFI — Error type
// ===========================================================================

/// Errors returned by the UniFFI-exported whisper API.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum WhisperError {
    #[error("Model not loaded — call load_model() first")]
    ModelNotLoaded,
    #[error("Model load failed: {reason}")]
    ModelLoadFailed { reason: String },
    #[error("Transcription failed: {reason}")]
    TranscriptionFailed { reason: String },
    #[error("Download failed: {reason}")]
    DownloadFailed { reason: String },
    #[error("IO error: {reason}")]
    Io { reason: String },
}

// ===========================================================================
// UniFFI — Record types (plain data structs passed across the FFI boundary)
// ===========================================================================

/// Progress payload emitted during a Whisper model download.
///
/// Callers of [`find_or_download_whisper_model`] receive this through
/// the progress callback and can forward it to the frontend without any
/// framework coupling in this crate.
///
/// Swift/Kotlin consumers receive this as a value type (struct / data class).
#[tsync]
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct WhisperModelDownloadProgress {
    /// The GGML model file being downloaded, e.g. `"ggml-base.bin"`.
    pub model_name: String,
    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,
    /// Human-readable downloaded size, e.g. `"45.2 MB"`.
    pub downloaded_display: String,
    /// Expected total size in bytes (0 if unknown).
    pub total_bytes: u64,
    /// Human-readable total size, e.g. `"141.1 MB"`.
    pub total_display: String,
    /// Download progress as a value between 0.0 and 1.0.
    pub progress: f64,
    /// Whether the download has completed.
    pub done: bool,
}

/// A single transcription segment with timing information.
///
/// Mirrors `transcribe_rs::TranscriptionSegment` with UniFFI-compatible
/// owned types so it can cross the FFI boundary.
#[derive(Debug, Clone, uniffi::Record)]
pub struct WhisperSegment {
    /// Start time in seconds.
    pub start_secs: f32,
    /// End time in seconds.
    pub end_secs: f32,
    /// The transcribed text for this segment.
    pub text: String,
}

/// Result of a Whisper transcription.
///
/// Mirrors `transcribe_rs::TranscriptionResult` with UniFFI-compatible
/// owned types.  `segments` is always a `Vec` (empty when the engine
/// returns `None`).
#[derive(Debug, Clone, uniffi::Record)]
pub struct WhisperResult {
    /// The full transcribed text.
    pub text: String,
    /// Individual timed segments (empty if the engine did not produce any).
    pub segments: Vec<WhisperSegment>,
}

// ===========================================================================
// UniFFI — Callback interface (Swift/Kotlin implements this to receive progress)
// ===========================================================================

/// Callback interface that Swift/Kotlin implements to receive download
/// progress updates.
///
/// ```swift
/// class MyProgressListener: WhisperProgressListener {
///     func onProgress(progress: WhisperModelDownloadProgress) {
///         print("Downloaded: \(progress.downloadedDisplay)")
///     }
/// }
/// ```
#[uniffi::export(callback_interface)]
pub trait WhisperProgressListener: Send + Sync {
    /// Called periodically during a model download with the current progress.
    fn on_progress(&self, progress: WhisperModelDownloadProgress);
}

// ===========================================================================
// UniFFI — Object wrapper around the foreign `WhisperEngine`
// ===========================================================================

/// A speech-to-text engine backed by Whisper (via whisper.cpp).
///
/// This is a UniFFI-compatible wrapper around `transcribe_rs::WhisperEngine`.
/// Construct with [`OndeWhisperEngine::new`], load a model with
/// [`load_model`], then transcribe audio with [`transcribe_file`] or
/// [`transcribe_samples`].
///
/// ```swift
/// let engine = OndeWhisperEngine()
/// try engine.loadModel(path: "/path/to/ggml-base.bin")
/// let result = try engine.transcribeFile(path: "/path/to/audio.wav", language: "en")
/// print(result.text)
/// ```
#[derive(uniffi::Object)]
pub struct OndeWhisperEngine {
    inner: std::sync::Mutex<Option<WhisperEngine>>,
}

#[uniffi::export]
impl OndeWhisperEngine {
    /// Create a new, unloaded Whisper engine.
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        // The inner Option<WhisperEngine> starts as None; a model must be
        // loaded via load_model() before transcription.
        Arc::new(Self {
            inner: std::sync::Mutex::new(None),
        })
    }

    /// Load a Whisper GGML model file.
    ///
    /// `path` is the absolute filesystem path to the `.bin` model file.
    /// This must be called before any transcription method.
    pub fn load_model(&self, path: String) -> Result<(), WhisperError> {
        let model_path = PathBuf::from(&path);
        let engine =
            WhisperEngine::load(&model_path).map_err(|e| WhisperError::ModelLoadFailed {
                reason: e.to_string(),
            })?;
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| WhisperError::ModelLoadFailed {
                reason: format!("Lock poisoned: {e}"),
            })?;
        *guard = Some(engine);
        Ok(())
    }

    /// Transcribe an audio file on disk.
    ///
    /// `path` is the absolute filesystem path to a WAV file (16-bit PCM,
    /// 16 kHz mono recommended).
    ///
    /// `language` is an optional BCP-47 language code (e.g. `"en"`, `"id"`).
    /// Pass `None` / `nil` for auto-detection.
    pub fn transcribe_file(
        &self,
        path: String,
        language: Option<String>,
    ) -> Result<WhisperResult, WhisperError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| WhisperError::TranscriptionFailed {
                reason: format!("Lock poisoned: {e}"),
            })?;

        let engine = guard.as_mut().ok_or(WhisperError::ModelNotLoaded)?;

        let params = WhisperInferenceParams {
            language,
            ..Default::default()
        };

        let file_path = Path::new(&path);
        let samples = transcribe_rs::audio::read_wav_samples(file_path).map_err(
            |e: transcribe_rs::TranscribeError| WhisperError::TranscriptionFailed {
                reason: e.to_string(),
            },
        )?;
        let result = engine.transcribe_with(&samples, &params).map_err(
            |e: transcribe_rs::TranscribeError| WhisperError::TranscriptionFailed {
                reason: e.to_string(),
            },
        )?;

        Ok(transcription_result_to_ffi(result))
    }

    /// Transcribe raw 16-bit PCM audio samples (16 kHz, mono, f32).
    ///
    /// `language` is an optional BCP-47 language code (e.g. `"en"`, `"id"`).
    /// Pass `None` / `nil` for auto-detection.
    pub fn transcribe_samples(
        &self,
        samples: Vec<f32>,
        language: Option<String>,
    ) -> Result<WhisperResult, WhisperError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| WhisperError::TranscriptionFailed {
                reason: format!("Lock poisoned: {e}"),
            })?;

        let engine = guard.as_mut().ok_or(WhisperError::ModelNotLoaded)?;

        let params = WhisperInferenceParams {
            language,
            ..Default::default()
        };

        let result = engine.transcribe_with(&samples, &params).map_err(
            |e: transcribe_rs::TranscribeError| WhisperError::TranscriptionFailed {
                reason: e.to_string(),
            },
        )?;

        Ok(transcription_result_to_ffi(result))
    }
}

// ===========================================================================
// UniFFI — Exported free functions
// ===========================================================================

/// Returns the default Whisper model filename for the current platform.
///
/// - **Android (CPU-only)**: `ggml-base.bin` (~142 MB) — best quality/speed
///   trade-off for mobile ARM CPUs.
/// - **macOS / iOS (Metal GPU)**: `ggml-large-v3-turbo.bin` (~1.6 GB) —
///   high quality, fast with GPU acceleration.
/// - **Other platforms**: `ggml-small.bin` (~466 MB) — reasonable default.
#[uniffi::export]
pub fn default_model_for_platform() -> String {
    if cfg!(target_os = "android") {
        "ggml-base.bin".to_string()
    } else if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        "ggml-large-v3-turbo.bin".to_string()
    } else {
        "ggml-small.bin".to_string()
    }
}

/// Returns the list of Whisper model candidates in order of preference
/// (best quality first).
#[uniffi::export]
pub fn whisper_model_candidates() -> Vec<String> {
    MODEL_CANDIDATES.iter().map(|s| s.to_string()).collect()
}

/// Finds a locally cached Whisper GGML model or downloads one via the
/// HuggingFace hub.
///
/// # Arguments
///
/// - `model_dir` — Legacy local directory to search for manually-placed
///   models.  Not used for downloaded models.
/// - `model_name` — Override the model filename to download.  Pass `nil`
///   to use the platform default.
/// - `listener` — Callback invoked periodically during the download.
///   Not called at all if a model is already cached.
/// - `app_data_dir` — **Always pass the app's data directory.**
///   The default `~/.cache` path is inaccessible on sandboxed platforms.
///   Pass `nil` only in CLI / test contexts.
///
/// Returns the absolute path to the resolved model file.
#[uniffi::export]
pub fn find_or_download_whisper_model_ffi(
    model_dir: String,
    model_name: Option<String>,
    listener: Box<dyn WhisperProgressListener>,
    app_data_dir: Option<String>,
) -> Result<String, WhisperError> {
    let model_dir_path = PathBuf::from(&model_dir);
    let app_data = app_data_dir.as_deref().map(Path::new);
    let name_ref = model_name.as_deref();

    let path = find_or_download_whisper_model(
        &model_dir_path,
        name_ref,
        move |progress| {
            listener.on_progress(progress);
        },
        app_data,
    )
    .map_err(|reason| WhisperError::DownloadFailed { reason })?;

    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| WhisperError::Io {
            reason: "Model path contains non-UTF-8 characters".to_string(),
        })
}

/// Read raw f32 samples from a WAV file (utility for feeding into
/// `OndeWhisperEngine.transcribe_samples`).
///
/// Returns 16 kHz mono f32 samples suitable for Whisper.
#[uniffi::export]
pub fn read_wav_samples_ffi(path: String) -> Result<Vec<f32>, WhisperError> {
    read_wav_samples(Path::new(&path)).map_err(|e| WhisperError::Io {
        reason: e.to_string(),
    })
}

// ===========================================================================
// Internal — callback-based API
// ===========================================================================

/// Resolve the `hf-hub` [`Cache`] instance for the current platform.
///
/// Callers should **always** provide `app_data_dir` (e.g.
/// `app.path().app_data_dir()`).  The default `~/.cache/huggingface/hub`
/// is inaccessible on every sandboxed platform:
///
/// - **Android** — `dirs::home_dir()` panics inside the sandbox.
/// - **iOS** — `~/.cache` is outside the container (`os error 1`).
/// - **macOS App Store** — the app is sandboxed; `~/.cache` is
///   inaccessible just like iOS.
///
/// Passing `app_data_dir` on non-sandboxed desktop builds (Linux,
/// Windows, side-loaded macOS) is harmless — the cache simply lives
/// under the app data directory instead of `~/.cache`.
///
/// `None` is accepted as a last-resort fallback for CLI / test contexts;
/// it uses `Cache::default()`.
fn resolve_hf_cache(app_data_dir: Option<&Path>) -> Result<hf_hub::Cache, String> {
    if let Some(data_dir) = app_data_dir {
        let hf_home = data_dir.join("huggingface");
        let hf_cache = hf_home.join("hub");
        std::fs::create_dir_all(&hf_cache)
            .map_err(|e| format!("Cannot create HF cache dir: {e}"))?;

        // Keep env vars in sync so hf_cache_dir() agrees.
        std::env::set_var("HF_HUB_CACHE", &hf_cache);
        std::env::set_var("HF_HOME", &hf_home);

        info!("HF cache resolved to app data path: {}", hf_cache.display());
        Ok(hf_hub::Cache::new(hf_cache))
    } else {
        warn!(
            "No app_data_dir provided — using Cache::default(). \
             This will fail on sandboxed platforms (iOS, Android, macOS App Store)."
        );
        Ok(hf_hub::Cache::default())
    }
}

/// Try to resolve a Whisper model file from the HF cache without
/// downloading.  Returns `Some(path)` if the file is already fully
/// cached, `None` otherwise.
///
/// This mirrors the mistral.rs strategy: the HF cache *is* the model
/// store — no separate `model_dir` copy is needed.
fn resolve_from_hf_cache(model_name: &str, cache: &hf_hub::Cache) -> Option<PathBuf> {
    let repo = cache.repo(hf_hub::Repo::model(HF_WHISPER_REPO.to_string()));
    // `repo.get(name)` returns `Some(path)` when the blob is fully
    // downloaded and the pointer symlink exists.
    let path = repo.get(model_name);
    if let Some(ref p) = path {
        // Double-check: the path must exist and be a real file (following
        // symlinks) with a non-zero size.
        match std::fs::metadata(p) {
            Ok(meta) if meta.len() > 0 => {
                debug!(
                    "HF cache hit for {}: {:?} ({} bytes)",
                    model_name,
                    p,
                    meta.len()
                );
            }
            _ => {
                debug!(
                    "HF cache entry for {} exists but is empty or broken",
                    model_name
                );
                return None;
            }
        }
    }
    path
}

/// Finds a locally cached Whisper GGML model or downloads one via the
/// HuggingFace hub.
///
/// # Resolution strategy (mirrors mistral.rs)
///
/// The HuggingFace hub cache is the **primary model store**.  Models are
/// downloaded into the cache's content-addressed blob directory and
/// accessed via symlinks — no copy or hard-link into a separate
/// `model_dir` is performed.  This avoids "Operation not permitted"
/// errors on sandboxed platforms (iOS, Android) and eliminates double
/// disk usage.
///
/// Search order:
///
/// 1. Check the HF cache for an already-downloaded model (preferred
///    candidate list, best quality first).
/// 2. Check `model_dir` for manually-placed `.bin` files (legacy
///    fallback).
/// 3. Download the platform default (or explicitly requested
///    `model_name`) from `ggerganov/whisper.cpp` on HuggingFace.
///
/// # Arguments
///
/// - `model_dir` — Legacy local directory to search for manually-placed
///   models.  Not used for downloaded models.
/// - `model_name` — Override the model filename to download.  Pass `None`
///   to use [`default_model_for_platform`].
/// - `on_progress` — Callback invoked periodically during the download.
///   Receives a [`WhisperModelDownloadProgress`] payload.  Not called at
///   all if a model is already cached.
/// - `app_data_dir` — **Always pass `Some(app.path().app_data_dir())`.**
///   The default `~/.cache` path is inaccessible on every sandboxed
///   platform (Android, iOS, macOS App Store).  On non-sandboxed desktop
///   builds it is harmless — the cache simply lives under the app data
///   directory.  `None` is only acceptable in CLI / test contexts.
///
/// # Errors
///
/// Returns a human-readable error string on failure (network, I/O, etc.).
pub fn find_or_download_whisper_model<F>(
    model_dir: &Path,
    model_name: Option<&str>,
    on_progress: F,
    app_data_dir: Option<&Path>,
) -> Result<PathBuf, String>
where
    F: Fn(WhisperModelDownloadProgress) + Send + Sync + 'static,
{
    // ── 1. Build the HF cache once (shared across all lookups) ────────
    let cache = resolve_hf_cache(app_data_dir)?;

    // ── 2. Check the HF cache for an already-downloaded model ─────────
    //
    // Walk the preferred candidate list (best quality first).  The first
    // cached model wins.  This is the primary lookup path — identical to
    // how mistral.rs resolves models from cache before triggering a
    // download.
    for candidate in MODEL_CANDIDATES {
        if let Some(path) = resolve_from_hf_cache(candidate, &cache) {
            info!("Whisper model found in HF cache: {:?}", path);
            return Ok(path);
        }
    }

    // ── 3. Legacy fallback: check model_dir for manually-placed files ─
    if model_dir.exists() {
        for candidate in MODEL_CANDIDATES {
            let path = model_dir.join(candidate);
            if path.exists() && path.is_file() {
                info!("Whisper model found in legacy model_dir: {:?}", path);
                return Ok(path);
            }
        }

        // Any .bin file at all.
        if let Ok(entries) = std::fs::read_dir(model_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "bin" {
                            warn!(
                                "No preferred Whisper model found; falling back to {:?}",
                                path
                            );
                            return Ok(path);
                        }
                    }
                }
            }
        }
    }

    // ── 4. Nothing cached — download via hf-hub ───────────────────────

    let target_name = model_name.unwrap_or(&default_model_for_platform_internal());

    info!(
        "No Whisper model cached; downloading {} from {}",
        target_name, HF_WHISPER_REPO,
    );

    let hf_path = download_whisper_model(target_name, on_progress, &cache)?;

    // The returned path lives inside the HF cache's blob store — use it
    // directly.  No copy / hard-link / symlink into model_dir.  This is
    // the same strategy mistral.rs uses: the HF cache *is* the model
    // directory.
    Ok(hf_path)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Internal version of `default_model_for_platform` that returns `&'static str`
/// (used by the generic callback-based API where we need a string slice).
fn default_model_for_platform_internal() -> &'static str {
    if cfg!(target_os = "android") {
        "ggml-base.bin"
    } else if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        "ggml-large-v3-turbo.bin"
    } else {
        "ggml-small.bin"
    }
}

/// Convert a `transcribe_rs::TranscriptionResult` into our UniFFI-compatible
/// [`WhisperResult`].
fn transcription_result_to_ffi(result: TranscriptionResult) -> WhisperResult {
    WhisperResult {
        text: result.text,
        segments: result
            .segments
            .unwrap_or_default()
            .into_iter()
            .map(|seg| WhisperSegment {
                start_secs: seg.start,
                end_secs: seg.end,
                text: seg.text,
            })
            .collect(),
    }
}

/// Download a single GGML model file from `ggerganov/whisper.cpp` using the
/// HuggingFace hub synchronous API.
///
/// Accepts a pre-built [`hf_hub::Cache`] (from [`resolve_hf_cache`]) so that
/// the caller controls where files land — essential on sandboxed platforms
/// (iOS, Android).
///
/// hf-hub handles:
/// - Resumable downloads (partial `.incomplete` files are continued).
/// - Content-addressed blob storage with symlinks.
///
/// Returns the absolute path to the downloaded file inside the HF cache.
fn download_whisper_model<F>(
    model_name: &str,
    on_progress: F,
    cache: &hf_hub::Cache,
) -> Result<PathBuf, String>
where
    F: Fn(WhisperModelDownloadProgress) + Send + Sync + 'static,
{
    use hf_hub::api::sync::ApiBuilder;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;

    let api = ApiBuilder::from_cache(cache.clone())
        .build()
        .map_err(|e| format!("Failed to create HF API: {e}"))?;

    let repo = api.model(HF_WHISPER_REPO.to_string());

    // Emit an initial "starting" progress event.
    let name = model_name.to_string();
    on_progress(WhisperModelDownloadProgress {
        model_name: name.clone(),
        downloaded_bytes: 0,
        downloaded_display: "0 B".into(),
        total_bytes: 0,
        total_display: "calculating…".into(),
        progress: 0.0,
        done: false,
    });

    // hf-hub's `repo.get()` is a blocking call that downloads the file.
    // We spin up a native monitor thread that polls the HF cache directory
    // for the blob size to provide progress updates — the same pattern
    // used in `hf_cache::download_model`.
    let finished = Arc::new(AtomicBool::new(false));
    let finished_clone = Arc::clone(&finished);
    let monitor_name = name.clone();

    // Try to figure out the model cache path for monitoring.
    let cache_path = crate::hf_cache::model_cache_path(HF_WHISPER_REPO);

    // Approximate expected sizes for common models so the progress bar
    // has a meaningful denominator even before the download starts.
    let expected_size = approximate_model_size(model_name);

    let monitor_handle = std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));

        if finished_clone.load(Ordering::Relaxed) {
            break;
        }

        let current_bytes = cache_path
            .as_ref()
            .filter(|p| p.exists())
            .map(|p| dir_size_recursive(p))
            .unwrap_or(0);

        let progress = if expected_size > 0 {
            (current_bytes as f64 / expected_size as f64).min(0.99)
        } else {
            0.0
        };

        on_progress(WhisperModelDownloadProgress {
            model_name: monitor_name.clone(),
            downloaded_bytes: current_bytes,
            downloaded_display: format_size(current_bytes),
            total_bytes: expected_size,
            total_display: format_size(expected_size),
            progress,
            done: false,
        });
    });

    // Blocking download — hf-hub handles resumption internally.
    let result = repo.get(model_name);

    finished.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    match result {
        Ok(path) => {
            info!(
                "Whisper model {} downloaded to HF cache: {:?}",
                model_name, path
            );
            // Final progress event.
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            // We need a new closure-compatible progress emitter here.
            // Since the monitor thread consumed `on_progress`, we log instead.
            // The caller will see the file appear and can emit its own "done".
            info!(
                "Whisper model download complete: {} ({})",
                model_name,
                format_size(file_size)
            );
            Ok(path)
        }
        Err(e) => {
            let msg = format!(
                "Failed to download Whisper model {} from {}: {}",
                model_name, HF_WHISPER_REPO, e
            );
            warn!("{}", msg);
            Err(msg)
        }
    }
}

/// Approximate expected download size for common Whisper GGML models.
/// These are rough estimates used for progress bar rendering; they don't
/// need to be exact.
fn approximate_model_size(model_name: &str) -> u64 {
    match model_name {
        "ggml-large-v3-turbo.bin" => 1_620_000_000,
        "ggml-large-v3.bin" => 3_090_000_000,
        "ggml-large-v2.bin" => 3_090_000_000,
        "ggml-large-v1.bin" => 3_090_000_000,
        "ggml-medium.bin" => 1_530_000_000,
        "ggml-medium.en.bin" => 1_530_000_000,
        "ggml-medium-q5_0.bin" => 539_000_000,
        "ggml-small.bin" => 466_000_000,
        "ggml-small.en.bin" => 466_000_000,
        "ggml-base.bin" => 142_000_000,
        "ggml-base.en.bin" => 142_000_000,
        "ggml-tiny.bin" => 75_000_000,
        "ggml-tiny.en.bin" => 75_000_000,
        _ => 0,
    }
}

/// Recursively compute the total size of a directory in bytes.
/// Follows symlinks to count the real file size and also counts in-progress
/// `.incomplete` staging files so progress polling works during download.
fn dir_size_recursive(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total += dir_size_recursive(&entry_path);
            } else if let Ok(meta) = entry_path.symlink_metadata() {
                if meta.is_symlink() {
                    if let Ok(real_meta) = entry_path.metadata() {
                        total += real_meta.len();
                    }
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

/// Format a byte count into a human-readable string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
