// Copyright 2026 Seto Elkahfi (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Browser `ChatEngine` for `wasm32-unknown-unknown`.
//!
//! Same public signatures as the native engine, so hosts such as `ed-agent`
//! compile unchanged. Inference runs on the CPU through candle (build the host
//! with `RUSTFLAGS="-C target-feature=+simd128"` for speed), single-threaded,
//! so it needs no cross-origin isolation. Run it in a Web Worker: loading a
//! model and every token's forward pass block the thread they run on. Between
//! tokens the generation loop yields to the event loop, so messages (such as a
//! cancel) and dropped streams are noticed promptly.
//!
//! Supported: GGUF Qwen2.5 (`qwen2`) and Qwen3 (`qwen3`) models, with the
//! tokenizer loaded from `GgufModelConfig::tok_model_id`. Files come from
//! [`crate::web_store`]. Tool calling is not available on web.

use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::{quantized_qwen2, quantized_qwen3};
use tokenizers::Tokenizer;
use tokio::sync::mpsc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use super::engine::split_think;
use super::text::{apply_penalties, chatml_prompt};
use super::types::*;
use crate::web_store;

/// Tokens fed to the model per forward pass while reading the prompt. Small
/// enough that a long history doesn't freeze the worker between yields.
const PREFILL_CHUNK: usize = 64;

/// How many stream chunks may queue before the token loop waits for the reader.
const STREAM_BUFFER: usize = 64;

fn unsupported_on_web(what: &str) -> InferenceError {
    InferenceError::Other {
        reason: format!("{what} is not supported on web."),
    }
}

fn inference_err(e: impl std::fmt::Display) -> InferenceError {
    InferenceError::Inference {
        reason: e.to_string(),
    }
}

fn build_err(e: impl std::fmt::Display) -> InferenceError {
    InferenceError::ModelBuild {
        reason: e.to_string(),
    }
}

/// Resolve after the event loop has had a turn (a macrotask, not a microtask,
/// so queued `postMessage` events run).
async fn yield_to_event_loop() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let global = js_sys::global();
        if let Ok(set_timeout) = js_sys::Reflect::get(&global, &"setTimeout".into()) {
            if let Ok(set_timeout) = set_timeout.dyn_into::<js_sys::Function>() {
                let _ = set_timeout.call2(&global, &resolve, &JsValue::from(0));
                return;
            }
        }
        let _ = resolve.call0(&JsValue::UNDEFINED);
    });
    let _ = JsFuture::from(promise).await;
}

fn now_ms() -> f64 {
    js_sys::Date::now()
}

fn elapsed(start_ms: f64) -> std::time::Duration {
    std::time::Duration::from_secs_f64(((now_ms() - start_ms) / 1000.0).max(0.0))
}

// ── Model plumbing ───────────────────────────────────────────────────────────

enum Weights {
    Qwen2(quantized_qwen2::ModelWeights),
    Qwen3(quantized_qwen3::ModelWeights),
}

impl Weights {
    fn forward(&mut self, input: &Tensor, pos: usize) -> candle_core::Result<Tensor> {
        match self {
            Weights::Qwen2(m) => m.forward(input, pos),
            Weights::Qwen3(m) => m.forward(input, pos),
        }
    }

    fn clear_kv_cache(&mut self) {
        match self {
            Weights::Qwen2(m) => m.clear_kv_cache(),
            Weights::Qwen3(m) => m.clear_kv_cache(),
        }
    }
}

struct Loaded {
    weights: Weights,
    tokenizer: Tokenizer,
    config: GgufModelConfig,
    /// Token ids that end a reply.
    eos: Vec<u32>,
}

struct State {
    loaded: Option<Loaded>,
    system_prompt: Option<String>,
    sampling: SamplingConfig,
    history: Vec<ChatMessage>,
    busy: bool,
}

/// Marks the engine busy for as long as it lives, so a dropped or cancelled
/// generation always releases it.
struct BusyGuard(Rc<RefCell<State>>);

