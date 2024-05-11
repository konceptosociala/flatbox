use thiserror::Error;

use crate::serializer::{BinarySerializerError, StringSerializerError};

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Error during (de-)serialization from string")]
    StringSerializerError(#[from] StringSerializerError),
    #[error("Error during binary (de-)serialization")]
    BinarySerializerError(#[from] BinarySerializerError),
    #[error("Asset I/O error")]
    IoError(#[from] std::io::Error),
}