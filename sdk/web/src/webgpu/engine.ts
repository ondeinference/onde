// Experimental: llama.cpp's WebGPU backend through wllama. The WebGPU spike
// measured it far faster than the CPU engine, but on a paravirtual GPU its
// output did not match the CPU reference, so it stays opt-in until that is
// checked on real hardware.

import { Wllama } from "@wllama/wllama";
import WasmFromCDN from "@wllama/wllama/esm/wasm-from-cdn.js";
import * as store from "../store.js";
import type {
  Engine,
  EngineInfo,
  LoadOptions,
  ModelInfo,
  ProgressCallback,
  Sampling,
} from "../types.js";

export interface WebGpuOptions {
  /**
   * URL of wllama's `wllama.wasm`. Defaults to the jsDelivr copy that matches
   * the installed @wllama/wllama. Set it to serve the file yourself.
   */
  wllamaWasmUrl?: string;
  /** Layers to offload to the GPU. Default: all of them. 0 runs wllama on the CPU. */
  gpuLayers?: number;
  /** Context window in tokens. Default 4096. */
  contextLength?: number;
}

type Message = { role: "system" | "user" | "assistant"; content: string };

const ALL_LAYERS = 999;
const MAX_TOKENS = 512;

function newWllama(wasmUrl: string | undefined): Wllama {
  return new Wllama({ default: wasmUrl ?? WasmFromCDN.default }, { suppressNativeLog: true });
}

export class WebGpuEngine implements Engine {
  readonly kind = "webgpu" as const;
  #options: WebGpuOptions;
  #wllama: Wllama | null = null;
  #system: string | undefined;
  #sampling: Sampling = {};
  #history: Message[] = [];
  #busy = false;
  #abort: AbortController | null = null;
  #disposed = false;

  constructor(options: WebGpuOptions = {}) {
    this.#options = options;
  }

  model(): Promise<ModelInfo> {
    return store.modelInfo();
  }

  isCached(): Promise<boolean> {
    return store.isCached();
  }

  download(onProgress?: ProgressCallback): Promise<void> {
    return store.download(onProgress);
  }

  deleteCached(): Promise<boolean> {
    return store.deleteCached();
  }

  async load(options: LoadOptions = {}): Promise<number> {
    this.#check();
    this.#busy = true;
    const started = performance.now();
    try {
      await this.#unload();
      await store.download();
      const gguf = await store.cachedBlob();
      const wllama = newWllama(this.#options.wllamaWasmUrl);
      await wllama.loadModel([gguf], {
        n_gpu_layers: this.#options.gpuLayers ?? ALL_LAYERS,
        n_ctx: this.#options.contextLength ?? 4096,
      });
      this.#wllama = wllama;
      this.#system = options.systemPrompt;
      this.#sampling = options.sampling ?? {};
      this.#history = [];
      return (performance.now() - started) / 1000;
    } finally {
      this.#busy = false;
    }
  }

  async send(message: string, onDelta?: (delta: string) => void): Promise<string> {
    this.#check();
    const wllama = this.#wllama;
    if (!wllama) throw new Error("No model loaded, call `load` first");
    this.#busy = true;
    this.#abort = new AbortController();
    const messages: Message[] = [
      ...(this.#system ? [{ role: "system" as const, content: this.#system }] : []),
      ...this.#history,
      { role: "user", content: message },
    ];
    const s = this.#sampling;
    let reply = "";
    try {
      await wllama.createChatCompletion({
        messages,
        stream: true,
        abortSignal: this.#abort.signal,
        max_tokens: s.maxTokens ?? MAX_TOKENS,
        temperature: s.temperature,
        top_p: s.topP,
        top_k: s.topK,
        seed: s.seed,
        onData: (chunk) => {
          const delta = chunk.choices?.[0]?.delta?.content;
          if (!delta) return;
          reply += delta;
          onDelta?.(delta);
        },
      });
      this.#history.push({ role: "user", content: message }, { role: "assistant", content: reply });
      return reply;
    } finally {
      this.#busy = false;
      this.#abort = null;
    }
  }

  async reset(): Promise<number> {
    this.#check();
    const dropped = this.#history.length;
    this.#history = [];
    return dropped;
  }

  async info(): Promise<EngineInfo> {
    const model = this.#wllama ? await store.modelInfo() : null;
    return {
      engine: this.kind,
      status: this.#busy ? "busy" : this.#wllama ? "ready" : "unloaded",
      modelName: model?.name ?? null,
      approxMemory: model?.approxSize ?? null,
      historyLength: this.#history.length,
    };
  }

  async cancel(): Promise<void> {
    this.#abort?.abort();
  }

  async dispose(): Promise<void> {
    if (this.#disposed) return;
    this.#disposed = true;
    this.#abort?.abort();
    await this.#unload();
  }

  async #unload() {
    const wllama = this.#wllama;
    this.#wllama = null;
    await wllama?.exit();
  }

  #check() {
    if (this.#disposed) throw new Error("The engine was disposed");
    if (this.#busy) throw new Error("The engine is busy with another request");
  }
}
