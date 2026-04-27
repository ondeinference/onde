//! Thin C API for Kotlin/Native (iOS) cinterop.
//!
//! These functions provide a simple, synchronous C interface to the async
//! [`ChatEngine`]. Each engine instance owns a tokio runtime and calls
//! `runtime.block_on()` internally. The Kotlin side dispatches these
//! blocking calls to `Dispatchers.IO`.
//!
//! This module exists because UniFFI generates JVM-only Kotlin bindings
//! (via JNA), which don't work on Kotlin/Native. Rather than reimplementing
//! the full UniFFI binary protocol in Kotlin/Native, we expose this
//! minimal C API that cinterop can consume directly.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::inference::engine::ChatEngine;
use crate::inference::types::*;

// ── Engine container ─────────────────────────────────────────────────────────

/// Wraps `ChatEngine` + its own tokio runtime for blocking calls.
struct CEngine {
    engine: ChatEngine,
    runtime: tokio::runtime::Runtime,
}

// ── Thread-local error ───────────────────────────────────────────────────────

std::thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = const { std::cell::RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

/// Get the last error message, or null if no error.
/// The pointer is valid until the next C API call on the same thread.
#[no_mangle]
pub extern "C" fn onde_last_error() -> *const c_char {
    LAST_ERROR.with(|e| match e.borrow().as_ref() {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    })
}

// ── String helpers ───────────────────────────────────────────────────────────

/// Free a string allocated by Rust. Pass NULL safely (no-op).
///
/// # Safety
///
/// `s` must be either null or a pointer previously returned by one of the
/// `onde_*` functions that document "caller frees with `onde_string_free()`".
/// Each pointer must be freed at most once.
#[no_mangle]
pub unsafe extern "C" fn onde_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

fn to_rust_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .ok()
            .map(|s| s.to_string())
    }
}

fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

// ── Engine lifecycle ─────────────────────────────────────────────────────────

/// Create a new inference engine. Returns an opaque pointer.
/// Returns NULL on failure (check `onde_last_error()`).
#[no_mangle]
pub extern "C" fn onde_engine_new() -> *mut CEngine {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(&format!("Failed to create tokio runtime: {e}"));
            return ptr::null_mut();
        }
    };
    let engine = ChatEngine::new();
    Box::into_raw(Box::new(CEngine { engine, runtime }))
}

/// Destroy an engine and free all resources. Pass NULL safely.
///
/// # Safety
///
/// `ptr` must be either null or a valid pointer returned by [`onde_engine_new`].
/// After this call the pointer is dangling and must not be reused.
#[no_mangle]
pub unsafe extern "C" fn onde_engine_destroy(ptr: *mut CEngine) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

// ── Environment setup ────────────────────────────────────────────────────────

/// Set environment variables for HuggingFace cache.
/// `data_dir` is the root directory (e.g. app's Documents directory).
///
/// # Safety
///
/// `data_dir` must be either null (no-op) or a valid, null-terminated UTF-8
/// C string that remains valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn onde_engine_setup(data_dir: *const c_char) {
    if let Some(dir) = to_rust_str(data_dir) {
        let hf_home = format!("{dir}/models");
        let hf_hub_cache = format!("{hf_home}/hub");
        let tmp_dir = format!("{dir}/tmp");

        let _ = std::fs::create_dir_all(&hf_home);
        let _ = std::fs::create_dir_all(&hf_hub_cache);
        let _ = std::fs::create_dir_all(&tmp_dir);

        std::env::set_var("HF_HOME", &hf_home);
        std::env::set_var("HF_HUB_CACHE", &hf_hub_cache);
        std::env::set_var("HUGGINGFACE_HUB_CACHE", &hf_hub_cache);
        std::env::set_var("TMPDIR", &tmp_dir);
    }
}

// ── Model lifecycle ──────────────────────────────────────────────────────────

