//! Hugging Face token resolution for on-device model loading.
//!
//! On iOS there is no way to run `mistralrs login` or place a token file at
//! `~/.cache/huggingface/token`, so we embed the token at **build time**.
//!
//! ## How it works
//!
//! 1. The consuming app's `build.rs` sets the `HF_TOKEN` env var (e.g. by
//!    loading a `.env` file with `dotenvy`, or via CI secrets).
//! 2. `option_env!("HF_TOKEN")` in this module picks it up at compile time
//!    and bakes it into the binary.
//!
//! This works regardless of whether the build is triggered from a terminal
//! (where shell env vars are available) or from Xcode (which strips the
//! shell environment).
//!
//! ## Setup
//!
//! Set the `HF_TOKEN` environment variable before building, or add it to
//! a `.env` file that your app's `build.rs` loads:
//!
//! ```text
//! HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
//! ```
//!
//! Then build normally — no extra env vars needed.
//!
//! ## Runtime behaviour
//!
//! [`hf_token_source`] returns:
//!
//! 1. `TokenSource::Literal(token)` — if `HF_TOKEN` was present at compile time.
//! 2. `TokenSource::CacheToken`    — otherwise (reads `~/.cache/huggingface/token`;
//!    works on macOS after `mistralrs login`).

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
use mistralrs::TokenSource;

/// Token baked in at compile time (if the `HF_TOKEN` env var was set).
///
/// `option_env!` is evaluated by `rustc` during compilation — the resulting
/// value is a `Option<&'static str>` embedded directly in the binary.
/// This means the token never needs to exist on the device's filesystem.
const BUILD_TIME_HF_TOKEN: Option<&str> = option_env!("HF_TOKEN");

/// Resolve the best available [`TokenSource`] for Hugging Face Hub requests.
///
/// Prefer the compile-time literal when available (required on iOS), otherwise
/// fall back to the on-disk cache token (the default on macOS where
/// `mistralrs login` has been run).
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
pub fn hf_token_source() -> TokenSource {
    match BUILD_TIME_HF_TOKEN {
        Some(token) if !token.is_empty() => {
            log::debug!("Using build-time HF_TOKEN for Hugging Face authentication.");
            TokenSource::Literal(token.to_string())
        }
        _ => {
            log::debug!(
                "No build-time HF_TOKEN found; falling back to cached token \
                 (~/.cache/huggingface/token)."
            );
            TokenSource::CacheToken
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_time_token_is_option() {
        // This just ensures the constant compiles and is the expected type.
        let _: Option<&str> = BUILD_TIME_HF_TOKEN;
    }

    #[test]
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
    fn token_source_returns_valid_variant() {
        let source = hf_token_source();
        // We can't know at test time whether HF_TOKEN was set, but we can
        // verify it returns *something* without panicking.
        let display = format!("{}", source);
        assert!(
            display.starts_with("literal:") || display == "cache",
            "Unexpected token source: {display}"
        );
    }
}
