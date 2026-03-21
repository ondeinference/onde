//! onde-wasm — Whisper speech-to-text compiled to WebAssembly.
//!
//! This crate exposes a single [`WhisperDecoder`] class to JavaScript via
//! `wasm-bindgen`.  It is backed by `candle-transformers`' pure-Rust Whisper
//! implementation, which compiles cleanly to `wasm32-unknown-unknown` without
//! any C FFI or platform intrinsics.
//!
//! # JavaScript API
//!
//! ```js
//! import init, { WhisperDecoder } from "./onde_wasm.js";
//!
//! await init();
//!
//! // All four byte arrays are fetched from HuggingFace (see build.sh).
//! const decoder = new WhisperDecoder(
//!   weightsBytes,    // model.safetensors  — or  model-tiny-en-q80.gguf
//!   tokenizerBytes,  // tokenizer.json
//!   melFiltersBytes, // mel_filters.safetensors
//!   configBytes,     // config.json
//!   quantized,       // bool — true for .gguf weights
//!   isMultilingual,  // bool — false for *.en models
//!   timestamps,      // bool — include per-segment timestamps
//!   task,            // "transcribe" | "translate" | null
//!   language,        // BCP-47 string | null  (null = auto-detect)
//! );
//!
//! // wavBytes is a Uint8Array of a 16 kHz mono WAV file.
//! const json = decoder.decode(wavBytes);
//!
//! // json is a string with the shape:
//! // {
//! //   "text": "Full transcript…",
//! //   "segments": [
//! //     { "start": 0.0, "end": 3.5, "text": " Hello world" },
//! //     …
//! //   ]
//! // }
//! const result = JSON.parse(json);
//! ```
//!
//! # Result JSON shape
//!
//! The JSON string returned by [`WhisperDecoder::decode`] mirrors the
//! `WhisperResult` / `WhisperSegment` types in `onde::whisper` so that the
//! same TypeScript interface can be reused across both the native Tauri app
//! and the browser connected-devices app.
//!
//! ```ts
//! interface OndeSegment { start: number; end: number; text: string; }
//! interface OndeResult  { text: string; segments: OndeSegment[]; }
//! ```
//!
//! # Model assets
//!
//! | File | Source |
//! |---|---|
//! | `model.safetensors` | `openai/whisper-tiny` (or tiny.en, base, …) |
//! | `model-tiny-en-q80.gguf` | `lmz/candle-whisper` (quantized) |
//! | `tokenizer.json` | same HF repo |
//! | `config.json` | same HF repo |
//! | `mel_filters.safetensors` | `huggingface/candle-whisper` space assets |
//!
//! # Build
//!
//! ```sh
//! wasm-pack build --target web --out-dir pkg
//! ```
//!
//! See `onde/crates/onde-wasm/build.sh` for the full reproducible build
//! script with asset download instructions.

use anyhow::anyhow;
use candle_core::{safetensors::Load, Device, IndexOp, Tensor, D};
use candle_nn::{ops::softmax, VarBuilder};
use candle_transformers::models::whisper::{self as m, Config};
use rand::{distr::Distribution, rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Panic hook
// ---------------------------------------------------------------------------

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ---------------------------------------------------------------------------
// Browser console bridge
// ---------------------------------------------------------------------------

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn console_warn(s: &str);
}

macro_rules! clog {
    ($($t:tt)*) => { console_log(&format!("[onde-wasm] {}", format_args!($($t)*))) }
}

macro_rules! cwarn {
    ($($t:tt)*) => { console_warn(&format!("[onde-wasm] {}", format_args!($($t)*))) }
}

// ---------------------------------------------------------------------------
// Language table
//
// Copied verbatim from candle-wasm-examples/whisper/src/languages.rs at
// rev c3bb5bf.  This constant is NOT re-exported by candle-transformers
// itself at that revision.
// ---------------------------------------------------------------------------