/// Load the platform-appropriate default model.
/// `system_prompt` may be NULL.
/// Returns load duration in seconds on success, or -1.0 on error.
///
/// # Safety
///
/// * `ptr` must be a valid pointer returned by [`onde_engine_new`].
/// * `system_prompt` must be either null or a valid, null-terminated UTF-8
///   C string that remains valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn onde_engine_load_default_model(
    ptr: *mut CEngine,
    system_prompt: *const c_char,
) -> f64 {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => {
            set_last_error("engine pointer is null");
            return -1.0;
        }
    };
    let prompt = to_rust_str(system_prompt);
    let config = GgufModelConfig::platform_default();

    match eng
        .runtime
        .block_on(eng.engine.load_gguf_model(config, prompt, None))
    {
        Ok(duration) => duration.as_secs_f64(),
        Err(e) => {
            set_last_error(&format!("{e}"));
            -1.0
        }
    }
}

/// Unload the current model. Returns the model name (caller frees) or NULL.
///
/// # Safety
///
/// `ptr` must be either null (returns NULL) or a valid pointer returned by
/// [`onde_engine_new`]. The returned string, if non-null, must be freed with
/// [`onde_string_free`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_unload(ptr: *mut CEngine) -> *mut c_char {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => return ptr::null_mut(),
    };
    match eng.runtime.block_on(eng.engine.unload_model()) {
        Some(name) => to_c_string(&name),
        None => ptr::null_mut(),
    }
}

/// Check whether a model is loaded.
///
/// # Safety
///
/// `ptr` must be either null (returns `false`) or a valid pointer returned by
/// [`onde_engine_new`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_is_loaded(ptr: *mut CEngine) -> bool {
    match ptr.as_mut() {
        Some(eng) => eng.runtime.block_on(eng.engine.is_loaded()),
        None => false,
    }
}

// ── Inference ────────────────────────────────────────────────────────────────

/// Send a message and get a complete reply.
/// Returns the reply text (caller frees with `onde_string_free()`), or NULL on error.
/// On success, also writes the duration to `*out_duration_secs` if non-NULL.
///
/// # Safety
///
/// * `ptr` must be a valid pointer returned by [`onde_engine_new`].
/// * `message` must be a valid, non-null, null-terminated UTF-8 C string.
/// * `out_duration_secs` must be either null or a valid pointer to a writable `f64`.
/// * The returned string must be freed with [`onde_string_free`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_chat(
    ptr: *mut CEngine,
    message: *const c_char,
    out_duration_secs: *mut f64,
) -> *mut c_char {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => {
            set_last_error("engine pointer is null");
            return ptr::null_mut();
        }
    };
    let msg = match to_rust_str(message) {
        Some(m) => m,
        None => {
            set_last_error("message is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match eng.runtime.block_on(eng.engine.send_message(msg)) {
        Ok(result) => {
            if !out_duration_secs.is_null() {
                *out_duration_secs = result.duration_secs;
            }
            to_c_string(&result.text)
        }
        Err(e) => {
            set_last_error(&format!("{e}"));
            ptr::null_mut()
        }
    }
}

/// Callback for streaming: `delta` is the token text, `done` is true for the last chunk.
/// `user_data` is passed through unchanged. Return `true` to continue, `false` to cancel.
pub type OndeStreamCallback = unsafe extern "C" fn(
    delta: *const c_char,
    done: bool,
    user_data: *mut std::ffi::c_void,
) -> bool;

/// Stream a chat message. Calls `callback` for each token delta.
/// Returns 0 on success, -1 on error (check `onde_last_error()`).
///
/// # Safety
///
/// * `ptr` must be a valid pointer returned by [`onde_engine_new`].
/// * `message` must be a valid, non-null, null-terminated UTF-8 C string.
/// * `callback` must be a valid function pointer matching [`OndeStreamCallback`].
/// * `user_data` is passed through to `callback` without dereference; it may be
///   null if the callback does not need it.
#[no_mangle]
pub unsafe extern "C" fn onde_engine_stream(
    ptr: *mut CEngine,
    message: *const c_char,
    callback: OndeStreamCallback,
    user_data: *mut std::ffi::c_void,
) -> i32 {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => {
            set_last_error("engine pointer is null");
            return -1;
        }
    };
    let msg = match to_rust_str(message) {
        Some(m) => m,
        None => {
            set_last_error("message is null or invalid UTF-8");
            return -1;
        }
    };

    let result = eng.runtime.block_on(async {
        let mut rx = eng.engine.stream_message(msg).await?;
        while let Some(chunk) = rx.recv().await {
            let delta_c = CString::new(chunk.delta.as_str()).unwrap_or_default();
            let should_continue = callback(delta_c.as_ptr(), chunk.done, user_data);
            if chunk.done || !should_continue {
                break;
            }
        }
        Ok::<(), InferenceError>(())
    });

    match result {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(&format!("{e}"));
            -1
        }
    }
}

