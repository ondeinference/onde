//! HuggingFace hub cache inspection, repair, and model download.
//!
//! This module owns all on-disk state management for locally cached
//! HuggingFace models and exposes a progress-callback-based download API
//! that is completely decoupled from any application framework.
//!
//! 1. Call [`list_local_hf_models`] / [`list_supported_hf_models`] /
//!    [`delete_local_hf_model`] directly from your command handlers.
//! 2. Call [`download_model`] with an `Arc<dyn Fn(ModelDownloadProgress)>`
//!    that forwards progress to whatever event system your app uses.

use {
    crate::inference::models::{SUPPORTED_MODELS, SUPPORTED_MODEL_INFO},
    log::{debug, info, warn},
    serde::{Deserialize, Serialize},
    std::{
        fs,
        path::{Path, PathBuf},
    },
    tsync::tsync,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Information about a locally cached HuggingFace model.
#[tsync]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalHfModel {
    /// The full model identifier, e.g. "bartowski/Qwen2.5-1.5B-Instruct-GGUF".
    pub model_id: String,
    /// The organisation or user that published the model.
    pub org: String,
    /// The model name without the org prefix.
    pub name: String,
    /// Total size on disk in bytes.
    pub size_bytes: u64,
    /// Human-readable size string, e.g. "4.2 GB".
    pub size_display: String,
    /// Absolute path to the model cache directory.
    pub path: String,
    /// List of snapshot revisions that are locally available.
    pub revisions: Vec<String>,
}

/// Response returned by [`list_local_hf_models`].
#[tsync]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalHfModelsResponse {
    /// The models found in the local HuggingFace cache.
    pub models: Vec<LocalHfModel>,
    /// The path that was scanned.
    pub cache_path: String,
    /// Total size of all cached models in bytes.
    pub total_size_bytes: u64,
    /// Human-readable total size string.
    pub total_size_display: String,
}

/// A supported model that may or may not be downloaded locally.
#[tsync]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedHfModel {
    /// The full model identifier, e.g. "bartowski/Qwen2.5-1.5B-Instruct-GGUF".
    pub model_id: String,
    /// Human-friendly display name.
    pub name: String,
    /// Organisation or publisher display name.
    pub org: String,
    /// Short description of the model.
    pub description: String,
    /// Whether this model is fully downloaded locally.
    pub is_downloaded: bool,
    /// Whether a partial (incomplete) download exists on disk.
    /// True when a cache directory exists but its size is less than 99 %
    /// of `expected_size_bytes`.
    pub is_incomplete: bool,
    /// Bytes currently on disk for this model (0 when not present at all).
    pub local_size_bytes: u64,
    /// Human-readable local size string, e.g. "4.2 GB".
    pub local_size_display: String,
    /// Approximate total size in bytes when fully downloaded.
    pub expected_size_bytes: u64,
    /// Human-readable expected size string, e.g. "23.80 GB".
    pub expected_size_display: String,
}

/// Response returned by [`list_supported_hf_models`].
#[tsync]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedHfModelsResponse {
    /// All models the app supports, with download status.
    pub models: Vec<SupportedHfModel>,
}

/// Progress payload passed to the [`download_model`] callback on every poll
/// tick and on completion.
///
/// Forward this value to your app's event system so the frontend can render
/// a progress bar.
#[tsync]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadProgress {
    /// The model being downloaded.
    pub model_id: String,
    /// Bytes downloaded so far (observed from cache directory size).
    pub downloaded_bytes: u64,
    /// Human-readable downloaded size, e.g. "1.2 GB".
    pub downloaded_display: String,
    /// Expected total size in bytes (approximate).
    pub total_bytes: u64,
    /// Human-readable total size, e.g. "23.80 GB".
    pub total_display: String,
    /// Download progress as a value between 0.0 and 1.0.
    pub progress: f64,
    /// Whether the download has completed.
    pub done: bool,
}

// ---------------------------------------------------------------------------
// Internal constants
// ---------------------------------------------------------------------------

