import OndeInferenceModule from "./OndeInferenceModule";
import type {
  ChatMessage,
  ChatRole,
  EngineInfo,
  GgufModelConfig,
  InferenceResult,
  SamplingConfig,
  StreamChunk,
  ToolCallInfo,
} from "./types";
import { OndeError } from "./types";

// Re-export all types
export type {
  ChatMessage,
  ChatRole,
  EngineInfo,
  EngineStatus,
  GgufModelConfig,
  InferenceResult,
  SamplingConfig,
  StreamChunk,
  ToolCallInfo,
} from "./types";
export { OndeError } from "./types";

// ── JSON helpers ─────────────────────────────────────────────────────────────

// Convert SamplingConfig from camelCase TS to snake_case Rust
function serializeSampling(config: SamplingConfig): string {
  return JSON.stringify({
    temperature: config.temperature,
    top_p: config.topP,
    top_k: config.topK,
    min_p: config.minP,
    max_tokens: config.maxTokens,
    frequency_penalty: config.frequencyPenalty,
    presence_penalty: config.presencePenalty,
  });
}

// Convert GgufModelConfig from camelCase TS to snake_case Rust
function serializeModelConfig(config: GgufModelConfig): string {
  return JSON.stringify({
    model_id: config.modelId,
    files: config.files,
    tok_model_id: config.tokModelId,
    display_name: config.displayName,
    approx_memory: config.approxMemory,
    chat_template: config.chatTemplate,
  });
}

// Parse a JSON result string, throwing OndeError if it contains an error field
function parseResult<T>(json: string): T {
  const parsed = JSON.parse(json);
  if (parsed.error) {
    throw new OndeError(parsed.error);
  }
  return parsed as T;
}

// Parse snake_case JSON from Rust into camelCase TS types
function parseInferenceResult(json: string): InferenceResult {
  const raw = parseResult<{
    text: string;
    duration_secs: number;
    duration_display: string;
    finish_reason?: string;
    tool_calls?: Array<{
      id: string;
      function_name: string;
      arguments: string;
    }>;
  }>(json);
  return {
    text: raw.text,
    durationSecs: raw.duration_secs,
    durationDisplay: raw.duration_display,
    finishReason: raw.finish_reason,
    toolCalls: (raw.tool_calls ?? []).map((tc) => ({
      id: tc.id,
      functionName: tc.function_name,
      arguments: tc.arguments,
    })),
  };
}

function parseEngineInfo(json: string): EngineInfo {
  const raw = parseResult<{
    status: string;
    model_name?: string;
    approx_memory?: string;
    history_length: number;
  }>(json);
  return {
    status: raw.status as EngineInfo["status"],
    modelName: raw.model_name,
    approxMemory: raw.approx_memory,
    historyLength: raw.history_length,
  };
}

function parseModelConfig(json: string): GgufModelConfig {
  const raw = JSON.parse(json);
  return {
    modelId: raw.model_id,
    files: raw.files,
    tokModelId: raw.tok_model_id,
    displayName: raw.display_name,
    approxMemory: raw.approx_memory,
    chatTemplate: raw.chat_template,
  };
}

function parseSamplingConfig(json: string): SamplingConfig {
  const raw = JSON.parse(json);
  return {
    temperature: raw.temperature,
    topP: raw.top_p,
    topK: raw.top_k,
    minP: raw.min_p,
    maxTokens: raw.max_tokens,
    frequencyPenalty: raw.frequency_penalty,
    presencePenalty: raw.presence_penalty,
  };
}

// ── OndeChatEngine ───────────────────────────────────────────────────────────

/**
 * On-device LLM chat inference engine for React Native.
 *
 * The engine is backed by a native Rust inference engine (mistral.rs)
 * with Metal acceleration on iOS and CPU inference on Android.
 *
 * @example
 * ```typescript
 * import { OndeChatEngine } from "@ondeinference/react-native";
 *
 * // Load the platform-appropriate default model
 * const elapsed = await OndeChatEngine.loadDefaultModel("You are helpful.");
 * console.log(`Model loaded in ${elapsed}s`);
 *
 * // Multi-turn chat
 * const result = await OndeChatEngine.sendMessage("Hello!");
 * console.log(result.text);
 *
 * // Cleanup
 * await OndeChatEngine.unloadModel();
 * ```
 */