impl BusyGuard {
    fn acquire(state: &Rc<RefCell<State>>) -> Result<Self, InferenceError> {
        let mut st = state.borrow_mut();
        if st.loaded.is_none() {
            return Err(InferenceError::NoModelLoaded);
        }
        if st.busy {
            return Err(InferenceError::Inference {
                reason: "the engine is already generating".into(),
            });
        }
        st.busy = true;
        Ok(Self(state.clone()))
    }
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        // `try_borrow_mut`: a drop during a panic unwind must not double-panic.
        if let Ok(mut st) = self.0.try_borrow_mut() {
            st.busy = false;
        }
    }
}

fn sampling_for(s: &SamplingConfig) -> Sampling {
    let temperature = s.temperature.unwrap_or(0.7);
    if temperature <= 1e-7 {
        return Sampling::ArgMax;
    }
    match (s.top_k, s.top_p) {
        (Some(k), Some(p)) => Sampling::TopKThenTopP {
            k: k as usize,
            p,
            temperature,
        },
        (Some(k), None) => Sampling::TopK {
            k: k as usize,
            temperature,
        },
        (None, Some(p)) => Sampling::TopP { p, temperature },
        (None, None) => Sampling::All { temperature },
    }
}

/// Turns a stream of token ids into text without splitting multi-byte
/// characters: a piece is emitted only once it decodes cleanly.
struct TokenStream {
    tokens: Vec<u32>,
    prev_index: usize,
    current_index: usize,
}

impl TokenStream {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            prev_index: 0,
            current_index: 0,
        }
    }

    fn decode(tokenizer: &Tokenizer, ids: &[u32]) -> Result<String, InferenceError> {
        tokenizer.decode(ids, true).map_err(inference_err)
    }

    fn push(
        &mut self,
        tokenizer: &Tokenizer,
        token: u32,
    ) -> Result<Option<String>, InferenceError> {
        let prev_text = if self.tokens.is_empty() {
            String::new()
        } else {
            Self::decode(tokenizer, &self.tokens[self.prev_index..self.current_index])?
        };
        self.tokens.push(token);
        let text = Self::decode(tokenizer, &self.tokens[self.prev_index..])?;
        if text.len() > prev_text.len() && !text.ends_with('\u{FFFD}') {
            let piece = text.get(prev_text.len()..).unwrap_or_default().to_string();
            self.prev_index = self.current_index;
            self.current_index = self.tokens.len();
            Ok(Some(piece))
        } else {
            Ok(None)
        }
    }

    fn flush(&mut self, tokenizer: &Tokenizer) -> Result<Option<String>, InferenceError> {
        let prev_text = if self.tokens.is_empty() {
            String::new()
        } else {
            Self::decode(tokenizer, &self.tokens[self.prev_index..self.current_index])?
        };
        let text = Self::decode(tokenizer, &self.tokens[self.prev_index..])?;
        Ok(text
            .get(prev_text.len()..)
            .filter(|rest| !rest.is_empty())
            .map(str::to_string))
    }
}

struct Generated {
    text: String,
    prompt_tokens: usize,
    completion_tokens: usize,
    finish_reason: String,
    /// The reader dropped the stream before generation finished.
    cancelled: bool,
}