const LANGUAGES: [(&str, &str); 99] = [
    ("en", "english"),
    ("zh", "chinese"),
    ("de", "german"),
    ("es", "spanish"),
    ("ru", "russian"),
    ("ko", "korean"),
    ("fr", "french"),
    ("ja", "japanese"),
    ("pt", "portuguese"),
    ("tr", "turkish"),
    ("pl", "polish"),
    ("ca", "catalan"),
    ("nl", "dutch"),
    ("ar", "arabic"),
    ("sv", "swedish"),
    ("it", "italian"),
    ("id", "indonesian"),
    ("hi", "hindi"),
    ("fi", "finnish"),
    ("vi", "vietnamese"),
    ("he", "hebrew"),
    ("uk", "ukrainian"),
    ("el", "greek"),
    ("ms", "malay"),
    ("cs", "czech"),
    ("ro", "romanian"),
    ("da", "danish"),
    ("hu", "hungarian"),
    ("ta", "tamil"),
    ("no", "norwegian"),
    ("th", "thai"),
    ("ur", "urdu"),
    ("hr", "croatian"),
    ("bg", "bulgarian"),
    ("lt", "lithuanian"),
    ("la", "latin"),
    ("mi", "maori"),
    ("ml", "malayalam"),
    ("cy", "welsh"),
    ("sk", "slovak"),
    ("te", "telugu"),
    ("fa", "persian"),
    ("lv", "latvian"),
    ("bn", "bengali"),
    ("sr", "serbian"),
    ("az", "azerbaijani"),
    ("sl", "slovenian"),
    ("kn", "kannada"),
    ("et", "estonian"),
    ("mk", "macedonian"),
    ("br", "breton"),
    ("eu", "basque"),
    ("is", "icelandic"),
    ("hy", "armenian"),
    ("ne", "nepali"),
    ("mn", "mongolian"),
    ("bs", "bosnian"),
    ("kk", "kazakh"),
    ("sq", "albanian"),
    ("sw", "swahili"),
    ("gl", "galician"),
    ("mr", "marathi"),
    ("pa", "punjabi"),
    ("si", "sinhala"),
    ("km", "khmer"),
    ("sn", "shona"),
    ("yo", "yoruba"),
    ("so", "somali"),
    ("af", "afrikaans"),
    ("oc", "occitan"),
    ("ka", "georgian"),
    ("be", "belarusian"),
    ("tg", "tajik"),
    ("sd", "sindhi"),
    ("gu", "gujarati"),
    ("am", "amharic"),
    ("yi", "yiddish"),
    ("lo", "lao"),
    ("uz", "uzbek"),
    ("fo", "faroese"),
    ("ht", "haitian creole"),
    ("ps", "pashto"),
    ("tk", "turkmen"),
    ("nn", "nynorsk"),
    ("mt", "maltese"),
    ("sa", "sanskrit"),
    ("lb", "luxembourgish"),
    ("my", "myanmar"),
    ("bo", "tibetan"),
    ("tl", "tagalog"),
    ("mg", "malagasy"),
    ("as", "assamese"),
    ("tt", "tatar"),
    ("haw", "hawaiian"),
    ("ln", "lingala"),
    ("ha", "hausa"),
    ("ba", "bashkir"),
    ("jw", "javanese"),
    ("su", "sundanese"),
];

// ---------------------------------------------------------------------------
// Audio processing
//
// Ported from candle-wasm-examples/whisper/src/audio.rs (rev c3bb5bf).
// Pure-Rust FFT + mel-spectrogram — no C dependencies, WASM-safe.
// ---------------------------------------------------------------------------

trait Float: num_traits::Float + num_traits::FloatConst + num_traits::NumAssign {}
impl Float for f32 {}
impl Float for f64 {}

