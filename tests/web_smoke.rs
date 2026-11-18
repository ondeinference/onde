// Copyright 2026 Seto Elkahfi (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Headless-browser smoke test for the web engine. It downloads Qwen 2.5 0.5B
//! (about 380 MB), so it only does anything when `ONDE_WEB_SMOKE` is set at
//! build time, and it is not part of the default CI run:
//!
//! ```text
//! ONDE_WEB_SMOKE=1 WASM_BINDGEN_TEST_TIMEOUT=1800 \
//!   RUSTFLAGS="-C target-feature=+simd128" \
//!   wasm-pack test --headless --chrome --release
//! ```
//!
//! It runs in a dedicated worker, which is where the engine is meant to live.

#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use onde::inference::{ChatEngine, GgufModelConfig, SamplingConfig};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[wasm_bindgen_test]
async fn qwen25_streams_a_reply() {
    if option_env!("ONDE_WEB_SMOKE").is_none() {
        return;
    }
    let config = GgufModelConfig::qwen25_0_5b();

    let mut last = (0u64, None);
    onde::web_store::download_model(&config, |received, total| last = (received, total))
        .await
        .expect("download");
    assert!(last.0 > 0, "no download progress reported");
    assert!(onde::web_store::is_cached(&config).await);

    let engine = ChatEngine::new();
    engine
        .load_gguf_model(
            config,
            Some("You are a terse assistant.".into()),
            Some(SamplingConfig {
                max_tokens: Some(24),
                ..SamplingConfig::deterministic()
            }),
        )
        .await
        .expect("load");
    assert!(engine.is_loaded().await);

    let mut rx = engine
        .stream_message("Say hello in one short sentence.")
        .await
        .expect("stream");
    let mut text = String::new();
    let mut chunks = 0;
    let mut finished = false;
    while let Some(chunk) = rx.recv().await {
        text.push_str(&chunk.delta);
        chunks += 1;
        if chunk.done {
            assert!(
                !chunk
                    .finish_reason
                    .as_deref()
                    .unwrap_or("")
                    .starts_with("error"),
                "{:?}",
                chunk.finish_reason
            );
            finished = true;
            break;
        }
    }
    assert!(finished, "stream ended without a done chunk");
    assert!(!text.trim().is_empty(), "empty reply");
    assert!(chunks > 1, "reply did not stream in pieces");

    // The turn is in history, so the engine reports both messages.
    assert_eq!(engine.history().await.len(), 2);

    let reply = engine.send_message("And in French?").await.expect("send");
    assert!(!reply.text.trim().is_empty());
    assert_eq!(engine.history().await.len(), 4);
}