/// Run one generation. `tx` receives each text piece as it is decoded; if its
/// receiver is dropped, generation stops and `cancelled` is set.
async fn generate_text(
    state: &Rc<RefCell<State>>,
    prompt: &str,
    sampling: &SamplingConfig,
    tx: Option<&mpsc::Sender<StreamChunk>>,
) -> Result<Generated, InferenceError> {
    let prompt_ids: Vec<u32> = {
        let st = state.borrow();
        let loaded = st.loaded.as_ref().ok_or(InferenceError::NoModelLoaded)?;
        loaded
            .tokenizer
            .encode(prompt, false)
            .map_err(inference_err)?
            .get_ids()
            .to_vec()
    };
    if prompt_ids.is_empty() {
        return Err(inference_err("empty prompt"));
    }

    let max_tokens = sampling.max_tokens.unwrap_or(512) as usize;
    let seed = (now_ms() as u64) ^ ((js_sys::Math::random() * 4_294_967_296.0) as u64);
    let mut processor = LogitsProcessor::from_sampling(seed, sampling_for(sampling));
    let frequency = sampling.frequency_penalty.unwrap_or(0.0);
    let presence = sampling.presence_penalty.unwrap_or(0.0);

    // Prefill. The whole conversation is re-read each turn, so start from an
    // empty KV cache. Chunked, with a yield between chunks.
    state
        .borrow_mut()
        .loaded
        .as_mut()
        .ok_or(InferenceError::NoModelLoaded)?
        .weights
        .clear_kv_cache();
    yield_to_event_loop().await;

    let mut logits = None;
    let mut pos = 0usize;
    for chunk in prompt_ids.chunks(PREFILL_CHUNK) {
        if tx.is_some_and(|tx| tx.is_closed()) {
            return Ok(Generated {
                text: String::new(),
                prompt_tokens: prompt_ids.len(),
                completion_tokens: 0,
                finish_reason: "cancelled".into(),
                cancelled: true,
            });
        }
        let input = Tensor::new(chunk, &Device::Cpu)
            .and_then(|t| t.unsqueeze(0))
            .map_err(inference_err)?;
        let out = {
            let mut st = state.borrow_mut();
            let loaded = st.loaded.as_mut().ok_or(InferenceError::NoModelLoaded)?;
            loaded.weights.forward(&input, pos).map_err(inference_err)?
        };
        pos += chunk.len();
        logits = Some(out);
        yield_to_event_loop().await;
    }
    let mut logits = logits.ok_or_else(|| inference_err("empty prompt"))?;

    let mut stream = TokenStream::new();
    let mut text = String::new();
    let mut generated: Vec<u32> = Vec::new();
    let mut finish_reason = "length".to_string();
    let mut cancelled = false;

    for _ in 0..max_tokens {
        let mut row = logits.squeeze(0).map_err(inference_err)?;
        if frequency != 0.0 || presence != 0.0 {
            let mut values = row.to_vec1::<f32>().map_err(inference_err)?;
            apply_penalties(&mut values, &generated, frequency, presence);
            row = Tensor::from_vec(values.clone(), values.len(), &Device::Cpu)
                .map_err(inference_err)?;
        }
        let next = processor.sample(&row).map_err(inference_err)?;

        let (is_eos, piece) = {
            let st = state.borrow();
            let loaded = st.loaded.as_ref().ok_or(InferenceError::NoModelLoaded)?;
            if loaded.eos.contains(&next) {
                (true, stream.flush(&loaded.tokenizer)?)
            } else {
                (false, stream.push(&loaded.tokenizer, next)?)
            }
        };
        if let Some(piece) = piece {
            text.push_str(&piece);
            if let Some(tx) = tx {
                let chunk = StreamChunk {
                    delta: piece,
                    done: false,
                    finish_reason: None,
                };
                if tx.send(chunk).await.is_err() {
                    cancelled = true;
                    break;
                }
            }
        }
        if is_eos {
            finish_reason = "stop".into();
            break;
        }
        generated.push(next);

        yield_to_event_loop().await;
        if tx.is_some_and(|tx| tx.is_closed()) {
            cancelled = true;
            break;
        }

        let input = Tensor::new(&[next], &Device::Cpu)
            .and_then(|t| t.unsqueeze(0))
            .map_err(inference_err)?;
        logits = {
            let mut st = state.borrow_mut();
            let loaded = st.loaded.as_mut().ok_or(InferenceError::NoModelLoaded)?;
            loaded.weights.forward(&input, pos).map_err(inference_err)?
        };
        pos += 1;
    }

    if cancelled {
        finish_reason = "cancelled".into();
    } else if finish_reason == "length" {
        // Out of budget: release anything still held back as an incomplete
        // character.
        let piece = {
            let st = state.borrow();
            match st.loaded.as_ref() {
                Some(loaded) => stream.flush(&loaded.tokenizer)?,
                None => None,
            }
        };
        if let Some(piece) = piece {
            text.push_str(&piece);
            if let Some(tx) = tx {
                let _ = tx
                    .send(StreamChunk {
                        delta: piece,
                        done: false,
                        finish_reason: None,
                    })
                    .await;
            }
        }
    }

    Ok(Generated {
        text,
        prompt_tokens: prompt_ids.len(),
        completion_tokens: generated.len(),
        finish_reason,
        cancelled,
    })
}

