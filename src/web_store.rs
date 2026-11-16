// Copyright 2026 Seto Elkahfi (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Browser model store for `wasm32-unknown-unknown`.
//!
//! Model files come from `https://huggingface.co/<repo>/resolve/main/<file>`
//! (Hugging Face serves CORS) and are kept in the Cache Storage API under
//! [`CACHE_NAME`]. Cache Storage works from a window and from a worker, needs
//! no origin-private-file-system handles, and survives reloads, so a model is
//! downloaded once per browser profile.
//!
//! [`download_model`] mirrors the native `hf_cache::download_model`: call it
//! first to show progress, then call `ChatEngine::load_gguf_model`, which reads
//! from the same cache (and downloads without progress if a file is missing).

use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, CacheStorage, ReadableStreamDefaultReader, Response};

use crate::inference::types::{GgufModelConfig, InferenceError};

/// Name of the Cache Storage bucket that holds downloaded model files.
pub const CACHE_NAME: &str = "onde-models-v1";

const HF_BASE: &str = "https://huggingface.co";

/// The tokenizer file fetched from `GgufModelConfig::tok_model_id`.
pub(crate) const TOKENIZER_FILE: &str = "tokenizer.json";

/// The Hugging Face URL a repository file is fetched from (and cached under).
pub fn model_url(repo: &str, file: &str) -> String {
    format!("{HF_BASE}/{repo}/resolve/main/{file}")
}