fn fft<T: Float>(inp: &[T]) -> Vec<T> {
    let n = inp.len();
    let zero = T::zero();
    if n == 1 {
        return vec![inp[0], zero];
    }
    if n % 2 == 1 {
        return dft(inp);
    }
    let mut out = vec![zero; n * 2];
    let mut even = Vec::with_capacity(n / 2);
    let mut odd = Vec::with_capacity(n / 2);
    for (i, &v) in inp.iter().enumerate() {
        if i % 2 == 0 { even.push(v) } else { odd.push(v) }
    }
    let even_fft = fft(&even);
    let odd_fft = fft(&odd);
    let two_pi = T::PI() + T::PI();
    let n_t = T::from(n).unwrap();
    for k in 0..n / 2 {
        let k_t = T::from(k).unwrap();
        let theta = two_pi * k_t / n_t;
        let re = theta.cos();
        let im = -theta.sin();
        let re_odd = odd_fft[2 * k];
        let im_odd = odd_fft[2 * k + 1];
        out[2 * k] = even_fft[2 * k] + re * re_odd - im * im_odd;
        out[2 * k + 1] = even_fft[2 * k + 1] + re * im_odd + im * re_odd;
        out[2 * (k + n / 2)] = even_fft[2 * k] - re * re_odd + im * im_odd;
        out[2 * (k + n / 2) + 1] = even_fft[2 * k + 1] - re * im_odd - im * re_odd;
    }
    out
}

