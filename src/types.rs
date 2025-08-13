use clap::ValueEnum;

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
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum HashAlg {
    Mean,
    Gradient,
    DoubleGradient,
}
