import type {
  Engine,
  EngineInfo,
  LoadOptions,
  ModelInfo,
  ProgressCallback,
} from "../types.js";
import type { Reply, Request } from "./protocol.js";

/** `Omit` applied to each member of a union, so the union stays discriminated. */
type DistributiveOmit<T, K extends PropertyKey> = T extends unknown ? Omit<T, K> : never;
type Body = DistributiveOmit<Request, "id">;

interface Pending {
  resolve(value: unknown): void;
  reject(error: Error): void;
  onProgress?: ProgressCallback;
  onDelta?: (delta: string) => void;
}

/** Starts the worker. Bundlers (Vite, webpack 5, esbuild) see this pattern and ship worker.js. */
function spawn(): Worker {
  return new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
}

/**
 * vectoron on the CPU (wasm simd128), in a Web Worker. Correct output on every
 * current browser, and the default engine.
 */
export class CpuEngine implements Engine {
  readonly kind = "cpu" as const;
  #worker: Worker;
  #pending = new Map<number, Pending>();
  #nextId = 1;
  #disposed = false;

  constructor() {
    this.#worker = this.#start();
  }

  #start(): Worker {
    const worker = spawn();
    worker.onmessage = ({ data }: MessageEvent<Reply>) => {
      const pending = this.#pending.get(data.id);
      if (!pending) return;
      switch (data.type) {
        case "progress":
          pending.onProgress?.(data.received, data.total);
          break;
        case "delta":
          pending.onDelta?.(data.text);
          break;
        case "result":
          this.#pending.delete(data.id);
          pending.resolve(data.value);
          break;
        case "error":
          this.#pending.delete(data.id);
          pending.reject(new Error(data.message));
          break;
      }
    };
    worker.onerror = (event) => {
      event.preventDefault();
      this.#failAll(new Error(`The CPU worker failed: ${event.message}`));
    };
    return worker;
  }

  #failAll(error: Error) {
    const pending = [...this.#pending.values()];
    this.#pending.clear();
    for (const p of pending) p.reject(error);
  }

  #call<T>(body: Body, hooks: Omit<Pending, "resolve" | "reject"> = {}): Promise<T> {
    if (this.#disposed) return Promise.reject(new Error("The engine was disposed"));
    const id = this.#nextId++;
    return new Promise<T>((resolve, reject) => {
      this.#pending.set(id, { resolve: resolve as (v: unknown) => void, reject, ...hooks });
      this.#worker.postMessage({ ...body, id } as Request);
    });
  }

  model(): Promise<ModelInfo> {
    return this.#call({ type: "model" });
  }

  isCached(): Promise<boolean> {
    return this.#call({ type: "isCached" });
  }

  download(onProgress?: ProgressCallback): Promise<void> {
    return this.#call({ type: "download" }, { onProgress });
  }

  deleteCached(): Promise<boolean> {
    return this.#call({ type: "deleteCached" });
  }

  load(options: LoadOptions = {}): Promise<number> {
    return this.#call({ type: "load", options });
  }

  send(message: string, onDelta?: (delta: string) => void): Promise<string> {
    return this.#call({ type: "send", message }, { onDelta });
  }

  reset(): Promise<number> {
    return this.#call({ type: "reset" });
  }

  async info(): Promise<EngineInfo> {
    const info = await this.#call<Omit<EngineInfo, "engine">>({ type: "info" });
    return { engine: this.kind, ...info };
  }

  async cancel(): Promise<void> {
    if (this.#disposed) return;
    // A forward pass can't be interrupted from outside, so stop the thread.
    this.#worker.terminate();
    this.#failAll(new Error("Cancelled"));
    this.#worker = this.#start();
  }

  async dispose(): Promise<void> {
    if (this.#disposed) return;
    this.#disposed = true;
    this.#worker.terminate();
    this.#failAll(new Error("The engine was disposed"));
  }
}
