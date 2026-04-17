//! Sends timing data to the Onde Inference dashboard via smbCloud GresIQ.
//!
//! Opt-in at the infra level, not the code level. Set three env vars and
//! telemetry starts flowing. Set none and nothing happens — no warnings,
//! no stubs, the branches just never run.
//!
//! The engine reads the vars once in `ChatEngine::new()`. You can't flip
//! telemetry on mid-run without restarting.
//!
//! | Variable            | Example                        |
//! |---------------------|--------------------------------|
//! | `GRESIQ_BASE_URL`   | `https://api.smbcloud.xyz`     |
//! | `GRESIQ_API_KEY`    | `gresiq_abc123...`             |
//! | `GRESIQ_API_SECRET` | `<secret from GresIQ console>` |
//!
//! `ONDE_EDGE_ID` is what this machine gets called in the dashboard.
//! Anything works — just keep it stable across restarts or you'll
//! end up with duplicate edges. Defaults to `"onde-unknown"`.

mod client;
mod events;

pub use client::PulseClient;
pub use events::{InferenceEvent, ModelLoadedEvent};

use std::sync::atomic::{AtomicU64, Ordering};

/// Cheap unique ID per inference request. No uuid crate needed.
/// Looks like `onde-1720000000000-42`.
pub(crate) fn next_request_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("onde-{}-{}", ms, seq)
}
