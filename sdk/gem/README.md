# Onde — Ruby Gem

Ruby bindings for the [Onde](https://ondeinference.com) on-device inference engine, powered by Rust via [magnus](https://github.com/matsadler/magnus).

Manage HuggingFace model caches, inspect supported models, and access inference configuration — all from Ruby, with zero C dependencies.

## Requirements

- Ruby >= 3.0
- Rust >= 1.87 (installed via [rustup](https://rustup.rs))
- macOS, Linux, or Windows

## Installation

Add to your Gemfile:

```ruby
gem 'onde'
```

Then:

```bash
bundle install
```

Or install directly:

```bash
gem install onde
```

The gem compiles a native Rust extension on install. Rust must be available on your `$PATH`.

## Development Setup

```bash
cd lib/crates/onde/gem

# Install dependencies and compile the native extension
bin/setup

# Or step by step:
bundle install
bundle exec rake compile

# Open an interactive console with the gem loaded
bin/console
```

## Building & Testing

```bash
# Compile the Rust extension
bundle exec rake compile

# Build the gem package
bundle exec rake build

# Run the default task (compile)
bundle exec rake
```

## API Reference

### `Onde.list_local_models` → Hash

Scans the local HuggingFace hub cache (`~/.cache/huggingface/hub/`) and returns all downloaded models that the inference engine supports.

```ruby
response = Onde.list_local_models

response["cache_path"]         # => "/Users/you/.cache/huggingface/hub"
response["total_size_bytes"]   # => 24768454656
response["total_size_display"] # => "23.07 GB"
response["models"].each do |model|
  puts "#{model["model_id"]} — #{model["size_display"]} (#{model["revisions"].length} revisions)"
  puts "  Path: #{model["path"]}"
end
```

### `Onde.list_supported_models` → Hash

Returns all models the engine supports with download status flags.

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

Each model Hash contains:

| Key                     | Type    | Description                                        |
|-------------------------|---------|----------------------------------------------------|
| `model_id`              | String  | Full HuggingFace ID, e.g. `"bartowski/Qwen2.5-1.5B-Instruct-GGUF"` |
| `name`                  | String  | Human-friendly display name                        |
| `org`                   | String  | Organisation or publisher                          |
| `description`           | String  | Short description of the model                     |
| `is_downloaded`         | Boolean | Fully downloaded locally                           |
| `is_incomplete`         | Boolean | Partial download exists on disk                    |
| `local_size_bytes`      | Integer | Bytes currently on disk                            |
| `local_size_display`    | String  | Human-readable local size                          |
| `expected_size_bytes`   | Integer | Approximate total size when fully downloaded       |
| `expected_size_display` | String  | Human-readable expected size                       |

### `Onde.delete_model(model_id)` → nil

Delete a locally cached model. Raises `RuntimeError` if the model is not found or deletion fails.

```ruby
Onde.delete_model("bartowski/Qwen2.5-1.5B-Instruct-GGUF")
```

### `Onde.model_info(model_id)` → Hash or nil

Look up rich metadata for a single supported model. Returns `nil` if the model ID is not in the supported list.

```ruby
info = Onde.model_info("bartowski/Qwen2.5-1.5B-Instruct-GGUF")
# => {
#   "id"                  => "bartowski/Qwen2.5-1.5B-Instruct-GGUF",
#   "name"                => "Qwen 2.5 1.5B (GGUF)",
#   "org"                 => "Qwen / Alibaba",
#   "description"         => "Lightest pre-quantized chat model — ideal for iOS & Android (~941 MB)...",
#   "expected_size_bytes" => 986048768
# }
```

### `Onde.supported_model_ids` → Array

Returns the list of all supported model IDs as strings.

```ruby
Onde.supported_model_ids
# => ["black-forest-labs/FLUX.1-schnell", "google/gemma-3n-E2B-it", ...]
```

### `Onde.cache_path` → String or nil

Returns the resolved HuggingFace cache directory path, or `nil` if it cannot be determined.

```ruby
Onde.cache_path
# => "/Users/you/.cache/huggingface/hub"
```

### Sampling Config Helpers

Return sampling parameter Hashes suitable for passing to inference engines:

```ruby
Onde.default_sampling_config
# => { "temperature" => 0.7, "top_p" => 0.95, "max_tokens" => 512, ... }

Onde.deterministic_sampling_config
# => { "temperature" => 0.0, "max_tokens" => 512, ... }

Onde.mobile_sampling_config
# => { "temperature" => 0.7, "top_p" => 0.95, "max_tokens" => 128, ... }
```

### Constants

```ruby
Onde::VERSION            # => "0.1.0" (gem version)
Onde::NATIVE_VERSION     # => "0.1.0" (Rust crate version)
Onde::SUPPORTED_MODELS   # => ["black-forest-labs/FLUX.1-schnell", ...] (frozen Array)
Onde::SUPPORTED_MODEL_INFO # => [{ "id" => ..., "name" => ..., ... }, ...] (frozen Array of frozen Hashes)
```

## Example: Rails Admin Dashboard

```ruby
# app/controllers/admin/models_controller.rb
class Admin::ModelsController < ApplicationController
  def index
    @supported = Onde.list_supported_models["models"]
    @local     = Onde.list_local_models
  end

  def destroy
    Onde.delete_model(params[:model_id])
    redirect_to admin_models_path, notice: "Model deleted."
  rescue RuntimeError => e
    redirect_to admin_models_path, alert: e.message
  end
end
```

## Architecture

```
gem/
├── onde.gemspec                 # Gem specification
├── Gemfile                     # Dev dependencies (rake-compiler, rb_sys)
├── Rakefile                    # Build tasks via RbSys::ExtensionTask
├── rust-toolchain.toml         # Pinned Rust toolchain version
├── lib/
│   ├── onde.rb                 # Entry point — loads native ext + defines module
│   └── onde/
│       └── version.rb          # Onde::VERSION
├── ext/
│   └── onde/
│       ├── Cargo.toml          # Rust crate — depends on `onde` + `magnus`
│       ├── extconf.rb          # rb_sys build hook
│       └── src/
│           └── lib.rs          # Magnus bindings — the bridge between Rust and Ruby
└── bin/
    ├── console                 # IRB with gem loaded
    └── setup                   # One-step dev setup
```

The native extension compiles the `onde` Rust crate (which wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) for LLM inference and HuggingFace Hub for model management) into a shared library (`.bundle` on macOS, `.so` on Linux) that Ruby loads at require time.

All Rust structs are converted to Ruby Hashes via `serde_json` serialization, keeping the binding layer thin and the Ruby API simple — no custom Ruby classes needed.

## License

MIT OR Apache-2.0 — same as the Onde crate.
