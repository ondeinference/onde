//! C FFI bridge for the Onde React Native npm SDK.
//!
//! This crate wraps [`onde::inference::engine::ChatEngine`] with `extern "C"`
//! functions callable from Swift (iOS) and Kotlin/JNI (Android).
//!
//! ## Design
//!
//! - **Opaque pointer** (`*mut c_void`) for the engine handle.
//! - **JSON strings** (via `serde_json`) for complex types crossing FFI.
//! - **Global `tokio::Runtime`** created once via `OnceLock`.
//! - Every function that returns a `*mut c_char` requires the caller to free
//!   it with [`onde_free_string`].
//! - Android JNI wrappers are gated behind `#[cfg(target_os = "android")]`.

use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::OnceLock;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::runtime::Runtime;

use onde::inference::engine::ChatEngine;
use onde::inference::types::{ChatMessage, GgufModelConfig, SamplingConfig};

// ── Global Tokio runtime ─────────────────────────────────────────────────────

/// Returns a reference to the global multi-threaded Tokio runtime.
///
/// The runtime is created on first access and lives for the process lifetime.
fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("onde-react-native: failed to create tokio runtime")
    })
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Serialize any `Serialize` value to a heap-allocated C string.
///
/// The caller **must** free the returned pointer with [`onde_free_string`].
fn to_json_cstring<T: Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(err) => error_json(&format!("JSON serialization failed: {err}")),
    }
}

/// Deserialize a JSON C string into a Rust type.
///
/// Returns `None` if the pointer is null or the JSON is malformed.
fn from_json_cstr<T: DeserializeOwned>(ptr: *const c_char) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let s = cstr.to_str().ok()?;
    serde_json::from_str(s).ok()
}

/// Read an optional C string parameter into an `Option<String>`.
///
/// Returns `None` if the pointer is null or the string is empty.
fn nullable_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let s = cstr.to_str().ok()?;
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

/// Create a heap-allocated JSON error string: `{ "error": "<msg>" }`.
fn error_json(msg: &str) -> *mut c_char {
    let json = serde_json::json!({ "error": msg });
    CString::new(json.to_string())
        .unwrap_or_default()
        .into_raw()
}

// ── Engine lifecycle ─────────────────────────────────────────────────────────

/// Create a new `ChatEngine` and return an opaque pointer to it.
///
/// The engine starts with no model loaded.  The caller must eventually call
/// [`onde_engine_destroy`] to free the memory.
#[no_mangle]
pub extern "C" fn onde_engine_create() -> *mut c_void {
    let engine = Box::new(ChatEngine::new());
    Box::into_raw(engine) as *mut c_void
}

/// Destroy an engine previously created with [`onde_engine_create`].
///
/// After this call the pointer is invalid and must not be used.
/// Passing a null pointer is a no-op.
#[no_mangle]
pub extern "C" fn onde_engine_destroy(engine: *mut c_void) {
    if engine.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(engine as *mut ChatEngine);
    }
}

/// Load the platform-default GGUF model.
///
/// # Parameters
///
/// - `engine` — opaque engine pointer from [`onde_engine_create`].
/// - `system_prompt` — optional C string (pass null to omit).
/// - `sampling_json` — optional JSON-encoded `SamplingConfig` (pass null for defaults).
///
/// # Returns
///
/// A heap-allocated JSON C string (free with [`onde_free_string`]):
/// - On success: `{ "elapsed_secs": <f64> }`
/// - On failure: `{ "error": "<description>" }`
#[no_mangle]
pub extern "C" fn onde_engine_load_default_model(
    engine: *mut c_void,
    system_prompt: *const c_char,
    sampling_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    let prompt = nullable_str(system_prompt);
    let sampling: Option<SamplingConfig> = from_json_cstr(sampling_json);
    let config = GgufModelConfig::platform_default();

    runtime().block_on(async {
        match engine_ref.load_gguf_model(config, prompt, sampling).await {
            Ok(duration) => {
                let result = serde_json::json!({ "elapsed_secs": duration.as_secs_f64() });
                to_json_cstring(&result)
            }
            Err(err) => error_json(&err.to_string()),
        }
    })
}

