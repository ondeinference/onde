//! Spike harness: honest CPU baseline for the onde web engine.
//!
//! Mirrors `generate_text` in onde `src/inference/web/mod.rs` (PREFILL_CHUNK
//! loop + decode loop) but times prefill, time-to-first-token and decode
//! separately. Works natively (CPU or Metal) and on wasm32-unknown-unknown.

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_qwen2::ModelWeights;
use serde::Serialize;
use std::future::Future;
use std::io::Cursor;
use tokenizers::Tokenizer;

pub const PREFILL_CHUNK: usize = 64;
/// Default: drain the GPU queue after every decode token. CI varies this
/// (1, 8, 32) to see how much decode speed the drain costs.
pub const DECODE_DRAIN_EVERY: usize = 1;

/// Wait until all submitted GPU work finished and let the fork recycle its
/// retained buffers. On wasm WebGPU has no blocking wait, so without this the
/// fork keeps every in-flight buffer alive and memory grows without bound
/// (see the 2026-11-17 incident). No-op for non-wgpu devices.
pub async fn drain(device: &Device) -> Result<(), String> {
    if device.is_wgpu() {
        device
            .as_wgpu_device()
            .map_err(|e| e.to_string())?
            .synchronize_async()
            .await
            .map_err(|e| format!("synchronize_async: {e}"))?;
    }
    Ok(())
}

#[derive(Serialize, Clone, Debug)]
pub struct RunResult {
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub prefill_ms: f64,
    /// prefill + sampling the first token.
    pub ttft_ms: f64,
    /// Time from first token sampled to last token sampled.
    pub decode_ms: f64,
    /// (generated - 1) / decode_ms: steady-state decode speed.
    pub decode_tps: f64,
    /// generated / total turn time: the ed#62 style figure.
    pub turn_tps: f64,
    pub total_ms: f64,
    pub yielded: bool,
    pub tokens: Vec<u32>,
    pub text: String,
    /// Top-5 (token, logit) of the very first sampled position, to tell a
    /// near-tie flip from a wrong backend.
    pub first_top5: Vec<(u32, f32)>,
}

fn argmax(v: &[f32]) -> u32 {
    let mut best = 0usize;
    for (i, x) in v.iter().enumerate() {
        if *x > v[best] {
            best = i;
        }
    }
    best as u32
}

pub fn chatml(system: &str, user: &str) -> String {
    format!(
        "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
    )
}

pub fn load(gguf: &[u8], device: &Device) -> candle_core::Result<ModelWeights> {
    let mut cur = Cursor::new(gguf);
    let content = gguf_file::Content::read(&mut cur)?;
    ModelWeights::from_gguf(content, &mut cur, device)
}

