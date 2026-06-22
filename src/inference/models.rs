/// Pre-quantized Qwen 2.5 1.5B Instruct (GGUF Q4_K_M) — lightest mobile option (~941 MB).
/// Fits comfortably on both iOS (iPhone 16e, 8 GB RAM) and Android memory-constrained devices.
/// Note: the 3B variant (~1.93 GB) caused OOM on iPhone 16e because iOS gives apps only ~2-3 GB;
/// the 1.5B variant at ~941 MB leaves comfortable headroom for KV cache, activations, and the app.
/// bartowski's GGUF embeds the full tokenizer and Qwen2.5 chat template, so on iOS/macOS no
/// separate tok_model_id download is needed. On Android an explicit tok_model_id is required.
pub const BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF: &str = "bartowski/Qwen2.5-1.5B-Instruct-GGUF";
/// The specific GGUF filename to download from the bartowski 1.5B repo.
pub const QWEN25_1_5B_GGUF_FILE: &str = "Qwen2.5-1.5B-Instruct-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
/// On Android, GGUF loading requires an explicit tokenizer source; on iOS/macOS the tokenizer
/// embedded in the GGUF file is used instead to avoid an extra network download.
pub const QWEN25_1_5B_TOK_MODEL_ID: &str = "Qwen/Qwen2.5-1.5B-Instruct";

/// Pre-quantized Qwen 2.5 0.5B Instruct (GGUF Q4_K_M), smallest model (~380 MB).
/// Used on tvOS (Apple TV 4K) where the hard memory limit is 2 GB.
pub const BARTOWSKI_QWEN25_0_5B_INSTRUCT_GGUF: &str = "bartowski/Qwen2.5-0.5B-Instruct-GGUF";
/// The specific GGUF filename to download from the bartowski 0.5B repo.
pub const QWEN25_0_5B_GGUF_FILE: &str = "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN25_0_5B_TOK_MODEL_ID: &str = "Qwen/Qwen2.5-0.5B-Instruct";

/// Pre-quantized Qwen 2.5 Coder 1.5B Instruct (GGUF Q4_K_M) — dedicated coding model (~941 MB).
///
/// Uses the `qwen2` GGUF architecture, identical to Qwen2.5-1.5B-Instruct, so it loads through
/// the existing `quantized_qwen.rs` path in mistral.rs without any code changes.
///
/// Strongly preferred over the general-purpose 1.5B for coding tasks: trained on 5.5T tokens of
/// code and math data, with fill-in-the-middle (FIM) and repo-level code understanding.
/// Same memory footprint as the general 1.5B (~941 MB) but dramatically better at code.
///
/// bartowski's GGUF embeds the full tokenizer and chat template, so on iOS/macOS no separate
/// tok_model_id download is needed. On Android an explicit tok_model_id is required.
pub const BARTOWSKI_QWEN25_CODER_1_5B_INSTRUCT_GGUF: &str =
    "bartowski/Qwen2.5-Coder-1.5B-Instruct-GGUF";
/// The specific GGUF filename to download from the bartowski Coder 1.5B repo.
pub const QWEN25_CODER_1_5B_GGUF_FILE: &str = "Qwen2.5-Coder-1.5B-Instruct-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN25_CODER_1_5B_TOK_MODEL_ID: &str = "Qwen/Qwen2.5-Coder-1.5B-Instruct";

/// HuggingFace repo for the full-precision Qwen 2.5 Coder 7B Instruct model.
///
/// Used by ISQ pipelines (`TextModelBuilder`) which download the safetensors weights
/// directly and quantise them in-situ on the device.  Unlike the bartowski GGUF
/// variants, this repo ships the original BF16 weights; mistral.rs handles
/// quantisation to Q4K or Q8_0 at load time via the `--isq` flag.
///
/// Requires ~8 GB RAM during the load phase (4-bit); ~12 GB for 8-bit.
/// Metal-accelerated on macOS; CPU fallback available but very slow.
pub const QWEN25_CODER_7B_INSTRUCT: &str = "Qwen/Qwen2.5-Coder-7B-Instruct";

/// Pre-quantized Qwen 2.5 Coder 3B Instruct (GGUF Q4_K_M) — best coding quality on macOS (~1.93 GB).
///
/// Same `qwen2` architecture as the 3B general model; ideal for macOS desktops where the extra
/// quality headroom over the 1.5B is worthwhile. Not recommended for iOS (OOM risk).
pub const BARTOWSKI_QWEN25_CODER_3B_INSTRUCT_GGUF: &str =
    "bartowski/Qwen2.5-Coder-3B-Instruct-GGUF";
