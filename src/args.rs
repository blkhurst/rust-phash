use crate::types as T;
use clap::{ArgAction, Parser, ValueHint};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "rust-phash",
    version,
    about = "Perceptual hashing to detect near-duplicate images.",
    author = "Blkhurst"
)]
pub struct Args {
    /// Directory to recursively scan
    #[arg(value_hint = ValueHint::DirPath)]
    pub directory: PathBuf,

    /// Hamming distance threshold
    #[arg(default_value_t = T::DEFAULT_THRESHOLD)]
    pub threshold: u32,

    /// Print JSON output
    #[arg(long = "json", action = ArgAction::SetTrue)]
    pub json: bool,

    /// Hashing Algorithm
    #[arg(long = "hash-alg", value_enum, default_value_t = T::DEFAULT_HASH_ALG)]
    pub hash_alg: T::HashAlg,

    /// Hash width (bits across).
    #[arg(long = "hash-w", default_value_t = T::DEFAULT_HASH_W)]
    pub hash_w: u32,

    /// Hash height (bits down).
    #[arg(long = "hash-h", default_value_t = T::DEFAULT_HASH_H)]
    pub hash_h: u32,

    /// Maximum parallelism (Rayon threads)
    #[arg(long = "parallel", default_value_t = T::DEFAULT_PARALLELISM)]
    pub parallel: usize,

    /// Cache file path.
    #[arg(long = "cache-file", default_value = T::DEFAULT_CACHE_FILE_NAME)]
    pub cache_file: PathBuf,
}
