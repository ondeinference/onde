/** Which inference engine runs the model. */
export type EngineKind = "cpu" | "webgpu";

/** What `createOnde` should pick. `"auto"` is experimental, see the README. */
export type EngineChoice = EngineKind | "auto";

/** The model the package runs, as reported by the wasm engine. */
export interface ModelInfo {
  name: string;
  approxSize: string;
  modelUrl: string;
  tokenizerUrl: string;
  /** The Cache Storage bucket both engines read the model from. */
  cacheName: string;
}

/**
 * Sampling settings. Every field is optional. A `temperature` of 0 is greedy,
 * and a fixed `seed` makes replies repeatable on the CPU engine.
 */
export interface Sampling {
  temperature?: number;
  topP?: number;
  topK?: number;
  seed?: number;
  maxTokens?: number;
}

export interface EngineInfo {
  engine: EngineKind;
  status: "unloaded" | "busy" | "ready";
  modelName: string | null;
  approxMemory: string | null;
  historyLength: number;
}

/** `total` is `null` when the server does not send a length. */
export type ProgressCallback = (received: number, total: number | null) => void;

export interface LoadOptions {
  systemPrompt?: string;
  sampling?: Sampling;
}

/** One chat engine over one model. Both engines implement it. */
export interface Engine {
  readonly kind: EngineKind;
  model(): Promise<ModelInfo>;
  isCached(): Promise<boolean>;
  download(onProgress?: ProgressCallback): Promise<void>;
  deleteCached(): Promise<boolean>;
  /** Loads the model, downloading it first if needed. Resolves to seconds taken. */
  load(options?: LoadOptions): Promise<number>;
  /** Sends one message. `onDelta` gets each new piece; resolves to the whole reply. */
  send(message: string, onDelta?: (delta: string) => void): Promise<string>;
  /** Forgets the conversation. Resolves to how many messages were dropped. */
  reset(): Promise<number>;
  info(): Promise<EngineInfo>;
  /**
   * Stops the reply in progress. On the CPU engine this restarts the worker,
   * so the model has to be loaded again (from the cache, no download).
   */
  cancel(): Promise<void>;
  /** Frees the model and any worker. The engine can't be used afterwards. */
  dispose(): Promise<void>;
}
