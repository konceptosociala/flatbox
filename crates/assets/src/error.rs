use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Error processing RON: {0}")]
    RonError(#[from] RonError),
    #[error("Asset I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Asset handle is invalid; requested asset does not exist")]
    InvalidHandle,
    #[error("Requested asset's rw-lock is blocked")]
    AssetBlocked,
    #[error("Specified asset type is wrong: `{asset_type}`")]
    WrongAssetType {
        asset_type: String,
    },
}

#[derive(Debug, Error)]
pub enum RonError {
    #[error("\n{0}")]
    Spanned(#[from] ron::error::SpannedError),
    #[error("{0}")]
    Regular(#[from] ron::Error),
}