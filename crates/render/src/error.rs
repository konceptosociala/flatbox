use image::ImageError;
use thiserror::Error;

use crate::hal::shader::ShaderError;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Error processing image data")]
    ImageProcessing(#[from] ImageError),
    #[error("Error processing shaders")]
    ShaderProcessing(#[from] ShaderError),
}