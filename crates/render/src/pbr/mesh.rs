use std::path::PathBuf;
use serde::{Serialize, Deserialize};
// use flatbox_core::math::*;

use crate::{
    hal::buffer::{Buffer, VertexArray}, 
    renderer::Renderer
};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
}

/// Represents the type of mesh in [`Model`] struct.
/// It indicates whether mesh must be created in runtime,
/// loaded from file (or resource) or created manually
/// with index and vertex buffers.
#[derive(Clone, Default, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum MeshType {
    /// Plane mesh
    Plane,
    /// Cube mesh
    #[default]
    Cube,
    /// Icosphere mesh
    Icosahedron,
    /// Refined icosphere mesh
    Sphere,
    /// Mesh which have been loaded from file or resource
    Loaded(PathBuf),
    /// Custom model type, which neither loaded from file, nor
    /// created in runtime. Unlike other meshes it's (de-)serialized.
    /// Use it when constructing models manually
    Generic,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertex_data: Vec<Vertex>,
    pub index_data: Vec<u32>,

    #[serde(skip)]
    pub(crate) vertex_array: Option<VertexArray>,
    #[serde(skip)]
    pub(crate) vertex_buffer: Option<Buffer>,
    #[serde(skip)]
    pub(crate) index_buffer: Option<Buffer>,
}

impl Mesh {
    pub fn new(vertices: &[Vertex], indices: &[u32]) -> Mesh {
        Mesh {
            vertex_data: Vec::from(vertices),
            index_data: Vec::from(indices),
            vertex_array: None,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
    
    pub fn bind(&mut self, renderer: &Renderer) {

    }

    pub fn draw(&self, renderer: &Renderer) {

    }
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Mesh {
            vertex_data: self.vertex_data.clone(),
            index_data: self.index_data.clone(),
            vertex_array: None,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}