/// Fraction of `expected_size_bytes` that must be present on disk for a
/// model to be considered fully downloaded (99 %).
const DOWNLOAD_COMPLETE_THRESHOLD: f64 = 0.99;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Resolve the HuggingFace hub cache directory.
///
/// Respects the `HF_HOME` / `HUGGINGFACE_HUB_CACHE` environment variables.
/// Falls back to `~/.cache/huggingface/hub`.
///
/// On iOS the `HF_HOME` variable is set during app setup (see
/// `setup_application_filesystem`) to a path inside the app's writable
/// `Library/Application Support/` container.  We create the directory if it
/// doesn't exist yet so that first-run and freshly-installed apps work
/// correctly.
fn hf_cache_dir() -> Option<PathBuf> {
    // 1. Explicit hub cache override
    if let Ok(cache) = std::env::var("HUGGINGFACE_HUB_CACHE") {
        let p = PathBuf::from(cache);
        let _ = fs::create_dir_all(&p);
        if p.is_dir() {
            return Some(p);
        }
    }

    // 2. HF_HOME override (hub is a subdirectory)
    if let Ok(hf_home) = std::env::var("HF_HOME") {
        let p = PathBuf::from(hf_home).join("hub");
        let _ = fs::create_dir_all(&p);
        if p.is_dir() {
            return Some(p);
        }
    }

    // 3. Default: ~/.cache/huggingface/hub
    if let Some(home) = home::home_dir() {
        let p = home.join(".cache").join("huggingface").join("hub");
        let _ = fs::create_dir_all(&p);
        if p.is_dir() {
            return Some(p);
        }
    }

    None
}

/// Recursively compute the total size of a directory in bytes.
///
/// Follows symlinks to count the real file size and also counts in-progress
/// `.incomplete` staging files so that progress polling reflects partial
/// downloads before they are renamed to their final blob name.
fn dir_size(path: &PathBuf) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total += dir_size(&entry_path);
            } else {
                // Count both completed blobs AND in-progress staging files
                // (hf-hub downloads to `<sha256>.incomplete` and renames on
                // completion — without this, progress stays at 0% for the
                // entire download and jumps to 100% at the end).
                if let Ok(meta) = entry_path.symlink_metadata() {
                    if meta.is_symlink() {
                        // Follow the symlink to get the real file size.
                        if let Ok(real_meta) = entry_path.metadata() {
                            total += real_meta.len();
                        }
                    } else {
                        total += meta.len();
                    }
                }
            }
        }
    }
    total
}

/// Format a byte count into a human-readable string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// List the snapshot revisions available for a cached model directory.
fn list_revisions(model_dir: &Path) -> Vec<String> {
    let snapshots_dir = model_dir.join("snapshots");
    if !snapshots_dir.is_dir() {
        return Vec::new();
    }

    let mut revisions = Vec::new();
    if let Ok(entries) = fs::read_dir(&snapshots_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    revisions.push(name.to_string());
                }
            }
        }
    }
    revisions.sort();
    revisions
}

/// Build a [`ModelDownloadProgress`] payload from raw byte counts.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "android"
))]
fn make_progress(model_id: &str, downloaded: u64, total: u64, done: bool) -> ModelDownloadProgress {
    let progress = if total > 0 {
        (downloaded as f64 / total as f64).min(1.0)
    } else {
        0.0
    };
    ModelDownloadProgress {
        model_id: model_id.to_string(),
        downloaded_bytes: downloaded,
        downloaded_display: format_size(downloaded),
        total_bytes: total,
        total_display: format_size(total),
        progress,
        done,
    }
}

// ---------------------------------------------------------------------------
// Public cache utilities
// ---------------------------------------------------------------------------

/// Return the cache sub-directory path for a given model ID.
///
/// E.g. for `"bartowski/Qwen2.5-1.5B-Instruct-GGUF"` the cache directory is
/// `<hf_cache>/models--bartowski--Qwen2.5-1.5B-Instruct-GGUF`.
pub fn model_cache_path(model_id: &str) -> Option<PathBuf> {
    let cache_dir = hf_cache_dir()?;
    let dir_name = format!("models--{}", model_id.replace('/', "--"));
    Some(cache_dir.join(dir_name))
}

/// Pre-create the HuggingFace cache directory tree for `model_id` so that the
/// progress-monitor thread can find a valid path immediately, even before
/// hf-hub creates it during the first download.
///
/// hf-hub won't be confused by a pre-existing (empty) directory — it will
/// simply populate it as normal.
pub fn ensure_model_cache_dir(model_id: &str) {
    if let Some(model_path) = model_cache_path(model_id) {
        let blobs_dir = model_path.join("blobs");
        if let Err(e) = fs::create_dir_all(&blobs_dir) {
            warn!(
                "Could not pre-create model cache dir {}: {}",
                blobs_dir.display(),
                e
            );
        } else {
            info!("Pre-created model cache dir: {}", model_path.display());
        }
    }
}