fn dft<T: Float>(inp: &[T]) -> Vec<T> {
    let zero = T::zero();
    let n = inp.len();
    let two_pi = T::PI() + T::PI();
    let mut out = Vec::with_capacity(2 * n);
    let n_t = T::from(n).unwrap();
    for k in 0..n {
        let k_t = T::from(k).unwrap();
        let mut re = zero;
        let mut im = zero;
        for (j, &v) in inp.iter().enumerate() {
            let j_t = T::from(j).unwrap();
            let angle = two_pi * k_t * j_t / n_t;
            re += v * angle.cos();
            im -= v * angle.sin();
        }
        out.push(re);
        out.push(im);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn log_mel_spectrogram_w<T: Float>(
    ith: usize,
    hann: &[T],
    samples: &[T],
    filters: &[T],
    fft_size: usize,
    fft_step: usize,
    speed_up: bool,
    n_len: usize,
    n_mel: usize,
    n_threads: usize,
) -> Vec<T> {
    let n_fft = if speed_up { 1 + fft_size / 4 } else { 1 + fft_size / 2 };
    let zero = T::zero();
    let half = T::from(0.5).unwrap();
    let mut fft_in = vec![zero; fft_size];
    let mut mel = vec![zero; n_len * n_mel];
    for i in (ith..n_len).step_by(n_threads) {
        let offset = i * fft_step;
        for j in 0..fft_size {
            fft_in[j] = if offset + j < samples.len() {
                hann[j] * samples[offset + j]
            } else {
                zero
            };
        }
        let mut fft_out: Vec<T> = fft(&fft_in);
        for j in 0..fft_size {
            fft_out[j] = fft_out[2 * j] * fft_out[2 * j]
                + fft_out[2 * j + 1] * fft_out[2 * j + 1];
        }
        for j in 1..fft_size / 2 {
            let v = fft_out[fft_size - j];
            fft_out[j] += v;
        }
        if speed_up {
            for j in 0..n_fft {
                fft_out[j] = half * (fft_out[2 * j] + fft_out[2 * j + 1]);
            }
        }
        for j in 0..n_mel {
            let mut sum = zero;
            for k in 0..n_fft {
                sum += fft_out[k] * filters[j * n_fft + k];
            }
            mel[j * n_len + i] = T::max(sum, T::from(1e-10).unwrap()).log10();
        }
    }
    mel
}

fn log_mel_spectrogram_<T: Float>(
    samples: &[T],
    filters: &[T],
    fft_size: usize,
    fft_step: usize,
    n_mel: usize,
    speed_up: bool,
) -> Vec<T> {
    let zero = T::zero();
    let two_pi = T::PI() + T::PI();
    let half = T::from(0.5).unwrap();
    let one = T::from(1.0).unwrap();
    let four = T::from(4.0).unwrap();
    let fft_size_t = T::from(fft_size).unwrap();
    let hann: Vec<T> = (0..fft_size)
        .map(|i| half * (one - ((two_pi * T::from(i).unwrap()) / fft_size_t).cos()))
        .collect();
    let n_len = samples.len() / fft_step;
    let pad = 100 * m::CHUNK_LENGTH / 2;
    let n_len = if n_len % pad != 0 {
        (n_len / pad + 1) * pad
    } else {
        n_len
    };
    let n_len = n_len + pad;
    let samples = {
        let mut padded = samples.to_vec();
        let to_add = n_len * fft_step - samples.len();
        padded.extend(std::iter::repeat_n(zero, to_add));
        padded
    };
    let mut mel = log_mel_spectrogram_w(
        0, &hann, &samples, filters, fft_size, fft_step, speed_up, n_len, n_mel, 1,
    );
    let mmax = mel
        .iter()
        .max_by(|u, v| u.partial_cmp(v).unwrap_or(std::cmp::Ordering::Greater))
        .copied()
        .unwrap_or(zero)
        - T::from(8).unwrap();
    for m in mel.iter_mut() {
        let v = T::max(*m, mmax);
        *m = v / four + one;
    }
    mel
}

fn pcm_to_mel(cfg: &Config, samples: &[f32], filters: &[f32]) -> anyhow::Result<Vec<f32>> {
    Ok(log_mel_spectrogram_(
        samples,
        filters,
        m::N_FFT,
        m::HOP_LENGTH,
        cfg.num_mel_bins,
        false,
    ))
}

// ---------------------------------------------------------------------------
// Model enum — wraps normal (f32 safetensors) and quantized (GGUF) variants
// ---------------------------------------------------------------------------

enum Model {
    Normal(m::model::Whisper),
    Quantized(m::quantized_model::Whisper),
}

impl Model {
    fn config(&self) -> &Config {
        match self {
            Self::Normal(m) => &m.config,
            Self::Quantized(m) => &m.config,
        }
    }

    fn encoder_forward(&mut self, x: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.encoder.forward(x, flush),
            Self::Quantized(m) => m.encoder.forward(x, flush),
        }
    }

    fn decoder_forward(
        &mut self,
        x: &Tensor,
        xa: &Tensor,
        flush: bool,
    ) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.decoder.forward(x, xa, flush),
            Self::Quantized(m) => m.decoder.forward(x, xa, flush),
        }
    }

    fn decoder_final_linear(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.decoder.final_linear(x),
            Self::Quantized(m) => m.decoder.final_linear(x),
        }
    }
}

// ---------------------------------------------------------------------------
// Token helpers
// ---------------------------------------------------------------------------

fn token_id(tokenizer: &Tokenizer, token: &str) -> anyhow::Result<u32> {
    tokenizer
        .token_to_id(token)
        .ok_or_else(|| anyhow!("no token-id for {token}"))
}

fn detect_language(
    model: &mut Model,
    tokenizer: &Tokenizer,
    mel: &Tensor,
) -> anyhow::Result<u32> {
    clog!("detecting language");
    let (_bsize, _, seq_len) = mel.dims3()?;
    let mel = mel.narrow(
        2,
        0,
        usize::min(seq_len, model.config().max_source_positions),
    )?;
    let device = mel.device();
    let language_token_ids = LANGUAGES
        .iter()
        .map(|(t, _)| token_id(tokenizer, &format!("<|{t}|>")))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let sot = token_id(tokenizer, m::SOT_TOKEN)?;
    let audio_features = model.encoder_forward(&mel, true)?;
    let tokens = Tensor::new(&[[sot]], device)?;
    let language_token_ids_t = Tensor::new(language_token_ids.as_slice(), device)?;
    let ys = model.decoder_forward(&tokens, &audio_features, true)?;
    let logits = model.decoder_final_linear(&ys.i(..1)?)?.i(0)?.i(0)?;
    let logits = logits.index_select(&language_token_ids_t, 0)?;
    let probs = softmax(&logits, D::Minus1)?.to_vec1::<f32>()?;
    let mut ranked: Vec<_> = LANGUAGES.iter().zip(probs.iter()).collect();
    ranked.sort_by(|(_, p1), (_, p2)| p2.total_cmp(p1));
    let best_code = ranked[0].0 .0;
    let token_str = format!("<|{best_code}|>");
    let language_id = token_id(tokenizer, &token_str)?;
    clog!("detected language: {best_code} ({token_str})");
    Ok(language_id)
}

