use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use flatbox_assets::AssetHandle;

use crate::{
    macros::set_vertex_attribute,
    hal::{
        buffer::{Buffer, VertexArray, BufferTarget, BufferUsage}, 
        shader::GraphicsPipeline
    }, 
    renderer::Renderer,
};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Primitive {
    pub first_index: u32,
    pub index_count: u32,
    /// Handle of material, which is attached to rendered mesh primitive
    pub material: AssetHandle,
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
    pub primitives: Vec<Primitive>,

    #[serde(skip)]
    pub vertex_array: VertexArray,
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
            vertex_array: VertexArray::new(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn cube() -> Mesh {
        Mesh::new(
            &[
                Vertex { position: [-0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
                Vertex { position: [-0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
                Vertex { position: [0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

                Vertex { position: [-0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
                Vertex { position: [-0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
                Vertex { position: [0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

                Vertex { position: [0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
                Vertex { position: [0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
                Vertex { position: [0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

                Vertex { position: [-0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
                Vertex { position: [-0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
                Vertex { position: [-0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
                Vertex { position: [-0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

                Vertex { position: [-0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
                Vertex { position: [-0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
                Vertex { position: [0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

                Vertex { position: [-0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
                Vertex { position: [-0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
                Vertex { position: [0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },
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
        set_vertex_attribute!(vertex_array, position_attribute, Vertex::position);
        set_vertex_attribute!(vertex_array, normal_attribute, Vertex::normal);
        set_vertex_attribute!(vertex_array, texcoord_attribute, Vertex::texcoord);
    }

    pub fn update_vertices(&self){     
        self.vertex_array.bind();

        if let (Some(ref vertex_buffer), Some(ref index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
            vertex_buffer.fill(&self.vertex_data);
            index_buffer.fill(&self.index_data);
        }
    }

    pub fn draw(&self, _renderer: &Renderer) {

    }
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Mesh {
            vertex_data: self.vertex_data.clone(),
            index_data: self.index_data.clone(),
            primitives: self.primitives.clone(),
            vertex_array: VertexArray::default(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}
