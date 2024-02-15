use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use as_any::AsAny;
use parking_lot::{RwLock, MappedRwLockReadGuard, RwLockReadGuard, MappedRwLockWriteGuard, RwLockWriteGuard};

pub trait Resource: AsAny + Send + Sync + 'static {}
impl<T: AsAny + Send + Sync + 'static> Resource for T {}

#[derive(Default)]
pub struct Resources {
    res: HashMap<TypeId, Arc<RwLock<dyn Resource>>>
}

impl Resources {
    pub fn new() -> Resources {
        Resources::default()
    }

    pub fn add_resource<R: Resource>(&mut self, resource: R) {
        self.res.insert(TypeId::of::<R>(), Arc::new(RwLock::new(resource)));
    }

    pub fn get_resource<R: Resource>(&self) -> Option<MappedRwLockReadGuard<R>> {
        if let Some(res) = self.res.get(&TypeId::of::<R>()) {
            let data = match res.try_read() {
                Some(data) => data,
                None => return None,
            };
            
            return RwLockReadGuard::try_map(data, |data| {
                (*data).as_any().downcast_ref::<R>()
            }).ok();
        }

        None
    }

    pub fn get_resource_mut<R: Resource>(&self) -> Option<MappedRwLockWriteGuard<R>> {
        if let Some(res) = self.res.get(&TypeId::of::<R>()) {
            let data = match res.try_write() {
                Some(data) => data,
                None => return None,
            };
            
            return RwLockWriteGuard::try_map(data, |data| {
                (*data).as_any_mut().downcast_mut::<R>()
            }).ok();
        }

        None
    }

    pub fn remove_resource<R: Resource>(&mut self) {
        self.res.remove(&TypeId::of::<R>());
    }
}