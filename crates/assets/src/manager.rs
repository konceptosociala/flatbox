use std::sync::Arc;
use as_any::AsAny;
use parking_lot::{RwLock, MappedRwLockWriteGuard, RwLockWriteGuard, MappedRwLockReadGuard, RwLockReadGuard};
use pretty_type_name::pretty_type_name;
use serde::{Serialize, Deserialize};
use slotmap::SlotMap;

use crate::error::AssetError;
use crate::AssetHandle;

#[typetag::serde(tag = "asset")]
pub trait Asset: AsAny + Send + Sync {}

#[derive(Default, Serialize, Deserialize)]
pub struct AssetManager {
    cache: SlotMap<AssetHandle, Arc<RwLock<Box<dyn Asset>>>>,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager::default()
    }

    pub fn insert<A: Asset>(&mut self, asset: A) -> AssetHandle {
        self.cache.insert(Arc::new(RwLock::new(Box::new(asset))))
    }

    pub fn get<A: Asset>(&self, handle: AssetHandle) -> Result<MappedRwLockReadGuard<A>, AssetError> {
        if let Some(asset) = self.cache.get(handle) {
            let data = match asset.try_read() {
                Some(data) => data,
                None => return Err(AssetError::AssetBlocked),
            };
            
            return RwLockReadGuard::try_map(data, |data| {
                (**data).as_any().downcast_ref::<A>()
            }).map_err(|_| AssetError::WrongAssetType { asset_type: pretty_type_name::<A>().to_string() });
        }

        Err(AssetError::InvalidHandle)
    }

    pub fn get_mut<A: Asset>(&mut self, handle: AssetHandle) -> Result<MappedRwLockWriteGuard<A>, AssetError> {
        if let Some(asset) = self.cache.get_mut(handle) {
            let data = match asset.try_write() {
                Some(data) => data,
                None => return Err(AssetError::AssetBlocked),
            };
            
            return RwLockWriteGuard::try_map(data, |data| {
                (**data).as_any_mut().downcast_mut::<A>()
            }).map_err(|_| AssetError::WrongAssetType { asset_type: pretty_type_name::<A>().to_string() });
        }

        Err(AssetError::InvalidHandle)
    }

    pub fn get_dynamic(&self, handle: AssetHandle) -> Result<RwLockReadGuard<Box<dyn Asset>>, AssetError> {
        if let Some(asset) = self.cache.get(handle) {
            return asset.try_read().ok_or(AssetError::AssetBlocked);  
        }

        Err(AssetError::InvalidHandle)
    }

    pub fn remove<A: Asset>(&mut self, handle: AssetHandle) {
        self.cache.remove(handle);
    }
}