//! The wasm engine behind `@ondeinference/web`, on vectoron.
//!
//! [`CpuEngine`] owns one model and is meant to live in a Web Worker: loading
//! the model and every token's forward pass block the thread they run on. The
//! npm package's `cpu/worker.js` drives it.
//!
//! Each method returns a `Promise` and holds its own `Rc` of the session,
//! because wasm-bindgen would otherwise borrow `&self` across the whole await.
//! While a reply streams, the chat is taken out of the session and the session
//! reads [`State::Busy`], so a call that arrives meanwhile (a `reset` during a
//! reply) is refused with an error.
//!
//! The model catalogue lives here, not in JavaScript: [`model_info`] hands the
//! URLs and cache name to the package, and the WebGPU engine uses them to find
//! the same cached GGUF.

mod chat;
mod store;

use std::cell::RefCell;
use std::rc::Rc;

use chat::{Chat, Sampling};
use js_sys::{Function, Promise};
use serde::Serialize;
use store::{ModelFiles, CACHE_NAME, QWEN25_0_5B};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

/// The model the package runs: Qwen 2.5 0.5B, about 380 MB.
const MODEL: ModelFiles = QWEN25_0_5B;

const BUSY: &str = "The engine is busy with another request";
const NOT_LOADED: &str = "No model loaded, call `load` first";

enum State {
    Unloaded,
    Ready(Box<Chat>),
    /// A reply or a load is running.
    Busy,
}

type Session = Rc<RefCell<State>>;