export const OndeChatEngine = {
  // ── Model lifecycle ──────────────────────────────────────────────────

  /**
   * Load the platform-appropriate default model.
   * - iOS → Qwen 2.5 1.5B (~941 MB, Metal)
   * - Android → Qwen 2.5 1.5B (~941 MB, CPU)
   * @returns Wall-clock loading time in seconds.
   */
  async loadDefaultModel(
    systemPrompt?: string,
    sampling?: SamplingConfig,
  ): Promise<number> {
    const json = await OndeInferenceModule.loadDefaultModel(
      systemPrompt ?? null,
      sampling ? serializeSampling(sampling) : null,
    );
    const result = parseResult<{ elapsed_secs: number }>(json);
    return result.elapsed_secs;
  },

  /**
   * Load the model assigned to this app via the Onde dashboard.
   *
   * Register your app at https://ondeinference.com to get an app ID
   * and secret. The dashboard lets you assign a model — the SDK
   * fetches it automatically. If no model is assigned, the platform
   * default is loaded instead.
   *
   * @param appId - Your Onde app ID from ondeinference.com
   * @param appSecret - Your Onde app secret from ondeinference.com
   * @returns Wall-clock loading time in seconds.
   */
  async loadAssignedModel(
    appId: string,
    appSecret: string,
    systemPrompt?: string,
    sampling?: SamplingConfig,
  ): Promise<number> {
    const json = await OndeInferenceModule.loadAssignedModel(
      appId,
      appSecret,
      systemPrompt ?? null,
      sampling ? serializeSampling(sampling) : null,
    );
    const result = parseResult<{ elapsed_secs: number }>(json);
    return result.elapsed_secs;
  },

  /**
   * Load a specific GGUF model.
   * @returns Wall-clock loading time in seconds.
   */
  async loadModel(
    config: GgufModelConfig,
    systemPrompt?: string,
    sampling?: SamplingConfig,
  ): Promise<number> {
    const json = await OndeInferenceModule.loadModel(
      serializeModelConfig(config),
      systemPrompt ?? null,
      sampling ? serializeSampling(sampling) : null,
    );
    const result = parseResult<{ elapsed_secs: number }>(json);
    return result.elapsed_secs;
  },

  /** Unload the current model, freeing all memory. */
  async unloadModel(): Promise<string | null> {
    const json = await OndeInferenceModule.unloadModel();
    const result = parseResult<{ model_name: string | null }>(json);
    return result.model_name;
  },

  /** Check whether a model is currently loaded. */
  isLoaded(): boolean {
    return OndeInferenceModule.isLoaded();
  },

  /** Get a snapshot of the engine's current state. */
  async info(): Promise<EngineInfo> {
    const json = await OndeInferenceModule.info();
    return parseEngineInfo(json);
  },

  // ── System prompt ────────────────────────────────────────────────────

  /** Set or replace the system prompt. */
  setSystemPrompt(prompt: string): void {
    OndeInferenceModule.setSystemPrompt(prompt);
  },

  /** Clear the system prompt. */
  clearSystemPrompt(): void {
    OndeInferenceModule.clearSystemPrompt();
  },

  // ── Sampling ─────────────────────────────────────────────────────────

  /** Replace the sampling configuration. */
  setSampling(sampling: SamplingConfig): void {
    OndeInferenceModule.setSampling(serializeSampling(sampling));
  },

  // ── History ──────────────────────────────────────────────────────────

  /** Get the full conversation history. */
  async history(): Promise<ChatMessage[]> {
    const json = await OndeInferenceModule.history();
    return JSON.parse(json) as ChatMessage[];
  },

  /** Clear the conversation history. Returns the number of removed turns. */
  clearHistory(): number {
    return OndeInferenceModule.clearHistory();
  },

  /** Append a message to history without running inference. */
  pushHistory(message: ChatMessage): void {
    OndeInferenceModule.pushHistory(JSON.stringify(message));
  },

  // ── Inference ────────────────────────────────────────────────────────

  /**
   * Send a user message and receive a complete assistant reply.
   * The user message and assistant reply are automatically appended
   * to the conversation history.
   */
  async sendMessage(message: string): Promise<InferenceResult> {
    const json = await OndeInferenceModule.sendMessage(message);
    return parseInferenceResult(json);
  },

  /**
   * Run inference on an explicit list of messages WITHOUT modifying
   * the engine's internal history. Useful for one-shot prompts.
   */
  async generate(
    messages: ChatMessage[],
    sampling?: SamplingConfig,
  ): Promise<InferenceResult> {
    const json = await OndeInferenceModule.generate(
      JSON.stringify(messages),
      sampling ? serializeSampling(sampling) : null,
    );
    return parseInferenceResult(json);
  },
};

// ── Free functions ─────────────────────────────────────────────────────────

/** Return the platform-appropriate default GGUF model config. */
export function defaultModelConfig(): GgufModelConfig {
  return parseModelConfig(OndeInferenceModule.defaultModelConfig());
}

/** Return the Qwen 2.5 1.5B config (~941 MB). */
export function qwen251_5bConfig(): GgufModelConfig {
  return parseModelConfig(OndeInferenceModule.qwen251_5bConfig());
}

/** Return the Qwen 2.5 3B config (~1.93 GB). */
export function qwen253bConfig(): GgufModelConfig {
  return parseModelConfig(OndeInferenceModule.qwen253bConfig());
}

/** Return default sampling parameters (temp=0.7, top_p=0.95, max_tokens=512). */
export function defaultSamplingConfig(): SamplingConfig {
  return parseSamplingConfig(OndeInferenceModule.defaultSamplingConfig());
}

/** Return deterministic (greedy) sampling parameters (temp=0.0). */
export function deterministicSamplingConfig(): SamplingConfig {
  return parseSamplingConfig(OndeInferenceModule.deterministicSamplingConfig());
}

/** Return conservative mobile sampling parameters (max_tokens=128). */
export function mobileSamplingConfig(): SamplingConfig {
  return parseSamplingConfig(OndeInferenceModule.mobileSamplingConfig());
}

// ── Message constructors ───────────────────────────────────────────────────

/** Create a system message. */
export function systemMessage(content: string): ChatMessage {
  return { role: "system", content };
}

/** Create a user message. */
export function userMessage(content: string): ChatMessage {
  return { role: "user", content };
}

/** Create an assistant message. */
export function assistantMessage(content: string): ChatMessage {
  return { role: "assistant", content };
}
