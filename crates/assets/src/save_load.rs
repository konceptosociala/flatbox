use flatbox_ecs::World;

use crate::prelude::AssetError;

pub trait SaveLoad {
    fn save<P: AsRef<std::path::Path>>(
        &mut self,
        world: &World,
        path: P,
    ) -> Result<(), AssetError>;
    
    fn load<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
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
///     let ws = MyWorldSaver::default();
/// 
///     ws.save("/path/to/save", &world)?;
/// }
/// 
/// ```
#[macro_export]
macro_rules! impl_save_load {
    {
        loader: $ctx:ident, 
        components: [ $( $comp:ty ),+ ]
    } => {
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
            fn save<P: AsRef<std::path::Path>>(
                &mut self,
                world: &World,
                path: P,
            ) -> Result<(), AssetError> {
                use std::fs::File;
                use std::io::Cursor;
                use ron::ser::PrettyConfig;

                let mut world_buf = vec![];                    
                let mut ser = ron::Serializer::new(&mut world_buf, Some(PrettyConfig::new()))
                    .map_err(|e| $crate::error::RonError::from(e))?;    

                serialize_world(&world, self, &mut ser)
                    .map_err(|e| $crate::error::RonError::from(e))?;

                let file = File::create(path)?;
                let mut encoder = $crate::lz4::EncoderBuilder::new()
                    .level(4)
                    .build(file)?;  

                std::io::copy(&mut &*world_buf, &mut encoder)?; 

                Ok(encoder.finish().1?)
            }
            
            fn load<P: AsRef<std::path::Path>>(
                &mut self,
                path: P,
            ) -> Result<World, AssetError> {
                use std::fs::File;
                use std::io::Read;
                use serde::Deserialize;

                let package = File::open(path)?;
                let mut decoded = $crate::lz4::Decoder::new(package)?;
                let mut buffer = vec![];
                decoded.read_to_end(&mut buffer)?;

                let mut de = ron::Deserializer::from_bytes(&buffer)
                    .map_err(|e| $crate::error::RonError::from(e))?;

                let world = deserialize_world(self, &mut de)
                    .map_err(|e| $crate::error::RonError::from(e))?;

                Ok(world)
            }
        }
    };
}