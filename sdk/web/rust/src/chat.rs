//! A chat session over one vectoron model: the conversation, its ChatML
//! prompt, and streaming generation.

use serde::Deserialize;
use tokenizers::Tokenizer;
use vectoron::{Qwen2Model, Sampler, SamplerConfig};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Prompt tokens run per forward pass. Between chunks the worker yields, so
/// a long prompt does not hold its event loop for seconds.
const PREFILL_CHUNK: usize = 64;
const MAX_TOKENS: usize = 512;

/// Sampling settings from JavaScript. Missing fields keep vectoron's
/// defaults; a `temperature` of 0 is greedy.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sampling {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    /// A fixed seed makes replies repeatable. Without one each reply differs.
    pub seed: Option<u64>,
    pub max_tokens: Option<usize>,
}

impl Sampling {
    fn config(&self) -> SamplerConfig {
        let defaults = SamplerConfig::default();
        SamplerConfig {
            temperature: self.temperature.unwrap_or(defaults.temperature),
            top_p: self.top_p.or(defaults.top_p),
            top_k: self.top_k.or(defaults.top_k),
            seed: self.seed.unwrap_or_else(seed),
            ..defaults
        }
    }
}

#[derive(Clone, Copy)]
enum Role {
    User,
    Assistant,
}

pub struct Chat {
    model: Qwen2Model,
    tokenizer: Tokenizer,
    eos: Vec<u32>,
    system: Option<String>,
    sampling: Sampling,
    history: Vec<(Role, String)>,
    /// The tokens whose keys and values are in the model's cache, so the
    /// next turn only runs what changed.
    cached: Vec<u32>,
}

impl Chat {
    pub fn new(
        model: Qwen2Model,
        tokenizer: Tokenizer,
        system: Option<String>,
        sampling: Sampling,
    ) -> Result<Self, String> {
        let eos: Vec<u32> = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .filter_map(|t| tokenizer.token_to_id(t))
            .collect();
        if eos.is_empty() {
            return Err("the tokenizer has no end-of-turn token".into());
        }
        Ok(Self {
            model,
            tokenizer,
            eos,
            system,
            sampling,
            history: Vec::new(),
            cached: Vec::new(),
        })
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Forgets the conversation. Returns how many messages were dropped.
    pub fn reset(&mut self) -> usize {
        let dropped = self.history.len();
        self.history.clear();
        self.cached.clear();
        self.model.clear_kv_cache();
        dropped
    }

    /// The conversation as a ChatML prompt ending in an open assistant turn.
    fn prompt(&self) -> String {
        let mut out = String::new();
        let mut turn = |role: &str, content: &str| {
            out.push_str("<|im_start|>");
            out.push_str(role);
            out.push('\n');
            out.push_str(content);
            out.push_str("<|im_end|>\n");
        };
        if let Some(system) = &self.system {
            turn("system", system);
        }
        for (role, content) in &self.history {
            let role = match role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            turn(role, content);
        }
        out.push_str("<|im_start|>assistant\n");
        out
    }

    /// Answers `message`, calling `on_piece` with each new piece of text.
    /// Returns the whole reply, which is added to the conversation.
    pub async fn send(
        &mut self,
        message: String,
        on_piece: impl FnMut(&str),
    ) -> Result<String, String> {
        self.history.push((Role::User, message));
        match self.generate(on_piece).await {
            Ok(reply) => {
                self.history.push((Role::Assistant, reply.clone()));
                Ok(reply)
            }
            Err(e) => {
                self.history.pop();
                Err(e)
            }
        }
    }

    async fn generate(&mut self, mut on_piece: impl FnMut(&str)) -> Result<String, String> {
        let ids = self
            .tokenizer
            .encode(self.prompt(), false)
            .map_err(|e| e.to_string())?
            .get_ids()
            .to_vec();

        // Reuse the cached positions the new prompt shares with the last one.
        let mut start = self
            .cached
            .iter()
            .zip(&ids)
            .take_while(|(a, b)| a == b)
            .count();
        if start == ids.len() {
            start -= 1;
        }
        // `cached` mirrors the model's cache, which forward() truncates to
        // `start` before running a chunk.
        self.cached.truncate(start);
        let mut logits = Vec::new();
        for chunk in ids[start..].chunks(PREFILL_CHUNK) {
            logits = self
                .model
                .forward(chunk, start)
                .map_err(|e| e.to_string())?;
            self.cached.extend_from_slice(chunk);
            start += chunk.len();
            yield_to_event_loop().await;
        }

        let mut sampler = Sampler::new(self.sampling.config());
        let max_tokens = self.sampling.max_tokens.unwrap_or(MAX_TOKENS);
        let mut generated: Vec<u32> = Vec::new();
        let mut shown = 0;
        let mut reply = String::new();
        for _ in 0..max_tokens {
            let next = sampler.sample(&mut logits, &generated);
            if self.eos.contains(&next) {
                break;
            }
            generated.push(next);
            let text = self
                .tokenizer
                .decode(&generated, true)
                .map_err(|e| e.to_string())?;
            // Hold back a piece that ends inside a multi-byte character.
            if !text.ends_with('\u{fffd}') && text.len() > shown {
                on_piece(&text[shown..]);
                shown = text.len();
                reply = text;
            }
            if self.model.cache_len() + 1 >= self.model.config().context_length {
                break;
            }
            logits = self
                .model
                .forward(&[next], self.model.cache_len())
                .map_err(|e| e.to_string())?;
            self.cached.push(next);
            yield_to_event_loop().await;
        }
        Ok(reply)
    }
}

/// A seed that differs between replies.
fn seed() -> u64 {
    let now = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * u32::MAX as f64) as u64;
    now ^ (random << 20)
}

/// Lets the worker's event loop run (a `setTimeout(0)` macrotask).
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
