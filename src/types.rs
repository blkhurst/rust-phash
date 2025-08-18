use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "avif", "heic", "heif",
];

pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "mov", "mkv", "avi", "webm", "mpg", "mpeg"];

/// Perceptual hash algorithm.
pub const DEFAULT_HASH_ALG: HashAlg = HashAlg::DoubleGradient;

/// Hash size
pub const DEFAULT_HASH_W: u32 = 16;
pub const DEFAULT_HASH_H: u32 = 16;

/// Hamming distance threshold.
pub const DEFAULT_THRESHOLD: u32 = 10;

/// Video ~ Frame to start sampling from.
pub const DEFAULT_SAMPLE_START: usize = 0;

/// Video ~ Number of frames sampled; evenly spaced between start and start+window
pub const DEFAULT_SAMPLE_COUNT: usize = 10;

/// Video ~ Number of frames to sample over; 0 = auto.
pub const DEFAULT_SAMPLE_WINDOW: usize = 0;

/// Video ~ Aggregation strategy default.
pub const DEFAULT_AGGREGATION: Aggregation = Aggregation::Medoid;

/// Default parallelism. If 0, Rayon decides.
pub const DEFAULT_PARALLELISM: usize = 0;

/// Cache filename
pub const DEFAULT_CACHE_FILE_NAME: &str = ".phash-cache.json";

/// Hashing algorithm choices, mirrored from `img_hash`.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlg {
    Mean,
    Gradient,
    DoubleGradient,
}

/// Video ~ Hash aggregation strategy
/// - Majority: Slower, bitwise majority vote across frame hashes.
/// - Medoid: Faster, picks the frame with the smallest hamming distance to all others.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aggregation {
    Majority,
    Medoid,
}

/// Cache
///
/// Cache Version
pub const CACHE_VERSION: u32 = 3;

/// Cache schema persisted to JSON.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheFile {
    pub version: u32,
    // pub by_blake3: std::collections::HashMap<String, Vec<CacheEntry>>,
    pub by_blake3: std::collections::BTreeMap<String, Vec<CacheEntry>>,
}

/// A single cache entry with the exact parameters used to compute the hash.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub hash_alg: HashAlg,
    pub hash_w: u32,
    pub hash_h: u32,
    pub perceptual_hash: String,
    // Video
    pub sample_start: Option<usize>,
    pub sample_count: Option<usize>,
    pub sample_window: Option<usize>,
    pub aggregation: Option<Aggregation>,
}

/// App-wide Config, reducing boiler-plate function arguments
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub media_paths: Vec<PathBuf>,
    pub hash_alg: HashAlg,
    pub hash_w: u32,
    pub hash_h: u32,
    pub parallelism: usize,
    // Video
    pub sample_start: usize,
    pub sample_count: usize,
    pub sample_window: usize,
    pub aggregation: Aggregation,
}

/// Pipeline Result for displaying information to user.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub path: PathBuf,
    pub blake3: String,
    pub perceptual_hash: String,
}
