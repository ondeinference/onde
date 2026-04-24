//! On-device LLM inference powered by [mistral.rs](https://github.com/EricLBuehler/mistral.rs).
//!
//! This module provides a high-level, framework-agnostic API for running
//! LLM chat inference on-device across macOS, iOS, Windows, Linux, and Android.
//!
//! # Modules
//!
//! - [`engine`] — The main [`ChatEngine`](engine::ChatEngine) that wraps mistral.rs
//!   with conversation history, sampling config, and model lifecycle management.
//! - [`types`] — Shared types ([`ChatMessage`](types::ChatMessage),
//!   [`SamplingConfig`](types::SamplingConfig), [`InferenceResult`](types::InferenceResult), etc.)
//!   used by the engine and any UI layer.
//! - [`models`] — Model ID constants and rich metadata for all supported models.
//! - [`token`] — HuggingFace token resolution (build-time literal or on-disk cache).
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use onde::inference::engine::ChatEngine;
//! use onde::inference::types::*;
//!
//! // Create engine and load the platform-appropriate default model.
//! let engine = ChatEngine::new();
//! engine.load_gguf_model(
//!     GgufModelConfig::platform_default(),
//!     Some("You are a helpful assistant.".into()),
//!     None, // platform-aware sampling defaults
//! ).await?;
//!
//! // Multi-turn chat (history is managed automatically).
//! let reply = engine.send_message("What is Rust's ownership model?").await?;
//! println!("{}", reply.text);
//!
//! let follow_up = engine.send_message("Can you give me an example?").await?;
//! println!("{}", follow_up.text);
//!
//! // One-shot generation (does NOT modify conversation history).
//! let enhanced = engine.generate(
//!     vec![ChatMessage::user("Expand this into a detailed prompt: a cat in space")],
//!     Some(SamplingConfig::deterministic()),
//! ).await?;
//! println!("{}", enhanced.text);
//!
//! // Streaming inference.
//! let mut rx = engine.stream_message("Tell me a story.").await?;
//! while let Some(chunk) = rx.recv().await {
//!     if !chunk.delta.is_empty() {
//!         print!("{}", chunk.delta);
//!     }
//!     if chunk.done {
//!         break;
//!     }
//! }
//!
//! // Cleanup.
//! engine.unload_model().await;
//! ```

pub mod engine;
pub mod ffi;
pub mod models;
pub mod token;
pub mod types;

// Re-export the most commonly used items at the `inference::` level for
// ergonomic imports like `use onde::inference::{ChatEngine, ChatMessage, ...}`.
pub use engine::ChatEngine;
pub use ffi::{OndeChatEngine, StreamChunkListener};
pub use types::{
    ChatMessage, ChatRole, EngineInfo, EngineStatus, GgufModelConfig, InferenceError,
    InferenceResult, IsqModelConfig, SamplingConfig, StreamChunk, ToolCallInfo, ToolDefinition,
    ToolResult,
};
