use flatbox_assets::{impl_save_load, serializer::{BinarySerializer, CompressionLevel}};
use flatbox_ecs::*;
use flatbox_assets::prelude::*;
use serde::{Deserialize, Serialize};

static SERIALIZER: BinarySerializer = BinarySerializer(Some(CompressionLevel(4)));

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
struct MyComponent(u32);

impl_save_load! {
    loader: SaveLoadImpl, 
    components: [
        MyComponent
    ]
}

fn main() -> anyhow::Result<()> {
    let mut world = World::new();
    let e = world.spawn((MyComponent(12),));

    SaveLoadImpl::save(&world, "/tmp/world", &SERIALIZER)?;

    world = SaveLoadImpl::load("/tmp/world", &SERIALIZER)?;

    assert_eq!(world.get::<&MyComponent>(e).ok().map(|v| *v), Some(MyComponent(12)));

    Ok(())
}