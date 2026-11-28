## 1.6.0

- The wasm engine moves to vectoron 0.3, which multiplies quantized weights in integers. In headless browsers on GitHub runners, decoding Qwen2.5-0.5B goes from 1.7 to 3.2 tok/s to 7.4 to 12.2, and reading the prompt from 10 to 11 s to 7 to 10 s. Chrome's peak memory is about 1.3 GB.
- Replies can differ from 1.5.1's at near-ties, because activations are rounded to 8 bits.
- First version published by the release workflow, through npm trusted publishing. 1.5.1 was published by hand.

## 1.5.1

- First release of `@ondeinference/web`. `createOnde()` runs Qwen 2.5 0.5B (Q4_K_M, about 380 MB) in the browser on the CPU through vectoron, in a Web Worker, with the model kept in Cache Storage.
- `load()` takes sampling options (`temperature`, `topP`, `topK`, `seed`, `maxTokens`). A `temperature` of 0 is greedy.
- An experimental WebGPU engine at `@ondeinference/web/webgpu`, on wllama 3.13.0 (llama.cpp's WebGPU backend). It is opt-in, and `engine: "auto"` falls back to the CPU if it doesn't load.

## 1.5.0

- Not published to npm. The package was added after 1.5.0 was tagged.
