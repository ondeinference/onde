---
name: uniffi-rs
description: Generate Swift, Kotlin, and Python bindings from Rust using UniFFI. Use when adding, modifying, or debugging FFI-exported types and functions in the onde crate.
allowed-tools: Read, Write, Edit, Glob, Grep
user-invocable: false
---

# uniffi-rs

## What This Skill Covers

How UniFFI is used in the `onde` crate to generate Swift, Kotlin, and Python bindings from a single Rust implementation.

## Version Pin

UniFFI is pinned to exactly `=0.31.0` everywhere. This version must stay in sync across:
- `onde/Cargo.toml` → `uniffi = { version = "=0.31.0", features = ["tokio"] }`
- `onde/Cargo.toml` build-deps → `uniffi = { version = "=0.31.0", features = ["build"] }`
- `onde/uniffi-bindgen/Cargo.toml` → `uniffi = { version = "=0.31.0", features = ["cli"] }`

Never bump one without bumping all three. Mismatched versions produce silently broken bindings.

## Project Structure

```
onde/
├── src/lib.rs                  # uniffi::setup_scaffolding!()
├── src/inference/
│   ├── types.rs                # #[derive(uniffi::Record/Enum/Error)]
│   └── ffi.rs                  # #[derive(uniffi::Object)], #[uniffi::export]
├── uniffi-bindgen/
│   ├── Cargo.toml              # Standalone bin crate for the bindgen CLI
│   └── uniffi-bindgen.rs       # fn main() { uniffi::uniffi_bindgen_main() }
├── uniffi.toml                 # Binding config (package names, cdylib name)
└── build.rs                    # uniffi build support (build-dep)
```

## Scaffolding Setup

`src/lib.rs` uses the proc-macro approach:

```rust
uniffi::setup_scaffolding!();
```

This is equivalent to the UDL approach but requires no `.udl` file. The `build.rs` still declares the `uniffi` build-dep to keep compatibility if the project ever switches to UDL.

## Type Annotations

| Rust construct | UniFFI derive | Crosses FFI as |
|---|---|---|
| Plain data struct | `#[derive(uniffi::Record)]` | Swift `struct` / Kotlin `data class` |
| C-like / rich enum | `#[derive(uniffi::Enum)]` | Swift `enum` / Kotlin `sealed class` |
| `thiserror::Error` enum | `#[derive(uniffi::Error)]` | Swift `Error enum` / Kotlin exception |
| `Arc<T>` wrapper struct | `#[derive(uniffi::Object)]` | Swift `class` / Kotlin `object` |
| Trait for callbacks | `#[uniffi::export(callback_interface)]` | Swift `protocol` / Kotlin `interface` |

All exported functions and impl blocks are annotated with `#[uniffi::export]`.

## Type Compatibility Rules

These constraints come directly from UniFFI 0.31 limitations:

- **`usize` / `isize` are not supported** — use `u64` / `i64` instead.
- **`impl Into<String>` is not FFI-safe** — use concrete `String` in all exported method signatures.
- **`tokio::sync::mpsc::Receiver` is not FFI-safe** — streaming is exposed via a `callback_interface` trait delivered through a free function, not an Object method.
- **`callback_interface` parameters must be on free functions** — UniFFI 0.31 does not support `callback_interface` arguments on `#[uniffi::Object]` methods.
- **Generics are not supported** — all exported types must be fully concrete.

## The uniffi-bindgen CLI

`onde/uniffi-bindgen/` is a minimal standalone binary crate:

```rust
// uniffi-bindgen.rs
fn main() {
    uniffi::uniffi_bindgen_main()
}
```

Build it once and run it to generate bindings:

```bash
cargo build --manifest-path uniffi-bindgen/Cargo.toml --release

# Generate Swift
./target/release/uniffi-bindgen generate \
    --library target/aarch64-apple-tvos/release/libonde.a \
    --language swift \
    --out-dir sdk/Onde/Sources/Onde \
    --config uniffi.toml

# Generate Kotlin
./target/release/uniffi-bindgen generate \
    --library target/aarch64-linux-android/release/libonde.so \
    --language kotlin \
    --out-dir sdk/kotlin/onde/src/main/kotlin \
    --config uniffi.toml

# Generate Python
./target/release/uniffi-bindgen generate \
    --library target/release/libonde.dylib \
    --language python \
    --out-dir sdk/python \
    --config uniffi.toml
```

The `--library` argument points to a compiled artifact (`.a`, `.so`, or `.dylib`). UniFFI reads the embedded metadata from the binary — any compiled artifact from any platform works for generating bindings for any language.

## uniffi.toml

```toml
[bindings.kotlin]
package_name = "com.ondeinference.onde"
cdylib_name = "onde"

[bindings.python]
cdylib_name = "onde"
```

No Swift-specific config is needed here. `cargo-swift` (when used) and the raw `uniffi-bindgen` both derive Swift names from the Rust struct/function names.

## Async Support

UniFFI 0.31 supports `async fn` exports when the crate enables the `tokio` feature:

```toml
uniffi = { version = "=0.31.0", features = ["tokio"] }
```

All `async fn` methods on `#[uniffi::Object]` and all `async` free functions annotated with `#[uniffi::export]` are automatically bridged to Swift `async throws` and Kotlin `suspend fun`.

## Streaming Pattern

Because `callback_interface` is not allowed on Object methods in UniFFI 0.31, streaming is exposed as a top-level free function that accepts the Object by `Arc<T>`:

```rust
// CORRECT — free function with Arc<Object>
#[uniffi::export]
pub async fn stream_chat_message(
    engine: Arc<OndeChatEngine>,
    message: String,
    listener: Box<dyn StreamChunkListener>,  // callback_interface
) -> Result<(), InferenceError> { ... }

// WRONG — callback_interface on Object method (not supported in 0.31)
// impl OndeChatEngine {
//     pub async fn stream(&self, listener: Box<dyn StreamChunkListener>) { ... }
// }
```

## Adding a New Exported Type

1. Define the type in `src/inference/types.rs` (or whichever module owns it).
2. Add the appropriate `#[derive(uniffi::Record)]` / `#[derive(uniffi::Enum)]` / `#[derive(uniffi::Error)]`.
3. Replace any `usize` fields with `u64`, and any `isize` with `i64`.
4. If the type needs to be exported from the FFI Object, add it to the method signatures in `src/inference/ffi.rs`.
5. Recompile for any Apple target and re-run `uniffi-bindgen generate` for Swift.
6. Re-run `build-kotlin.sh` for Kotlin.

## Common Pitfalls

- **Version mismatch**: If the `uniffi` version in `onde/Cargo.toml` differs from the one in `onde/uniffi-bindgen/Cargo.toml`, the generated bindings will be incompatible with the compiled library at runtime. Always keep them identical with the `=` exact pin.
- **Forgetting `setup_scaffolding!()`**: Without `uniffi::setup_scaffolding!()` in `lib.rs`, no scaffolding code is generated and all `#[uniffi::export]` annotations are silently ignored.
- **`usize` in struct fields**: UniFFI will produce a hard compile error or a confusing runtime panic if `usize` appears in any exported type. Always use `u64`.
- **Callback interface in Object method**: This compiles fine but panics at runtime in UniFFI 0.31. Move the method to a free function.
- **Not rebuilding the bindgen binary**: If you update `uniffi` in the main `Cargo.toml` without rebuilding `uniffi-bindgen`, the CLI will use the old schema and generate stale/broken bindings.