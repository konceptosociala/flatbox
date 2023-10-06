use std::{collections::HashMap, any::type_name};
use std::any::TypeId;

use flatbox_core::{
    logger::error,
    math::glm,
};

use crate::{
    hal::{
        shader::{GraphicsPipeline, Shader, ShaderType},
        GlInitFunction,
    },
    pbr::material::Material,
};

pub type GraphicsPipelines = HashMap<TypeId, GraphicsPipeline>;

pub struct Renderer {
    pub graphics_pipelines: GraphicsPipelines,
    pub clear_color: glm::Vec3,
}

impl Renderer {
    pub fn init<F: GlInitFunction>(mut init_function: F) -> Renderer {
        gl::load_with(|ptr| init_function(ptr) );

        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);  
        }

        Renderer {
            graphics_pipelines: GraphicsPipelines::new(),
            clear_color: glm::vec3(0.1, 0.1, 0.1),
        }
    }

    pub fn bind_material<M: Material>(&mut self) {
        let vertex_shader = Shader::new_from_source(M::vertex_shader(), ShaderType::VertexShader)
            .expect("Cannot compile vertex shader");

        let fragment_shader = Shader::new_from_source(M::fragment_shader(), ShaderType::FragmentShader)
            .expect("Cannot compile fragment shader");

        let material_type = TypeId::of::<M>();
        let pipeline = GraphicsPipeline::new(&[vertex_shader, fragment_shader]).expect("Cannot initialize graphics pipeline");

        if self.graphics_pipelines.contains_key(&material_type) {
            error!("Material type `{}` is already bound", type_name::<M>());
        } else {
            self.graphics_pipelines.insert(material_type, pipeline);
        }
    }

    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(self.clear_color.x, self.clear_color.y, self.clear_color.z, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn render(
        &self,
    ){
        unsafe {
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
        }
    }

    pub fn push_uniform_matrix(&self, location: i32, matrix: &glm::Mat4) {
        unsafe { gl::UniformMatrix4fv(location as i32, 1, gl::FALSE, glm::value_ptr(&matrix).as_ptr()); }
    }
}