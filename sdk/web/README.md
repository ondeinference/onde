<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference</h1>

<p align="center">
  <strong>Run LLMs in the browser with <a href="https://ondeinference.com/">Onde Inference</a>. The model downloads once, runs on the user's machine, and no prompt leaves the page.</strong>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@ondeinference/web"><img src="https://img.shields.io/npm/v/@ondeinference/web?style=flat-square&color=235843&labelColor=17211D&label=npm" alt="npm"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://github.com/ondeinference/onde/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
</p>

<p align="center">
  <a href="https://github.com/ondeinference/onde">Rust SDK</a> · <a href="https://swiftpackageindex.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference">Kotlin Multiplatform SDK</a> · <a href="https://pub.dev/packages/onde_inference">Flutter SDK</a> · <a href="https://www.npmjs.com/package/@ondeinference/react-native">React Native SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

`@ondeinference/web` runs Qwen 2.5 0.5B Instruct inside a web page. There's no server and no API key. The first `download()` fetches the model from Hugging Face (about 380 MB) into the browser's Cache Storage. After that it loads from disk, even offline.

The default engine is [vectoron](https://crates.io/crates/vectoron), a small Rust inference engine compiled to WebAssembly with SIMD. It runs in a Web Worker, so generation never blocks the page. Expect a few tokens per second on a laptop CPU, which is usable for short replies.

## Installation

```bash
npm install @ondeinference/web
```

## Quick start

```ts
import { createOnde } from "@ondeinference/web";

const onde = await createOnde();

if (!(await onde.isCached())) {
  await onde.download((received, total) => {
    console.log(`${received} / ${total ?? "?"} bytes`);
  });
}

await onde.load({ systemPrompt: "You are a helpful assistant." });

for await (const piece of onde.stream("What is the capital of Sweden?")) {
  output.textContent += piece;
}

await onde.reset();   // forget the conversation
await onde.dispose(); // free the model and the worker
```

`send(message, onDelta)` does the same as `stream` with a callback, and resolves to the whole reply. `cancel()` stops a reply. On the CPU engine that restarts the worker, so call `load()` again afterwards. The model comes from the cache, with no new download.

### Sampling

```ts
await onde.load({ sampling: { temperature: 0, maxTokens: 128 } }); // greedy
await onde.load({ sampling: { temperature: 0.7, topP: 0.95, seed: 42 } });
```

Every field is optional. A `temperature` of 0 is greedy, and a fixed `seed` makes CPU replies repeatable.

## WebGPU (experimental)

There's a second engine on llama.cpp's WebGPU backend through [wllama](https://github.com/ngxson/wllama). In our tests on a GitHub macOS runner it was far faster than the CPU engine. But on that VM's paravirtualized GPU, its output didn't match the CPU reference. Until that's checked on real hardware, the engine is opt-in and lives in its own entry point:

```bash
npm install @ondeinference/web @wllama/wllama@3.13.0
```

```ts
import { createOnde } from "@ondeinference/web";
import { webgpu } from "@ondeinference/web/webgpu";

// "webgpu" throws if WebGPU is missing; "auto" falls back to the CPU engine.
const onde = await createOnde({ engine: "auto", webgpu: webgpu() });
console.log(onde.engine); // "webgpu" or "cpu"
```

`webgpu({ gpuLayers, contextLength, wllamaWasmUrl })` takes options. `gpuLayers: 0` runs wllama on the CPU. wllama's own `wllama.wasm` loads from jsDelivr by default. Set `wllamaWasmUrl` to serve it yourself. For wllama's multi-threaded build, the page must be cross-origin isolated (`Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Embedder-Policy: require-corp`).

Both engines read the same cached model file, so switching engines doesn't download it again. Apps that never import `@ondeinference/web/webgpu` don't need wllama, and their bundle doesn't include it.

## Requirements

- A browser with WebAssembly SIMD: current Chrome, Edge, Firefox and Safari.
- A secure context (HTTPS or `localhost`) for Cache Storage.
- A bundler that understands `new URL("…", import.meta.url)` for the worker and the `.wasm` file. Vite, webpack 5, Rollup and esbuild all do, with no plugin.
- Roughly 1 GB of free memory while the model loads.

## Model

| Model | Size | Source |
|---|---|---|
| Qwen 2.5 0.5B Instruct, Q4_K_M | ~380 MB | [bartowski/Qwen2.5-0.5B-Instruct-GGUF](https://huggingface.co/bartowski/Qwen2.5-0.5B-Instruct-GGUF) |

`onde.model()` returns its name, size and download URLs. `deleteCached()` removes it from the browser.

## License

MIT OR Apache-2.0. Copyright © Onde Inference.