/// One generation. `now` returns milliseconds. `yield_now` is awaited between
/// prefill chunks and between decode tokens when `do_yield` is set (the engine
/// does this; native has nothing to yield to).
pub async fn run<F, Fut>(
    model: &mut ModelWeights,
    tokenizer: &Tokenizer,
    device: &Device,
    prompt: &str,
    max_new: usize,
    now: &dyn Fn() -> f64,
    do_yield: bool,
    drain_every: usize,
    yield_now: F,
) -> Result<RunResult, String>
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let e = |x: candle_core::Error| x.to_string();
    let ids: Vec<u32> = tokenizer
        .encode(prompt, false)
        .map_err(|x| x.to_string())?
        .get_ids()
        .to_vec();
    let mut processor = ();

    model.clear_kv_cache();
    let t0 = now();
    if do_yield {
        yield_now().await;
    }
    let mut logits = None;
    let mut pos = 0usize;
    for chunk in ids.chunks(PREFILL_CHUNK) {
        let input = Tensor::new(chunk, device)
            .and_then(|t| t.unsqueeze(0))
            .map_err(e)?;
        let out = model.forward(&input, pos).map_err(e)?;
        pos += chunk.len();
        logits = Some(out);
        drain(device).await?;
        if do_yield {
            yield_now().await;
        }
    }
    let mut logits = logits.ok_or("empty prompt")?;
    // Force completion of any queued GPU work and time prefill honestly.
    let mut row = logits.squeeze(0).map_err(e)?;
    let _ = &mut processor;
    let first_vec = row
        .to_dtype(candle_core::DType::F32)
        .map_err(e)?
        .to_vec1_async::<f32>()
        .await
        .map_err(e)?;
    let mut next = argmax(&first_vec);
    let first_top5 = {
        let mut v: Vec<(u32, f32)> = first_vec
            .into_iter()
            .into_iter()
            .enumerate()
            .map(|(i, x)| (i as u32, x))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v.truncate(5);
        v
    };
    let t_first = now();
    let prefill_ms = t_first - t0; // includes sampling token 1 (sync point)

    let mut generated = vec![next];
    while generated.len() < max_new {
        if do_yield {
            yield_now().await;
        }
        let input = Tensor::new(&[next], device)
            .and_then(|t| t.unsqueeze(0))
            .map_err(e)?;
        logits = model.forward(&input, pos).map_err(e)?;
        pos += 1;
        if generated.len() % drain_every.max(1) == 0 {
            drain(device).await?;
        }
        row = logits.squeeze(0).map_err(e)?;
        next = row
            .argmax(0)
            .map_err(e)?
            .to_scalar_async::<u32>()
            .await
            .map_err(e)?;
        generated.push(next);
    }
    let t_end = now();
    let decode_ms = t_end - t_first;
    let total_ms = t_end - t0;
    let n = generated.len();
    Ok(RunResult {
        prompt_tokens: ids.len(),
        generated_tokens: n,
        prefill_ms,
        ttft_ms: prefill_ms,
        decode_ms,
        decode_tps: if n > 1 { (n - 1) as f64 / (decode_ms / 1000.0) } else { 0.0 },
        turn_tps: n as f64 / (total_ms / 1000.0),
        total_ms,
        yielded: do_yield,
        text: tokenizer.decode(&generated, true).unwrap_or_default(),
        tokens: generated,
        first_top5,
    })
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use wasm_bindgen::prelude::*;

    fn perf_now() -> f64 {
        js_sys::Reflect::get(&js_sys::global(), &"performance".into())
            .ok()
            .and_then(|p| {
                let f = js_sys::Reflect::get(&p, &"now".into()).ok()?;
                f.dyn_into::<js_sys::Function>()
                    .ok()?
                    .call0(&p)
                    .ok()?
                    .as_f64()
            })
            .unwrap_or_else(js_sys::Date::now)
    }

    async fn yield_to_event_loop() {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let g = js_sys::global();
            if let Ok(st) = js_sys::Reflect::get(&g, &"setTimeout".into()) {
                if let Ok(st) = st.dyn_into::<js_sys::Function>() {
                    let _ = st.call2(&g, &resolve, &JsValue::from(0));
                    return;
                }
            }
            let _ = resolve.call0(&JsValue::UNDEFINED);
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }

    #[wasm_bindgen]
    pub struct GpuBench {
        model: ModelWeights,
        tokenizer: Tokenizer,
        device: Device,
    }

    #[wasm_bindgen]
    impl GpuBench {
        /// Returns the model; load time is measured by the caller too.
        pub async fn load(gguf: Vec<u8>, tokenizer_json: Vec<u8>) -> Result<GpuBench, JsValue> {
            console_error_panic_hook::set_once();
            let tokenizer = Tokenizer::from_bytes(&tokenizer_json)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let device = Device::new_wgpu_async(0)
                .await
                .map_err(|e| JsValue::from_str(&format!("new_wgpu_async: {e}")))?;
            let model = load(&gguf, &device).map_err(|e| JsValue::from_str(&format!("load: {e}")))?;
            super::drain(&device).await.map_err(|e| JsValue::from_str(&e))?;
            Ok(GpuBench { model, tokenizer, device })
        }

        /// Run one generation, return JSON of RunResult.
        pub async fn run(
            &mut self,
            prompt: String,
            max_new: usize,
            do_yield: bool,
            drain_every: usize,
        ) -> Result<String, JsValue> {
            let r = super::run(
                &mut self.model,
                &self.tokenizer,
                &self.device,
                &prompt,
                max_new,
                &perf_now,
                do_yield,
                drain_every,
                yield_to_event_loop,
            )
            .await
            .map_err(|e| JsValue::from_str(&e))?;
            serde_json::to_string(&r).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}
