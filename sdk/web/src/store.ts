// The model store on the page's thread, for the WebGPU engine. It calls the
// same wasm functions the CPU worker uses, so both engines share one Cache
// Storage entry and one model catalogue (which lives in the Rust crate).

import init, {
  delete_cached,
  download as wasmDownload,
  is_cached,
  model_info,
} from "../wasm/onde_web.js";
import type { ModelInfo, ProgressCallback } from "./types.js";

let ready: Promise<unknown> | null = null;
const wasm = () => (ready ??= init());

export async function modelInfo(): Promise<ModelInfo> {
  await wasm();
  return model_info() as ModelInfo;
}

export async function isCached(): Promise<boolean> {
  await wasm();
  return is_cached();
}

export async function download(onProgress?: ProgressCallback): Promise<void> {
  await wasm();
  await wasmDownload((received: number, total: number | null) => onProgress?.(received, total));
}

export async function deleteCached(): Promise<boolean> {
  await wasm();
  return delete_cached();
}

/** The cached GGUF as a Blob, which wllama reads without copying it into JS memory first. */
export async function cachedBlob(): Promise<Blob> {
  const { cacheName, modelUrl } = await modelInfo();
  const cache = await caches.open(cacheName);
  const hit = await cache.match(modelUrl);
  if (!hit) throw new Error(`${modelUrl} is not in the cache, call download() first`);
  return hit.blob();
}
