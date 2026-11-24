// Entry point for the experimental WebGPU engine: `@ondeinference/web/webgpu`.
// It is a separate entry so that only apps importing it need @wllama/wllama;
// the main entry never references wllama, and bundlers never look for it.

import type { Engine } from "../types.js";
import { WebGpuEngine, type WebGpuOptions } from "./engine.js";

export { WebGpuEngine, type WebGpuOptions };

/** Pass the result as `createOnde({ webgpu: webgpu() })`. */
export function webgpu(options: WebGpuOptions = {}): () => Engine {
  return () => new WebGpuEngine(options);
}
