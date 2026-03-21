//! Ruby native extension for the Onde on-device inference engine.
//!
//! This crate uses [magnus](https://github.com/matsadler/magnus) to expose
//! Onde's HuggingFace cache management, supported model metadata, and
//! inference types to Ruby as the `Onde` module.
//!
//! ## Ruby usage
//!
//! ```ruby
//! require "onde"
//!
//! # List models cached locally on disk.
//! response = Onde.list_local_models
//! response["models"].each do |m|
//!   puts "#{m["model_id"]} — #{m["size_display"]}"
//! end
//!
//! # List all supported models (with download status).
//! Onde.list_supported_models["models"].each do |m|
//!   status = m["is_downloaded"] ? "✓" : "✗"
//!   puts "[#{status}] #{m["name"]} (#{m["org"]}) — #{m["expected_size_display"]}"
//! end
//!
//! # Delete a cached model.
//! Onde.delete_model("bartowski/Qwen2.5-1.5B-Instruct-GGUF")
//!
//! # Inspect supported model IDs.
//! Onde::SUPPORTED_MODELS  # => ["black-forest-labs/FLUX.1-schnell", ...]
//!
//! # Access model metadata.
//! Onde.model_info("bartowski/Qwen2.5-1.5B-Instruct-GGUF")
//! # => { "id" => "bartowski/…", "name" => "Qwen 2.5 1.5B (GGUF)", … }
//!
//! # Sampling config helpers.
//! Onde.default_sampling_config
//! Onde.deterministic_sampling_config
//! Onde.mobile_sampling_config
//! ```

use magnus::{function, prelude::*, Error, RHash, Ruby};

use onde::hf_cache;
use onde::inference::models::{SUPPORTED_MODELS, SUPPORTED_MODEL_INFO};
use onde::inference::types::SamplingConfig;

// ---------------------------------------------------------------------------
// Helpers — convert Rust structs to Ruby hashes via serde_json
// ---------------------------------------------------------------------------

/// Serialize any `serde::Serialize` value into a Ruby Hash (or Array of
/// Hashes) by round-tripping through serde_json.  This keeps the binding
/// layer thin — we don't need to define Ruby classes for every Onde struct.
fn to_ruby_value<T: serde::Serialize>(ruby: &Ruby, value: &T) -> Result<magnus::Value, Error> {
    let json = serde_json::to_value(value).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("serialization error: {e}"),
        )
    })?;
    json_to_ruby(ruby, &json)
}

fn json_to_ruby(ruby: &Ruby, value: &serde_json::Value) -> Result<magnus::Value, Error> {
    match value {
        serde_json::Value::Null => Ok(ruby.qnil().as_value()),
        serde_json::Value::Bool(b) => {
            if *b {
                Ok(ruby.qtrue().as_value())
            } else {
                Ok(ruby.qfalse().as_value())
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(ruby.integer_from_i64(i).as_value())
            } else if let Some(f) = n.as_f64() {
                Ok(ruby.float_from_f64(f).as_value())
            } else {
                Ok(ruby.qnil().as_value())
            }
        }
        serde_json::Value::String(s) => Ok(ruby.str_new(s).as_value()),
        serde_json::Value::Array(arr) => {
            let ary = ruby.ary_new_capa(arr.len());
            for item in arr {
                ary.push(json_to_ruby(ruby, item)?)?;
            }
            Ok(ary.as_value())
        }
        serde_json::Value::Object(map) => {
            let hash = ruby.hash_new();
            for (k, v) in map {
                hash.aset(ruby.str_new(k), json_to_ruby(ruby, v)?)?;
            }
            Ok(hash.as_value())
        }
    }
}

// ---------------------------------------------------------------------------
// Exported Ruby methods
// ---------------------------------------------------------------------------

/// `Onde.list_local_models` → Hash
///
/// Scans the local HuggingFace hub cache and returns all downloaded models
/// that the inference engine supports.
///
/// Returns a Hash with keys: `"models"`, `"cache_path"`,
/// `"total_size_bytes"`, `"total_size_display"`.
fn list_local_models() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    let response = hf_cache::list_local_hf_models();
    to_ruby_value(&ruby, &response)
}

/// `Onde.list_supported_models` → Hash
///
/// Returns all models the engine supports, together with flags indicating
/// whether each one is fully downloaded, partially downloaded, or absent.
///
/// Returns a Hash with key `"models"` containing an Array of model Hashes.
fn list_supported_models() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    let response = hf_cache::list_supported_hf_models();
    to_ruby_value(&ruby, &response)
}

/// `Onde.delete_model(model_id)` → nil
///
/// Delete a locally cached HuggingFace model.
/// `model_id` is the full identifier, e.g. `"black-forest-labs/FLUX.1-schnell"`.
///
/// Raises `RuntimeError` if the model is not found or deletion fails.
fn delete_model(model_id: String) -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    hf_cache::delete_local_hf_model(model_id)
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e))?;
    Ok(ruby.qnil().as_value())
}

