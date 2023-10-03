use super::mesh::{MeshType, Mesh};

#[derive(Debug, Clone, Default)]
#[readonly::make]
pub struct Model {
    /// Model mesh type. It can be selected manually and is
    /// readonly during future use
    #[readonly]
    pub mesh_type: MeshType,
    pub mesh: Option<Mesh>,
}