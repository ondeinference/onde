//! The browser model store.
//!
//! Model files come from `https://huggingface.co/<repo>/resolve/main/<file>`
//! (Hugging Face serves CORS) and are kept in the Cache Storage API under
//! [`CACHE_NAME`]. Cache Storage works from a window and from a worker and
//! survives reloads, so a model is downloaded once per browser profile.
//!
//! The cache name and URLs match the ones the onde crate's `web_store` uses, so
//! a model cached by a page built on the crate is found without downloading
//! again. The WebGPU engine reads the same entries from JavaScript.

use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, CacheStorage, ReadableStreamDefaultReader, Response};

/// Name of the Cache Storage bucket that holds downloaded model files.
pub const CACHE_NAME: &str = "onde-models-v1";

const HF_BASE: &str = "https://huggingface.co";

/// A model and its tokenizer, both hosted on Hugging Face.
pub struct ModelFiles {
    pub display_name: &'static str,
    pub approx_size: &'static str,
    pub repo: &'static str,
    pub file: &'static str,
    pub tokenizer_repo: &'static str,
}

/// Qwen 2.5 0.5B Instruct, Q4_K_M.
pub const QWEN25_0_5B: ModelFiles = ModelFiles {
    display_name: "Qwen 2.5 0.5B",
    approx_size: "~380 MB (GGUF Q4_K_M)",
    repo: "bartowski/Qwen2.5-0.5B-Instruct-GGUF",
    file: "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf",
    tokenizer_repo: "Qwen/Qwen2.5-0.5B-Instruct",
};

const TOKENIZER_FILE: &str = "tokenizer.json";

impl ModelFiles {
    pub fn model_url(&self) -> String {
        url(self.repo, self.file)
    }

    pub fn tokenizer_url(&self) -> String {
        url(self.tokenizer_repo, TOKENIZER_FILE)
    }

    fn urls(&self) -> [String; 2] {
        [self.tokenizer_url(), self.model_url()]
    }
}

fn url(repo: &str, file: &str) -> String {
    format!("{HF_BASE}/{repo}/resolve/main/{file}")
}

/// Downloads the tokenizer, then the model, into the cache.
/// `on_progress(received, total)` reports the model file's bytes; `total` is
/// `None` when the server hides the length. Cached files are skipped.
pub async fn download(
    files: &ModelFiles,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<(), String> {
    ensure_cached(&files.tokenizer_url(), &mut |_, _| {}).await?;
    ensure_cached(&files.model_url(), &mut on_progress).await
}

/// Whether the model and its tokenizer are both cached.
pub async fn is_cached(files: &ModelFiles) -> bool {
    let Ok(cache) = open_cache().await else {
        return false;
    };
    for url in files.urls() {
        if !matches!(cache_match(&cache, &url).await, Ok(Some(_))) {
            return false;
        }
    }
    true
}

/// Removes the model and its tokenizer. Returns whether anything was removed.
pub async fn delete(files: &ModelFiles) -> Result<bool, String> {
    let cache = open_cache().await?;
    let mut removed = false;
    for url in files.urls() {
        let deleted = JsFuture::from(cache.delete_with_str(&url))
            .await
            .map_err(|e| js_message(&e))?;
        removed |= deleted.as_bool().unwrap_or(false);
    }
    Ok(removed)
}

/// Reads a cached file into wasm memory, downloading it first if missing.
///
/// The bytes cross from a JS `ArrayBuffer` into one `Vec`; the engine then
/// works on that `Vec` in place, so the peak is about twice the file size
/// until the browser collects the `ArrayBuffer`.
pub async fn load(url: &str) -> Result<Vec<u8>, String> {
    ensure_cached(url, &mut |_, _| {}).await?;
    let cache = open_cache().await?;
    let response = cache_match(&cache, url)
        .await?
        .ok_or_else(|| format!("{url} vanished from the cache"))?;
    let buffer = JsFuture::from(response.array_buffer().map_err(|e| js_message(&e))?)
        .await
        .map_err(|e| js_message(&e))?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

pub fn js_message(value: &JsValue) -> String {
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

/// Makes sure `url` is in the cache. One clone of the response goes to
/// `Cache.put`, which writes it to disk as it arrives; the other is read only
/// to count bytes, so the file is never held in memory here.
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
