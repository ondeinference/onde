// Shared by every harness: read model bytes from the same Cache Storage entry
// onde's web_store.rs uses (bucket "onde-models-v1", key = the HF resolve URL).
// If the entry is missing it is seeded from the local dev server (/model/*),
// which serves the same bytes (checked by SHA-256 below).
export const CACHE_NAME = "onde-models-v1";
export const MODEL_URL =
  "https://huggingface.co/bartowski/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/Qwen2.5-0.5B-Instruct-Q4_K_M.gguf";
export const TOK_URL =
  "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/main/tokenizer.json";
export const MODEL_SHA256 = "6eb923e7d26e9cea28811e1a8e852009b21242fb157b26149d3b188f3a8c8653";

async function ensure(cache, url, localPath) {
  let hit = await cache.match(url);
  let seeded = false;
  if (!hit) {
    const r = await fetch(localPath);
    if (!r.ok) throw new Error(`seed ${localPath}: ${r.status}`);
    await cache.put(url, r);
    hit = await cache.match(url);
    seeded = true;
  }
  return { response: hit, seeded };
}

export async function loadModel({ verify = true } = {}) {
  const t0 = performance.now();
  const cache = await caches.open(CACHE_NAME);
  const m = await ensure(cache, MODEL_URL, "/model/model.gguf");
  const t = await ensure(cache, TOK_URL, "/model/tokenizer.json");
  const gguf = new Uint8Array(await m.response.arrayBuffer());
  const tok = new Uint8Array(await t.response.arrayBuffer());
  const readMs = performance.now() - t0;
  let sha = null;
  if (verify) {
    const h = await crypto.subtle.digest("SHA-256", gguf);
    sha = [...new Uint8Array(h)].map((b) => b.toString(16).padStart(2, "0")).join("");
    if (sha !== MODEL_SHA256) throw new Error(`model sha mismatch: ${sha}`);
  }
  return { gguf, tok, readMs, seeded: m.seeded, sha };
}

export async function loadBench() {
  const q = new URLSearchParams(self.location.search).get("cfg") || "bench";
  return await (await fetch(`/common/${q}.json`)).json();
}

export function chatml(system, user) {
  return `<|im_start|>system\n${system}<|im_end|>\n<|im_start|>user\n${user}<|im_end|>\n<|im_start|>assistant\n`;
}