// ---------------------------------------------------------------------------
// Internal decoding state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct DecodingResult {
    tokens: Vec<u32>,
    text: String,
    avg_logprob: f64,
    no_speech_prob: f64,
    compression_ratio: f64,
}

struct Decoder {
    model: Model,
    rng: StdRng,
    tokenizer: Tokenizer,
    mel_filters: Vec<f32>,
    language: Option<String>,
    is_multilingual: bool,
    timestamps: bool,
    task_tokens: Vec<u32>,  // [task_token] or [task_token, no_timestamps_token]
    suppress_tokens: Tensor,
    sot_token: u32,
    eot_token: u32,
    no_speech_token: u32,
    no_timestamps_token: u32,
}

impl Decoder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: Model,
        tokenizer: Tokenizer,
        mel_filters: Vec<f32>,
        device: &Device,
        task: Option<&str>,
        language: Option<String>,
        is_multilingual: bool,
        timestamps: bool,
    ) -> anyhow::Result<Self> {
        let suppress_tokens_vec: Vec<f32> = (0..model.config().vocab_size as u32)
            .map(|i| {
                if model.config().suppress_tokens.contains(&i) {
                    f32::NEG_INFINITY
                } else {
                    0f32
                }
            })
            .collect();
        let suppress_tokens = Tensor::new(suppress_tokens_vec.as_slice(), device)?;

        let sot_token = token_id(&tokenizer, m::SOT_TOKEN)?;
        let transcribe_token = token_id(&tokenizer, m::TRANSCRIBE_TOKEN)?;
        let translate_token = token_id(&tokenizer, m::TRANSLATE_TOKEN)?;
        let eot_token = token_id(&tokenizer, m::EOT_TOKEN)?;
        let no_timestamps_token = token_id(&tokenizer, m::NO_TIMESTAMPS_TOKEN)?;
        let no_speech_token = m::NO_SPEECH_TOKENS
            .iter()
            .find_map(|t| token_id(&tokenizer, t).ok())
            .ok_or_else(|| anyhow!("no no-speech token found"))?;

        let task_token = match task {
            Some("translate") => translate_token,
            _ => transcribe_token,
        };

        let mut task_tokens = vec![task_token];
        if !timestamps {
            task_tokens.push(no_timestamps_token);
        }

        Ok(Self {
            model,
            rng: StdRng::seed_from_u64(299_792_458),
            tokenizer,
            mel_filters,
            language,
            is_multilingual,
            timestamps,
            task_tokens,
            suppress_tokens,
            sot_token,
            eot_token,
            no_speech_token,
            no_timestamps_token,
        })
    }

    fn decode(&mut self, mel: &Tensor, temperature: f64) -> anyhow::Result<DecodingResult> {
        let language_token = match (self.is_multilingual, &self.language) {
            (true, None) => Some(detect_language(&mut self.model, &self.tokenizer, mel)?),
            (false, None) => None,
            (true, Some(lang)) => {
                let tok = format!("<|{lang}|>");
                Some(token_id(&self.tokenizer, &tok).map_err(|_| {
                    anyhow!("language '{lang}' is not supported by this model")
                })?)
            }
            (false, Some(_)) => {
                anyhow::bail!("language cannot be set for non-multilingual models")
            }
        };

        let audio_features = self.model.encoder_forward(mel, true)?;
        let sample_len = self.model.config().max_target_positions / 2;

        // Build the initial prompt token sequence:
        //   [SOT, (language), task, (no_timestamps)]
        let mut tokens = vec![self.sot_token];
        if let Some(lang_tok) = language_token {
            tokens.push(lang_tok);
        }
        tokens.extend_from_slice(&self.task_tokens);

        let mut sum_logprob = 0f64;
        let mut no_speech_prob = f64::NAN;

        for i in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), mel.device())?.unsqueeze(0)?;
            let ys = self.model.decoder_forward(&tokens_t, &audio_features, i == 0)?;

            // Extract no-speech probability from the first decoder step.
            if i == 0 {
                let logits = self.model.decoder_final_linear(&ys.i(..1)?)?.i(0)?.i(0)?;
                no_speech_prob = softmax(&logits, 0)?
                    .i(self.no_speech_token as usize)?
                    .to_scalar::<f32>()? as f64;
            }

            let (_, seq_len, _) = ys.dims3()?;
            let logits = self
                .model
                .decoder_final_linear(&ys.i((..1, seq_len - 1..))?)?
                .i(0)?
                .i(0)?;
            let logits = logits.broadcast_add(&self.suppress_tokens)?;

            let next_token = if temperature > 0.0 {
                let prs = softmax(&(&logits / temperature)?, 0)?;
                let logits_v: Vec<f32> = prs.to_vec1()?;
                let distr = rand::distr::weighted::WeightedIndex::new(&logits_v)?;
                distr.sample(&mut self.rng) as u32
            } else {
                let logits_v: Vec<f32> = logits.to_vec1()?;
                logits_v
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.total_cmp(b))
                    .map(|(i, _)| i as u32)
                    .unwrap_or(self.eot_token)
            };

            if next_token == self.eot_token
                || tokens.len() > self.model.config().max_target_positions
            {
                break;
            }

            let prob = softmax(&logits, D::Minus1)?
                .i(next_token as usize)?
                .to_scalar::<f32>()? as f64;
            sum_logprob += prob.ln();
            tokens.push(next_token);
        }

        let text = self
            .tokenizer
            .decode(&tokens, true)
            .map_err(|e| anyhow!("{e}"))?;
        let avg_logprob = sum_logprob / tokens.len().max(1) as f64;

        Ok(DecodingResult {
            tokens,
            text,
            avg_logprob,
            no_speech_prob,
            compression_ratio: f64::NAN,
        })
    }

    fn decode_with_fallback(&mut self, segment: &Tensor) -> anyhow::Result<DecodingResult> {
        for (i, &t) in m::TEMPERATURES.iter().enumerate() {
            let result = self.decode(segment, t);
            if i == m::TEMPERATURES.len() - 1 {
                return result;
            }
            match result {
                Ok(dr) => {
                    let needs_fallback = dr.compression_ratio > m::COMPRESSION_RATIO_THRESHOLD
                        || dr.avg_logprob < m::LOGPROB_THRESHOLD;
                    if !needs_fallback || dr.no_speech_prob > m::NO_SPEECH_THRESHOLD {
                        return Ok(dr);
                    }
                }
                Err(e) => cwarn!("decode at temperature {t} failed: {e}"),
            }
        }
        unreachable!()
    }

    fn run(&mut self, mel: &Tensor) -> anyhow::Result<Vec<Segment>> {
        let (_, _, content_frames) = mel.dims3()?;
        let mut seek = 0;
        let mut segments = vec![];
        while seek < content_frames {
            let time_offset = (seek * m::HOP_LENGTH) as f64 / m::SAMPLE_RATE as f64;
            let segment_size = usize::min(content_frames - seek, m::N_FRAMES);
            let mel_segment = mel.narrow(2, seek, segment_size)?;
            let segment_duration =
                (segment_size * m::HOP_LENGTH) as f64 / m::SAMPLE_RATE as f64;
            let dr = self.decode_with_fallback(&mel_segment)?;
            seek += segment_size;
            if dr.no_speech_prob > m::NO_SPEECH_THRESHOLD
                && dr.avg_logprob < m::LOGPROB_THRESHOLD
            {
                clog!("skipping silent segment at seek={seek}");
                continue;
            }
            clog!("segment [{:.1}s] {}", time_offset, dr.text.trim());
            segments.push(Segment {
                start: time_offset,
                end: time_offset + segment_duration,
                dr,
            });
        }
        Ok(segments)
    }
}

