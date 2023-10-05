use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Asset handle is invalid; requested asset does not exist")]
    InvalidHandle,
    #[error("Requested asset's rw-lock is blocked")]
    AssetBlocked,
    #[error("Specified asset type is wrong: `{asset_type}`")]
    WrongAssetType {
        asset_type: String,
    },
}