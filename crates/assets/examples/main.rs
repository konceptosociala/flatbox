use flatbox_assets::manager::{AssetManager, Asset};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct MyAsset;

#[typetag::serde]
impl Asset for MyAsset {}

fn main() {
    let mut assets = AssetManager::new();
    let handle = assets.insert(MyAsset);

    println!("`get()`: {:#?}", assets.get::<MyAsset>(handle).unwrap());
}