use std::{path::PathBuf, sync::Arc};
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use flatbox_core::math::glm;

use crate::{
    macros::set_vertex_attribute,
    hal::{
        buffer::{Buffer, VertexArray, BufferTarget, BufferUsage, AttributeType}, 
        shader::GraphicsPipeline
    }, 
};

#[allow(unused_imports)]
use crate::pbr::model::Model;

use super::material::Material;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub texcoord: glm::Vec2,
}

impl Vertex {
    /// Get middle point between two vertices
    pub fn midpoint(a: &Vertex, b: &Vertex) -> Vertex {
        Vertex {
            position: glm::vec3(
                0.5 * (a.position[0] + b.position[0]),
                0.5 * (a.position[1] + b.position[1]),
                0.5 * (a.position[2] + b.position[2]),
            ),
            normal: Self::normalize(glm::vec3(
                0.5 * (a.normal[0] + b.normal[0]),
                0.5 * (a.normal[1] + b.normal[1]),
                0.5 * (a.normal[2] + b.normal[2]),
            )),
            texcoord: glm::vec2(
                0.5 * (a.texcoord[0] + b.texcoord[0]),
                0.5 * (a.texcoord[1] + b.texcoord[1]),
            ),
        }
    }
    
    /// Normalize vector/vertex. Returns vector with the same direction and `1` lenght
    pub fn normalize(v: glm::Vec3) -> glm::Vec3 {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        glm::vec3(v[0] / l, v[1] / l, v[2] / l)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    pub first_index: u32,
    pub index_count: u32,
    /// Handle of material, which is attached to rendered mesh primitive
    pub material: Arc<Mutex<Box<dyn Material>>>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub vertex_data: Vec<Vertex>,
    pub index_data: Vec<u32>,
    pub primitives: Vec<Primitive>,

    #[serde(skip)]
    pub(crate) prepared: bool,
    #[serde(skip)]
    pub(crate) vertex_array: VertexArray,
    #[serde(skip)]
    pub(crate) vertex_buffer: Option<Buffer>,
    #[serde(skip)]
    pub(crate) index_buffer: Option<Buffer>,
}

impl Mesh {
    pub fn new(vertices: &[Vertex], indices: &[u32], primitives: &[Primitive]) -> Mesh {
        Mesh {
            vertex_data: vertices.to_vec(),
            index_data: indices.to_vec(),
            primitives: primitives.to_vec(),
            prepared: false,
            vertex_array: VertexArray::new(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn cube() -> Mesh {
        Mesh::new(
            &[
                Vertex { position: glm::vec3(-0.5,0.5,-0.5), normal: glm::vec3(0.0, 0.0, -1.0), texcoord: glm::vec2(0.0, 0.0) },
                Vertex { position: glm::vec3(-0.5,-0.5,-0.5), normal: glm::vec3(0.0, 0.0, -1.0), texcoord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec3(0.5,-0.5,-0.5), normal: glm::vec3(0.0, 0.0, -1.0), texcoord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec3(0.5,0.5,-0.5), normal: glm::vec3(0.0, 0.0, -1.0), texcoord: glm::vec2(1.0, 0.0) },

                Vertex { position: glm::vec3(-0.5,0.5,0.5), normal: glm::vec3(0.0, 0.0, 1.0), texcoord: glm::vec2(0.0, 0.0) },
                Vertex { position: glm::vec3(-0.5,-0.5,0.5), normal: glm::vec3(0.0, 0.0, 1.0), texcoord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec3(0.5,-0.5,0.5), normal: glm::vec3(0.0, 0.0, 1.0), texcoord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec3(0.5,0.5,0.5), normal: glm::vec3(0.0, 0.0, 1.0), texcoord: glm::vec2(1.0, 0.0) },

                Vertex { position: glm::vec3(0.5,0.5,-0.5), normal: glm::vec3(1.0, 0.0, 0.0), texcoord: glm::vec2(0.0, 0.0) },
                Vertex { position: glm::vec3(0.5,-0.5,-0.5), normal: glm::vec3(1.0, 0.0, 0.0), texcoord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec3(0.5,-0.5,0.5), normal: glm::vec3(1.0, 0.0, 0.0), texcoord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec3(0.5,0.5,0.5), normal: glm::vec3(1.0, 0.0, 0.0), texcoord: glm::vec2(1.0, 0.0) },

                Vertex { position: glm::vec3(-0.5,0.5,-0.5), normal: glm::vec3(-1.0, 0.0, 0.0), texcoord: glm::vec2(0.0, 0.0) },
                Vertex { position: glm::vec3(-0.5,-0.5,-0.5), normal: glm::vec3(-1.0, 0.0, 0.0), texcoord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec3(-0.5,-0.5,0.5), normal: glm::vec3(-1.0, 0.0, 0.0), texcoord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec3(-0.5,0.5,0.5), normal: glm::vec3(-1.0, 0.0, 0.0), texcoord: glm::vec2(1.0, 0.0) },

                Vertex { position: glm::vec3(-0.5,0.5,0.5), normal: glm::vec3(0.0, 1.0, 0.0), texcoord: glm::vec2(0.0, 0.0) },
                Vertex { position: glm::vec3(-0.5,0.5,-0.5), normal: glm::vec3(0.0, 1.0, 0.0), texcoord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec3(0.5,0.5,-0.5), normal: glm::vec3(0.0, 1.0, 0.0), texcoord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec3(0.5,0.5,0.5), normal: glm::vec3(0.0, 1.0, 0.0), texcoord: glm::vec2(1.0, 0.0) },

                Vertex { position: glm::vec3(-0.5,-0.5,0.5), normal: glm::vec3(0.0, -1.0, 0.0), texcoord: glm::vec2(0.0, 0.0) },
                Vertex { position: glm::vec3(-0.5,-0.5,-0.5), normal: glm::vec3(0.0, -1.0, 0.0), texcoord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec3(0.5,-0.5,-0.5), normal: glm::vec3(0.0, -1.0, 0.0), texcoord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec3(0.5,-0.5,0.5), normal: glm::vec3(0.0, -1.0, 0.0), texcoord: glm::vec2(1.0, 0.0) },
            ],
            &[
                0,1,3, 3,1,2,
                4,5,7, 7,5,6,
                8,9,11, 11,9,10,
                12,13,15, 15,13,14,
                16,17,19, 19,17,18,
                20,21,23, 23,21,22
            ],
            &[],
        )
    }
    
    pub fn setup(&mut self, pipeline: &GraphicsPipeline) {
        if self.vertex_buffer.is_some() && self.index_buffer.is_some() {
            return;
        }

        self.vertex_buffer = Some(Buffer::new(BufferTarget::ArrayBuffer, BufferUsage::StaticDraw));
        self.index_buffer = Some(Buffer::new(BufferTarget::ElementArrayBuffer, BufferUsage::StaticDraw));

        self.update_vertices();

        let position_attribute = pipeline.get_attribute_location("position");
        let normal_attribute = pipeline.get_attribute_location("normal");
        let texcoord_attribute = pipeline.get_attribute_location("texcoord");

        let vertex_array = &self.vertex_array;
        set_vertex_attribute!(vertex_array, position_attribute, Vertex::position, AttributeType::Float);
        set_vertex_attribute!(vertex_array, normal_attribute, Vertex::normal, AttributeType::Float);
        set_vertex_attribute!(vertex_array, texcoord_attribute, Vertex::texcoord, AttributeType::Float);
    }

    pub fn update_vertices(&self){     
        self.vertex_array.bind();

        if let (Some(ref vertex_buffer), Some(ref index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
            vertex_buffer.fill(&self.vertex_data);
            index_buffer.fill(&self.index_data);
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh::cube()
    }
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Mesh {
            vertex_data: self.vertex_data.clone(),
            index_data: self.index_data.clone(),
            primitives: self.primitives.clone(),
            prepared: false,
            vertex_array: VertexArray::default(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}