/// The specific GGUF filename to download from the bartowski Coder 3B repo.
pub const QWEN25_CODER_3B_GGUF_FILE: &str = "Qwen2.5-Coder-3B-Instruct-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN25_CODER_3B_TOK_MODEL_ID: &str = "Qwen/Qwen2.5-Coder-3B-Instruct";

/// Pre-quantized Qwen 2.5 Coder 7B Instruct (GGUF Q4_K_M) — strong coding model with tool calling (~4.4 GB).
///
/// Uses the `qwen2` GGUF architecture. Trained on 5.5T tokens of code/math data with
/// fill-in-the-middle and repo-level code understanding. Supports function/tool calling
/// via the Qwen2.5 chat template. Requires ~8 GB RAM.
pub const BARTOWSKI_QWEN25_CODER_7B_INSTRUCT_GGUF: &str =
    "bartowski/Qwen2.5-Coder-7B-Instruct-GGUF";
/// The specific GGUF filename to download from the bartowski Coder 7B repo.
pub const QWEN25_CODER_7B_GGUF_FILE: &str = "Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN25_CODER_7B_TOK_MODEL_ID: &str = "Qwen/Qwen2.5-Coder-7B-Instruct";

/// Pre-quantized Qwen 2.5 3B Instruct (GGUF Q4_K_M) — balanced option (~1.93 GB).
/// Ideal for macOS desktops and Android devices with sufficient RAM.
/// Not recommended as default on iOS: the 3B variant caused OOM on iPhone 16e (8 GB RAM)
/// because iOS gives apps only ~2-3 GB; use the 1.5B variant for iOS instead.
/// No in-situ quantization needed; loads directly at quantized size.
pub const BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF: &str = "bartowski/Qwen2.5-3B-Instruct-GGUF";
/// The specific GGUF filename to download from the bartowski 3B repo.
pub const QWEN25_3B_GGUF_FILE: &str = "Qwen2.5-3B-Instruct-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer (tokenizer.json + tokenizer_config.json).
pub const QWEN25_3B_TOK_MODEL_ID: &str = "Qwen/Qwen2.5-3B-Instruct";

// ── Qwen 3 family (GGUF Q4_K_M) ──────────────────────────────────────────────
//
// The Qwen 3 line uses the `qwen3` GGUF architecture (and `qwen3moe` for the
// 30B-A3B mixture-of-experts variant), both supported by mistral.rs's quantized
// loader. Every Qwen 3 model is a hybrid reasoner with an extended thinking mode
// (`<think>…</think>`); always load with `max_tokens ≥ 4096` so the thinking
// block does not exhaust the token budget before the real reply.
//
// bartowski's repos embed the tokenizer and chat template inside the GGUF, so on
// iOS/macOS no separate tokenizer download is needed. On Android the candle GGUF
// backend cannot parse the embedded tokenizer, so each model also declares a
// `TOK_MODEL_ID` pointing at the official Qwen repo for a standalone tokenizer.

/// Pre-quantized Qwen 3 0.6B (GGUF Q4_K_M) — smallest Qwen 3 variant (~0.5 GB).
///
/// Lightest tool-capable Qwen 3 model. Suitable for tvOS and the most
/// memory-constrained mobile devices where even the 1.7B is too large.
pub const BARTOWSKI_QWEN3_0_6B_GGUF: &str = "bartowski/Qwen_Qwen3-0.6B-GGUF";
/// The specific GGUF filename for the bartowski Qwen 3 0.6B repo.
pub const QWEN3_0_6B_GGUF_FILE: &str = "Qwen_Qwen3-0.6B-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_0_6B_TOK_MODEL_ID: &str = "Qwen/Qwen3-0.6B";

/// Pre-quantized Qwen 3 1.7B (GGUF Q4_K_M) — lightweight tool-calling model (~1.3 GB).
///
/// Smallest Qwen 3 variant with comfortable tool calling. Suitable for mobile
/// devices where the 4B model would be too large.
pub const BARTOWSKI_QWEN3_1_7B_GGUF: &str = "bartowski/Qwen_Qwen3-1.7B-GGUF";
/// The specific GGUF filename for the Qwen3 1.7B repo.
pub const QWEN3_1_7B_GGUF_FILE: &str = "Qwen_Qwen3-1.7B-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_1_7B_TOK_MODEL_ID: &str = "Qwen/Qwen3-1.7B";