/// Remove stale `.lock` and orphaned `.part` files from the HF cache
/// `blobs/` directory for a model.
///
/// The HF hub client creates `.lock` files during download. If a download is
/// interrupted these locks persist and prevent any subsequent access (download
/// **or** model load). Calling this before loading or resuming a download
/// allows the hub client to re-acquire the lock and proceed normally.
pub fn clean_stale_lock_files(model_id: &str) {
    if let Some(cache_path) = model_cache_path(model_id) {
        let blobs_dir = cache_path.join("blobs");
        if blobs_dir.is_dir() {
            // Collect the names of complete blobs (no extension) so we can
            // decide whether a `.part` file is stale.
            let complete_blobs: Vec<String> = fs::read_dir(&blobs_dir)
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|e| {
                    let p = e.path();
                    if p.is_file() && p.extension().is_none() {
                        p.file_name().map(|n| n.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if let Ok(entries) = fs::read_dir(&blobs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let ext = path.extension().and_then(|e| e.to_str());

                    match ext {
                        Some("lock") => {
                            info!("Removing stale lock file: {}", path.display());
                            if let Err(e) = fs::remove_file(&path) {
                                warn!("Failed to remove lock file {}: {}", path.display(), e);
                            }
                        }
                        Some("part") => {
                            // A `.part` file is a partial download left by hf-hub.
                            // If the corresponding complete blob exists (same name
                            // without `.part`), the partial file is stale and
                            // causes hf-hub to issue a spurious Range-resume
                            // download instead of using the complete blob.
                            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                            if complete_blobs.iter().any(|b| b == stem) {
                                let part_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                info!(
                                    "Removing stale .part file ({} bytes, complete blob exists): {}",
                                    part_size,
                                    path.display()
                                );
                                if let Err(e) = fs::remove_file(&path) {
                                    warn!("Failed to remove .part file {}: {}", path.display(), e);
                                }
                            } else {
                                let part_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                debug!(
                                    "Keeping .part file ({} bytes, no complete blob yet): {}",
                                    part_size,
                                    path.display()
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Log the complete HF cache structure for a model to aid debugging.
///
/// This is called before model load so the developer can see exactly what's
/// present in the cache, whether `refs/main` resolves, whether symlinks are
/// intact, and whether any `.part` files are left over from interrupted
/// downloads.
pub fn diagnose_hf_cache(model_id: &str) {
    let cache_path = match model_cache_path(model_id) {
        Some(p) => p,
        None => {
            info!("diagnose_hf_cache({}): cache path not found", model_id);
            return;
        }
    };

    if !cache_path.is_dir() {
        info!(
            "diagnose_hf_cache({}): directory does not exist: {}",
            model_id,
            cache_path.display()
        );
        return;
    }

    info!(
        "diagnose_hf_cache({}): scanning {}",
        model_id,
        cache_path.display()
    );

    // ── refs/ ────────────────────────────────────────────────────────────
    let refs_dir = cache_path.join("refs");
    if refs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&refs_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                match fs::read_to_string(entry.path()) {
                    Ok(content) => {
                        let hash = content.trim();
                        info!(
                            "  refs/{} → \"{}\" (len={})",
                            name,
                            if hash.len() > 12 {
                                format!("{}…", &hash[..12])
                            } else {
                                hash.to_string()
                            },
                            hash.len()
                        );
                    }
                    Err(e) => {
                        warn!("  refs/{} → <unreadable: {}>", name, e);
                    }
                }
            }
        }
    } else {
        warn!("  refs/ directory missing!");
    }

    // ── blobs/ ───────────────────────────────────────────────────────────
    let blobs_dir = cache_path.join("blobs");
    if blobs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&blobs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let kind = if name.ends_with(".lock") {
                    "LOCK"
                } else if name.ends_with(".part") {
                    "PART (incomplete)"
                } else {
                    "blob"
                };
                info!(
                    "  blobs/{} — {} ({} bytes / {:.1} MB)",
                    name,
                    kind,
                    size,
                    size as f64 / 1_048_576.0
                );
            }
        }
    } else {
        warn!("  blobs/ directory missing!");
    }

    // ── snapshots/ ───────────────────────────────────────────────────────
    let snapshots_dir = cache_path.join("snapshots");
    if snapshots_dir.is_dir() {
        if let Ok(revs) = fs::read_dir(&snapshots_dir) {
            for rev_entry in revs.flatten() {
                let rev_name = rev_entry.file_name().to_string_lossy().to_string();
                let rev_display = if rev_name.len() > 12 {
                    format!("{}…", &rev_name[..12])
                } else {
                    rev_name.clone()
                };
                info!("  snapshots/{}/", rev_display);

                if let Ok(files) = fs::read_dir(rev_entry.path()) {
                    for file_entry in files.flatten() {
                        let file_name = file_entry.file_name().to_string_lossy().to_string();
                        let file_path = file_entry.path();

                        if file_path.is_symlink() {
                            let target = fs::read_link(&file_path)
                                .map(|t| t.to_string_lossy().to_string())
                                .unwrap_or_else(|_| "<unreadable>".to_string());
                            let accessible = fs::metadata(&file_path).is_ok();
                            let status = if accessible { "OK" } else { "BROKEN" };
                            info!("    {} → {} [symlink: {}]", file_name, target, status);
                        } else if file_path.is_file() {
                            let size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                            info!(
                                "    {} [{} bytes / {:.1} MB, real file]",
                                file_name,
                                size,
                                size as f64 / 1_048_576.0
                            );
                        } else {
                            warn!("    {} [not a file or symlink!]", file_name);
                        }
                    }
                }
            }
        }
    } else {
        warn!("  snapshots/ directory missing!");
    }

    info!("diagnose_hf_cache({}): done", model_id);
}

/// Repair broken symlinks in the HF hub cache for a given model.
///
/// The `hf-hub` crate stores downloaded files as blobs keyed by SHA-256 and
/// creates **symlinks** from `snapshots/<rev>/<filename>` → `../../blobs/<hash>`.
/// When the cache directory is copied into an iOS app container (e.g. via
/// Xcode "Replace Container" or `ideviceinstaller`), symlinks are often
/// silently dropped or broken.  The `hf-hub` client then thinks the file is
/// missing and re-downloads it — even though the 940 MB blob is right there.
///
/// This function walks every snapshot entry for `model_id`.  If a path is a
/// broken symlink **or** a missing file, and a matching blob exists, it
/// replaces the symlink with a hard copy of the blob so that `hf-hub`'s
/// `Cache::get()` finds a real file at the expected path.
///
/// It also ensures a `refs/main` file exists pointing at the snapshot
/// revision, since that file is needed for `hf-hub` to resolve the "main"
/// ref to a concrete snapshot directory.
///
/// This is idempotent and safe to call on every app launch.
pub fn repair_hf_cache_symlinks(model_id: &str) {
    let cache_path = match model_cache_path(model_id) {
        Some(p) if p.is_dir() => p,
        _ => return,
    };

    let blobs_dir = cache_path.join("blobs");
    let snapshots_dir = cache_path.join("snapshots");
    let refs_dir = cache_path.join("refs");

    if !blobs_dir.is_dir() || !snapshots_dir.is_dir() {
        return;
    }

    // Collect blob filenames (excluding .lock files) for lookup.
    let blob_names: Vec<String> = match fs::read_dir(&blobs_dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".lock") {
                    None
                } else if e.path().is_file() {
                    Some(name)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return,
    };

    if blob_names.is_empty() {
        return;
    }

    // Walk each snapshot revision directory.
    let snapshot_revs: Vec<(String, PathBuf)> = match fs::read_dir(&snapshots_dir) {
        Ok(entries) => entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| {
                let rev = e.file_name().to_string_lossy().to_string();
                (rev, e.path())
            })
            .collect(),
        Err(_) => return,
    };

    for (rev, snap_dir) in &snapshot_revs {
        let entries = match fs::read_dir(snap_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let snap_file = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Check if the file is accessible (broken symlink → metadata fails).
            let needs_repair = if snap_file.is_symlink() {
                // Symlink exists but may be broken — check if target is readable.
                fs::metadata(&snap_file).is_err()
            } else if snap_file.exists() {
                // Real file, no repair needed.
                false
            } else {
                // Path doesn't exist at all (dangling entry in readdir).
                true
            };

            if !needs_repair {
                continue;
            }

            // Try to find a matching blob.  For GGUF repos there's usually
            // just one blob; for multi-file repos we match by the symlink
            // target name if we can read it, otherwise fall back to the
            // single-blob heuristic.
            let blob_name = if let Ok(target) = fs::read_link(&snap_file) {
                target
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .filter(|n| blob_names.contains(n))
            } else {
                None
            };

            let resolved_blob = if let Some(ref name) = blob_name {
                Some(blobs_dir.join(name))
            } else if blob_names.len() == 1 {
                // Single-blob repo (typical for GGUF) — safe to assume it
                // corresponds to this snapshot entry.
                Some(blobs_dir.join(&blob_names[0]))
            } else {
                None
            };

            if let Some(blob_path) = resolved_blob {
                if !blob_path.is_file() {
                    continue;
                }

                // Remove the broken symlink before copying.
                if snap_file.is_symlink() || snap_file.exists() {
                    let _ = fs::remove_file(&snap_file);
                }

                info!(
                    "repair_hf_cache_symlinks: replacing broken symlink with copy: {} → {}",
                    blob_path.display(),
                    snap_file.display()
                );

                // Prefer hard link (instant, no extra disk space) over copy.
                if fs::hard_link(&blob_path, &snap_file).is_err() {
                    if let Err(e) = fs::copy(&blob_path, &snap_file) {
                        warn!(
                            "repair_hf_cache_symlinks: failed to copy blob to snapshot: {}",
                            e
                        );
                    }
                }
            } else {
                warn!(
                    "repair_hf_cache_symlinks: broken symlink for '{}' but no matching blob found",
                    file_name
                );
            }
        }

        // Ensure refs/main exists and points to this revision so that
        // `hf-hub` can resolve the snapshot.
        if !refs_dir.join("main").is_file() {
            let _ = fs::create_dir_all(&refs_dir);
            if let Err(e) = fs::write(refs_dir.join("main"), rev.as_bytes()) {
                warn!("repair_hf_cache_symlinks: failed to write refs/main: {}", e);
            } else {
                info!("repair_hf_cache_symlinks: created refs/main → {}", rev);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Query / mutation functions
// ---------------------------------------------------------------------------

/// Scan the local HuggingFace hub cache and return all downloaded models that
/// the inference engine supports.
///
/// The cache directory is typically `~/.cache/huggingface/hub/` and contains
/// sub-directories named `models--{org}--{model}` for each cached model.
pub fn list_local_hf_models() -> LocalHfModelsResponse {
    let cache_dir = match hf_cache_dir() {
        Some(dir) => dir,
        None => {
            debug!("HuggingFace cache directory not found.");
            return LocalHfModelsResponse {
                models: Vec::new(),
                cache_path: String::new(),
                total_size_bytes: 0,
                total_size_display: "0 B".to_string(),
            };
        }
    };

    let cache_path_str = cache_dir.to_string_lossy().to_string();
    debug!("Scanning HuggingFace cache at: {}", cache_path_str);

    let mut models: Vec<LocalHfModel> = Vec::new();

    let entries = match fs::read_dir(&cache_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read HuggingFace cache directory: {:?}", e);
            return LocalHfModelsResponse {
                models: Vec::new(),
                cache_path: cache_path_str,
                total_size_bytes: 0,
                total_size_display: "0 B".to_string(),
            };
        }
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let dir_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        // HuggingFace hub caches model repos as "models--{org}--{name}"
        if !dir_name.starts_with("models--") {
            continue;
        }

        // Strip the "models--" prefix and split by "--" to get org/name
        let remainder = &dir_name["models--".len()..];
        let parts: Vec<&str> = remainder.splitn(2, "--").collect();
        if parts.len() != 2 {
            // Some models have no org (single segment), handle gracefully
            let model_id = remainder.replace("--", "/");
            let size = dir_size(&entry_path);
            let revisions = list_revisions(&entry_path);

            models.push(LocalHfModel {
                model_id: model_id.clone(),
                org: String::new(),
                name: model_id,
                size_bytes: size,
                size_display: format_size(size),
                path: entry_path.to_string_lossy().to_string(),
                revisions,
            });
            continue;
        }

        let org = parts[0].to_string();
        let name = parts[1].to_string();
        let model_id = format!("{}/{}", org, name);
        let size = dir_size(&entry_path);
        let revisions = list_revisions(&entry_path);

        models.push(LocalHfModel {
            model_id,
            org,
            name,
            size_bytes: size,
            size_display: format_size(size),
            path: entry_path.to_string_lossy().to_string(),
            revisions,
        });
    }

    // Only keep models that the inference engine actually supports.
    models.retain(|m| SUPPORTED_MODELS.iter().any(|&s| s == m.model_id));

    // Sort by model_id for a stable, predictable order.
    models.sort_by(|a, b| a.model_id.to_lowercase().cmp(&b.model_id.to_lowercase()));

    let total_size_bytes: u64 = models.iter().map(|m| m.size_bytes).sum();

    debug!(
        "Found {} local HuggingFace model(s), total size: {}",
        models.len(),
        format_size(total_size_bytes)
    );

    LocalHfModelsResponse {
        models,
        cache_path: cache_path_str,
        total_size_bytes,
        total_size_display: format_size(total_size_bytes),
    }
}

/// Delete a locally cached HuggingFace model by removing its directory from
/// the hub cache.
///
/// `model_id` should be the full identifier, e.g. "bartowski/Qwen2.5-1.5B-Instruct-GGUF".
/// Returns `Ok(())` on success or an error string describing what went wrong.
pub fn delete_local_hf_model(model_id: String) -> Result<(), String> {
    let cache_dir = hf_cache_dir().ok_or("HuggingFace cache directory not found.".to_string())?;

    // Convert model_id (e.g. "org/name") to the cache directory name ("models--org--name")
    let dir_name = format!("models--{}", model_id.replace('/', "--"));
    let model_path = cache_dir.join(&dir_name);

    if !model_path.exists() {
        return Err(format!(
            "Model cache directory not found: {}",
            model_path.display()
        ));
    }

    if !model_path.is_dir() {
        return Err(format!(
            "Expected a directory but found a file: {}",
            model_path.display()
        ));
    }

    debug!(
        "Deleting local HuggingFace model: {} at {:?}",
        model_id, model_path
    );

    fs::remove_dir_all(&model_path).map_err(|e| {
        let msg = format!("Failed to delete model {}: {}", model_id, e);
        warn!("{}", msg);
        msg
    })?;

    debug!("Successfully deleted local model: {}", model_id);
    Ok(())
}

/// Return the full list of models the inference engine supports, together with
/// flags indicating whether each one is fully downloaded, partially downloaded,
/// or not present at all.
pub fn list_supported_hf_models() -> SupportedHfModelsResponse {
    let models = SUPPORTED_MODEL_INFO
        .iter()
        .map(|info| {
            let cache_path = model_cache_path(info.id);
            let local_size = cache_path
                .as_ref()
                .filter(|p| p.exists())
                .map(dir_size)
                .unwrap_or(0);

            let has_cache = local_size > 0;
            let is_complete = info.expected_size_bytes > 0
                && local_size as f64
                    >= info.expected_size_bytes as f64 * DOWNLOAD_COMPLETE_THRESHOLD;

            SupportedHfModel {
                model_id: info.id.to_string(),
                name: info.name.to_string(),
                org: info.org.to_string(),
                description: info.description.to_string(),
                is_downloaded: has_cache && is_complete,
                is_incomplete: has_cache && !is_complete,
                local_size_bytes: local_size,
                local_size_display: format_size(local_size),
                expected_size_bytes: info.expected_size_bytes,
                expected_size_display: format_size(info.expected_size_bytes),
            }
        })
        .collect();

    SupportedHfModelsResponse { models }
}

// ---------------------------------------------------------------------------
// Model download — platform-specific implementations
// ---------------------------------------------------------------------------
//
// The download function accepts a generic progress callback so that callers
// can forward progress to any event system without coupling to this crate.
//
// Callback signature: `Fn(ModelDownloadProgress) + Send + Sync + Clone + 'static`
//   - Called every ~2 s from a native monitor thread.
//   - Called once with `done: true` after the build completes.

// ── Apple (macOS + iOS): Metal-accelerated, all model types ─────────────────

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos"
))]
pub async fn download_model<F>(
    model_id: String,
    on_progress: F,
    app_data_dir: Option<PathBuf>,
) -> Result<(), String>
where
    F: Fn(ModelDownloadProgress) + Send + Sync + Clone + 'static,
{
    use crate::inference::models::{
        BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF, BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF,
        QWEN25_1_5B_GGUF_FILE, QWEN25_3B_GGUF_FILE,
    };
    use crate::inference::token::hf_token_source;
    use mistralrs::GgufModelBuilder;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;

    // When app_data_dir is provided, seed the HF env vars so that hf-hub,
    // mistral.rs, and hf_cache_dir() all write inside the app's writable
    // container.  This is required on every sandboxed platform:
    //   - iOS: ~/.cache is outside the container (os error 1).
    //   - macOS App Store: the app is sandboxed, ~/.cache is inaccessible.
    // It is harmless on non-sandboxed desktop builds (the cache simply
    // lives under the app data dir instead of ~/.cache).
    if let Some(ref data_dir) = app_data_dir {
        let hf_home = data_dir.join("models");
        let hf_hub_cache = hf_home.join("hub");
        fs::create_dir_all(&hf_hub_cache)
            .map_err(|e| format!("Cannot create HF cache dir: {e}"))?;
        std::env::set_var("HF_HUB_CACHE", &hf_hub_cache);
        std::env::set_var("HF_HOME", &hf_home);
        info!(
            "HF hub cache resolved to app data path: {}",
            hf_hub_cache.display()
        );
    }

    if !SUPPORTED_MODELS.contains(&model_id.as_str()) {
        return Err(format!("Model {} is not supported.", model_id));
    }

    let expected_size = SUPPORTED_MODEL_INFO
        .iter()
        .find(|m| m.id == model_id)
        .map(|m| m.expected_size_bytes)
        .unwrap_or(0);

    info!(
        "Starting download for model: {} (expected ~{})",
        model_id,
        format_size(expected_size)
    );

    clean_stale_lock_files(&model_id);
    repair_hf_cache_symlinks(&model_id);
    ensure_model_cache_dir(&model_id);

    // Emit an initial progress event based on the actual cache size so that
    // resumed downloads don't visually restart from 0%.
    let initial_bytes = model_cache_path(&model_id)
        .filter(|p| p.exists())
        .map(|p| dir_size(&p))
        .unwrap_or(0);
    on_progress(make_progress(
        &model_id,
        initial_bytes,
        expected_size,
        false,
    ));

    // Shared flag so the monitor task knows when to stop.
    let finished = Arc::new(AtomicBool::new(false));
    let finished_clone = Arc::clone(&finished);
    let monitor_model_id = model_id.clone();
    let monitor_cb = on_progress.clone();

    // Spawn a native OS thread to poll the cache directory size every 2 seconds.
    // We use std::thread instead of tokio::spawn because the model builder's
    // .build().await internally calls the synchronous hf-hub API (api.get()),
    // which blocks the tokio worker thread. This starves any tokio-spawned
    // monitor tasks, preventing progress updates. A native thread runs
    // independently of the tokio runtime and is immune to this starvation.
    let monitor_handle = std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));

        if finished_clone.load(Ordering::Relaxed) {
            break;
        }

        let current_bytes = model_cache_path(&monitor_model_id)
            .filter(|p| p.exists())
            .map(|p| dir_size(&p))
            .unwrap_or(0);

        monitor_cb(make_progress(
            &monitor_model_id,
            current_bytes,
            expected_size,
            false,
        ));
    });

    // Build (and immediately drop) the model using the GGUF chat builder.
    // Building triggers the HuggingFace hub download.
    let result: Result<mistralrs::Model, anyhow::Error> = match model_id.as_str() {
        BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF => {
            GgufModelBuilder::new(&model_id, vec![QWEN25_1_5B_GGUF_FILE])
                .with_token_source(hf_token_source())
                .build()
                .await
        }
        BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF => {
            GgufModelBuilder::new(&model_id, vec![QWEN25_3B_GGUF_FILE])
                .with_token_source(hf_token_source())
                .build()
                .await
        }
        _ => Err(anyhow::anyhow!(
            "Model '{}' is not part of the chat-only release.",
            model_id
        )),
    };

    finished.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    match result {
        Ok(_model) => {
            let final_bytes = model_cache_path(&model_id)
                .filter(|p| p.exists())
                .map(|p| dir_size(&p))
                .unwrap_or(expected_size);
            on_progress(make_progress(&model_id, final_bytes, final_bytes, true));
            info!("Successfully downloaded model: {}", model_id);
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to download model {}: {}", model_id, e);
            warn!("{}", msg);
            Err(msg)
        }
    }
}

