//! Sends timing data to the Onde Inference dashboard.
//!
//! GresIQ credentials are embedded in the SDK at build time — consumer apps
//! never set them.  Telemetry activates automatically when the SDK was built
//! with a valid `.env` file containing `GRESIQ_API_KEY` and `GRESIQ_API_SECRET`.
//!
//! `ONDE_EDGE_ID` (read from the process environment at runtime) is what this
//! machine gets called in the dashboard.  Anything works — just keep it stable
//! across restarts or you'll end up with duplicate edges.  Defaults to
//! `"onde-unknown"`.

mod client;
mod events;

pub use client::PulseClient;
pub use events::{InferenceEvent, ModelLoadedEvent};

use std::sync::atomic::{AtomicU64, Ordering};

/// Cheap unique ID per inference request.  No uuid crate needed.
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
