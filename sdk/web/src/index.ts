import { CpuEngine } from "./cpu/engine.js";
import { hasWasmSimd, hasWebGpu } from "./detect.js";
import type {
  Engine,
  EngineChoice,
  EngineInfo,
  EngineKind,
  LoadOptions,
  ModelInfo,
  ProgressCallback,
} from "./types.js";

export type {
  Engine,
  EngineChoice,
  EngineInfo,
  EngineKind,
  LoadOptions,
  ModelInfo,
  ProgressCallback,
  Sampling,
} from "./types.js";
export { CpuEngine, hasWebGpu, hasWasmSimd };

export interface OndeOptions {
  /**
   * `"cpu"` (default) runs vectoron in a Web Worker. `"webgpu"` and `"auto"`
   * are experimental and need `webgpu`; `"auto"` falls back to the CPU when
   * WebGPU or wllama doesn't come up.
   */
  engine?: EngineChoice;
  /**
   * The WebGPU engine, from `import { webgpu } from "@ondeinference/web/webgpu"`
   * (which needs `@wllama/wllama` installed).
   */
  webgpu?: () => Engine;
}

/**
 * The chat engine a page talks to. It forwards to the engine `createOnde`
 * picked, and on `"auto"` swaps a failed WebGPU load for the CPU engine.
 */
export class Onde {
  #engine: Engine;
  #fallback: boolean;

  constructor(engine: Engine, fallback = false) {
    this.#engine = engine;
    this.#fallback = fallback;
  }

  /** The engine in use. Can change from "webgpu" to "cpu" during `load` on `"auto"`. */
  get engine(): EngineKind {
    return this.#engine.kind;
  }

  model(): Promise<ModelInfo> {
    return this.#engine.model();
  }

  isCached(): Promise<boolean> {
    return this.#engine.isCached();
  }

  download(onProgress?: ProgressCallback): Promise<void> {
    return this.#engine.download(onProgress);
  }

  deleteCached(): Promise<boolean> {
    return this.#engine.deleteCached();
  }

  async load(options?: LoadOptions): Promise<number> {
    try {
      return await this.#engine.load(options);
    } catch (error) {
      if (!this.#fallback || this.#engine.kind !== "webgpu") throw error;
      console.warn("onde: WebGPU engine failed to load, using the CPU engine", error);
      await this.#engine.dispose();
      this.#engine = new CpuEngine();
      this.#fallback = false;
      return this.#engine.load(options);
    }
  }

  send(message: string, onDelta?: (delta: string) => void): Promise<string> {
    return this.#engine.send(message, onDelta);
  }

  /** The reply to `message`, piece by piece. */
  async *stream(message: string): AsyncGenerator<string, string> {
    const queue: string[] = [];
    let wake: (() => void) | null = null;
    let finished = false;
    let failure: unknown;
    const reply = this.#engine
      .send(message, (delta) => {
        queue.push(delta);
        wake?.();
      })
      .catch((error: unknown) => {
        failure = error ?? new Error("send failed");
        return "";
      })
      .finally(() => {
        finished = true;
        wake?.();
      });
    while (true) {
      while (queue.length) yield queue.shift()!;
      if (finished) break;
      await new Promise<void>((resolve) => (wake = resolve));
      wake = null;
    }
    while (queue.length) yield queue.shift()!;
    if (failure !== undefined) throw failure;
    return await reply;
  }

  reset(): Promise<number> {
    return this.#engine.reset();
  }

  info(): Promise<EngineInfo> {
    return this.#engine.info();
  }

  cancel(): Promise<void> {
    return this.#engine.cancel();
  }

  dispose(): Promise<void> {
    return this.#engine.dispose();
  }
}

/** Picks an engine and returns an `Onde`. Nothing is downloaded until `download` or `load`. */
export async function createOnde(options: OndeOptions = {}): Promise<Onde> {
  const choice = options.engine ?? "cpu";
  if (choice !== "cpu" && !options.webgpu) {
    throw new Error(
      `engine "${choice}" needs the WebGPU engine: pass webgpu() from "@ondeinference/web/webgpu"`,
    );
  }
  if (choice === "webgpu") {
    if (!(await hasWebGpu())) throw new Error("WebGPU is not available in this browser");
    return new Onde(options.webgpu!());
  }
  if (choice === "auto" && (await hasWebGpu())) {
    return new Onde(options.webgpu!(), true);
  }
  if (!hasWasmSimd()) {
    throw new Error("The CPU engine needs WebAssembly SIMD, which this browser lacks");
  }
  return new Onde(new CpuEngine());
}
