## 1.5.1

- First release of `@ondeinference/web`. `createOnde()` runs Qwen 2.5 0.5B (Q4_K_M, about 380 MB) in the browser on the CPU through vectoron, in a Web Worker, with the model kept in Cache Storage.
- `load()` takes sampling options (`temperature`, `topP`, `topK`, `seed`, `maxTokens`). A `temperature` of 0 is greedy.
- An experimental WebGPU engine at `@ondeinference/web/webgpu`, on wllama 3.13.0 (llama.cpp's WebGPU backend). It is opt-in, and `engine: "auto"` falls back to the CPU if it doesn't load.

## 1.5.0

- Not published to npm. The package was added after 1.5.0 was tagged.
