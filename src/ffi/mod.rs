//! FFI bindings for cross-platform consumers.
//!
//! This module groups the foreign-function-interface layers that expose
//! [`ChatEngine`](crate::inference::engine::ChatEngine) to non-Rust callers.
//!
//! # Sub-modules
//!
//! - [`uniffi`]: UniFFI-generated bindings for Swift and JVM Kotlin.
//!   Uses `#[uniffi::export]` proc macros to produce idiomatic Swift classes
//!   and Kotlin JNA wrappers at build time. This is the primary FFI layer
//!   for the Swift SDK (`onde-swift`) and the Android/JVM Kotlin SDK
//!   (`onde-inference` on Maven Central).
//!
//! - [`c_api`]: Thin C ABI for Kotlin/Native (iOS) cinterop.
//!   Hand-written `#[no_mangle] extern "C"` functions with plain C types
//!   (`char*`, `double`, function pointers). Exists because UniFFI 0.31's
//!   Kotlin codegen uses JNA, which is unavailable on Kotlin/Native. Each
//!   engine instance owns a tokio runtime and calls `block_on()` internally;
//!   the Kotlin side dispatches to `Dispatchers.IO`.

pub mod uniffi;

#[allow(private_interfaces)]
pub mod c_api;

// Re-export the most commonly used items so existing `use crate::ffi::*`
// imports continue to work.
pub use self::uniffi::{OndeChatEngine, StreamChunkListener};
