use std::sync::Arc;
use std::path::Path;
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use flatbox_ecs::{World, EntityBuilder};

use crate::serializer::AssetSerializer;
use crate::{
    error::AssetError,
    ser_component::SerializableComponent,
};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "Entity")]
pub struct SerializableEntity {
    pub components: Vec<Arc<Mutex<Box<dyn SerializableComponent>>>>
}

/// Macro for easy [`SerializableEntity`] creation. Often used along with
/// `scene!` macro during [`Scene`] creating
/// 
/// # Usage example
/// ```rust
/// let entity = entity![
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
    pub entities: Vec<SerializableEntity>,
}

impl Scene {    
    pub fn new() -> Self {
        Scene::default()
    }
    
    pub fn load(
        path: impl AsRef<Path>, 
        serializer: impl AssetSerializer,
    ) -> Result<Self, AssetError> {
        serializer.load(path)
    }
    
    pub fn save(
        &self,
        path: impl AsRef<Path>, 
        serializer: impl AssetSerializer,
    ) -> Result<(), AssetError> {     
        serializer.save(&self, path)
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
        entities: [$( $entity:expr ),+]
    } => {
        {
            let mut entities = Vec::new();
            $(
                entities.push($entity);
            )+

            Scene {
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