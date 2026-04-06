//! # Onde
//!
//! **On-device inference for cross-platform apps.**
//!
//! Run LLMs, diffusion models, and speech-to-text locally — no cloud,
//! no latency, no data leaving the device.
//!
//! Onde wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) for
//! LLM and image generation, and [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
//! (via [transcribe-rs](https://github.com/cjpais/transcribe-rs)) for
//! speech-to-text — with a unified API that handles model discovery,
//! HuggingFace Hub downloads, cache management, and GPU acceleration
//! across every platform.
//!
//! Built by [Onde Inference](https://ondeinference.com)
//!
//! ## Modules
//!
//! - [`hf_cache`] — HuggingFace Hub cache inspection, repair, and model
//!   download with a framework-agnostic progress-callback API.
//! - [`inference::models`] — Model ID constants and rich metadata (download
//!   size, display name, org, description) for all supported models.
//! - [`inference::token`] — HuggingFace token resolution (build-time literal
//!   or on-disk cache; required on iOS where the filesystem is sandboxed).
//! - [`whisper`] — Whisper speech-to-text transcription with automatic model
//!   download (feature-gated behind `whisper`).
//!
//! ## Re-exports
//!
//! `mistralrs`, `hf_hub`, and `mistralrs_core` are re-exported so that apps
//! depending on `onde` do not need their own direct dependency on those crates.
//! Access them as `onde::mistralrs`, `onde::hf_hub`, and `onde::mistralrs_core`.
//!
//! ## Example
//!
//! ```rust,ignore
//! use onde::whisper::{
//!     find_or_download_whisper_model, WhisperEngine,
//!     WhisperInferenceParams, TranscriptionEngine,
//! };
//!
//! let model_path = find_or_download_whisper_model(
//!     &"./models/whisper".into(),
//!     None,  // platform-smart default
//!     |p| println!("{}: {}", p.model_name, p.downloaded_display),
//!     None,
//! )?;
//!
//! let mut engine = WhisperEngine::new();
//! engine.load_model(&model_path)?;
//! let result = engine.transcribe_file("audio.wav", None)?;
//! println!("{}", result.text);
//! ```

pub mod hf_cache;
pub mod inference;

uniffi::setup_scaffolding!();

// Whisper speech-to-text via transcribe-rs (opt-in via `whisper` feature).
#[cfg(feature = "whisper")]
pub mod whisper;

// Re-export mistralrs for every platform that onde supports.
// Apps use `onde::mistralrs::Model` etc. instead of declaring a direct dep.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
))]
pub use mistralrs;

// Re-exports needed for the GLOBAL_HF_CACHE workaround on sandboxed platforms.
// On iOS/tvOS `~/.cache` is outside the container; on Android `dirs::home_dir()`
// panics.  All three need `hf_hub::Cache` + `mistralrs_core::GLOBAL_HF_CACHE`.
#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]
pub use hf_hub;
#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]
pub use mistralrs_core;