// ---------------------------------------------------------------------------
// Segment — intermediate result per 30-second chunk
// ---------------------------------------------------------------------------

struct Segment {
    start: f64,
    end: f64,
    dr: DecodingResult,
}

// ---------------------------------------------------------------------------
// Public JSON output types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct OndeSegment {
    start: f64,
    end: f64,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct OndeResult {
    text: String,
    segments: Vec<OndeSegment>,
}

// ---------------------------------------------------------------------------
// WhisperDecoder — the public wasm-bindgen export
// ---------------------------------------------------------------------------

/// Whisper decoder bound to a pre-loaded set of model weights.
///
/// Construct once, then call `decode` for each audio clip.
#[wasm_bindgen]
pub struct WhisperDecoder {
    decoder: Decoder,
}

#[wasm_bindgen]
impl WhisperDecoder {
    /// Construct a new `WhisperDecoder` from raw byte buffers.
    ///
    /// All four byte slices must stay alive for the duration of this call;
    /// they are consumed (weights are moved into the model, everything else
    /// is copied into small owned structures) and the originals are no
    /// longer needed once the constructor returns.
    ///
    /// # Arguments
    ///
    /// - `weights`     — `model.safetensors` bytes, OR `.gguf` bytes when `quantized = true`
    /// - `tokenizer`   — `tokenizer.json` bytes
    /// - `mel_filters` — `mel_filters.safetensors` bytes
    /// - `config`      — `config.json` bytes
    /// - `quantized`   — `true` when `weights` is a `.gguf` file
    /// - `is_multilingual` — `false` for `*.en` models
    /// - `timestamps`  — include per-segment timestamps in the output
    /// - `task`        — `"transcribe"` | `"translate"` | `null`
    /// - `language`    — BCP-47 code (`"id"`, `"en"`, …) or `null` for auto-detect
    #[wasm_bindgen(constructor)]
    pub fn new(
        weights: &[u8],
        tokenizer: &[u8],
        mel_filters: &[u8],
        config: &[u8],
        quantized: bool,
        is_multilingual: bool,
        timestamps: bool,
        task: Option<String>,
        language: Option<String>,
    ) -> Result<WhisperDecoder, JsError> {
        set_panic_hook();
        Self::build(
            weights,
            tokenizer,
            mel_filters,
            config,
            quantized,
            is_multilingual,
            timestamps,
            task.as_deref(),
            language,
        )
        .map_err(|e| JsError::new(&e.to_string()))
    }

