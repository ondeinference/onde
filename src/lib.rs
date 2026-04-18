//! # Onde
//!
//! **On-device chat inference for cross-platform apps.**
//!
//! Run LLM chat locally — no cloud, no latency, no data leaving the device.
//!
//! Onde wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) with a
//! unified API for model discovery, HuggingFace Hub downloads, cache
//! management, and GPU acceleration across every platform.
//!
//! Built by [Onde Inference](https://ondeinference.com)
//!
//! ## Modules
//!
//! - [`hf_cache`] — HuggingFace Hub cache inspection, repair, and model
//!   download with a framework-agnostic progress-callback API.
//! - [`inference`] — Chat inference engine, UniFFI FFI wrapper, model metadata,
//!   and HuggingFace token resolution.
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
//! use onde::inference::ChatEngine;
//! use onde::inference::GgufModelConfig;
//!
//! let engine = ChatEngine::new();
//! engine
//!     .load_gguf_model(
//!         GgufModelConfig::platform_default(),
//!         Some("You are a helpful assistant.".into()),
//!         None,
//!     )
//!     .await?;
//!
//! let result = engine.send_message("Hello!").await?;
//! println!("{}", result.text);
//! ```

pub mod hf_cache;

pub mod inference;
pub mod pulse;

uniffi::setup_scaffolding!();

// Re-export mistralrs for every platform that onde supports.
// Apps use `onde::mistralrs::Model` etc. instead of declaring a direct dep.
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
pub use mistralrs;

// Re-exports needed for the GLOBAL_HF_CACHE workaround on sandboxed platforms.
// On iOS/tvOS `~/.cache` is outside the container; on Android `dirs::home_dir()`
// panics.  All three need `hf_hub::Cache` + `mistralrs_core::GLOBAL_HF_CACHE`.
#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]
pub use hf_hub;
#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]
pub use mistralrs_core;
