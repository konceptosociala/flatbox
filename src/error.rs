use flatbox_assets::error::AssetError;
use flatbox_render::error::RenderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlatboxError {
    #[error("Asset processing error")]
    AssetError(#[from] AssetError),
    #[error("Rendering error")]
    RenderError(#[from] RenderError)
}

pub type FlatboxResult<T> = Result<T, FlatboxError>;