// ── Engine state ─────────────────────────────────────────────────────────────

/// Get the engine status as a string: "unloaded", "loading", "ready", "generating", "error".
/// Caller frees with `onde_string_free()`.
///
/// # Safety
///
/// `ptr` must be either null (returns `"unloaded"`) or a valid pointer returned by
/// [`onde_engine_new`]. The returned string must be freed with [`onde_string_free`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_status(ptr: *mut CEngine) -> *mut c_char {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => return to_c_string("unloaded"),
    };
    let info = eng.runtime.block_on(eng.engine.info());
    to_c_string(&format!("{}", info.status))
}

/// Get the loaded model name, or NULL if no model is loaded.
/// Caller frees with `onde_string_free()`.
///
/// # Safety
///
/// `ptr` must be either null (returns NULL) or a valid pointer returned by
/// [`onde_engine_new`]. The returned string, if non-null, must be freed with
/// [`onde_string_free`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_model_name(ptr: *mut CEngine) -> *mut c_char {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => return ptr::null_mut(),
    };
    let info = eng.runtime.block_on(eng.engine.info());
    match info.model_name {
        Some(name) => to_c_string(&name),
        None => ptr::null_mut(),
    }
}

/// Get the approximate memory usage string, or NULL.
/// Caller frees with `onde_string_free()`.
///
/// # Safety
///
/// `ptr` must be either null (returns NULL) or a valid pointer returned by
/// [`onde_engine_new`]. The returned string, if non-null, must be freed with
/// [`onde_string_free`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_approx_memory(ptr: *mut CEngine) -> *mut c_char {
    let eng = match ptr.as_mut() {
        Some(e) => e,
        None => return ptr::null_mut(),
    };
    let info = eng.runtime.block_on(eng.engine.info());
    match info.approx_memory {
        Some(mem) => to_c_string(&mem),
        None => ptr::null_mut(),
    }
}

/// Get the number of messages in history.
///
/// # Safety
///
/// `ptr` must be either null (returns 0) or a valid pointer returned by
/// [`onde_engine_new`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_history_length(ptr: *mut CEngine) -> u64 {
    match ptr.as_mut() {
        Some(eng) => eng.runtime.block_on(eng.engine.info()).history_length,
        None => 0,
    }
}

// ── History ──────────────────────────────────────────────────────────────────

/// Clear conversation history. Returns the number of entries removed.
///
/// # Safety
///
/// `ptr` must be either null (returns 0) or a valid pointer returned by
/// [`onde_engine_new`].
#[no_mangle]
pub unsafe extern "C" fn onde_engine_clear_history(ptr: *mut CEngine) -> u64 {
    match ptr.as_mut() {
        Some(eng) => eng.runtime.block_on(eng.engine.clear_history()) as u64,
        None => 0,
    }
}

/// Set or replace the system prompt. Pass NULL to clear.
///
/// # Safety
///
/// * `ptr` must be either null (no-op) or a valid pointer returned by
///   [`onde_engine_new`].
/// * `prompt` must be either null (clears the prompt) or a valid,
///   null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn onde_engine_set_system_prompt(ptr: *mut CEngine, prompt: *const c_char) {
    if let Some(eng) = ptr.as_mut() {
        match to_rust_str(prompt) {
            Some(p) => eng.runtime.block_on(eng.engine.set_system_prompt(p)),
            None => eng.runtime.block_on(eng.engine.clear_system_prompt()),
        }
    }
}
