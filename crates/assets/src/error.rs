use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Error processing RON: {0}")]
    RonError(#[from] RonError),
    #[error("Asset I/O error")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum RonError {
    #[error("\n{0}")]
    Spanned(#[from] ron::error::SpannedError),
    #[error("{0}")]
    Regular(#[from] ron::Error),
}