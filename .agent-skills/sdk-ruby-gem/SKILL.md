---
name: sdk-ruby-gem
description: Build and publish the onde-inference Ruby gem (Magnus-based native extension). Use when working on the Ruby SDK under sdk/gem/, adding Ruby-exposed functions, or publishing to RubyGems.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: true
---

# Skill: Ruby Gem SDK (`sdk/gem/`)

## What This Is

A Magnus-based Ruby native extension that exposes Onde's HuggingFace cache management and model metadata to Ruby. The gem is named `onde-inference` on RubyGems. It does **not** expose the `ChatEngine` / `OndeChatEngine` — it is a cache + model metadata utility layer.

## Source Layout

```
sdk/gem/
├── Cargo.toml           # Rust extension crate (magnus + onde dep)
├── Gemfile
├── Rakefile             # `rake compile` → builds the .so via rb-sys-dock
├── onde-inference.gemspec
├── bin/
│   ├── console          # irb + require 'onde'
│   └── setup            # bundle install + rake compile
├── ext/onde/            # Rust extension source
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs       # #[magnus::init] entry point
└── lib/
    └── onde/
        └── version.rb
```

## Build Commands

```bash
cd sdk/gem

# First-time setup
bin/setup               # bundle install + rake compile

# Recompile after Rust changes
bundle exec rake compile

# Interactive console
bin/console

# Verify
bundle exec ruby -e "require 'onde'; puts Onde::VERSION; puts Onde::SUPPORTED_MODELS"
```

## Key Facts

- **Magnus** is the Rust↔Ruby binding layer (`magnus` crate). All Ruby-visible types are wrapped with `#[magnus::init]` and `wrap_class!` / `rb_class!` macros.
- The gem depends on `onde` as a relative path in `sdk/gem/ext/onde/Cargo.toml` — never use a published git ref here during development.
- **`rb-sys`** drives `rake compile` — it invokes `cargo build` with the correct Ruby extension flags.
- The gem is published as `onde-inference` on RubyGems (`gem push onde-inference-*.gem`).
- `Onde::SUPPORTED_MODELS` is a Ruby Array of model ID strings matching `onde::inference::models::SUPPORTED_MODELS`.
- Cache functions (`list_local_models`, `list_supported_models`, `delete_model`, `diagnose_cache`, `repair_symlinks`) are exposed as module-level methods under the `Onde` namespace.

## Pitfalls

- `rake compile` must be re-run after any change to Rust source under `ext/onde/src/` — the gem does not hot-reload.
- On macOS, `LIBRARY_PATH` and `DYLD_LIBRARY_PATH` must include the Ruby lib path if linking fails at test time.
- Magnus `Value` / `RArray` / `RHash` types are **not** `Send` — do not store them across Ruby GC boundaries.
- `bundle exec` is required for all rake/ruby invocations so the correct gem/ruby version is used.
- `onde-inference` (hyphen) on RubyGems; `require 'onde'` (no hyphen, no suffix) in Ruby code.