/// Pre-quantized Qwen 3 4B (GGUF Q4_K_M) — full OpenAI-compatible tool calling (~2.7 GB).
///
/// Recommended model for siGit Code (coding agent with tool calling on macOS/Linux/Windows).
pub const BARTOWSKI_QWEN3_4B_GGUF: &str = "bartowski/Qwen_Qwen3-4B-GGUF";
/// The specific GGUF filename to download from the bartowski Qwen 3 4B repo.
pub const QWEN3_4B_GGUF_FILE: &str = "Qwen_Qwen3-4B-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_4B_TOK_MODEL_ID: &str = "Qwen/Qwen3-4B";

/// Pre-quantized Qwen 3 8B (GGUF Q4_K_M) — strong tool-calling model (~5 GB).
///
/// Best balance of quality and memory for macOS with 24+ GB RAM.
/// Full tool calling and extended thinking mode support.
pub const BARTOWSKI_QWEN3_8B_GGUF: &str = "bartowski/Qwen_Qwen3-8B-GGUF";
pub const QWEN3_8B_GGUF_FILE: &str = "Qwen_Qwen3-8B-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_8B_TOK_MODEL_ID: &str = "Qwen/Qwen3-8B";

/// Pre-quantized Qwen 3 14B (GGUF Q4_K_M) — strong reasoning and tool-calling model (~8.4 GB).
///
/// Best all-around model for macOS with 16+ GB RAM. Full tool calling support.
pub const BARTOWSKI_QWEN3_14B_GGUF: &str = "bartowski/Qwen_Qwen3-14B-GGUF";
/// The specific GGUF filename.
pub const QWEN3_14B_GGUF_FILE: &str = "Qwen_Qwen3-14B-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_14B_TOK_MODEL_ID: &str = "Qwen/Qwen3-14B";

/// Pre-quantized Qwen 3 32B (GGUF Q4_K_M) — largest dense Qwen 3 model (~19.8 GB).
///
/// Highest-quality dense Qwen 3 variant. Requires a high-memory desktop
/// (32+ GB RAM / unified memory). Full tool calling and extended thinking.
pub const BARTOWSKI_QWEN3_32B_GGUF: &str = "bartowski/Qwen_Qwen3-32B-GGUF";
/// The specific GGUF filename for the bartowski Qwen 3 32B repo.
pub const QWEN3_32B_GGUF_FILE: &str = "Qwen_Qwen3-32B-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_32B_TOK_MODEL_ID: &str = "Qwen/Qwen3-32B";

// ── Qwen 3 "2507" updated releases ───────────────────────────────────────────
//
// The July/August 2025 refresh split Qwen 3 into dedicated non-thinking
// (`-Instruct-2507`) and thinking-only (`-Thinking-2507`) checkpoints with
// markedly improved instruction following, tool use, and long-context quality.
// These are the latest open Qwen 3 weights available as on-device GGUF.

/// Pre-quantized Qwen 3 4B Instruct 2507 (GGUF Q4_K_M) — latest non-thinking 4B (~2.5 GB).
///
/// Updated instruction-tuned checkpoint. Unlike the base 4B it does *not* emit a
/// `<think>` block, so it is faster and more predictable for chat and tool use.
pub const BARTOWSKI_QWEN3_4B_INSTRUCT_2507_GGUF: &str =
    "bartowski/Qwen_Qwen3-4B-Instruct-2507-GGUF";
/// The specific GGUF filename for the bartowski Qwen 3 4B Instruct 2507 repo.
pub const QWEN3_4B_INSTRUCT_2507_GGUF_FILE: &str = "Qwen_Qwen3-4B-Instruct-2507-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_4B_INSTRUCT_2507_TOK_MODEL_ID: &str = "Qwen/Qwen3-4B-Instruct-2507";

/// Pre-quantized Qwen 3 4B Thinking 2507 (GGUF Q4_K_M) — latest reasoning 4B (~2.5 GB).
///
/// Updated thinking-only checkpoint with stronger reasoning and tool-use accuracy.
/// Always emits a `<think>` block — load with `max_tokens ≥ 4096`.
pub const BARTOWSKI_QWEN3_4B_THINKING_2507_GGUF: &str =
    "bartowski/Qwen_Qwen3-4B-Thinking-2507-GGUF";
