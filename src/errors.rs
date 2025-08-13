use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error(transparent)]
    Hash(#[from] HashError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unexpected cache version: expected {expected}, got {found}")]
    VersionMismatch { expected: u32, found: u32 },
}

#[derive(Debug, Error)]
pub enum HashError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),
}