/// `Onde.model_info(model_id)` → Hash or nil
///
/// Look up rich metadata for a supported model by its ID.
/// Returns `nil` if the model ID is not in the supported list.
fn model_info(model_id: String) -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");

    let info = SUPPORTED_MODEL_INFO.iter().find(|i| i.id == model_id);

    match info {
        None => Ok(ruby.qnil().as_value()),
        Some(i) => {
            let hash = ruby.hash_new();
            hash.aset(ruby.str_new("id"), ruby.str_new(i.id))?;
            hash.aset(ruby.str_new("name"), ruby.str_new(i.name))?;
            hash.aset(ruby.str_new("org"), ruby.str_new(i.org))?;
            hash.aset(ruby.str_new("description"), ruby.str_new(i.description))?;
            hash.aset(
                ruby.str_new("expected_size_bytes"),
                ruby.integer_from_u64(i.expected_size_bytes),
            )?;
            Ok(hash.as_value())
        }
    }
}

/// `Onde.supported_model_ids` → Array of String
///
/// Returns the list of all supported model IDs.
fn supported_model_ids() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    let ary = ruby.ary_new_capa(SUPPORTED_MODELS.len());
    for id in SUPPORTED_MODELS {
        ary.push(ruby.str_new(id))?;
    }
    Ok(ary.as_value())
}

/// `Onde.default_sampling_config` → Hash
fn default_sampling_config() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    to_ruby_value(&ruby, &SamplingConfig::default())
}

/// `Onde.deterministic_sampling_config` → Hash
fn deterministic_sampling_config() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    to_ruby_value(&ruby, &SamplingConfig::deterministic())
}

/// `Onde.mobile_sampling_config` → Hash
fn mobile_sampling_config() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    to_ruby_value(&ruby, &SamplingConfig::mobile())
}

/// `Onde.cache_path` → String or nil
///
/// Returns the resolved HuggingFace cache directory path, or nil if it
/// cannot be determined (e.g. `$HOME` is unset).
fn cache_path() -> Result<magnus::Value, Error> {
    let ruby = Ruby::get().expect("called outside Ruby");
    let response = hf_cache::list_local_hf_models();
    if response.cache_path.is_empty() {
        Ok(ruby.qnil().as_value())
    } else {
        Ok(ruby.str_new(&response.cache_path).as_value())
    }
}

// ---------------------------------------------------------------------------
// Init — called by Ruby when `require "onde/onde"` loads the shared library
// ---------------------------------------------------------------------------

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Onde")?;

    // -- Singleton methods ----------------------------------------------------
    module.define_singleton_method("list_local_models", function!(list_local_models, 0))?;
    module.define_singleton_method("list_supported_models", function!(list_supported_models, 0))?;
    module.define_singleton_method("delete_model", function!(delete_model, 1))?;
    module.define_singleton_method("model_info", function!(model_info, 1))?;
    module.define_singleton_method("supported_model_ids", function!(supported_model_ids, 0))?;
    module.define_singleton_method(
        "default_sampling_config",
        function!(default_sampling_config, 0),
    )?;
    module.define_singleton_method(
        "deterministic_sampling_config",
        function!(deterministic_sampling_config, 0),
    )?;
    module.define_singleton_method(
        "mobile_sampling_config",
        function!(mobile_sampling_config, 0),
    )?;
    module.define_singleton_method("cache_path", function!(cache_path, 0))?;

    // -- Constants ------------------------------------------------------------

    // Onde::NATIVE_VERSION (Rust crate version for parity checks).
    module.const_set("NATIVE_VERSION", ruby.str_new(env!("CARGO_PKG_VERSION")))?;

    // Onde::SUPPORTED_MODELS — frozen Array of model ID strings.
    let model_ids = ruby.ary_new_capa(SUPPORTED_MODELS.len());
    for id in SUPPORTED_MODELS {
        model_ids.push(ruby.str_new(id))?;
    }
    model_ids.freeze();
    module.const_set("SUPPORTED_MODELS", model_ids)?;

    // Onde::SUPPORTED_MODEL_INFO — frozen Array of frozen Hashes.
    let info_ary = ruby.ary_new_capa(SUPPORTED_MODEL_INFO.len());
    for info in SUPPORTED_MODEL_INFO {
        let hash: RHash = ruby.hash_new();
        hash.aset(ruby.str_new("id"), ruby.str_new(info.id))?;
        hash.aset(ruby.str_new("name"), ruby.str_new(info.name))?;
        hash.aset(ruby.str_new("org"), ruby.str_new(info.org))?;
        hash.aset(ruby.str_new("description"), ruby.str_new(info.description))?;
        hash.aset(
            ruby.str_new("expected_size_bytes"),
            ruby.integer_from_u64(info.expected_size_bytes),
        )?;
        hash.freeze();
        info_ary.push(hash)?;
    }
    info_ary.freeze();
    module.const_set("SUPPORTED_MODEL_INFO", info_ary)?;

    Ok(())
}
