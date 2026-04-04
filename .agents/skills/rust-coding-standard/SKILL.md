---
name: rust-coding-standard
description: Rust coding standards for the onde crate. Covers error handling, naming, async patterns, platform gating, clippy compliance, and cross-platform SDK conventions. Apply to all Rust edits.
allowed-tools: Read, Write, Edit, Glob, Grep
user-invocable: false
---

# Rust Coding Standard

Rules for writing Rust in the `onde` crate. These are traps to avoid, not maps to follow.

---

## Correctness First

Prioritize correctness and clarity. Performance is secondary unless explicitly stated. Complete, mostly-correct code beats perfect code that doesn't solve the problem.

## Error Handling

- Propagate errors with `?`. Never call `.unwrap()` or `.expect()` outside of `#[cfg(test)]` code.
- Never silently discard errors with `let _ =` on fallible operations:
  - Propagate with `?` when the caller should handle it.
  - Use `log::warn!` or `log::error!` when you need to ignore but want visibility.
  - Use `match` or `if let Err(e)` when custom recovery logic is needed.
- Use `anyhow` for internal application errors in `engine.rs`.
- Use `thiserror` for the `InferenceError` enum — it crosses the FFI boundary via UniFFI.
- Be careful with indexing operations that may panic if out of bounds. Prefer `.get()` and safe alternatives.

## Naming

- Use full words for all variable and parameter names. No single-letter names, no abbreviations (`environment` not `env`, `client` not `c`, `message` not `msg`).
- `snake_case` for functions and variables, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Model constants live in `src/inference/models.rs`. Prefix HuggingFace repo IDs with the org name: `BARTOWSKI_QWEN25_...`.

## Comments

- Comments explain **why**, never **what**. Do not write comments that summarize or re-describe the code.
- Doc comments (`///`) on public items are fine and encouraged — they explain the contract, not the implementation.
- Do not add organizational separators like `// --- Helper functions ---` unless they are already established in the file's style.

## Module Structure

- **No `mod.rs` for new modules.** Use `src/some_module.rs` instead of `src/some_module/mod.rs`. Exception: `inference/mod.rs` already exists and is the intentional module root — don't move it.
- Prefer implementing functionality in existing files unless it's a genuinely new logical component. Avoid creating many small single-purpose files.

## Async

- Runtime is `tokio`. All async functions use `tokio::sync::Mutex`, not `std::sync::Mutex`.
- Use variable shadowing to scope `.clone()` calls inside async closures, minimizing borrow lifetimes:

```rust
tokio::spawn({
    let engine = engine.clone();
    async move {
        engine.do_work().await;
    }
});
```

## Logging

- Use `log` crate macros: `log::debug!`, `log::info!`, `log::warn!`, `log::error!`.
- No `println!` or `eprintln!` in library code. Ever.
- Log at `debug` for routine operations, `info` for lifecycle events (model loaded/unloaded), `warn` for recoverable issues, `error` for failures.

## Platform Gating

- Use `#[cfg(target_os = "...")]` blocks. Match `Cargo.toml`'s target-conditional dependency sections.
- When a function is only used on specific platforms, gate the function itself — not just the call site. This prevents dead-code warnings on other targets.
- When an import is only used on specific platforms, gate the `use` statement separately. Do not put platform-only imports inside a shared `use` block.
- The supported `target_os` values are: `macos`, `ios`, `tvos`, `windows`, `linux`, `android`.
- ISQ model loading is `#[cfg(target_os = "macos")]` only. Gate the `Isq` variant, its match arms, and its imports (`IsqBits`, `TextModelBuilder`) to macOS.

## Clippy Compliance

All code must pass `cargo clippy --all-targets -- -D warnings`. Common traps:

- `&PathBuf` in function parameters → use `&Path` instead.
- `unwrap_or_else(Type::default)` → use `unwrap_or_default()`.
- `filter_map(|x| { ... Some(y) })` where every path returns `Some` → use `map`.
- `.map(|x| f(x))` where `f` is a plain function → use `.map(f)`.
- Redundant type annotations in closures: `.map(|c: &String| c.trim())` → `.map(|c| c.trim())`.
- Unused imports/variants on platform-gated code → gate the import/variant to the same platform.

## Types for FFI

- `usize` / `isize` are not FFI-safe. Use `u64` / `i64` in any type that crosses the UniFFI boundary.
- `impl Into<String>` is not FFI-safe. Use concrete `String` in exported signatures.
- All UniFFI-exported types go in `src/inference/types.rs`. The FFI wrapper lives in `src/inference/ffi.rs`.
- `ChatEngine` (engine.rs) owns Rust-idiomatic logic. `OndeChatEngine` (ffi.rs) is the thin FFI wrapper. Never put `#[uniffi::export]` on `ChatEngine`.

## Re-exports

`mistralrs`, `hf_hub`, and `mistralrs_core` are re-exported from `lib.rs` for downstream Rust consumers. Keep these in sync with what's actually available per platform.

## Testing

- Unit tests go in a `#[cfg(test)] mod tests` block at the bottom of the file they test.
- Use `#[tokio::test]` for async tests.
- Tests may use `.unwrap()` and `.expect()` — the panics are the assertions.
- Test names describe the scenario: `engine_send_without_model_errors`, not `test_send` or `test_1`.
- Run `cargo test --lib` to skip integration tests that require model downloads.

## Adding a New Model

This is a common task with a specific checklist:

1. Add `pub const` entries in `src/inference/models.rs` for repo ID, GGUF filename, and `TOK_MODEL_ID`.
2. Add the repo ID to `SUPPORTED_MODELS`.
3. Add a `SupportedModelInfo` entry to `SUPPORTED_MODEL_INFO` with `expected_size_bytes` from the HF API.
4. Add a constructor to `GgufModelConfig` in `engine.rs` (follow `qwen25_1_5b` pattern).
5. Export a free function in `ffi.rs` (follow `qwen25_1_5b_config` pattern).

## Build Verification

Always run `cargo check` before returning work. If code does not compile, fix it — do not return with TODOs. On macOS, also run `cargo clippy --all-targets -- -D warnings` to catch lint issues before CI does.