fn to_js_error(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn number(n: u64) -> JsValue {
    JsValue::from_f64(n as f64)
}

/// Takes the chat out of the session, leaving it busy.
fn take_chat(session: &Session) -> Result<Box<Chat>, JsValue> {
    let mut state = session.borrow_mut();
    match std::mem::replace(&mut *state, State::Busy) {
        State::Ready(chat) => Ok(chat),
        State::Unloaded => {
            *state = State::Unloaded;
            Err(to_js_error(NOT_LOADED))
        }
        State::Busy => Err(to_js_error(BUSY)),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelInfo {
    name: &'static str,
    approx_size: &'static str,
    model_url: String,
    tokenizer_url: String,
    cache_name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    status: &'static str,
    model_name: Option<&'static str>,
    approx_memory: Option<&'static str>,
    history_length: u64,
}

/// The model's name, size, download URLs and Cache Storage bucket.
#[wasm_bindgen]
pub fn model_info() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&ModelInfo {
        name: MODEL.display_name,
        approx_size: MODEL.approx_size,
        model_url: MODEL.model_url(),
        tokenizer_url: MODEL.tokenizer_url(),
        cache_name: CACHE_NAME,
    })
    .map_err(to_js_error)
}

/// Resolves to whether the model is already in the browser cache.
#[wasm_bindgen]
pub fn is_cached() -> Promise {
    future_to_promise(async move { Ok(JsValue::from_bool(store::is_cached(&MODEL).await)) })
}

/// Download the model into the browser cache. `on_progress(received, total)`
/// gets byte counts; `total` is `null` when the server hides the length.
#[wasm_bindgen]
pub fn download(on_progress: Function) -> Promise {
    future_to_promise(async move {
        store::download(&MODEL, |received, total| {
            let total = total.map_or(JsValue::NULL, number);
            let _ = on_progress.call2(&JsValue::NULL, &number(received), &total);
        })
        .await
        .map_err(to_js_error)?;
        Ok(JsValue::UNDEFINED)
    })
}

/// Remove the downloaded model from the browser cache.
#[wasm_bindgen]
pub fn delete_cached() -> Promise {
    future_to_promise(async move {
        let removed = store::delete(&MODEL).await.map_err(to_js_error)?;
        Ok(JsValue::from_bool(removed))
    })
}

#[wasm_bindgen]
pub struct CpuEngine {
    session: Session,
}

impl Default for CpuEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl CpuEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CpuEngine {
        console_error_panic_hook::set_once();
        CpuEngine {
            session: Rc::new(RefCell::new(State::Unloaded)),
        }
    }

    /// Load the model into memory (downloading it first if it isn't cached).
    /// `sampling` is `{ temperature, topP, topK, seed, maxTokens }`, every
    /// field optional. Resolves to the load time in seconds.
    pub fn load(&self, system_prompt: Option<String>, sampling: JsValue) -> Promise {
        let session = self.session.clone();
        future_to_promise(async move {
            let sampling: Sampling = if sampling.is_undefined() || sampling.is_null() {
                Sampling::default()
            } else {
                serde_wasm_bindgen::from_value(sampling).map_err(to_js_error)?
            };
            let previous = std::mem::replace(&mut *session.borrow_mut(), State::Busy);
            if matches!(previous, State::Busy) {
                return Err(to_js_error(BUSY));
            }
            // Free the old model before reading the new one.
            drop(previous);
            let started = js_sys::Date::now();
            match load_chat(system_prompt, sampling).await {
                Ok(chat) => {
                    *session.borrow_mut() = State::Ready(Box::new(chat));
                    Ok(JsValue::from_f64((js_sys::Date::now() - started) / 1000.0))
                }
                Err(e) => {
                    *session.borrow_mut() = State::Unloaded;
                    Err(e)
                }
            }
        })
    }

    /// Send one message. `on_delta(text)` is called for each piece as it is
    /// generated; the promise resolves with the whole reply.
    pub fn send(&self, message: String, on_delta: Function) -> Promise {
        let session = self.session.clone();
        future_to_promise(async move {
            let mut chat = take_chat(&session)?;
            let reply = chat
                .send(message, |piece| {
                    let _ = on_delta.call1(&JsValue::NULL, &JsValue::from_str(piece));
                })
                .await;
            *session.borrow_mut() = State::Ready(chat);
            Ok(JsValue::from_str(&reply.map_err(to_js_error)?))
        })
    }

    /// Forget the conversation. Resolves to how many messages were dropped.
    pub fn reset(&self) -> Promise {
        let session = self.session.clone();
        future_to_promise(async move {
            let dropped = match &mut *session.borrow_mut() {
                State::Ready(chat) => chat.reset(),
                State::Unloaded => 0,
                State::Busy => return Err(to_js_error(BUSY)),
            };
            Ok(number(dropped as u64))
        })
    }

    /// Engine status, model name and history length.
    pub fn info(&self) -> Promise {
        let session = self.session.clone();
        future_to_promise(async move {
            let info = match &*session.borrow() {
                State::Unloaded => Info {
                    status: "unloaded",
                    model_name: None,
                    approx_memory: None,
                    history_length: 0,
                },
                State::Busy => Info {
                    status: "busy",
                    model_name: Some(MODEL.display_name),
                    approx_memory: Some(MODEL.approx_size),
                    history_length: 0,
                },
                State::Ready(chat) => Info {
                    status: "ready",
                    model_name: Some(MODEL.display_name),
                    approx_memory: Some(MODEL.approx_size),
                    history_length: chat.history_len() as u64,
                },
            };
            serde_wasm_bindgen::to_value(&info).map_err(to_js_error)
        })
    }
}

/// Reads the tokenizer and the model from the cache and starts a chat.
async fn load_chat(system_prompt: Option<String>, sampling: Sampling) -> Result<Chat, JsValue> {
    let tokenizer = store::load(&MODEL.tokenizer_url())
        .await
        .map_err(to_js_error)?;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer).map_err(to_js_error)?;
    let gguf = store::load(&MODEL.model_url()).await.map_err(to_js_error)?;
    let model = vectoron::Qwen2Model::load(gguf).map_err(to_js_error)?;
    Chat::new(model, tokenizer, system_prompt, sampling).map_err(to_js_error)
}
