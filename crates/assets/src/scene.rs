use std::sync::Arc;
use std::path::Path;
use std::fs::{File, read_to_string};
use parking_lot::Mutex;
use ron::ser::{Serializer, PrettyConfig};
use serde::{Serialize, Deserialize};
use flatbox_ecs::{World, EntityBuilder};

use crate::error::RonError;
use crate::{
    error::AssetError,
    manager::AssetManager,
    ser_component::SerializableComponent,
};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "Entity")]
pub struct SerializableEntity {
    pub components: Vec<Arc<Mutex<Box<dyn SerializableComponent + 'static>>>>
}

/// Macro for easy [`SerializableEntity`] creation. Often used along with
/// `scene!` macro during [`Scene`] creating
/// 
/// # Usage example
/// ```rust
/// let entity = entity![
///     AssetHandle::null(), 
///     Model::cube(),
///     Transform::default()
/// ];
/// ```
#[macro_export]
macro_rules! entity {
    [$( $comp:expr ),+] => {
        {
            use ::std::sync::Arc;
            use $crate::parking_lot::Mutex;

            let mut entity = SerializableEntity::default();
            $(
                entity.components.push(Arc::new(Mutex::new(Box::new($comp))));
            )+
            entity
        }
    };
}

#[derive(Default, Serialize, Deserialize)]
pub struct Scene {
    pub assets: AssetManager,
    pub entities: Vec<SerializableEntity>,
}

impl Scene {    
    pub fn new() -> Self {
        Scene::default()
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, AssetError> {     
        Ok(ron::from_str::<Scene>(
            &read_to_string(path)?
        ).map_err(RonError::from)?)
    }
    
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), AssetError> {     
        let buf = File::create(path)?;                    
        let mut ser = Serializer::new(buf, Some(
            PrettyConfig::new()
                .struct_names(true)
        )).map_err(RonError::from)?;   
        
        self.serialize(&mut ser).map_err(RonError::from)?;
                        
        Ok(())
    }
}

/// Macro for easy [`Scene`] creation. `entities` can be created with [`entity!`] 
/// macro or manually:
/// ```rust
/// let entity = SerializableEntity {
///     components: vec![
///         Arc::new(comp1),
///         Arc::new(comp2),
///     ],
/// };
/// ```
/// 
/// # Usage example
/// ```rust
/// let scene = scene! {
///     assets: AssetManager::new(),
///     entities: [
///         entity![
///             Camera::builder()
///                 .is_active(true)
///                 .camera_type(CameraType::FirstPerson)
///                 .build(),
///             Transform::default()
///         ],
///         entity![
///             AssetHandle::null(), 
///             Model::cube(),
///             Transform::default()
///         ]
///     ]
/// };
/// ```
#[macro_export]
macro_rules! scene {
    {
        assets: $assets:expr,
        entities: [$( $entity:expr ),+]
    } => {
        {
            let mut entities = Vec::new();
            $(
                entities.push($entity);
            )+

            Scene {
                assets: $assets,
                entities,
            }
        }
    };
}

pub trait SpawnSceneExt {
    fn spawn_scene(&mut self, scene: Scene);
}

impl SpawnSceneExt for World {
    fn spawn_scene(&mut self, scene: Scene) {
        self.clear();

        for entity in scene.entities {
            let mut entity_builder = EntityBuilder::new();
            
            for component in entity.components {
                component.lock().add_into(&mut entity_builder);
            }
            
            self.spawn(entity_builder.build());
        }        
    }
}