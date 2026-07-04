//! # Onde
//!
//! **On-device inference for cross-platform apps.**
//!
//! Run LLMs, diffusion models, and speech-to-text locally — no cloud,
//! no latency, no data leaving the device.
//!
//! Onde wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) for
//! LLM and image generation — with a unified API that handles model discovery,
//! HuggingFace Hub downloads, cache management, and GPU acceleration
//! across every platform. (Speech-to-text moved out of onde: apps depend on
//! `transcribe-rs` directly.)
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
//!
//! ## Re-exports
//!
//! `mistralrs`, `hf_hub`, and `mistralrs_core` are re-exported so that apps
//! depending on `onde` do not need their own direct dependency on those crates.
//! Access them as `onde::mistralrs`, `onde::hf_hub`, and `onde::mistralrs_core`.

pub mod hf_cache;

pub mod inference;
pub mod pulse;

static PANIC_HOOK_ONCE: std::sync::Once = std::sync::Once::new();

pub(crate) fn install_panic_hook_once() {
    PANIC_HOOK_ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            let location = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                .unwrap_or_else(|| "<unknown>".to_string());

            let message = if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                (*msg).to_string()
            } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
                msg.clone()
            } else {
                "<non-string panic payload>".to_string()
            };

            log::error!("Rust panic at {}: {}", location, message);
        }));
    });
}

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
