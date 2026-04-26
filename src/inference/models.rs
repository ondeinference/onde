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

/// Pre-quantized Qwen 3 4B Instruct (GGUF Q4_K_M) — full OpenAI-compatible tool calling (~2.7 GB).
///
/// Qwen 3 uses an extended thinking mode (`<think>…</think>`) that significantly improves
/// reasoning and tool-use accuracy. Load with `max_tokens ≥ 4096` to avoid empty replies caused
/// by the model exhausting its token budget on thinking before producing a response.
///
/// Recommended model for siGit Code (coding agent with tool calling on macOS/Linux/Windows).
pub const BARTOWSKI_QWEN3_4B_GGUF: &str = "bartowski/Qwen_Qwen3-4B-GGUF";
/// The specific GGUF filename to download from the bartowski Qwen 3 4B repo.
pub const QWEN3_4B_GGUF_FILE: &str = "Qwen_Qwen3-4B-Q4_K_M.gguf";

/// Pre-quantized Qwen 3 8B (GGUF Q4_K_M) — strong tool-calling model (~5 GB).
///
/// Best balance of quality and memory for macOS with 24+ GB RAM.
/// Full tool calling and extended thinking mode support.
pub const BARTOWSKI_QWEN3_8B_GGUF: &str = "bartowski/Qwen_Qwen3-8B-GGUF";
pub const QWEN3_8B_GGUF_FILE: &str = "Qwen_Qwen3-8B-Q4_K_M.gguf";

/// Pre-quantized Qwen 3 1.7B (GGUF Q4_K_M) — lightweight tool-calling model (~1.3 GB).
///
/// Smallest Qwen 3 variant with tool calling support. Suitable for mobile devices
/// where the 4B model would be too large.
pub const BARTOWSKI_QWEN3_1_7B_GGUF: &str = "bartowski/Qwen_Qwen3-1.7B-GGUF";
/// The specific GGUF filename for the Qwen3 1.7B repo.
pub const QWEN3_1_7B_GGUF_FILE: &str = "Qwen_Qwen3-1.7B-Q4_K_M.gguf";

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
    BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF,
    BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF,
    BARTOWSKI_QWEN3_4B_GGUF,
    BARTOWSKI_QWEN3_8B_GGUF,
    BARTOWSKI_QWEN3_1_7B_GGUF,
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
        id: BARTOWSKI_QWEN3_1_7B_GGUF,
        name: "Qwen 3 1.7B (GGUF)",
        org: "Qwen / Alibaba",
        description: "Lightweight tool-calling model for mobile (~1.3 GB). \
             Smallest Qwen 3 variant with tool calling support.",
        expected_size_bytes: 1_282_439_584,
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
        BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF => Some(QWEN25_1_5B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF => Some(QWEN25_3B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_CODER_1_5B_INSTRUCT_GGUF => Some(QWEN25_CODER_1_5B_TOK_MODEL_ID),
        BARTOWSKI_QWEN25_CODER_3B_INSTRUCT_GGUF => Some(QWEN25_CODER_3B_TOK_MODEL_ID),
        THEBLOKE_DEEPSEEK_CODER_6_7B_INSTRUCT_GGUF => Some(DEEPSEEK_CODER_6_7B_TOK_MODEL_ID),
        _ => None,
    }
}