/// The specific GGUF filename for the bartowski Qwen 3 4B Thinking 2507 repo.
pub const QWEN3_4B_THINKING_2507_GGUF_FILE: &str = "Qwen_Qwen3-4B-Thinking-2507-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_4B_THINKING_2507_TOK_MODEL_ID: &str = "Qwen/Qwen3-4B-Thinking-2507";

/// Pre-quantized Qwen 3 30B-A3B Instruct 2507 (GGUF Q4_K_M) — flagship MoE (~18.6 GB).
///
/// Mixture-of-experts model: 30B total parameters but only ~3B active per token,
/// so inference is far cheaper than a 30B dense model while quality rivals it.
/// Loads via the `qwen3moe` GGUF architecture in mistral.rs. Requires a
/// high-memory desktop (32+ GB RAM / unified memory).
pub const BARTOWSKI_QWEN3_30B_A3B_INSTRUCT_2507_GGUF: &str =
    "bartowski/Qwen_Qwen3-30B-A3B-Instruct-2507-GGUF";
/// The specific GGUF filename for the bartowski Qwen 3 30B-A3B Instruct 2507 repo.
pub const QWEN3_30B_A3B_INSTRUCT_2507_GGUF_FILE: &str =
    "Qwen_Qwen3-30B-A3B-Instruct-2507-Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const QWEN3_30B_A3B_INSTRUCT_2507_TOK_MODEL_ID: &str = "Qwen/Qwen3-30B-A3B-Instruct-2507";

/// DeepSeek Coder v1 6.7B Instruct (GGUF Q4_K_M) — dedicated code generation model (~3.8 GB).
///
/// Uses the `llama` GGUF architecture, so it loads through the existing
/// `quantized_llama.rs` path in mistral.rs without any code changes.
/// Strong coding performance. Requires ~8 GB RAM.
pub const THEBLOKE_DEEPSEEK_CODER_6_7B_INSTRUCT_GGUF: &str =
    "TheBloke/deepseek-coder-6.7B-instruct-GGUF";
/// The specific GGUF filename.
pub const DEEPSEEK_CODER_6_7B_GGUF_FILE: &str = "deepseek-coder-6.7b-instruct.Q4_K_M.gguf";
/// Base model repo used for the HF tokenizer on Android.
pub const DEEPSEEK_CODER_6_7B_TOK_MODEL_ID: &str = "deepseek-ai/deepseek-coder-6.7b-instruct";

/// All model IDs that the Onde inference engine supports.
/// Used by `list_local_hf_models` to filter the HuggingFace cache
/// to only show models that can actually be used for generation.
pub const SUPPORTED_MODELS: &[&str] = &[
    BARTOWSKI_QWEN25_0_5B_INSTRUCT_GGUF,
    BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF,
    BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF,
    BARTOWSKI_QWEN3_0_6B_GGUF,
    BARTOWSKI_QWEN3_1_7B_GGUF,
    BARTOWSKI_QWEN3_4B_GGUF,
    BARTOWSKI_QWEN3_8B_GGUF,
    BARTOWSKI_QWEN3_14B_GGUF,
    BARTOWSKI_QWEN3_32B_GGUF,
    BARTOWSKI_QWEN3_4B_INSTRUCT_2507_GGUF,
    BARTOWSKI_QWEN3_4B_THINKING_2507_GGUF,
    BARTOWSKI_QWEN3_30B_A3B_INSTRUCT_2507_GGUF,
    BARTOWSKI_QWEN25_CODER_7B_INSTRUCT_GGUF,
    THEBLOKE_DEEPSEEK_CODER_6_7B_INSTRUCT_GGUF,
];

/// Rich metadata for a supported model, used by the frontend to display
/// unavailable models that can be downloaded.
pub struct SupportedModelInfo {
    /// The full HuggingFace model identifier, e.g. "bartowski/Qwen2.5-1.5B-Instruct-GGUF".
    pub id: &'static str,
    /// Human-friendly display name for the model.
    pub name: &'static str,
    /// Organisation or publisher display name.
    pub org: &'static str,
    /// Short description of the model's purpose / capabilities.
    pub description: &'static str,
    /// Approximate total size in bytes when fully downloaded.
    /// Used by the progress monitor to estimate download percentage.
    ///
    /// These values are computed by summing every file in the model's
    /// HuggingFace repository (via the `/api/models/{id}?blobs=true`
    /// endpoint's `siblings[].size` fields).
    pub expected_size_bytes: u64,
}

