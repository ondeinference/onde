# WebGPU spike: findings

Question: can onde's browser engine use the GPU through WebGPU? Today it runs
candle on the CPU (wasm simd128, one thread), see ondeinference/onde#50.

Status: **Complete.** Both spikes ran on GitHub-hosted runners. CI results
below are from [run 44087888201](https://github.com/ondeinference/onde/actions/runs/44087888201)
(2026-11-17, commit 34f2221).

## Pinned versions

| thing | version |
|---|---|
| candle (CPU harness) | crates.io 0.11.1 |
| candle (GPU harness) | FerrisMind/candle `bde164da9269619aeb16d5b18f8267e25d188fa4` + `patches/*.patch` |
| Rust | 1.94.0, target `wasm32-unknown-unknown`, `+simd128` |
| wasm-bindgen / CLI | 0.2.130 |
| Playwright | 1.65.1 (see `package-lock.json`) |
| model | bartowski Qwen2.5-0.5B-Instruct Q4_K_M, SHA-256 `6eb923e7d26e9cea28811e1a8e852009b21242fb157b26149d3b188f3a8c8653`, checked in CI |
| wllama | 3.13.0 (llama.cpp WebGPU backend) |
| tokenizer | Qwen/Qwen2.5-0.5B-Instruct `tokenizer.json` from `main`, SHA-256 `c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539`, checked in CI (the same hash in all 22 downloads of runs 44087888201 and 44142779991) |

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

All rows from [run 44087888201](https://github.com/ondeinference/onde/actions/runs/44087888201).
Bench config: 179-token prompt, 128 new tokens, greedy, mean of 3 runs (warmup
1). Rows that only reached `bench-quick` (24 tokens, 1 run) or `bench-tiny`
(8 tokens, 1 run) are noted.

### Native (macos-26 VM, no browser)

| path | prefill ms | decode tok/s | load ms | correctness |
|---|---|---|---|---|
| candle CPU | 5410 | 25.0 | 1055 | 20/20 (reference) |
| candle Metal | 177 | 62.5 | 1690 | 0/20 (differs at 0) |
| FerrisMind wgpu→Metal | 8288 | 2.6 | 905 | 0/20 (differs at 0) |

Metal and wgpu output differ from CPU. The VM's paravirtualized Metal may
introduce rounding differences; this needs investigation on real hardware.

### Browser — candle CPU baseline (wasm simd128)

| runner | browser | prefill ms | decode tok/s | load ms | peak RSS GB | correctness |
|---|---|---|---|---|---|---|
| macos-26 | chrome | 28706 | 4.9 | 3827 | 2.30 | 19/20 |
| macos-26 | webkit | 30588 | 5.3 | 6674 | 0.10 | 19/20 |
| ubuntu | chrome | 35429 | 3.6 | 1108 | 2.00 | 19/20 |

This is the current onde web-engine path. ~29–35 s prefill, ~4–5 tok/s decode.

### Browser — Spike A: FerrisMind candle wgpu

| runner | browser | drain | rung | prefill ms | decode tok/s | load ms | peak RSS GB | correctness | status |
|---|---|---|---|---|---|---|---|---|---|
| macos-26 | chrome | 32 | bench-quick | 10887 | 0.3 | 1013 | 1.61 | 20/20 | bench failed: map_async |
| macos-26 | chrome | 8 | bench-quick | 32420 | 0.2 | 5795 | 1.75 | 20/20 | bench failed: map_async |
| macos-26 | webkit | 1 | bench | 7225 | 2.6 | 7579 | 0.10 | 20/20 | ok |
| macos-26 | webkit | 32 | bench | 3120 | 2.6 | 5201 | 0.11 | 20/20 | ok |
| macos-26 | webkit | 8 | bench | 4517 | 2.4 | 6867 | 0.10 | 20/20 | ok |
| ubuntu | chrome | 1 | none | – | – | – | 1.20 | – | shader-f16 not implemented (SwiftShader) |

Chrome on macos-26: prefill is faster than CPU (3–11 s vs 29 s), but decode is
extremely slow (0.2–0.3 tok/s) and long runs hit `map_async` failures with
device-lost events. WebKit on macos-26: prefill is faster (3–7 s) with correct
output, but decode (2.4–2.6 tok/s) is slower than CPU baseline (5.3 tok/s).
SwiftShader on ubuntu can't load the model (needs `shader-f16`).

### Browser — Spike B: wllama (llama.cpp WebGPU)

| runner | browser | GPU layers | rung | prefill ms | decode tok/s | load ms | peak RSS GB | correctness | status |
|---|---|---|---|---|---|---|---|---|---|
| macos-26 | chrome | all | bench | 111 | 26.7 | 5983 | 1.65 | 0/60 chars | text mismatch |
| macos-26 | chrome | 0 (CPU) | bench | 587 | 4.7 | 6643 | 1.48 | 60/60 chars | ok |
| macos-26 | webkit | all | none | – | – | – | 0.11 | – | I/O read failed |
| ubuntu | chrome | all | bench | 322 | 6.3 | 2536 | 1.13 | 60/60 chars | ok |
| ubuntu | chrome | 0 (CPU) | bench | 324 | 6.3 | 2592 | 1.16 | 60/60 chars | ok |

**wllama with GPU offloading on macos-26 Chrome is the fastest path by far**:
111 ms prefill (vs 28706 ms candle CPU wasm), 26.7 tok/s decode (vs 4.9).
However, the generated text does not match the CPU reference (0/60 chars). The
CPU-only wllama path (ngl=0) matches perfectly (60/60 chars). On ubuntu
SwiftShader, GPU and CPU paths produce identical speed (6.3 tok/s) and identical
correct output — SwiftShader is not actually offloading.

The correctness gap on macos-26's paravirtual GPU could be:
- Paravirtualized Metal rounding (the native Metal path also differs from CPU)
- A real llama.cpp WebGPU bug with quantized weights
- Temperature or sampling differences (the harness uses greedy/temp=0)

wllama on WebKit failed with an I/O read error during model loading. This is a
known WebKit limitation with large file reads through wllama's multipart worker.

### GPU events

Chrome on macos-26 reported device-lost events during the longer wgpu-candle
runs:
- drain/32 bench: `A valid external Instance reference no longer exists` at ~496 s
- drain/8 bench: `A valid external Instance reference no longer exists` at ~847 s

## CPU engines: candle and vectoron

Added after the spike. The question was whether vectoron, the engine behind
`@ondeinference/web`, could replace candle in onde's browser engine.
`vectoron-cpu` is `cpu-baseline` with vectoron in place of candle: same prompt,
prefill chunk, greedy decode loop and release profile. `engine-bench.yml`
runs both in one job per runner and browser. Run 45602570325, `bench.json`
(179-token prompt, 128 new tokens, mean of 3 runs after 1 warmup), wasm
simd128, no event-loop yields. vectoron is at `d15c228`, which is 0.3.0 apart
from a test helper and the manifest.

| wasm | candle prefill / decode | vectoron 0.2 | vectoron 0.3 |
|---|---|---|---|
| macos-26 Chrome | 31.0 s / 4.5 tok/s | 10.8 s / 2.4 | 7.7 s / 10.3 |
| macos-26 WebKit | 35.0 s / 4.6 | 9.9 s / 3.2 | 6.9 s / 12.2 |
| ubuntu Chrome | 44.6 s / 3.6 | 11.0 s / 1.7 | 10.2 s / 7.4 |

Peak Chrome RSS: 2.0 to 2.3 GB for candle, 1.0 to 1.3 GB for vectoron.

Natively (CPU, one process), candle still decodes faster on macos-26: 20.3
against 18.5 tok/s for vectoron 0.3, and 5.7 s against 4.5 s of prefill. That
is candle using NEON and threads. It does not apply to the single-threaded web
engine.

vectoron 0.2 read the prompt 3 to 4 times faster than candle but decoded at
about half its speed, because it dequantized every weight block to `f32` with
scalar code on each token. 0.3 quantizes the activations to 8 bits and
multiplies in integers, which is where the decode gain comes from.

The first generated token differs between the engines on this prompt. It is
a near-tie, not a bug: the top two logits are 70151 (18.68) and 4498 (18.61)
in candle, and 4498 (18.73) and 70151 (18.68) in vectoron, with the same top
five. The report's "first 20 tokens vs native CPU" column is therefore not a
useful check across engines. Correctness is covered by vectoron's own tests:
its `f32` path reproduces candle's greedy tokens on Qwen2.5-0.5B and
Qwen3-0.6B, and the integer kernels agree with it.

Follow-ups left open: wasm threads (they need COOP/COEP, which the web engine
avoids), and an `f16` key/value cache.

## Recommendation

**Use wllama (llama.cpp WebGPU) as the GPU acceleration path.** It is the only
path that delivered real speedup on a GPU-capable runner.

Concretely:
1. **Investigate the correctness gap first.** Run wllama GPU on real Apple
   Silicon (not the paravirtual GPU) to determine whether the text mismatch is a
   VM artifact or a real bug. If the native Metal path also differs from CPU on
   the VM but not on real hardware, the mismatch is likely the VM.
2. **Park FerrisMind candle wgpu.** It's immature: Chrome map_async failures on
   long runs, decode slower than CPU on WebKit, can't load on SwiftShader
   (needs shader-f16). Revisit if upstream candle merges PR #3379 and
   stabilizes.
3. **Ship candle CPU wasm (the current path) for now.** It works, produces
   correct output, and runs everywhere. WebGPU acceleration via wllama is the
   upgrade path once correctness is confirmed.
4. **If wllama GPU is correct on real hardware**, the integration path is: load
   the same GGUF, use wllama's JS API for GPU inference, fall back to candle
   wasm CPU when WebGPU is unavailable. This is a runtime choice, not a
   build-time one.

## What hosted runners can't tell us

`macos-26` is an Apple Silicon VM with a paravirtualized GPU, so WebGPU may not
come up there, and if it does its numbers compare paths against each other
rather than predicting an M4's GPU. `ubuntu-latest` has no GPU: Chrome's WebGPU
runs in software (SwiftShader), which is good for correctness and for catching
runaway memory safely, and useless for speed. Measuring real GPU speed needs a
real-hardware run that you choose to do.
