// ── Message role ─────────────────────────────────────────────────────────────

/** Role of a message in a chat conversation. */
export type ChatRole = "system" | "user" | "assistant";

// ── Chat message ─────────────────────────────────────────────────────────────

/** A single message in a conversation. */
export interface ChatMessage {
  role: ChatRole;
  content: string;
}

// ── Sampling configuration ───────────────────────────────────────────────────

/**
 * Sampling parameters for text generation.
 * All fields are optional — `undefined` means "use the engine default".
 */
export interface SamplingConfig {
  /** Sampling temperature (higher = more random). Typical range: 0.0–2.0. */
  temperature?: number;
  /** Nucleus (top-p) sampling threshold. Typical value: 0.9–0.95. */
  topP?: number;
  /** Top-k sampling limit. */
  topK?: number;
  /** Min-p sampling threshold. */
  minP?: number;
  /** Maximum number of tokens to generate. */
  maxTokens?: number;
  /** Frequency penalty (penalise tokens proportional to occurrence count). */
  frequencyPenalty?: number;
  /** Presence penalty (penalise tokens that appeared at all). */
  presencePenalty?: number;
}

// ── Model configuration ──────────────────────────────────────────────────────

/** Configuration for loading a GGUF model. */
export interface GgufModelConfig {
  /** HuggingFace model repository ID. */
  modelId: string;
  /** GGUF filename(s) within the repository. */
  files: string[];
  /** Optional: explicit tokenizer model ID (required on Android). */
  tokModelId?: string;
  /** Human-friendly display name. */
  displayName: string;
  /** Approximate memory footprint description. */
  approxMemory: string;
}

// ── Inference result ─────────────────────────────────────────────────────────

/** Result of a completed inference request. */
export interface InferenceResult {
  /** The generated text. */
  text: string;
  /** Wall-clock inference duration in seconds. */
  durationSecs: number;
  /** Human-readable duration string, e.g. "1.23s". */
  durationDisplay: string;
  /** Finish reason, if available. */
  finishReason?: string;
}

// ── Streaming ────────────────────────────────────────────────────────────────

/** A single streaming token chunk. */
export interface StreamChunk {
  /** The token text delta. */
  delta: string;
  /** Whether this is the final chunk. */
  done: boolean;
  /** Finish reason (only present on the final chunk). */
  finishReason?: string;
}

// ── Engine status ────────────────────────────────────────────────────────────

/** Current status of the inference engine. */
export type EngineStatus =
  | "Unloaded"
  | "Loading"
  | "Ready"
  | "Generating"
  | "Error";

/** Snapshot of the engine's current state. */
export interface EngineInfo {
  /** Current engine status. */
  status: EngineStatus;
  /** Name of the loaded model, if any. */
  modelName?: string;
  /** Approximate memory footprint of the loaded model. */
  approxMemory?: string;
  /** Number of messages in the conversation history. */
  historyLength: number;
}

// ── Errors ───────────────────────────────────────────────────────────────────

/** Error thrown by the Onde inference engine. */
export class OndeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "OndeError";
  }
}
