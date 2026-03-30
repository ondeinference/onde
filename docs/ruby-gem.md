---
title: "Ruby Gem"
description: "Build and use the onde-inference Ruby gem — a Magnus-based native extension exposing HuggingFace cache management and model metadata to Ruby."
---

# Onde Ruby Gem

Ruby bindings for the [Onde](https://ondeinference.com) on-device inference engine, powered by Rust via [magnus](https://github.com/matsadler/magnus).

Manage HuggingFace model caches, inspect supported models, and access inference configuration — all from Ruby with native Rust performance.

## Prerequisites

- **Ruby** >= 3.0
- **Rust** >= 1.92 (installed via [rustup](https://rustup.rs))
- **Bundler** (`gem install bundler`)
- macOS, Linux, or Windows

The gem compiles a native Rust extension at install time. Rust must be available on your `$PATH`.

## Setup

```bash
cd lib/crates/onde/gem

# Install Ruby dependencies and compile the Rust native extension
bin/setup

# Or step by step:
bundle install
bundle exec rake compile
```

### Verify the installation

```bash
bundle exec ruby -e "require 'onde'; puts Onde::VERSION"
# => 0.1.0
```

### Interactive console

```bash
bin/console
```

Opens an IRB session with the gem pre-loaded. Useful for exploring the API:

```ruby
irb> Onde::SUPPORTED_MODELS
=> ["bartowski/Qwen2.5-1.5B-Instruct-GGUF", "bartowski/Qwen2.5-3B-Instruct-GGUF"]

irb> Onde.cache_path
=> "/Users/you/.cache/huggingface/hub"
```

## Build Commands

| Command | Description |
|---------|-------------|
| `bundle exec rake compile` | Compile the Rust native extension (release profile) |
| `bundle exec rake compile:dev` | Compile with debug profile (faster builds, slower runtime) |
| `bundle exec rake build` | Build the `.gem` package |
| `bundle exec rake` | Default task — compiles the extension |
| `bin/setup` | One-step: `bundle install` + `rake compile` |
| `bin/console` | IRB with the gem loaded |

## API Reference

### Cache Management

#### `Onde.list_local_models` → Hash

Scans the local HuggingFace hub cache (`~/.cache/huggingface/hub/`) and returns all downloaded models that the inference engine supports.

```ruby
response = Onde.list_local_models

response["cache_path"]         # => "/Users/you/.cache/huggingface/hub"
response["total_size_bytes"]   # => 24768454656
response["total_size_display"] # => "23.07 GB"

response["models"].each do |model|
  puts "#{model["model_id"]} — #{model["size_display"]}"
  puts "  Path: #{model["path"]}"
  puts "  Revisions: #{model["revisions"].join(", ")}"
end
```

**Return value keys:**

| Key | Type | Description |
|-----|------|-------------|
| `cache_path` | String | Absolute path that was scanned |
| `total_size_bytes` | Integer | Total size of all cached models |
| `total_size_display` | String | Human-readable total size (e.g. `"23.07 GB"`) |
| `models` | Array | Array of model Hashes (see below) |

**Each model Hash:**

| Key | Type | Description |
|-----|------|-------------|
| `model_id` | String | Full HuggingFace ID (e.g. `"bartowski/Qwen2.5-1.5B-Instruct-GGUF"`) |
| `org` | String | Organisation or publisher |
| `name` | String | Model name without org prefix |
| `size_bytes` | Integer | Total size on disk in bytes |
| `size_display` | String | Human-readable size (e.g. `"1.84 GB"`) |
| `path` | String | Absolute path to the model cache directory |
| `revisions` | Array | List of snapshot revision strings |

---

#### `Onde.list_supported_models` → Hash

Returns all models the engine supports, together with flags indicating whether each one is fully downloaded, partially downloaded, or absent.

```ruby
Onde.list_supported_models["models"].each do |m|
  status = if m["is_downloaded"]
             "✓ downloaded"
           elsif m["is_incomplete"]
             "⏳ incomplete (#{m["local_size_display"]} / #{m["expected_size_display"]})"
           else
             "✗ not downloaded"
           end
  puts "[#{status}] #{m["name"]} (#{m["org"]})"
  puts "  #{m["description"]}"
end
```

**Each model Hash:**

| Key | Type | Description |
|-----|------|-------------|
| `model_id` | String | Full HuggingFace ID |
| `name` | String | Human-friendly display name |
| `org` | String | Organisation or publisher |
| `description` | String | Short description of the model |
| `is_downloaded` | Boolean | Fully downloaded locally |
| `is_incomplete` | Boolean | Partial download exists on disk |
| `local_size_bytes` | Integer | Bytes currently on disk (0 if absent) |
| `local_size_display` | String | Human-readable local size |
| `expected_size_bytes` | Integer | Approximate total size when fully downloaded |
| `expected_size_display` | String | Human-readable expected size |

---

#### `Onde.delete_model(model_id)` → nil

Delete a locally cached HuggingFace model by removing its directory from the hub cache.

```ruby
Onde.delete_model("bartowski/Qwen2.5-1.5B-Instruct-GGUF")
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `model_id` | String | Full model identifier (e.g. `"bartowski/Qwen2.5-1.5B-Instruct-GGUF"`) |

**Raises:** `RuntimeError` if the model is not found or deletion fails.

---

#### `Onde.cache_path` → String or nil

Returns the resolved HuggingFace cache directory path, or `nil` if it cannot be determined (e.g. `$HOME` is unset).

```ruby
Onde.cache_path
# => "/Users/you/.cache/huggingface/hub"
```

---

### Model Metadata

#### `Onde.model_info(model_id)` → Hash or nil

Look up rich metadata for a single supported model. Returns `nil` if the model ID is not in the supported list.

```ruby
info = Onde.model_info("bartowski/Qwen2.5-1.5B-Instruct-GGUF")
# => {
#   "id"                  => "bartowski/Qwen2.5-1.5B-Instruct-GGUF",
#   "name"                => "Qwen 2.5 1.5B (GGUF)",
#   "org"                 => "Qwen / Alibaba",
#   "description"         => "Lightest pre-quantized chat model — ...",
#   "expected_size_bytes" => 986048768
# }

Onde.model_info("nonexistent/model")
# => nil
```

**Return value keys:**

| Key | Type | Description |
|-----|------|-------------|
| `id` | String | Full HuggingFace model identifier |
| `name` | String | Human-friendly display name |
| `org` | String | Organisation or publisher |
| `description` | String | Short description |
| `expected_size_bytes` | Integer | Approximate total size in bytes |

---

#### `Onde.supported_model_ids` → Array

Returns the list of all supported model IDs as strings.

```ruby
Onde.supported_model_ids
# => [
#   "bartowski/Qwen2.5-1.5B-Instruct-GGUF",
#   "bartowski/Qwen2.5-3B-Instruct-GGUF"
# ]
```

---

### Sampling Configuration

These methods return Hash representations of the inference sampling parameters. They are useful for inspecting defaults or passing to future inference APIs.

#### `Onde.default_sampling_config` → Hash

Creative chat defaults.

```ruby
Onde.default_sampling_config
# => {
#   "temperature"       => 0.7,
#   "top_p"             => 0.95,
#   "top_k"             => nil,
#   "min_p"             => nil,
#   "max_tokens"        => 512,
#   "frequency_penalty" => nil,
#   "presence_penalty"  => nil
# }
```

#### `Onde.deterministic_sampling_config` → Hash

Greedy decoding (temperature = 0.0).

```ruby
Onde.deterministic_sampling_config
# => { "temperature" => 0.0, "top_p" => nil, ..., "max_tokens" => 512 }
```

#### `Onde.mobile_sampling_config` → Hash

Conservative defaults for constrained devices (lower `max_tokens`).

```ruby
Onde.mobile_sampling_config
# => { "temperature" => 0.7, "top_p" => 0.95, ..., "max_tokens" => 128 }
```

---

### Constants

| Constant | Type | Description |
|----------|------|-------------|
| `Onde::VERSION` | String | Gem version (e.g. `"0.1.0"`) |
| `Onde::NATIVE_VERSION` | String | Rust crate version (e.g. `"0.1.0"`) |
| `Onde::SUPPORTED_MODELS` | Array | Frozen array of all supported model ID strings |
| `Onde::SUPPORTED_MODEL_INFO` | Array | Frozen array of frozen Hashes with model metadata |

```ruby
Onde::SUPPORTED_MODELS
# => ["bartowski/Qwen2.5-1.5B-Instruct-GGUF", "bartowski/Qwen2.5-3B-Instruct-GGUF"]

Onde::SUPPORTED_MODEL_INFO.first
# => {
#   "id"                  => "bartowski/Qwen2.5-1.5B-Instruct-GGUF",
#   "name"                => "Qwen 2.5 1.5B (GGUF)",
#   "org"                 => "Qwen / Alibaba",
#   "description"         => "Lightest pre-quantized chat model — ...",
#   "expected_size_bytes" => 986048768
# }
```

Both constants are frozen at load time — attempting to modify them raises `FrozenError`.

## Usage Examples

### Rails Admin Dashboard

```ruby
# app/controllers/admin/models_controller.rb
class Admin::ModelsController < ApplicationController
  def index
    @supported = Onde.list_supported_models["models"]
    @local     = Onde.list_local_models
  end

  def destroy
    Onde.delete_model(params[:model_id])
    redirect_to admin_models_path, notice: "Model cache deleted."
  rescue RuntimeError => e
    redirect_to admin_models_path, alert: e.message
  end
end
```

### Background Job — Cache Cleanup

```ruby
# app/jobs/cleanup_hf_cache_job.rb
class CleanupHfCacheJob < ApplicationJob
  queue_as :maintenance

  def perform
    local = Onde.list_local_models
    logger.info "[Onde] HF cache: #{local["total_size_display"]} across #{local["models"].length} models"

    # Delete models that are no longer in the supported list
    local["models"].each do |model|
      unless Onde::SUPPORTED_MODELS.include?(model["model_id"])
        logger.info "[Onde] Removing unsupported model: #{model["model_id"]} (#{model["size_display"]})"
        Onde.delete_model(model["model_id"])
      end
    end
  end
end
```

### Rake Task — Model Status Report

```ruby
# lib/tasks/onde.rake
namespace :onde do
  desc "Show HuggingFace model cache status"
  task status: :environment do
    puts "Cache: #{Onde.cache_path || "(not found)"}"
    puts

    Onde.list_supported_models["models"].each do |m|
      icon = m["is_downloaded"] ? "✓" : (m["is_incomplete"] ? "⏳" : "✗")
      local = m["is_downloaded"] ? m["local_size_display"] : (m["is_incomplete"] ? m["local_size_display"] : "—")
      puts "[#{icon}] #{m["name"].ljust(25)} #{local.rjust(10)} / #{m["expected_size_display"].rjust(10)}  (#{m["org"]})"
    end

    local = Onde.list_local_models
    puts
    puts "Total on disk: #{local["total_size_display"]}"
  end
end
```

Run with:

```bash
bundle exec rake onde:status
```

Example output:

```
Cache: /Users/you/.cache/huggingface/hub

[✓] Qwen 2.5 1.5B (GGUF)        1.84 GB /  940.4 MB  (Qwen / Alibaba)
[✓] Qwen 2.5 3B (GGUF)          3.59 GB /    1.80 GB  (Qwen / Alibaba)

Total on disk: 5.43 GB
```

### API Endpoint

```ruby
# app/controllers/api/v1/models_controller.rb
# frozen_string_literal: true

module Api
  module V1
    class ModelsController < ApplicationController
      def index
        render json: Onde.list_supported_models
      end

      def show
        info = Onde.model_info(params[:id])
        if info
          render json: info
        else
          render json: { error: "Model not found" }, status: :not_found
        end
      end
    end
  end
end
```

## Architecture

```
gem/
├── Cargo.toml                      # Workspace root (isolates from parent onde crate)
├── Gemfile                         # Dev deps: rake-compiler, rb_sys ~> 0.9.63
├── Rakefile                        # RbSys::ExtensionTask build config
├── onde.gemspec                    # Gem specification
├── rust-toolchain.toml             # Pinned to Rust 1.92
├── .gitignore
├── bin/
│   ├── console                     # IRB with gem pre-loaded
│   └── setup                       # One-step dev setup script
├── lib/
│   ├── onde.rb                     # Entry point — loads native ext + defines module
│   └── onde/
│       ├── version.rb              # Onde::VERSION = '0.1.0'
│       └── onde_ruby.bundle        # Compiled native extension (git-ignored)
└── ext/
    └── onde-ruby/
        ├── Cargo.toml              # Rust crate: depends on `onde` + `magnus`
        ├── extconf.rb              # rb_sys build hook
        └── src/
            └── lib.rs              # Magnus bindings — the Rust ↔ Ruby bridge
```

### How the binding works

```
┌───────────────────────────┐
│   Ruby                    │
│   require "onde"          │
│   Onde.list_local_models  │
└───────────┬───────────────┘
            │ FFI call via magnus
            ▼
┌───────────────────────────┐
│   onde-ruby (cdylib)      │
│   ext/onde-ruby/src/lib.rs│
│                           │
│   #[magnus::init]         │
│   fn init(ruby) {         │
│     module.define_*()     │
│   }                       │
└───────────┬───────────────┘
            │ Rust dependency
            ▼
┌───────────────────────────┐
│   onde (Rust crate)       │
│   lib/crates/onde/src/    │
│                           │
│   hf_cache::              │
│   inference::models::     │
│   inference::types::      │
└───────────┬───────────────┘
            │
            ▼
┌───────────────────────────┐
│   mistral.rs              │
│   HuggingFace Hub         │
│   Metal / CUDA / CPU      │
└───────────────────────────┘
```

### Data flow

All Rust structs are converted to Ruby Hashes via `serde_json` round-tripping:

```
Rust struct ─→ serde_json::Value ─→ Ruby Hash/Array/String/Integer/nil
```

This keeps the binding layer thin — no custom Ruby classes are needed. Every Onde struct that implements `serde::Serialize` can be exposed to Ruby with a single `to_ruby_value()` call.

### Workspace isolation

The `gem/Cargo.toml` defines a Cargo workspace with `ext/onde-ruby` as its only member. This prevents `cargo metadata` from walking up to the parent `lib/crates/onde/Cargo.toml` and confusing the rb_sys build system.

Without this workspace file, running `cargo metadata` from `gem/` resolves to the parent `onde` crate, and `RbSys::Cargo::Metadata` cannot find the `onde-ruby` package.

### Naming convention

| Layer | Name | Why |
|-------|------|-----|
| Cargo package | `onde-ruby` | Avoids collision with the parent `onde` crate in the dependency graph |
| Cargo lib output | `onde_ruby` | Rust convention: hyphens become underscores in lib names |
| Ruby require | `onde/onde_ruby` | Matches the compiled `.bundle`/`.so` filename |
| Ruby module | `Onde` | Clean public API name |
| Gem name | `onde` | Simple, matches the project name |

## Internals

### Key files

| File | Purpose |
|------|---------|
| `ext/onde-ruby/src/lib.rs` | All Magnus bindings — defines the `Onde` Ruby module, singleton methods, and constants |
| `ext/onde-ruby/Cargo.toml` | Rust crate config — depends on `onde` (path) + `magnus` 0.7 + `serde_json` |
| `ext/onde-ruby/extconf.rb` | Build hook — calls `create_rust_makefile("onde/onde_ruby")` |
| `lib/onde.rb` | Ruby entry point — `require_relative 'onde/onde_ruby'` loads the native ext |
| `Rakefile` | `RbSys::ExtensionTask.new('onde-ruby', GEMSPEC)` wires up compilation |
| `Cargo.toml` (root) | Workspace file isolating the gem from the parent crate |

### Dependencies

**Ruby side:**

- `rake` ~> 13.0 — build orchestration
- `rake-compiler` — native extension compilation framework
- `rb_sys` ~> 0.9.63 — Rust ↔ Ruby build glue

**Rust side:**

- `magnus` 0.7 — Ruby FFI bindings for Rust
- `serde` + `serde_json` — struct serialization to Ruby Hashes
- `onde` (path dependency) — the core Onde crate

### Adding a new method

1. Add the Rust function in `ext/onde-ruby/src/lib.rs`:

   ```rust
   fn my_new_method(arg: String) -> Result<magnus::Value, Error> {
       let ruby = Ruby::get().expect("called outside Ruby");
       // ... call into onde crate ...
       Ok(ruby.str_new("result").as_value())
   }
   ```

2. Register it in the `init` function:

   ```rust
   module.define_singleton_method("my_new_method", function!(my_new_method, 1))?;
   ```

3. Recompile:

   ```bash
   bundle exec rake compile
   ```

4. Test:

   ```bash
   bundle exec ruby -e "require 'onde'; puts Onde.my_new_method('hello')"
   ```

### Adding a new constant

In the `init` function in `ext/onde-ruby/src/lib.rs`:

```rust
module.const_set("MY_CONSTANT", ruby.str_new("value"))?;
```

Constants are available as `Onde::MY_CONSTANT` in Ruby. Call `.freeze()` on Arrays and Hashes before setting them as constants to prevent mutation.

## Troubleshooting

### `cargo` not found during `bundle install`

Rust is not on your `$PATH`. Install via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Compilation fails with `rustc X.Y.Z is not supported`

The gem requires Rust >= 1.92. Update your toolchain:

```bash
rustup update
# or install the specific version:
rustup toolchain install 1.92
```

The `rust-toolchain.toml` in the gem directory pins the version — rustup will select it automatically when you `cd` into the directory.

### `Could not find Cargo package metadata for "onde-ruby"`

The workspace `Cargo.toml` at `gem/` is missing or corrupted. It must exist and contain:

```toml
[workspace]
members = ["ext/onde-ruby"]
resolver = "2"
```

Without this file, `cargo metadata` resolves to the parent `onde` crate and rb_sys cannot find `onde-ruby`.

### Output filename collision warning

You may see:

```
warning: output filename collision at .../libonde.dylib
  note: the lib target `onde` in package `onde-ruby` has the same output
        filename as the lib target `onde` in package `onde`
```

This is a Cargo warning (not an error) caused by the extension crate's lib output name (`onde_ruby`) sharing a target directory with the parent `onde` crate. It does not affect compilation. The warning will become a hard error in a future Cargo version — at that point, set a separate `target-dir` in the workspace.

### `LoadError: cannot load such file -- onde/onde_ruby`

The native extension hasn't been compiled. Run:

```bash
bundle exec rake compile
```

### Stale `.bundle` / `.so` after code changes

If you change `ext/onde-ruby/src/lib.rs` and the changes don't take effect:

```bash
bundle exec rake clobber compile
```

`clobber` cleans all build artifacts before recompiling.

## Version Compatibility

| Onde gem | Onde crate | Rust toolchain | Ruby | magnus |
|----------|-----------|----------------|------|--------|
| 0.1.0 | 0.1.0 | >= 1.92 | >= 3.0 | 0.7 |

The `Onde::NATIVE_VERSION` constant always reflects the Rust crate version compiled into the gem. Compare it against `Onde::VERSION` to detect mismatches:

```ruby
if Onde::VERSION != Onde::NATIVE_VERSION
  warn "Onde version mismatch: gem=#{Onde::VERSION} native=#{Onde::NATIVE_VERSION}"
end
```