/// Download every file `config` needs into the cache, reporting progress.
///
/// `on_progress(received, total)` is called as bytes arrive for the model
/// file(s). `total` is `None` when the server does not expose a length. Files
/// already cached are skipped and reported as complete. The small tokenizer
/// file is fetched first, without progress.
pub async fn download_model(
    config: &GgufModelConfig,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<(), InferenceError> {
    if let Some(tok) = &config.tok_model_id {
        ensure_cached(&model_url(tok, TOKENIZER_FILE), &mut |_, _| {})
            .await
            .map_err(other)?;
    }
    let mut base = 0u64;
    for file in &config.files {
        let url = model_url(&config.model_id, file);
        let mut file_total = None;
        let mut file_received = 0u64;
        ensure_cached(&url, &mut |received, total| {
            file_received = received;
            file_total = total;
            on_progress(base + received, total.map(|t| base + t));
        })
        .await
        .map_err(other)?;
        base += file_total.unwrap_or(file_received);
    }
    Ok(())
}

/// Whether every file `config` needs is already in the cache.
pub async fn is_cached(config: &GgufModelConfig) -> bool {
    let Ok(cache) = open_cache().await else {
        return false;
    };
    for url in urls(config) {
        if !matches!(cache_match(&cache, &url).await, Ok(Some(_))) {
            return false;
        }
    }
    true
}

/// Remove `config`'s files from the cache. Returns whether anything was removed.
pub async fn delete_model(config: &GgufModelConfig) -> Result<bool, InferenceError> {
    let cache = open_cache().await.map_err(other)?;
    let mut removed = false;
    for url in urls(config) {
        let deleted = JsFuture::from(cache.delete_with_str(&url))
            .await
            .map_err(|e| other(js_message(&e)))?;
        removed |= deleted.as_bool().unwrap_or(false);
    }
    Ok(removed)
}

/// Read a cached file fully into memory, downloading it first if missing.
pub(crate) async fn load_file(repo: &str, file: &str) -> Result<Vec<u8>, InferenceError> {
    let url = model_url(repo, file);
    ensure_cached(&url, &mut |_, _| {}).await.map_err(other)?;
    let cache = open_cache().await.map_err(other)?;
    let response = cache_match(&cache, &url)
        .await
        .map_err(other)?
        .ok_or_else(|| other(format!("{url} vanished from the cache")))?;
    let buffer = JsFuture::from(response.array_buffer().map_err(|e| other(js_message(&e)))?)
        .await
        .map_err(|e| other(js_message(&e)))?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

fn urls(config: &GgufModelConfig) -> Vec<String> {
    let mut urls: Vec<String> = config
        .files
        .iter()
        .map(|f| model_url(&config.model_id, f))
        .collect();
    if let Some(tok) = &config.tok_model_id {
        urls.push(model_url(tok, TOKENIZER_FILE));
    }
    urls
}

fn other(reason: String) -> InferenceError {
    InferenceError::Other { reason }
}

fn js_message(value: &JsValue) -> String {
    if let Some(s) = value.as_string() {
        return s;
    }
    if let Ok(message) = Reflect::get(value, &"message".into()) {
        if let Some(s) = message.as_string() {
            return s;
        }
    }
    format!("{value:?}")
}

async fn open_cache() -> Result<Cache, String> {
    let global = js_sys::global();
    let caches = Reflect::get(&global, &"caches".into()).map_err(|e| js_message(&e))?;
    if caches.is_undefined() || caches.is_null() {
        return Err("Cache Storage is not available (it needs a secure context)".into());
    }
    let storage: CacheStorage = caches
        .dyn_into()
        .map_err(|_| "`caches` is not a CacheStorage".to_string())?;
    JsFuture::from(storage.open(CACHE_NAME))
        .await
        .map_err(|e| js_message(&e))?
        .dyn_into::<Cache>()
        .map_err(|_| "Cache Storage returned something that is not a Cache".to_string())
}

async fn cache_match(cache: &Cache, url: &str) -> Result<Option<Response>, String> {
    let found = JsFuture::from(cache.match_with_str(url))
        .await
        .map_err(|e| js_message(&e))?;
    if found.is_undefined() || found.is_null() {
        return Ok(None);
    }
    found
        .dyn_into::<Response>()
        .map(Some)
        .map_err(|_| "cache hit is not a Response".to_string())
}

fn content_length(response: &Response) -> Option<u64> {
    response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
}

async fn fetch(url: &str) -> Result<Response, String> {
    let global = js_sys::global();
    let fetch: Function = Reflect::get(&global, &"fetch".into())
        .map_err(|e| js_message(&e))?
        .dyn_into()
        .map_err(|_| "`fetch` is not available".to_string())?;
    let promise: Promise = fetch
        .call1(&global, &JsValue::from_str(url))
        .map_err(|e| js_message(&e))?
        .dyn_into()
        .map_err(|_| "`fetch` did not return a promise".to_string())?;
    let response: Response = JsFuture::from(promise)
        .await
        .map_err(|e| format!("could not fetch {url}: {}", js_message(&e)))?
        .dyn_into()
        .map_err(|_| "`fetch` did not resolve to a Response".to_string())?;
    if !response.ok() {
        return Err(format!("HTTP {} for {url}", response.status()));
    }
    Ok(response)
}

/// Make sure `url` is in the cache. Streams the body: one clone goes to
/// `Cache.put` (which writes it to disk as it arrives) while the other is read
/// only to count bytes, so the file is never held in memory here.
async fn ensure_cached(
    url: &str,
    on_progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<(), String> {
    let cache = open_cache().await?;
    if let Some(hit) = cache_match(&cache, url).await? {
        let size = content_length(&hit).unwrap_or(0);
        on_progress(size, Some(size));
        return Ok(());
    }

    let response = fetch(url).await?;
    let total = content_length(&response);
    let for_cache = response.clone().map_err(|e| js_message(&e))?;
    let put = cache.put_with_str(url, &for_cache);

    let body = response
        .body()
        .ok_or_else(|| format!("{url} has no response body"))?;
    let reader: ReadableStreamDefaultReader = body
        .get_reader()
        .dyn_into()
        .map_err(|_| "response body has no default reader".to_string())?;
    let mut received = 0u64;
    loop {
        let step = JsFuture::from(reader.read())
            .await
            .map_err(|e| js_message(&e))?;
        if Reflect::get(&step, &"done".into())
            .map(|d| d.as_bool().unwrap_or(false))
            .unwrap_or(true)
        {
            break;
        }
        if let Ok(chunk) = Reflect::get(&step, &"value".into()) {
            received += chunk.unchecked_into::<Uint8Array>().length() as u64;
            on_progress(received, total);
        }
    }
    JsFuture::from(put)
        .await
        .map_err(|e| format!("could not cache {url}: {}", js_message(&e)))?;
    on_progress(received, total.or(Some(received)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_urls_point_at_the_resolve_endpoint() {
        assert_eq!(
            model_url("Qwen/Qwen2.5-0.5B-Instruct", "tokenizer.json"),
            "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/main/tokenizer.json"
        );
    }
}
