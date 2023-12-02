use flatbox_assets::{impl_ser_component, typetag};
use flatbox_core::math::transform::Transform;
use serde::{
    Serialize, 
    Deserialize,
    Serializer, 
    Deserializer, 
    de::*,
    de::Error as DeError,
    ser::SerializeStruct,
};

use crate::pbr::{
    mesh::{MeshType, Mesh},
    material::Material,
};

#[derive(Debug, Clone, Default)]
#[readonly::make]
pub struct Model {
    /// Model mesh type. It can be selected manually and is
    /// readonly during future use
    #[readonly]
    pub mesh_type: MeshType,
    pub mesh: Option<Mesh>,
}

impl Model {
    pub fn cube() -> Model {
        Model {
            mesh_type: MeshType::Cube,
            mesh: Some(Mesh::cube()),
        }
    }
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut model = serializer.serialize_struct("Model", 2)?;
        model.serialize_field("mesh_type", &self.mesh_type)?;

        match self.mesh_type {
            MeshType::Generic => {
                model.serialize_field("mesh", &self.mesh)?;
            },
            _ => {
                model.serialize_field("mesh", &Option::<Mesh>::None)?;
            }
        }

        model.end()
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum ModelField { 
            MeshType,
            Mesh,
        }

        struct ModelVisitor;

        impl<'de> Visitor<'de> for ModelVisitor {
            type Value = Model;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Model")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Model, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mesh_type: MeshType = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;

                let mesh = match mesh_type {
                    MeshType::Cube => { Some(Mesh::cube()) },
                    // MeshType::Icosahedron => { Some(Mesh::icosahedron()) },
                    // MeshType::Sphere => { Some(Mesh::sphere()) },
                    // MeshType::Plane => { Some(Mesh::plane()) },
                    // MeshType::Loaded(path) => {
                        // return Ok(Model::load_obj(path)
                            // .expect("Cannot load deserialized model from path"));
                    // },
                    MeshType::Generic => { 
                        seq.next_element()?.ok_or_else(|| DeError::invalid_length(1, &self))? 
                    },
                    _ => todo!("Mesh types: `icosahedron`, `sphere`, `plane` etc."),
                };

                Ok(Model {
                    mesh_type,
                    mesh,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Model, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut mesh_type: Option<MeshType> = None;
                let mut mesh: Option<Option<Mesh>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        ModelField::MeshType => {
                            if mesh_type.is_some() {
                                return Err(DeError::duplicate_field("mesh_type"));
                            }
                            mesh_type = Some(map.next_value()?);
                        },
                        ModelField::Mesh => {
                            if mesh.is_some() {
                                return Err(DeError::duplicate_field("mesh"));
                            }
                            mesh = Some(map.next_value()?);
                        },
                    }
                }

                let mesh_type = mesh_type.ok_or_else(|| DeError::missing_field("mesh_type"))?;

                let mesh = match mesh_type {
                    MeshType::Cube => { Some(Mesh::cube()) },
                    // MeshType::Icosahedron => { Some(Mesh::icosahedron()) },
                    // MeshType::Sphere => { Some(Mesh::sphere()) },
                    // MeshType::Plane => { Some(Mesh::plane()) },
                    // MeshType::Loaded(path) => {
                        // return Ok(Model::load_obj(path)
                            // .expect("Cannot load deserialized model from path"));
                    // },
                    MeshType::Generic => { 
                        mesh.ok_or_else(|| DeError::missing_field("mesh"))?
                    },
                    _ => todo!("Mesh types: `icosahedron`, `sphere`, `plane` etc."),
                };

                Ok(Model {
                    mesh_type,
                    mesh,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "mesh_type",
            "mesh"
        ];
        deserializer.deserialize_struct("Model", FIELDS, ModelVisitor)
    }
}

impl_ser_component!(Model);

pub struct ModelBundle<M: Material> {
    pub model: Model,
    pub material: M,
    pub transform: Transform,
}