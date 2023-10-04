use std::collections::HashMap;
use std::any::TypeId;

use as_any::{Downcast, AsAny};
use atomic_refcell::AtomicRefCell;
use parking_lot::RwLock;

pub struct AssetsManager {
    // assets: RwLock<HashMap<TypeId, AtomicRefCell<dyn AsAny>>>
}