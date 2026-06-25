# Engine patches for `onde-mistralrs`

These patches fix bugs that live in the inference engine (`onde-mistralrs` /
`onde-mistralrs-core`, the `setoelkahfi/mistral.rs` fork published to crates.io),
not in this crate. They are kept here so the fix is reviewable and reproducible
even though it has to be applied and republished from the fork.

## `0001-prefer-f32-over-f16-gguf-attention.patch` — Qwen3-14B NaN logits

Fixes: **getsigit/sigit#5** — *"Can't run inference with the Qwen3-14B-GGUF
model"*, which fails with:

```
Inference failed: inference error: Invalid sampling probability at index 0: NaN.
The model likely produced NaN/Inf logits.
```

### Root cause

1. The GGUF pipeline always loads with `ModelDType::Auto`
   (`mistralrs/src/gguf.rs` has no dtype setter; `build_gguf_pipeline` passes
   `&ModelDType::Auto`), so **onde cannot choose the compute dtype** from this
   side — the fix has to be in the engine.
2. `determine_auto_dtype_all` (`mistralrs-core/src/utils/normal.rs`) probes
   `BF16 -> F16 -> F32` and returns the first that supports a trivial matmul.
   On a device **without hardware bf16** (Intel Mac Metal; some CPUs), the BF16
   probe fails and **F16** is selected.
3. GGUF attention materialises the `Q·Kᵀ` score matrix in the compute dtype —
   the eager path `naive_sdpa` and, on Metal prefill, the fused kernel
   `candle_nn::ops::sdpa` (see `attention/mod.rs::run_attention_noflash`). F16's
   maximum is **65504**. The 14B model's larger scaled scores exceed it →
   `Inf` → `NaN` after softmax. `sample_multinomial` then rejects the
   distribution with the error above.

Smaller Qwen models stay under the F16 overflow threshold, which is why only
the 14B fails. **Apple Silicon (BF16) and CPU (BF16/F32) already run it fine** —
only the F16 fallback path is affected.

### The fix

On non-CUDA devices, drop F16 from the auto-dtype candidates so resolution is
`BF16 -> F32`. BF16 has F32's exponent range and F32 is safe by definition;
F16 is the only unsafe option. CUDA is untouched because flash-attention
accumulates the softmax in F32 and never materialises an overflowing F16 score
matrix.

`0002-optional-naive-sdpa-f32-scores.patch` is **optional** hardening: it makes
the eager `naive_sdpa` path compute scores in F32 even if F16 is forced
explicitly. It does **not** cover the Metal fused-SDPA path — only `0001` does.
Apply `0001` regardless.

### Apply, publish, and consume

```sh
# In a checkout of the fork (setoelkahfi/mistral.rs, branch fix/all-platform-fixes):
git apply /path/to/onde/patches/0001-prefer-f32-over-f16-gguf-attention.patch
# optional:
git apply /path/to/onde/patches/0002-optional-naive-sdpa-f32-scores.patch

cargo test -p onde-mistralrs-core
# bump onde-mistralrs / onde-mistralrs-core to 0.8.3, commit, tag, publish:
cargo publish -p onde-mistralrs-core   # then the dependent crates
```

Then in **this** repo (`onde`), bump the dependency in `Cargo.toml` from
`0.8.2` to `0.8.3` (every `onde-mistralrs` / `onde-mistralrs-core` line) and run
`cargo update -p onde-mistralrs-core`.

> The path prefix in the patches is `mistralrs-core/` (the fork's workspace
> directory). The published crate ships the same file as `src/utils/normal.rs`
> and `src/attention/backends/naive.rs`. If `git apply` reports an offset,
> apply with `git apply --recount`, or make the one-line change by hand — the
> hunks are tiny.

## In-repo safety net (already applied here)

Until `onde-mistralrs 0.8.3` ships, `src/inference/engine.rs` detects the
NaN/Inf sampler error (`augment_inference_error`) and appends an actionable hint
("half-precision overflow … try a smaller model or run on Apple Silicon")
instead of surfacing only the cryptic engine message. This is a message-only
change; it does not alter inference behaviour.
