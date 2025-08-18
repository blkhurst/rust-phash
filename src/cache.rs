use crate::{errors::CacheError, types as T};
use serde_json;
use std::{fs, io::Write, path::Path};

/// Load + Deserialise, or Create CacheFile
pub fn load_cache(path: &Path) -> Result<T::CacheFile, CacheError> {
    if !path.exists() {
        return Ok(T::CacheFile {
            version: T::CACHE_VERSION,
            ..Default::default()
        });
    }

    let bytes = fs::read(path)?;
    let cache: T::CacheFile = serde_json::from_slice(&bytes)?;

    if cache.version != T::CACHE_VERSION {
        return Err(CacheError::VersionMismatch {
            expected: T::CACHE_VERSION,
            found: cache.version,
        });
    }

    Ok(cache)
}

/// Save CacheFile
pub fn save_cache(path: &Path, cache: &T::CacheFile) -> Result<(), CacheError> {
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(cache)?;

    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(&json)?;
        file.sync_all().ok();
    }

    fs::rename(&tmp, path)?;
    Ok(())
}

/// Lookup cache entry for a given BLAKE3 key AND the current config.
pub fn lookup(
    cache: &T::CacheFile,
    key: &str,
    cfg: &T::AppConfig,
    is_video: bool,
) -> Option<T::CacheEntry> {
    let entries = cache.by_blake3.get(key)?;
    if is_video {
        entries
            .iter()
            .find(|e| matches_video_params(e, cfg))
            .cloned()
    } else {
        entries
            .iter()
            .find(|e| matches_image_params(e, cfg))
            .cloned()
    }
}

/// Insert or replace the cache entry for this BLAKE3 key and config.
pub fn upsert(cache: &mut T::CacheFile, key: String, entry: T::CacheEntry) {
    let vec = cache.by_blake3.entry(key).or_default();
    // Replace if same config already stored, else append.
    if let Some(existing) = vec.iter_mut().find(|e| {
        // Compare config fields
        e.hash_alg == entry.hash_alg
            && e.hash_w == entry.hash_w
            && e.hash_h == entry.hash_h
            && e.sample_start == entry.sample_start
            && e.sample_count == entry.sample_count
            && e.sample_window == entry.sample_window
            && e.aggregation == entry.aggregation
    }) {
        *existing = entry;
    } else {
        vec.push(entry);
    }
}

/// Check if image configuration exists
fn matches_image_params(e: &T::CacheEntry, cfg: &T::AppConfig) -> bool {
    e.hash_alg == cfg.hash_alg
        && e.hash_w == cfg.hash_w
        && e.hash_h == cfg.hash_h
        && e.sample_start.is_none()
        && e.sample_count.is_none()
        && e.sample_window.is_none()
        && e.aggregation.is_none()
}

/// Check if video configuration exists
fn matches_video_params(e: &T::CacheEntry, cfg: &T::AppConfig) -> bool {
    e.hash_alg == cfg.hash_alg
        && e.hash_w == cfg.hash_w
        && e.hash_h == cfg.hash_h
        && e.sample_start == Some(cfg.sample_start)
        && e.sample_count == Some(cfg.sample_count)
        && e.sample_window == Some(cfg.sample_window)
        && e.aggregation == Some(cfg.aggregation)
}
