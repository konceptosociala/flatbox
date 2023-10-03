use std::{collections::HashMap, any::TypeId};

use crate::{
    hal::{
        shader::GraphicsPipeline,
        GlInitFunction,
    },
    pbr::material::Material,
};

pub type GraphicsPipelines = HashMap<TypeId, GraphicsPipeline>;

pub struct Renderer {
    graphics_pipelines: GraphicsPipelines,
}

impl Renderer {
    pub fn init<F: GlInitFunction>(mut init_function: F) -> Renderer {
        gl::load_with(|ptr| init_function(ptr) );

        Renderer {
            graphics_pipelines: GraphicsPipelines::new()
        }
    }

    pub fn bind_material<M: Material>(&mut self) {

    }

    pub fn render(&self){
        unsafe {
            gl::ClearColor(1.0, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}