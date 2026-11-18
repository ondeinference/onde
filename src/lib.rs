//! # Onde
//!
//! **On-device AI inference for cross-platform apps.**
//!
//! Run LLM chat and image generation locally. No cloud, no per-call cost, no data leaving the device.
//!
//! Onde wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) with a
//! unified API for model discovery, HuggingFace Hub downloads, cache
//! management, and GPU acceleration across every platform.
//!
//! Built by [Onde Inference](https://ondeinference.com)
//!
//! ## Modules
//!
//! - `hf_cache` — HuggingFace Hub cache inspection, repair, and model
//!   download with a framework-agnostic progress-callback API.
//! - `pulse` — usage telemetry.
//! - [`inference`] — Text: the chat inference engine, UniFFI FFI wrapper, model
//!   metadata, and HuggingFace token resolution.
//! - Image generation — FLUX.1 Schnell behind the `diffusion` feature, run
//!   through `onde::mistralrs` (see below).
//!
//! ## wasm32
//!
//! On `wasm32-unknown-unknown` the crate has a real browser engine:
//! [`inference::ChatEngine`] runs GGUF Qwen2.5 and Qwen3 models on the CPU
//! through candle, single-threaded, so it needs no COOP/COEP headers. Build the
//! host with `RUSTFLAGS="-C target-feature=+simd128"` for speed, and run the
//! engine in a Web Worker, because a forward pass blocks its thread. Tool
//! calling is not available there, and WebGPU is not used yet.
//!
//! The `web_store` module downloads model files from Hugging Face and keeps
//! them in the Cache Storage API (bucket `onde-models-v1`), which works from
//! a window and from a worker alike. Call `web_store::download_model` first to
//! show progress; `ChatEngine::load_gguf_model` then reads from the cache.
//!
//! Other wasm and unknown targets (wasi, emscripten) compile against a stub
//! `ChatEngine` whose calls all return an "unsupported" error. On every wasm32
//! target there is no UniFFI, `hf_cache` or `pulse`: the FFI wrapper is
//! native-only, and the other two depend on the filesystem, OS threads and
//! `SystemTime`.
//!
//! ## Image generation (feature `diffusion`)
//!
//! Off by default. With `features = ["diffusion"]`, FLUX.1 Schnell joins the
//! model catalogue and [`hf_cache::download_model`] can fetch it. Generation is
//! plain mistral.rs: build a `DiffusionModelBuilder` from `onde::mistralrs` and
//! call `Model::generate_image`. See the README for a full example.
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

#[cfg(not(target_arch = "wasm32"))]
pub mod hf_cache;

pub mod inference;
#[cfg(not(target_arch = "wasm32"))]
pub mod pulse;
#[cfg(onde_web)]
pub mod web_store;

#[cfg_attr(not(onde_native), allow(dead_code))]
static PANIC_HOOK_ONCE: std::sync::Once = std::sync::Once::new();

#[cfg_attr(not(onde_native), allow(dead_code))]
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

#[cfg(not(target_arch = "wasm32"))]
uniffi::setup_scaffolding!();

// Re-export mistralrs for every platform that onde supports.
// Apps use `onde::mistralrs::Model` etc. instead of declaring a direct dep.
#[cfg(onde_native)]
pub use mistralrs;

// Re-exports needed for the GLOBAL_HF_CACHE workaround on sandboxed platforms.
// On iOS/tvOS `~/.cache` is outside the container; on Android `dirs::home_dir()`
// panics.  All three need `hf_hub::Cache` + `mistralrs_core::GLOBAL_HF_CACHE`.
#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]
pub use hf_hub;
#[cfg(any(target_os = "android", target_os = "ios", target_os = "tvos"))]
pub use mistralrs_core;
