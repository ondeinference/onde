# WebGPU spike: findings

Question: can onde's browser engine use the GPU through WebGPU? Today it runs
candle on the CPU (wasm simd128, one thread), see ondeinference/onde#50.

Status: **Spike A is in its first CI round.** The CI table below is empty until
a run exists; nothing in it is filled in from memory.

## Pinned versions

| thing | version |
|---|---|
| candle (CPU harness) | crates.io 0.11.1 |
| candle (GPU harness) | FerrisMind/candle `bde164da9269619aeb16d5b18f8267e25d188fa4` + `patches/*.patch` |
| Rust | 1.94.0, target `wasm32-unknown-unknown`, `+simd128` |
| wasm-bindgen / CLI | 0.2.130 |
| Playwright | 1.65.1 (see `package-lock.json`) |
| model | bartowski Qwen2.5-0.5B-Instruct Q4_K_M, SHA-256 `6eb923e7d26e9cea28811e1a8e852009b21242fb157b26149d3b188f3a8c8653`, checked in CI |
| tokenizer | Qwen/Qwen2.5-0.5B-Instruct `tokenizer.json` (unpinned `main`; the job log prints its SHA-256) |

## Fork patches and the errors that needed them

| patch | error it fixed |
|---|---|
| `01-wgsl-f32-limit.patch` | Chrome rejected two reduce shaders: `value -340282349999999991754788743781432688640.0 cannot be represented as 'f32'`. The constants `±3.4028235e38` become `±3.4028234e38`. Native wgpu accepted the original. |

## Local M4 numbers (pre-incident, not CI)

Laptop numbers, 179-token prompt, 128 new tokens, greedy, mean of 3 runs.
Files are in `results/`; see `results/README.md`.

| path | prefill | decode tok/s |
|---|---|---|
| native candle CPU | ~2.2 s | ~49 |
| native candle Metal | ~0.09 s | ~125 |
| native FerrisMind wgpu (Metal) | ~2.8 s | ~14 |
| Chrome wasm CPU, simd128 | ~13.7 s | ~11 |
| Chrome wasm CPU, no simd | ~30.6 s | ~4.4 |
| WebKit (Playwright) wasm CPU | ~14–18 s | ~9.3 |
| Chrome WebGPU (patched fork), 30-token prompt, 8 new tokens | 0.12 s | 23 |
| Chrome WebGPU (patched fork), 179-token prompt, 24 new tokens | 0.44 s | 7.7 |

The two WebGPU rows are single runs from before the buffer-recycling fix. They
show the GPU path working and prefill about 30x faster than wasm CPU at the
same prompt length. Decode was not better than CPU in the 24-token run. They
are not a trustworthy decode benchmark.

## Incident, 2026-11-17

Running the browser GPU harness locally panicked the M4 three times (09:18,
09:31, 09:51), each a `watchdog timeout`, with one Chrome GPU helper process at
49–52 GB on a 24 GiB machine and about 15 MB free.

Cause in my harness: on wasm the FerrisMind fork frees in-flight GPU buffers
only when the caller awaits `WgpuDevice::synchronize_async()`, and the harness
never did. Chrome let a single page reach about 50 GB. The 09:51 panic happened
with nothing of mine running; Chrome's data was not readable (macOS privacy
block), so what Chrome had open then is unknown.

Fixes: `wgpu-candle/src/lib.rs` now awaits `synchronize_async()` after load,
after every prefill chunk and after every decode token (`drain()`), and
`guard.mjs` kills the browser past 4 GB per process or the per-run time limit
and records the peak RSS. Nothing runs locally now; every build and benchmark
runs on GitHub-hosted runners (`.github/workflows/webgpu-spike.yml`).

## CI results

Not run yet. Each row will link its run.

| path | runner | browser | prefill ms | TTFT ms | decode tok/s | load ms | peak RSS GB | first 20 tokens vs native | run |
|---|---|---|---|---|---|---|---|---|---|

## Recommendation

None yet. The local pre-incident rows suggest WebGPU prefill is worth pursuing;
decode is unproven. Decide after the first CI table.

## What hosted runners can't tell us

`macos-26` is an Apple Silicon VM with a paravirtualized GPU, so WebGPU may not
come up there, and if it does its numbers compare paths against each other
rather than predicting an M4's GPU. `ubuntu-latest` has no GPU: Chrome's WebGPU
runs in software (SwiftShader), which is good for correctness and for catching
runaway memory safely, and useless for speed. Measuring real GPU speed needs a
real-hardware run that you choose to do.