    fn build(
        weights: &[u8],
        tokenizer_bytes: &[u8],
        mel_filters_bytes: &[u8],
        config_bytes: &[u8],
        quantized: bool,
        is_multilingual: bool,
        timestamps: bool,
        task: Option<&str>,
        language: Option<String>,
    ) -> anyhow::Result<WhisperDecoder> {
        let device = Device::Cpu;

        // Tokenizer
        let tokenizer = Tokenizer::from_bytes(tokenizer_bytes)
            .map_err(|e| anyhow!("tokenizer load failed: {e}"))?;
        clog!("tokenizer loaded");

        // Mel filters  — shape (n_mels, n_fft/2+1) stored as "mel_80" tensor
        let mel_st = safetensors::SafeTensors::deserialize(mel_filters_bytes)
            .map_err(|e| anyhow!("mel_filters load failed: {e}"))?;
        let mel_tensor = mel_st
            .tensor("mel_80")
            .map_err(|e| anyhow!("mel_80 tensor missing: {e}"))?
            .load(&device)?;
        let mel_filters = mel_tensor.flatten_all()?.to_vec1::<f32>()?;
        clog!("mel filters loaded: {:?}", mel_tensor.shape());

        // Config
        let config: Config = serde_json::from_slice(config_bytes)
            .map_err(|e| anyhow!("config.json parse failed: {e}"))?;

        // Model weights
        let model = if quantized {
            let vb =
                candle_transformers::quantized_var_builder::VarBuilder::from_gguf_buffer(
                    weights,
                    &device,
                )?;
            Model::Quantized(m::quantized_model::Whisper::load(&vb, config)?)
        } else {
            let vb = VarBuilder::from_buffered_safetensors(
                weights.to_vec(),
                m::DTYPE,
                &device,
            )?;
            Model::Normal(m::model::Whisper::load(&vb, config)?)
        };
        clog!("model loaded (quantized={quantized})");

        let decoder = Decoder::new(
            model,
            tokenizer,
            mel_filters,
            &device,
            task,
            language,
            is_multilingual,
            timestamps,
        )?;

        Ok(WhisperDecoder { decoder })
    }

