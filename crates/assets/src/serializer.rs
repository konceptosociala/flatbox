use std::io::{Read, Write};
use std::path::Path;
use std::fs;
use lz4::{Decoder, EncoderBuilder};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::AssetError;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct CompressionLevel(pub u32);

pub trait AssetSerializer {
    fn load<T>(&self, path: impl AsRef<Path>) -> Result<T, AssetError>
    where 
        T: for<'de> Deserialize<'de>; 

    fn save<T>(&self, value: &T, path: impl AsRef<Path>) -> Result<(), AssetError>
    where
        T: ?Sized + Serialize;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StringSerializer;

impl AssetSerializer for StringSerializer {
    fn load<T>(&self, path: impl AsRef<Path>) -> Result<T, AssetError>
    where 
        T: for<'de> Deserialize<'de>
    {
        ron::from_str::<T>(&fs::read_to_string(path)?)
            .map_err(StringSerializerError::from)
            .map_err(AssetError::from)
    }

    fn save<T>(&self, value: &T, path: impl AsRef<Path>) -> Result<(), AssetError>
    where
        T: ?Sized + Serialize 
    {
        let mut file = fs::File::create(path)?;
        let string = ron::ser::to_string_pretty(value, PrettyConfig::new())
            .map_err(StringSerializerError::from)?;

        writeln!(&mut file, "{string}")?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum StringSerializerError {
    #[error("RON error (spanned): \n{0}")]
    Spanned(#[from] ron::error::SpannedError),
    #[error("RON error: \n{0}")]
    Regular(#[from] ron::Error),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct BinarySerializer(pub Option<CompressionLevel>);

impl AssetSerializer for BinarySerializer {
    fn load<T>(&self, path: impl AsRef<Path>) -> Result<T, AssetError>
    where 
        T: for<'de> Deserialize<'de>
    {
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();

        if self.0.is_some() {
            let mut decoder = Decoder::new(file)?;
            decoder.read_to_end(&mut buffer)?;
        } else {
            file.read_to_end(&mut buffer)?;
        }
        
        bincode::deserialize::<T>(&buffer)
            .map_err(|e| AssetError::from(*e))
    }

    fn save<T>(&self, value: &T, path: impl AsRef<Path>) -> Result<(), AssetError>
    where
        T: ?Sized + Serialize 
    {
        let mut file = fs::File::create(path)?;
        let encoded = bincode::serialize(value)
            .map_err(|e| AssetError::from(*e))?;

        if let Some(level) = self.0 {
            let mut encoder = EncoderBuilder::new()
                .level(level.0)
                .build(&mut file)?;

            encoder.write_all(&encoded)?;
            encoder.finish().1?;
        } else {
            file.write_all(&encoded)?;
        }

        Ok(())
    }
}

pub type BinarySerializerError = bincode::ErrorKind;
