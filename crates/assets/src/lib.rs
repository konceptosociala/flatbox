use slotmap::new_key_type;

pub mod error;
pub mod manager;
pub mod save_load;
pub mod scene;
pub mod ser_component;

pub use tar;
pub use lz4;
pub use typetag;

new_key_type! {
    pub struct AssetHandle;
}