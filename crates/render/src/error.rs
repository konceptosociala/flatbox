use std::path::PathBuf;

use image::ImageError;
use thiserror::Error;

use crate::hal::shader::ShaderError;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Error processing image data")]
    ImageProcessing(#[from] ImageError),
    #[error("Wrong image data passed")]
    WrongImageData,
    #[error("Error processing shaders")]
    ShaderProcessing(#[from] ShaderError),
    #[error("Material not bound: {0}")]
    MaterialNotBound(String),
    #[error("Model is not prepared for drawing. Before `DrawModelCommand` call `PrepareModelCommand` first")]
    ModelNotPrepared,
    #[error("There can be only one active camera at once")]
    MultipleActiveCameras,
    #[error("Cannot load model `{0}`")]
    ModelLoadError(PathBuf)
}