use flatbox_core::math::transform::Transform;

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

pub struct ModelBundle<M: Material> {
    pub model: Model,
    pub material: M,
    pub transform: Transform,
}