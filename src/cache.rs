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

/// Lookup cache entry using Blake3 key
pub fn lookup(cache: &T::CacheFile, key: &str) -> Option<T::CacheEntry> {
    cache.by_blake3.get(key).cloned()
}

/// Insert or update cache entry using Blake3 key
pub fn upsert(cache: &mut T::CacheFile, key: String, entry: T::CacheEntry) {
    cache.by_blake3.insert(key, entry);
}