/// Load a specific GGUF model from a JSON-encoded [`GgufModelConfig`].
///
/// # Parameters
///
/// - `config_json` — JSON-encoded `GgufModelConfig`.
/// - `system_prompt` — optional C string (pass null to omit).
/// - `sampling_json` — optional JSON-encoded `SamplingConfig` (pass null for defaults).
///
/// # Returns
///
/// Same JSON shape as [`onde_engine_load_default_model`].
#[no_mangle]
pub extern "C" fn onde_engine_load_model(
    engine: *mut c_void,
    config_json: *const c_char,
    system_prompt: *const c_char,
    sampling_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };

    let config: GgufModelConfig = match from_json_cstr(config_json) {
        Some(c) => c,
        None => return error_json("invalid or null config_json"),
    };
    let prompt = nullable_str(system_prompt);
    let sampling: Option<SamplingConfig> = from_json_cstr(sampling_json);

    runtime().block_on(async {
        match engine_ref.load_gguf_model(config, prompt, sampling).await {
            Ok(duration) => {
                let result = serde_json::json!({ "elapsed_secs": duration.as_secs_f64() });
                to_json_cstring(&result)
            }
            Err(err) => error_json(&err.to_string()),
        }
    })
}

/// Unload the currently loaded model.
///
/// # Returns
///
/// JSON: `{ "model_name": "<name>" }` or `{ "model_name": null }`.
#[no_mangle]
pub extern "C" fn onde_engine_unload_model(engine: *mut c_void) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    let name = runtime().block_on(engine_ref.unload_model());
    let result = serde_json::json!({ "model_name": name });
    to_json_cstring(&result)
}

/// Check whether a model is currently loaded.
#[no_mangle]
pub extern "C" fn onde_engine_is_loaded(engine: *mut c_void) -> bool {
    if engine.is_null() {
        return false;
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    runtime().block_on(engine_ref.is_loaded())
}

// ── Engine info ──────────────────────────────────────────────────────────────

/// Get a snapshot of the engine's current state as JSON.
///
/// # Returns
///
/// JSON-encoded [`EngineInfo`](onde::inference::types::EngineInfo).
#[no_mangle]
pub extern "C" fn onde_engine_info(engine: *mut c_void) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    let info = runtime().block_on(engine_ref.info());
    to_json_cstring(&info)
}

// ── System prompt ────────────────────────────────────────────────────────────