// ═════════════════════════════════════════════════════════════════════════════
// ChatEngine
// ═════════════════════════════════════════════════════════════════════════════

/// On-device chat engine for the browser. See the [module docs](self).
///
/// Not `Send`: it lives on the thread (usually a Web Worker) that created it.
pub struct ChatEngine {
    state: Rc<RefCell<State>>,
}

impl ChatEngine {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(State {
                loaded: None,
                system_prompt: None,
                sampling: SamplingConfig::default(),
                history: Vec::new(),
                busy: false,
            })),
        }
    }

    /// Mirror of the native `with_app_id`. Telemetry is not sent from the
    /// browser, so the app id is dropped.
    pub fn with_app_id(_onde_app_id: Option<String>) -> Self {
        Self::new()
    }

    /// Mirror of the native `with_pulse_country`; no telemetry is sent.
    pub fn with_pulse_country(self, _country_code: Option<String>) -> Self {
        self
    }

    /// Load a GGUF Qwen2.5 or Qwen3 model, downloading it into the browser
    /// cache first if [`web_store::download_model`] hasn't already.
    ///
    /// `config.tok_model_id` is required: the tokenizer comes from that
    /// repository's `tokenizer.json`. The `qwen25_*` and `qwen3_*` presets set
    /// it on web.
    pub async fn load_gguf_model(
        &self,
        config: GgufModelConfig,
        system_prompt: Option<String>,
        sampling: Option<SamplingConfig>,
    ) -> Result<std::time::Duration, InferenceError> {
        let start = now_ms();
        let id = config.model_id.to_ascii_lowercase();
        let qwen3 = id.contains("qwen3");
        if !qwen3 && !id.contains("qwen2.5") {
            return Err(unsupported_on_web(&format!(
                "Loading {} (only Qwen2.5 and Qwen3 GGUF models run in the browser)",
                config.model_id
            )));
        }
        let tok_repo = config
            .tok_model_id
            .clone()
            .ok_or_else(|| build_err("tok_model_id is required on web"))?;
        let file = config
            .files
            .first()
            .ok_or_else(|| build_err("the model config lists no GGUF file"))?;

        let tokenizer_bytes = web_store::load_file(&tok_repo, web_store::TOKENIZER_FILE).await?;
        let tokenizer = Tokenizer::from_bytes(tokenizer_bytes).map_err(build_err)?;
        let gguf = web_store::load_file(&config.model_id, file).await?;

        // Parsing the file and copying its tensors blocks for a few seconds.
        yield_to_event_loop().await;
        let mut cursor = Cursor::new(gguf);
        let content = gguf_file::Content::read(&mut cursor).map_err(build_err)?;
        let weights = if qwen3 {
            Weights::Qwen3(
                quantized_qwen3::ModelWeights::from_gguf(content, &mut cursor, &Device::Cpu)
                    .map_err(build_err)?,
            )
        } else {
            Weights::Qwen2(
                quantized_qwen2::ModelWeights::from_gguf(content, &mut cursor, &Device::Cpu)
                    .map_err(build_err)?,
            )
        };
        drop(cursor);

        let eos: Vec<u32> = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .filter_map(|t| tokenizer.token_to_id(t))
            .collect();
        if eos.is_empty() {
            return Err(build_err("the tokenizer has no ChatML end-of-turn token"));
        }

        let sampling = sampling.unwrap_or_else(|| {
            if config.uses_extended_thinking() {
                SamplingConfig::reasoning()
            } else {
                SamplingConfig::default()
            }
        });
        let mut st = self.state.borrow_mut();
        st.loaded = Some(Loaded {
            weights,
            tokenizer,
            config,
            eos,
        });
        st.system_prompt = system_prompt;
        st.sampling = sampling;
        st.history.clear();
        Ok(elapsed(start))
    }

    pub async fn load_uqff_model(
        &self,
        _config: UqffModelConfig,
        _system_prompt: Option<String>,
        _sampling: Option<SamplingConfig>,
    ) -> Result<std::time::Duration, InferenceError> {
        Err(unsupported_on_web("UQFF models"))
    }

    pub async fn unload_model(&self) -> Option<String> {
        let mut st = self.state.borrow_mut();
        st.history.clear();
        st.loaded.take().map(|l| l.config.display_name)
    }

    pub async fn is_loaded(&self) -> bool {
        self.state.borrow().loaded.is_some()
    }

    pub async fn info(&self) -> EngineInfo {
        let st = self.state.borrow();
        let status = match (&st.loaded, st.busy) {
            (None, _) => EngineStatus::Unloaded,
            (Some(_), true) => EngineStatus::Generating,
            (Some(_), false) => EngineStatus::Ready,
        };
        EngineInfo {
            status,
            model_name: st.loaded.as_ref().map(|l| l.config.display_name.clone()),
            approx_memory: st.loaded.as_ref().map(|l| l.config.approx_memory.clone()),
            history_length: st.history.len() as u64,
        }
    }

    pub async fn set_system_prompt(&self, prompt: impl Into<String>) {
        self.state.borrow_mut().system_prompt = Some(prompt.into());
    }

    pub async fn clear_system_prompt(&self) {
        self.state.borrow_mut().system_prompt = None;
    }

    pub async fn set_sampling(&self, sampling: SamplingConfig) {
        self.state.borrow_mut().sampling = sampling;
    }

    pub async fn history(&self) -> Vec<ChatMessage> {
        self.state.borrow().history.clone()
    }

    pub async fn clear_history(&self) -> usize {
        let mut st = self.state.borrow_mut();
        let n = st.history.len();
        st.history.clear();
        n
    }

    pub async fn push_history(&self, message: ChatMessage) {
        self.state.borrow_mut().history.push(message);
    }

    /// Tool calling is not available on web.
    pub async fn tool_calling_support(&self) -> ToolCallingSupport {
        ToolCallingSupport::Unsupported
    }

    pub async fn record_tool_results(&self, _results: Vec<ToolResult>) -> usize {
        0
    }

    /// Prompt and sampling for the next turn. Called after the busy guard, which
    /// itself reports `NoModelLoaded` for an unloaded engine.
    fn prepare_turn(&self, user_message: &str) -> (String, SamplingConfig) {
        let st = self.state.borrow();
        let mut messages = st.history.clone();
        messages.push(ChatMessage::user(user_message));
        (
            chatml_prompt(st.system_prompt.as_deref(), &messages),
            st.sampling.clone(),
        )
    }

    /// Record a finished turn. Qwen3's reasoning block stays out of history.
    fn commit_turn(&self, user_message: String, reply: &str) {
        let (reply, _) = split_think(reply);
        let mut st = self.state.borrow_mut();
        st.history.push(ChatMessage::user(user_message));
        st.history.push(ChatMessage::assistant(reply));
    }

    pub async fn send_message(
        &self,
        user_message: impl Into<String>,
    ) -> Result<InferenceResult, InferenceError> {
        let user_message = user_message.into();
        let _busy = BusyGuard::acquire(&self.state)?;
        let (prompt, sampling) = self.prepare_turn(&user_message);
        let start = now_ms();
        let out = generate_text(&self.state, &prompt, &sampling, None).await?;
        self.commit_turn(user_message, &out.text);
        let duration = elapsed(start);
        Ok(InferenceResult {
            text: split_think(&out.text).0,
            duration_secs: duration.as_secs_f64(),
            duration_display: format_duration(duration),
            finish_reason: out.finish_reason,
            tool_calls: Vec::new(),
        })
    }

    /// Stateless generation over an explicit message list; history and the
    /// engine's system prompt are not used or changed.
    pub async fn generate(
        &self,
        messages: Vec<ChatMessage>,
        sampling: Option<SamplingConfig>,
    ) -> Result<InferenceResult, InferenceError> {
        let _busy = BusyGuard::acquire(&self.state)?;
        let sampling = sampling.unwrap_or_else(|| self.state.borrow().sampling.clone());
        let prompt = chatml_prompt(None, &messages);
        let start = now_ms();
        let out = generate_text(&self.state, &prompt, &sampling, None).await?;
        let duration = elapsed(start);
        Ok(InferenceResult {
            text: out.text,
            duration_secs: duration.as_secs_f64(),
            duration_display: format_duration(duration),
            finish_reason: out.finish_reason,
            tool_calls: Vec::new(),
        })
    }

    /// Stream a reply. Generation runs on a `spawn_local` task that yields to
    /// the event loop between tokens and stops when the receiver is dropped
    /// (history is then left unchanged). A failure mid-stream arrives as a
    /// final chunk with `finish_reason` starting `error:`.
    pub async fn stream_message(
        &self,
        user_message: impl Into<String>,
    ) -> Result<mpsc::Receiver<StreamChunk>, InferenceError> {
        let user_message = user_message.into();
        let busy = BusyGuard::acquire(&self.state)?;
        let (prompt, sampling) = self.prepare_turn(&user_message);
        let (tx, rx) = mpsc::channel(STREAM_BUFFER);
        let engine = ChatEngine {
            state: self.state.clone(),
        };
        spawn_local(async move {
            let _busy = busy;
            match generate_text(&engine.state, &prompt, &sampling, Some(&tx)).await {
                Ok(out) if out.cancelled => {}
                Ok(out) => {
                    engine.commit_turn(user_message, &out.text);
                    let _ = tx
                        .send(StreamChunk {
                            delta: String::new(),
                            done: true,
                            finish_reason: Some(out.finish_reason),
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(StreamChunk {
                            delta: String::new(),
                            done: true,
                            finish_reason: Some(format!("error: {e}")),
                        })
                        .await;
                }
            }
        });
        Ok(rx)
    }

    pub async fn send_message_with_tools(
        &self,
        _user_message: impl Into<String>,
        _tools: &[ToolDefinition],
    ) -> Result<ToolAwareResult, InferenceError> {
        Err(unsupported_on_web("Tool calling"))
    }

    pub async fn send_tool_results(
        &self,
        _results: Vec<ToolResult>,
        _next_tools: Option<&[ToolDefinition]>,
    ) -> Result<ToolAwareResult, InferenceError> {
        Err(unsupported_on_web("Tool calling"))
    }

    pub async fn stream_tool_results(
        &self,
        _results: Vec<ToolResult>,
        _tools: Option<Vec<ToolDefinition>>,
    ) -> Result<mpsc::Receiver<StreamChunk>, InferenceError> {
        Err(unsupported_on_web("Tool calling"))
    }

    /// One completion over explicit turns. Only plain system, user and
    /// assistant turns are accepted: there is no tool calling on web.
    pub async fn complete(
        &self,
        turns: Vec<ChatTurn>,
        tools: &[ToolDefinition],
        sampling: Option<SamplingConfig>,
    ) -> Result<CompletionResult, InferenceError> {
        if !tools.is_empty() {
            return Err(unsupported_on_web("Tool calling"));
        }
        let mut messages = Vec::with_capacity(turns.len());
        for turn in turns {
            messages.push(match turn {
                ChatTurn::System(content) => ChatMessage::system(content),
                ChatTurn::User(content) => ChatMessage::user(content),
                ChatTurn::Assistant {
                    content,
                    tool_calls,
                } if tool_calls.is_empty() => ChatMessage::assistant(content),
                _ => return Err(unsupported_on_web("Tool calling")),
            });
        }
        let _busy = BusyGuard::acquire(&self.state)?;
        let sampling = sampling.unwrap_or_else(|| self.state.borrow().sampling.clone());
        let prompt = chatml_prompt(None, &messages);
        let start = now_ms();
        let out = generate_text(&self.state, &prompt, &sampling, None).await?;
        let (text, reasoning) = split_think(&out.text);
        Ok(CompletionResult {
            text,
            reasoning,
            tool_calls: Vec::new(),
            finish_reason: out.finish_reason,
            prompt_tokens: out.prompt_tokens,
            completion_tokens: out.completion_tokens,
            duration_secs: elapsed(start).as_secs_f64(),
        })
    }
}

impl Default for ChatEngine {
    fn default() -> Self {
        Self::new()
    }
}