// ── Android: GGUF-only download ───────────────────────────────────────────
//
// On Android there is no Metal GPU, so only pre-quantized GGUF models
// (CPU inference via candle) are supported.  We mirror the Apple
// implementation but restrict to GGUF presets and skip all Metal / ISQ
// model builders.
//
// `app_data_dir` is **required** on Android: `dirs::home_dir()` returns
// `None` inside the Android sandbox, which causes `hf_hub::Cache::default()`
// to panic.  The caller resolves the app data directory and passes it here.

#[cfg(target_os = "android")]
pub async fn download_model<F>(
    model_id: String,
    on_progress: F,
    app_data_dir: Option<PathBuf>,
) -> Result<(), String>
where
    F: Fn(ModelDownloadProgress) + Send + Sync + Clone + 'static,
{
    use crate::inference::models::{
        BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF, BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF,
        QWEN25_1_5B_GGUF_FILE, QWEN25_1_5B_TOK_MODEL_ID, QWEN25_3B_GGUF_FILE,
        QWEN25_3B_TOK_MODEL_ID,
    };
    use crate::inference::token::hf_token_source;
    use hf_hub::Cache;
    use mistralrs::GgufModelBuilder;
    use mistralrs_core::GLOBAL_HF_CACHE;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;

    let (gguf_file, tok_model_id) = match model_id.as_str() {
        id if id == BARTOWSKI_QWEN25_1_5B_INSTRUCT_GGUF => {
            (QWEN25_1_5B_GGUF_FILE, QWEN25_1_5B_TOK_MODEL_ID)
        }
        id if id == BARTOWSKI_QWEN25_3B_INSTRUCT_GGUF => {
            (QWEN25_3B_GGUF_FILE, QWEN25_3B_TOK_MODEL_ID)
        }
        _ => {
            return Err(format!(
                "Model '{}' is not supported on Android. \
                 Only Qwen 2.5 1.5B or 3B (GGUF) are available.",
                model_id
            ));
        }
    };

    // ── Resolve the HF cache path ─────────────────────────────────────────
    //
    // `app_data_dir` is required on Android — `dirs::home_dir()` returns None
    // in the Android sandbox and causes Cache::default() to panic.
    let resolved_app_data = app_data_dir
        .ok_or_else(|| "app_data_dir is required on Android for HF cache resolution".to_string())?;

    let hf_home = resolved_app_data.join("models");
    let hf_hub_cache = hf_home.join("hub");

    fs::create_dir_all(&hf_hub_cache)
        .map_err(|e| format!("Failed to create HF hub cache dir: {e}"))?;

    // Seed GLOBAL_HF_CACHE so get_paths_gguf! never falls back to Cache::default().
    GLOBAL_HF_CACHE.get_or_init(|| Cache::new(hf_hub_cache.clone()));

    // Keep env vars in sync so hf_hub_cache_dir() and hf_cache_dir() agree.
    std::env::set_var("HF_HUB_CACHE", &hf_hub_cache);
    std::env::set_var("HF_HOME", &hf_home);

    info!(
        "Android HF hub cache resolved to: {}",
        hf_hub_cache.display()
    );

    let expected_size = SUPPORTED_MODEL_INFO
        .iter()
        .find(|m| m.id == model_id)
        .map(|m| m.expected_size_bytes)
        .unwrap_or(0);

    info!(
        "Starting Android download for model: {} (expected ~{})",
        model_id,
        format_size(expected_size)
    );

    clean_stale_lock_files(&model_id);
    ensure_model_cache_dir(&model_id);

    let initial_bytes = model_cache_path(&model_id)
        .filter(|p| p.exists())
        .map(|p| dir_size(&p))
        .unwrap_or(0);
    on_progress(make_progress(
        &model_id,
        initial_bytes,
        expected_size,
        false,
    ));

    // Native monitor thread — same reasoning as the Apple implementation.
    let finished = Arc::new(AtomicBool::new(false));
    let finished_clone = Arc::clone(&finished);
    let monitor_model_id = model_id.clone();
    let monitor_cb = on_progress.clone();

    let monitor_handle = std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));
        if finished_clone.load(Ordering::Relaxed) {
            break;
        }
        let current_bytes = model_cache_path(&monitor_model_id)
            .filter(|p| p.exists())
            .map(|p| dir_size(&p))
            .unwrap_or(0);
        monitor_cb(make_progress(
            &monitor_model_id,
            current_bytes,
            expected_size,
            false,
        ));
    });

    let result = GgufModelBuilder::new(&model_id, vec![gguf_file])
        .with_tok_model_id(tok_model_id)
        .with_token_source(hf_token_source())
        .with_logging()
        .build()
        .await;

    finished.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    match result {
        Ok(_model) => {
            let final_bytes = model_cache_path(&model_id)
                .filter(|p| p.exists())
                .map(|p| dir_size(&p))
                .unwrap_or(expected_size);
            on_progress(make_progress(&model_id, final_bytes, final_bytes, true));
            info!("Successfully downloaded model on Android: {}", model_id);
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to download model {}: {}", model_id, e);
            warn!("{}", msg);
            Err(msg)
        }
    }
}

// ── Other platforms: no-op ────────────────────────────────────────────────

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
    target_os = "android"
)))]
pub async fn download_model<F>(
    model_id: String,
    _on_progress: F,
    _app_data_dir: Option<PathBuf>,
) -> Result<(), String>
where
    F: Fn(ModelDownloadProgress) + Send + Sync + Clone + 'static,
{
    debug!(
        "download_model: not supported on this platform, ignoring request for {}.",
        model_id
    );
    Err(format!(
        "Model downloads are not supported on this platform (requested: {}).",
        model_id
    ))
}