/// Set or replace the engine's system prompt.
#[no_mangle]
pub extern "C" fn onde_engine_set_system_prompt(engine: *mut c_void, prompt: *const c_char) {
    if engine.is_null() {
        return;
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    if let Some(text) = nullable_str(prompt) {
        runtime().block_on(engine_ref.set_system_prompt(text));
    }
}

/// Clear the engine's system prompt.
#[no_mangle]
pub extern "C" fn onde_engine_clear_system_prompt(engine: *mut c_void) {
    if engine.is_null() {
        return;
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    runtime().block_on(engine_ref.clear_system_prompt());
}

// ── Sampling ─────────────────────────────────────────────────────────────────

/// Replace the engine's sampling configuration from a JSON string.
#[no_mangle]
pub extern "C" fn onde_engine_set_sampling(engine: *mut c_void, sampling_json: *const c_char) {
    if engine.is_null() {
        return;
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    if let Some(sampling) = from_json_cstr::<SamplingConfig>(sampling_json) {
        runtime().block_on(engine_ref.set_sampling(sampling));
    }
}

// ── History ──────────────────────────────────────────────────────────────────

/// Get the conversation history as a JSON array of `ChatMessage` objects.
#[no_mangle]
pub extern "C" fn onde_engine_history(engine: *mut c_void) -> *mut c_char {
    if engine.is_null() {
        return to_json_cstring(&serde_json::json!([]));
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    let history = runtime().block_on(engine_ref.history());
    to_json_cstring(&history)
}

/// Clear the conversation history.  Returns the number of turns removed.
#[no_mangle]
pub extern "C" fn onde_engine_clear_history(engine: *mut c_void) -> u64 {
    if engine.is_null() {
        return 0;
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    runtime().block_on(engine_ref.clear_history()) as u64
}

/// Append a message to the conversation history without running inference.
///
/// `message_json` must be a JSON-encoded `ChatMessage`.
#[no_mangle]
pub extern "C" fn onde_engine_push_history(engine: *mut c_void, message_json: *const c_char) {
    if engine.is_null() {
        return;
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    if let Some(message) = from_json_cstr::<ChatMessage>(message_json) {
        runtime().block_on(engine_ref.push_history(message));
    }
}

// ── Non-streaming inference ──────────────────────────────────────────────────

/// Send a user message and receive a complete assistant reply.
///
/// The message and reply are appended to the conversation history.
///
/// # Returns
///
/// JSON-encoded [`InferenceResult`](onde::inference::types::InferenceResult)
/// on success, or `{ "error": "<description>" }` on failure.
#[no_mangle]
pub extern "C" fn onde_engine_send_message(
    engine: *mut c_void,
    message: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    let text = match nullable_str(message) {
        Some(t) => t,
        None => return error_json("message is null or empty"),
    };

    runtime().block_on(async {
        match engine_ref.send_message(text).await {
            Ok(result) => to_json_cstring(&result),
            Err(err) => error_json(&err.to_string()),
        }
    })
}

/// One-shot generation without modifying conversation history.
///
/// # Parameters
///
/// - `messages_json` — JSON array of `ChatMessage`.
/// - `sampling_json` — optional JSON-encoded `SamplingConfig` (null for engine defaults).
///
/// # Returns
///
/// JSON-encoded [`InferenceResult`] or `{ "error": "..." }`.
#[no_mangle]
pub extern "C" fn onde_engine_generate(
    engine: *mut c_void,
    messages_json: *const c_char,
    sampling_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };

    let messages: Vec<ChatMessage> = match from_json_cstr(messages_json) {
        Some(m) => m,
        None => return error_json("invalid or null messages_json"),
    };
    let sampling: Option<SamplingConfig> = from_json_cstr(sampling_json);

    runtime().block_on(async {
        match engine_ref.generate(messages, sampling).await {
            Ok(result) => to_json_cstring(&result),
            Err(err) => error_json(&err.to_string()),
        }
    })
}

// ── Streaming inference ──────────────────────────────────────────────────────

/// Send a user message and stream token chunks via a C callback.
///
/// The callback is invoked for each chunk with:
/// - `chunk_json` — a JSON-encoded [`StreamChunk`](onde::inference::types::StreamChunk)
///   (valid only for the duration of the callback invocation).
/// - `done` — `true` when this is the final chunk.
///
/// # Returns
///
/// - `null` on success (streaming completed).
/// - A heap-allocated JSON error string on failure (free with [`onde_free_string`]).
#[no_mangle]
pub extern "C" fn onde_engine_stream_message(
    engine: *mut c_void,
    message: *const c_char,
    callback: extern "C" fn(*const c_char, bool),
) -> *mut c_char {
    if engine.is_null() {
        return error_json("engine pointer is null");
    }
    let engine_ref = unsafe { &*(engine as *const ChatEngine) };
    let text = match nullable_str(message) {
        Some(t) => t,
        None => return error_json("message is null or empty"),
    };

    runtime().block_on(async {
        let receiver = match engine_ref.stream_message(text).await {
            Ok(rx) => rx,
            Err(err) => return error_json(&err.to_string()),
        };

        // Drain the channel and forward each chunk through the callback.
        let mut receiver = receiver;
        loop {
            match receiver.recv().await {
                Some(chunk) => {
                    let done = chunk.done;
                    match serde_json::to_string(&chunk) {
                        Ok(json) => {
                            if let Ok(cstr) = CString::new(json) {
                                callback(cstr.as_ptr(), done);
                            }
                        }
                        Err(err) => {
                            log::error!(
                                "onde_engine_stream_message: JSON serialization failed: {err}"
                            );
                        }
                    }
                    if done {
                        break;
                    }
                }
                None => {
                    // Channel closed without a done chunk — send a synthetic one.
                    let final_json = serde_json::json!({
                        "delta": "",
                        "done": true,
                        "finish_reason": "channel_closed"
                    });
                    if let Ok(cstr) = CString::new(final_json.to_string()) {
                        callback(cstr.as_ptr(), true);
                    }
                    break;
                }
            }
        }

        std::ptr::null_mut()
    })
}

// ── Model config presets ─────────────────────────────────────────────────────

/// Return the platform-default GGUF model config as JSON.
#[no_mangle]
pub extern "C" fn onde_default_model_config() -> *mut c_char {
    to_json_cstring(&GgufModelConfig::platform_default())
}

/// Return the Qwen 2.5 1.5B GGUF model config as JSON.
#[no_mangle]
pub extern "C" fn onde_qwen25_1_5b_config() -> *mut c_char {
    to_json_cstring(&GgufModelConfig::qwen25_1_5b())
}

/// Return the Qwen 2.5 3B GGUF model config as JSON.
#[no_mangle]
pub extern "C" fn onde_qwen25_3b_config() -> *mut c_char {
    to_json_cstring(&GgufModelConfig::qwen25_3b())
}

// ── Sampling presets ─────────────────────────────────────────────────────────

/// Return the default sampling config as JSON.
#[no_mangle]
pub extern "C" fn onde_default_sampling_config() -> *mut c_char {
    to_json_cstring(&SamplingConfig::default())
}

/// Return the deterministic (greedy) sampling config as JSON.
#[no_mangle]
pub extern "C" fn onde_deterministic_sampling_config() -> *mut c_char {
    to_json_cstring(&SamplingConfig::deterministic())
}

/// Return the mobile sampling config as JSON.
#[no_mangle]
pub extern "C" fn onde_mobile_sampling_config() -> *mut c_char {
    to_json_cstring(&SamplingConfig::mobile())
}

// ── Memory management ────────────────────────────────────────────────────────

/// Free a C string previously returned by any `onde_*` function.
///
/// Passing a null pointer is a no-op.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn onde_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Android JNI bindings
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jlong, jstring, JNI_FALSE, JNI_TRUE};
    use jni::JNIEnv;

    use std::ffi::{c_char, c_void, CStr, CString};

    use super::*;

    // ── JNI helpers ──────────────────────────────────────────────────────

    /// Convert a JNI `JString` to a `CString`.
    ///
    /// Returns `None` if the JString is null or conversion fails.
    /// The caller must keep the returned `CString` alive for as long as
    /// the `.as_ptr()` result is used.
    fn jstring_to_cstring(env: &mut JNIEnv, jstr: &JString) -> Option<CString> {
        if jstr.is_null() {
            return None;
        }
        let java_str = env.get_string(jstr).ok()?;
        let rust_str: String = java_str.into();
        CString::new(rust_str).ok()
    }

    /// Convert a `*mut c_char` (owned C string) to a JNI `jstring`.
    ///
    /// Frees the C string after conversion.  Returns null jstring on failure.
    fn cstring_ptr_to_jstring(env: &mut JNIEnv, ptr: *mut c_char) -> jstring {
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        let cstr = unsafe { CStr::from_ptr(ptr) };
        let result = match cstr.to_str() {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(std::ptr::null_mut()),
            Err(_) => std::ptr::null_mut(),
        };
        // Free the C string.
        onde_free_string(ptr);
        result
    }

    // ── Engine lifecycle ─────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineCreate(
        _env: JNIEnv,
        _class: JClass,
    ) -> jlong {
        onde_engine_create() as jlong
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineDestroy(
        _env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) {
        onde_engine_destroy(engine as *mut c_void);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineLoadDefaultModel(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        system_prompt: JString,
        sampling_json: JString,
    ) -> jstring {
        let prompt_cstr = jstring_to_cstring(&mut env, &system_prompt);
        let sampling_cstr = jstring_to_cstring(&mut env, &sampling_json);

        let prompt_ptr = prompt_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        let sampling_ptr = sampling_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());

        let result =
            onde_engine_load_default_model(engine as *mut c_void, prompt_ptr, sampling_ptr);
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineLoadModel(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        config_json: JString,
        system_prompt: JString,
        sampling_json: JString,
    ) -> jstring {
        let config_cstr = jstring_to_cstring(&mut env, &config_json);
        let prompt_cstr = jstring_to_cstring(&mut env, &system_prompt);
        let sampling_cstr = jstring_to_cstring(&mut env, &sampling_json);

        let config_ptr = config_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        let prompt_ptr = prompt_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        let sampling_ptr = sampling_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());

        let result =
            onde_engine_load_model(engine as *mut c_void, config_ptr, prompt_ptr, sampling_ptr);
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineUnloadModel(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) -> jstring {
        let result = onde_engine_unload_model(engine as *mut c_void);
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineIsLoaded(
        _env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) -> jboolean {
        if onde_engine_is_loaded(engine as *mut c_void) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    }

    // ── Engine info ──────────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineInfo(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) -> jstring {
        let result = onde_engine_info(engine as *mut c_void);
        cstring_ptr_to_jstring(&mut env, result)
    }

    // ── System prompt ────────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineSetSystemPrompt(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        prompt: JString,
    ) {
        let prompt_cstr = jstring_to_cstring(&mut env, &prompt);
        let prompt_ptr = prompt_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        onde_engine_set_system_prompt(engine as *mut c_void, prompt_ptr);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineClearSystemPrompt(
        _env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) {
        onde_engine_clear_system_prompt(engine as *mut c_void);
    }

    // ── Sampling ─────────────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineSetSampling(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        sampling_json: JString,
    ) {
        let sampling_cstr = jstring_to_cstring(&mut env, &sampling_json);
        let sampling_ptr = sampling_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        onde_engine_set_sampling(engine as *mut c_void, sampling_ptr);
    }

    // ── History ──────────────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineHistory(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) -> jstring {
        let result = onde_engine_history(engine as *mut c_void);
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineClearHistory(
        _env: JNIEnv,
        _class: JClass,
        engine: jlong,
    ) -> jlong {
        onde_engine_clear_history(engine as *mut c_void) as jlong
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEnginePushHistory(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        message_json: JString,
    ) {
        let msg_cstr = jstring_to_cstring(&mut env, &message_json);
        let msg_ptr = msg_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        onde_engine_push_history(engine as *mut c_void, msg_ptr);
    }

    // ── Inference ────────────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineSendMessage(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        message: JString,
    ) -> jstring {
        let msg_cstr = jstring_to_cstring(&mut env, &message);
        let msg_ptr = msg_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        let result = onde_engine_send_message(engine as *mut c_void, msg_ptr);
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeEngineGenerate(
        mut env: JNIEnv,
        _class: JClass,
        engine: jlong,
        messages_json: JString,
        sampling_json: JString,
    ) -> jstring {
        let msgs_cstr = jstring_to_cstring(&mut env, &messages_json);
        let sampling_cstr = jstring_to_cstring(&mut env, &sampling_json);

        let msgs_ptr = msgs_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        let sampling_ptr = sampling_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());

        let result = onde_engine_generate(engine as *mut c_void, msgs_ptr, sampling_ptr);
        cstring_ptr_to_jstring(&mut env, result)
    }

    // ── Model config presets ─────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeDefaultModelConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result = onde_default_model_config();
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeQwen251_5bConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result = onde_qwen25_1_5b_config();
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeQwen253bConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result = onde_qwen25_3b_config();
        cstring_ptr_to_jstring(&mut env, result)
    }

    // ── Sampling presets ─────────────────────────────────────────────────

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeDefaultSamplingConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result = onde_default_sampling_config();
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeDeterministicSamplingConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result = onde_deterministic_sampling_config();
        cstring_ptr_to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_ondeinference_OndeInferenceModule_ondeMobileSamplingConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result = onde_mobile_sampling_config();
        cstring_ptr_to_jstring(&mut env, result)
    }
}