/// Complete list of supported models with display metadata.
///
/// When adding a new model, add its constant ID to [`SUPPORTED_MODELS`] **and**
/// a corresponding entry here so the frontend can show it in the model list UI.
pub const SUPPORTED_MODEL_INFO: &[SupportedModelInfo] = &[
    SupportedModelInfo {
        id: BARTOWSKI_QWEN25_0_5B_INSTRUCT_GGUF,
        name: "Qwen 2.5 0.5B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Smallest pre-quantized chat model, used on tvOS Apple TV (~380 MB).",
        expected_size_bytes: 397_808_192,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF,
        name: "Qwen 2.5 1.5B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Lightest pre-quantized chat model — ideal for iOS & Android (~941 MB). \
             Fits comfortably within iOS memory limits (iPhone 16e, 8 GB RAM).",
        // Qwen2.5-1.5B-Instruct-Q4_K_M.gguf from bartowski repo.
        // Exact file size from HuggingFace API siblings[].size.
        expected_size_bytes: 986_048_768,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF,
        name: "Qwen 2.5 3B (GGUF)",
        org: "Qwen / Alibaba",
        description:
            "Pre-quantized chat model for macOS & Android — balanced quality and size (~1.93 GB). \
             Not recommended as default on iOS due to memory constraints.",
        // Qwen2.5-3B-Instruct-Q4_K_M.gguf from bartowski repo.
        // Exact file size from HuggingFace API siblings[].size.
        expected_size_bytes: 1_929_903_264,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_0_6B_GGUF,
        name: "Qwen 3 0.6B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Smallest Qwen 3 variant with tool calling (~0.5 GB). \
             Suitable for tvOS and the most memory-constrained mobile devices.",
        // Approximate Q4_K_M size; HF API unreachable at authoring time.
        expected_size_bytes: 483_000_000,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_1_7B_GGUF,
        name: "Qwen 3 1.7B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Lightweight tool-calling model for mobile (~1.3 GB). \
             Smallest Qwen 3 variant with comfortable tool calling support.",
        expected_size_bytes: 1_282_439_584,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_4B_GGUF,
        name: "Qwen 3 4B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Full tool-calling support with extended reasoning mode (~2.7 GB). \
                      Recommended for siGit Code on macOS, Linux, and Windows.",
        // Qwen_Qwen3-4B-Q4_K_M.gguf from bartowski repo.
        expected_size_bytes: 2_596_306_912,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_8B_GGUF,
        name: "Qwen 3 8B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Strong tool-calling model with extended thinking (~5 GB). \
                      Best balance of quality and memory for macOS with 24+ GB RAM.",
        expected_size_bytes: 5_131_567_104,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_14B_GGUF,
        name: "Qwen 3 14B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Strong reasoning and tool-calling model with extended thinking (~8.4 GB). \
             Best all-around model for macOS with 16+ GB RAM.",
        // Qwen_Qwen3-14B-Q4_K_M.gguf from bartowski repo.
        // Exact file size from HuggingFace API siblings[].size.
        expected_size_bytes: 9_001_753_632,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_32B_GGUF,
        name: "Qwen 3 32B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Largest dense Qwen 3 model with extended thinking (~19.8 GB). \
             Highest-quality dense variant; requires 32+ GB RAM.",
        // Approximate Q4_K_M size; HF API unreachable at authoring time.
        expected_size_bytes: 21_260_000_000,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_4B_INSTRUCT_2507_GGUF,
        name: "Qwen 3 4B Instruct 2507 (GGUF)",
        org: "Qwen / Alibaba",
        description: "Latest non-thinking 4B checkpoint — faster, predictable chat and \
             tool use (~2.5 GB). Recommended general-purpose Qwen 3 model.",
        // Approximate Q4_K_M size; HF API unreachable at authoring time.
        expected_size_bytes: 2_500_000_000,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_4B_THINKING_2507_GGUF,
        name: "Qwen 3 4B Thinking 2507 (GGUF)",
        org: "Qwen / Alibaba",
        description: "Latest reasoning-focused 4B checkpoint with extended thinking (~2.5 GB). \
             Stronger reasoning and tool-use accuracy; load with max_tokens ≥ 4096.",
        // Approximate Q4_K_M size; HF API unreachable at authoring time.
        expected_size_bytes: 2_500_000_000,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN3_30B_A3B_INSTRUCT_2507_GGUF,
        name: "Qwen 3 30B-A3B Instruct 2507 (GGUF)",
        org: "Qwen / Alibaba",
        description: "Flagship mixture-of-experts model: 30B total / ~3B active (~18.6 GB). \
             Near-dense quality at far lower inference cost; requires 32+ GB RAM.",
        // Approximate Q4_K_M size; HF API unreachable at authoring time.
        expected_size_bytes: 19_971_600_000,
    },
    SupportedModelInfo {
        id: BARTOWSKI_QWEN25_CODER_7B_INSTRUCT_GGUF,
        name: "Qwen 2.5 Coder 7B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Strong coding model with tool calling support (~4.4 GB). \
             Trained on 5.5T code tokens. Requires 8+ GB RAM.",
        expected_size_bytes: 4_683_074_336,
    },
    SupportedModelInfo {
        id: THEBLOKE_DEEPSEEK_CODER_6_7B_INSTRUCT_GGUF,
        name: "DeepSeek Coder 6.7B (GGUF)",
        org: "DeepSeek AI",
        description: "Strong code generation model using the llama architecture (~3.8 GB). \
             Requires 8+ GB RAM. Not recommended for mobile devices.",
        expected_size_bytes: 4_083_015_904,
    },
];

