# Local results (M4 MacBook Pro, 24 GiB), measured before the 2026-11-17 incident

These files were produced by running the harnesses on a developer laptop. They
were taken **before** the `synchronize_async()` fix in `wgpu-candle/src/lib.rs`,
so the `spikeA-chrome-wgpu-*` runs exercised the leaking path and are kept as
evidence, not as a benchmark. Nothing under `results/` was produced by CI.
CI results live in the workflow run summaries and are copied into
`../findings.md`.

| file | what |
|---|---|
| `step0-native-cpu.jsonl`, `quick-native.jsonl` | native candle CPU |
| `step0-native-metal.jsonl` | native candle Metal |
| `spikeA-native-wgpu.jsonl` | native FerrisMind wgpu (Metal backend) |
| `step0-chrome-cpu-raw.json`, `step0-chrome-cpu-nosimd-raw.json`, `quick-www*.json` | Chrome, wasm CPU, with and without simd128 |
| `step0-webkit-cpu-raw.json` | Playwright WebKit, wasm CPU |
| `spikeA-chrome-wgpu-*.json` | Chrome WebGPU; `-raw` is the failed f32-limit WGSL run, `-tiny`/`-quick` ran with the fork patch |
