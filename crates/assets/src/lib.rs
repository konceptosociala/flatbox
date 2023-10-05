use serde::{Serialize, Deserialize};
use slotmap::{KeyData, Key};

pub mod error;
pub mod manager;

pub use tar;
pub use lz4;
pub use typetag;

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AssetHandle(KeyData);

impl From<KeyData> for AssetHandle {
    fn from(value: KeyData) -> Self {
        AssetHandle(value)
    }
}

unsafe impl Key for AssetHandle {
    fn data(&self) -> KeyData {
        self.0
    }
}