    /// Decode a 16 kHz mono PCM WAV file.
    ///
    /// Returns a JSON string with shape `{ text, segments: [{start, end, text}] }`.
    pub fn decode(&mut self, wav_bytes: &[u8]) -> Result<String, JsError> {
        self.decode_inner(wav_bytes)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    fn decode_inner(&mut self, wav_bytes: &[u8]) -> anyhow::Result<String> {
        let device = Device::Cpu;

        // Decode WAV
        let mut cursor = std::io::Cursor::new(wav_bytes);
        let reader = hound::WavReader::new(&mut cursor)?;
        let spec = reader.spec();
        clog!("wav: {spec:?}");
        if spec.sample_rate != m::SAMPLE_RATE as u32 {
            anyhow::bail!(
                "expected {}Hz WAV, got {}Hz — please resample before passing to decode()",
                m::SAMPLE_RATE,
                spec.sample_rate
            );
        }

        // Read samples, take only the first channel if stereo
        let raw: Vec<_> = reader.into_samples::<i16>().collect::<hound::Result<Vec<_>>>()?;
        let mono: Vec<f32> = raw
            .chunks(spec.channels as usize)
            .map(|ch| ch[0] as f32 / 32768.0)
            .collect();
        clog!("pcm samples: {}", mono.len());

        // Build mel spectrogram tensor (shape: [1, n_mels, n_frames])
        let cfg = self.decoder.model.config();
        let mel = pcm_to_mel(cfg, &mono, &self.decoder.mel_filters)?;
        let mel_len = mel.len();
        let n_mels = cfg.num_mel_bins;
        let mel_t = Tensor::from_vec(mel, (1, n_mels, mel_len / n_mels), &device)?;
        clog!("mel tensor: {:?}", mel_t.dims());

        // Run the sliding-window decoder
        let segments = self.decoder.run(&mel_t)?;

        // Build output
        let mut full_text = String::new();
        let mut out_segments = Vec::with_capacity(segments.len());
        for seg in &segments {
            if !full_text.is_empty() && !seg.dr.text.starts_with(' ') {
                full_text.push(' ');
            }
            full_text.push_str(seg.dr.text.trim());
            out_segments.push(OndeSegment {
                start: seg.start,
                end: seg.end,
                text: seg.dr.text.trim().to_string(),
            });
        }

        let result = OndeResult {
            text: full_text,
            segments: out_segments,
        };
        Ok(serde_json::to_string(&result)?)
    }
}