/// Return the explicit tokenizer model ID required on Android for `hf_repo_id`.
///
/// The candle GGUF backend cannot parse the tokenizer embedded inside GGUF
/// files; an explicit `tok_model_id` triggers a separate tokenizer download
/// from the base model repo. Returns `None` on iOS and macOS where the
/// embedded tokenizer is used automatically.
pub fn tok_model_id_for_repo(hf_repo_id: &str) -> Option<&'static str> {
    match hf_repo_id {
        BARTOWSKI_QWEN25_0_5B_INSTRUCT_GGUF => Some(QWEN25_0_5B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF => Some(QWEN25_1_5B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF => Some(QWEN25_3B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_CODER_1_5B_INSTRUCT_GGUF => Some(QWEN25_CODER_1_5B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_CODER_3B_INSTRUCT_GGUF => Some(QWEN25_CODER_3B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_CODER_7B_INSTRUCT_GGUF => Some(QWEN25_CODER_7B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_0_6B_GGUF => Some(QWEN3_0_6B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_1_7B_GGUF => Some(QWEN3_1_7B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_4B_GGUF => Some(QWEN3_4B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_8B_GGUF => Some(QWEN3_8B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_14B_GGUF => Some(QWEN3_14B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_32B_GGUF => Some(QWEN3_32B_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_4B_INSTRUCT_2507_GGUF => Some(QWEN3_4B_INSTRUCT_2507_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_4B_THINKING_2507_GGUF => Some(QWEN3_4B_THINKING_2507_TOK_MODEL_ID),
        BARTOWSKI_QWEN3_30B_A3B_INSTRUCT_2507_GGUF => {
            Some(QWEN3_30B_A3B_INSTRUCT_2507_TOK_MODEL_ID)
        }
        THEBLOKE_DEEPSEEK_CODER_6_7B_INSTRUCT_GGUF => Some(DEEPSEEK_CODER_6_7B_TOK_MODEL_ID),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every supported model must have a corresponding display-metadata entry
    /// so the model-list UI can render it.
    #[test]
    fn every_supported_model_has_info() {
        for id in SUPPORTED_MODELS {
            assert!(
                SUPPORTED_MODEL_INFO.iter().any(|info| info.id == *id),
                "{id} is in SUPPORTED_MODELS but missing from SUPPORTED_MODEL_INFO"
            );
        }
    }

    /// Every Qwen 3 repo must resolve to an Android tokenizer ID so loading
    /// works on the candle GGUF backend.
    #[test]
    fn qwen3_repos_have_android_tokenizer() {
        let qwen3 = SUPPORTED_MODELS.iter().filter(|id| id.contains("Qwen3"));
        for id in qwen3 {
            assert!(
                tok_model_id_for_repo(id).is_some(),
                "{id} has no Android tokenizer mapping"
            );
        }
    }
}
