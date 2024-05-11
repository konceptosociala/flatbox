use flatbox_ecs::{serialize_world, deserialize_world, DeserializeContext, SerializeContext, World};
use serde::{Deserialize, Serialize, Serializer};
use std::{marker::PhantomData, path::Path};

use crate::{prelude::AssetError, serializer::AssetSerializer};

pub struct SerializeWorld<'a, C>(&'a World, PhantomData<C>);

impl<'a, C: SaveLoad> SerializeWorld<'a, C> {
    pub fn new(world: &'a World) -> Self {
        SerializeWorld(world, PhantomData)
    }
}

impl<'a, C: SaveLoad> Serialize for SerializeWorld<'a, C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer 
    {
        let mut ctx = C::default();
        serialize_world(self.0, &mut ctx, serializer)
    }
}

pub struct DeserializeWorld<C>(World, PhantomData<C>);

impl<C: SaveLoad> DeserializeWorld<C> {
    pub fn into_inner(self) -> World {
        self.0
    }
}

impl<'de, C: SaveLoad> Deserialize<'de> for DeserializeWorld<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> 
    {
        let mut ctx = C::default();
        Ok(DeserializeWorld(
            deserialize_world(&mut ctx, deserializer)?,
            PhantomData,
        ))
    }
}

pub trait SaveLoad: SerializeContext + DeserializeContext + Default {
    fn save(
        world: &World,
        path: impl AsRef<Path>,
        serializer: &impl AssetSerializer,
    ) -> Result<(), AssetError>;
    
    fn load(
        path: impl AsRef<Path>,
        serializer: &impl AssetSerializer,
    ) -> Result<World, AssetError>;
}

/// Macro that is used to create custom [`SaveLoad`]ers, 
/// that are capable of saving and loading individual serializable
/// components from the [`World`]
/// 
/// # Usage example
/// 
/// ```rust 
/// #[derive(Serialize, Deserialize)]
/// struct MyComponent(u32);
/// 
/// #[derive(Default)]
/// struct MySaveLoader {
///     components: Vec<String>, // required field
/// }
/// 
/// impl_save_load! {
///     loader: MySaveLoader, 
///     components: [
///         Camera, 
///         Timer, 
///         Transform,
///         MyComponent
///     ]
/// }
/// 
/// fn save_world(
///     world: Read<World>,
/// ) -> FlatboxResult<()> {
///     let ws = MySaveLoader::default();
/// 
///     ws.save("/path/to/save", &world)?;
/// 
///     Ok(())
/// }
/// 
/// ```
#[macro_export]
macro_rules! impl_save_load {
    {
        loader: $ctx:ident, 
        components: [ $( $comp:ty ),+ ]
    } => {
        #[derive(Default)]
        pub struct $ctx {
            components: Vec<String>,
        }

        impl SerializeContext for $ctx {
            fn component_count(&self, archetype: &Archetype) -> usize {                
                archetype.component_types()
                    .filter(|&t|
                        $(
                            t == std::any::TypeId::of::<$comp>() ||
                        )*
                        false
                    )
                    .count()
            }
            
            fn serialize_component_ids<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                    try_serialize_id::<$comp, _, _>(archetype, stringify!($comp), &mut out)?;
                )*
                
                out.end()
            }
            
            fn serialize_components<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                    try_serialize::<$comp, _>(archetype, &mut out)?;
                )*
                
                out.end()
            }
        }
        
        impl DeserializeContext for $ctx {
            fn deserialize_component_ids<'de, A: serde::de::SeqAccess<'de>>(
                &mut self,
                mut seq: A,
            ) -> Result<ColumnBatchType, A::Error> {
                self.components.clear();
                let mut batch = ColumnBatchType::new();
                while let Some(id) = seq.next_element::<String>()? {
                    match id.as_str() {                        
                        $(                            
                            stringify!($comp) => {
                                batch.add::<$comp>();
                            }
                        )*
                        
                        _ => {},
                    }
                    self.components.push(id);
                }
                
                Ok(batch)
            }
            
            fn deserialize_components<'de, A: serde::de::SeqAccess<'de>>(
                &mut self,
                entity_count: u32,
                mut seq: A,
                batch: &mut ColumnBatchBuilder,
            ) -> Result<(), A::Error> {
                for component in &self.components {
                    match component.as_str() {
                        $(                            
                            stringify!($comp) => {
                                deserialize_column::<$comp, _>(entity_count, &mut seq, batch)?;
                            }
                        )*
                        
                        _ => {},
                    }
                }
                
                Ok(())
            }

        }
        
        impl SaveLoad for $ctx {
            fn save(
                world: &World,
                path: impl AsRef<std::path::Path>,
                serializer: &impl flatbox_assets::serializer::AssetSerializer,
            ) -> Result<(), AssetError> {
                let serialize_world = SerializeWorld::<$ctx>::new(&world);
                serializer.save(&serialize_world, path)
            }
            
            fn load(
                path: impl AsRef<std::path::Path>,
                serializer: &impl flatbox_assets::serializer::AssetSerializer,
            ) -> Result<World, AssetError> {
                let deserialize_world = serializer.load::<DeserializeWorld<$ctx>>(path)?;
                Ok(deserialize_world.into_inner())
            }
        }
    };
}