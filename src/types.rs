use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "avif", "heic", "heif",
];

/// Perceptual hash algorithm.
pub const DEFAULT_HASH_ALG: HashAlg = HashAlg::DoubleGradient;

/// Hash size
pub const DEFAULT_HASH_W: u32 = 16;
pub const DEFAULT_HASH_H: u32 = 16;

/// Hamming distance threshold.
pub const DEFAULT_THRESHOLD: u32 = 20;

/// Default parallelism. If 0, Rayon decides.
pub const DEFAULT_PARALLELISM: usize = 0;

/// Cache filename
pub const DEFAULT_CACHE_FILE_NAME: &str = ".vhash-cache.json";

/// Hashing algorithm choices, mirrored from `img_hash`.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlg {
    Mean,
    Gradient,
    DoubleGradient,
}

/// Cache
///
/// Cache Version
pub const CACHE_VERSION: u32 = 1;

/// Cache schema persisted to JSON.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheFile {
    pub version: u32,
    pub by_blake3: std::collections::HashMap<String, CacheEntry>,
}

/// A single cache entry with the exact parameters used to compute the hash.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub hash_alg: HashAlg,
    pub hash_w: u32,
    pub hash_h: u32,
    pub perceptual_hash: String,
}

/// App-wide Config, reducing boiler-plate function arguments
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub media_paths: Vec<PathBuf>,
    pub hash_alg: HashAlg,
    pub hash_w: u32,
    pub hash_h: u32,
    pub threshold: u32,
    pub parallelism: usize,
}
