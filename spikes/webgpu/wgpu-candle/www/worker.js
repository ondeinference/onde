import init, { GpuBench } from "./pkg/spike_wgpu_candle.js";
import { loadModel, loadBench, chatml } from "/common/store.js";

// WebGPU adapter facts first: if there is no adapter nothing else matters.
async function probe() {
  if (!("gpu" in navigator)) return { available: false, reason: "navigator.gpu missing in worker" };
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) return { available: false, reason: "requestAdapter() returned null" };
  const info = adapter.info || (await adapter.requestAdapterInfo?.()) || {};
  const l = adapter.limits;
  return {
    available: true,
    vendor: info.vendor, architecture: info.architecture, device: info.device, description: info.description,
    features: [...adapter.features],
    maxStorageBufferBindingSize: l.maxStorageBufferBindingSize,
    maxBufferSize: l.maxBufferSize,
    maxComputeWorkgroupStorageSize: l.maxComputeWorkgroupStorageSize,
    maxComputeInvocationsPerWorkgroup: l.maxComputeInvocationsPerWorkgroup,
  };
}

self.onmessage = async () => {
  const out = { runs: [] };
  try {
    out.gpu = await probe();
    if (!out.gpu.available) throw new Error("no WebGPU: " + out.gpu.reason);
    const cfg = await loadBench();
    const prompt = chatml(cfg.system, cfg.user);
    const model = await loadModel();
    out.read_ms = model.readMs; out.seeded = model.seeded; out.sha256 = model.sha;
    const w0 = performance.now();
    const wasm = await init();
    out.wasm_bytes = (await (await fetch("./pkg/spike_wgpu_candle_bg.wasm")).arrayBuffer()).byteLength;
    out.wasm_init_ms = performance.now() - w0;
    const l0 = performance.now();
    const bench = await GpuBench.load(model.gguf, model.tok);
    out.load_ms = performance.now() - l0;
    out.wasm_memory_bytes = wasm.memory.buffer.byteLength;
    for (const doYield of (new URLSearchParams(self.location.search).get("yield") === "0" ? [false] : [false, true])) {
      for (let i = 0; i < cfg.warmup_runs + cfg.measured_runs; i++) {
        const r = JSON.parse(await bench.run(prompt, cfg.max_new_tokens, doYield));
        r.warmup = i < cfg.warmup_runs;
        out.runs.push(r);
        self.postMessage({ progress: `yield=${doYield} run ${i}: prefill ${r.prefill_ms.toFixed(0)} ms, decode ${r.decode_tps.toFixed(2)} tok/s` });
      }
    }
    out.wasm_memory_bytes_after = wasm.memory.buffer.byteLength;
    self.postMessage({ done: out });
  } catch (e) {
    self.postMessage({ error: String(e && e.stack || e), partial: out });
  }
};
