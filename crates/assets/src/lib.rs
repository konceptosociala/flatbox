use slotmap::new_key_type;

pub mod error;
pub mod manager;
pub mod resources;
pub mod save_load;
pub mod scene;
pub mod ser_component;

pub use ron;
pub use tar;
pub use lz4;
pub use typetag;
pub use parking_lot;

new_key_type! {
    pub struct AssetHandle;
}