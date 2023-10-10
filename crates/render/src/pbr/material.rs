use flatbox_core::math::glm;

use crate::hal::shader::GraphicsPipeline;

pub trait Material: Send + Sync + 'static {
    fn vertex_shader() -> &'static str;

    fn fragment_shader() -> &'static str;

    fn setup_pipeline(&self, pipeline: &GraphicsPipeline);

    fn process_pipeline(&self, pipeline: &GraphicsPipeline);
}

#[repr(C)]
pub struct DefaultMaterial {
    pub color: glm::Vec3,
}

impl Material for DefaultMaterial {
    fn vertex_shader() -> &'static str {
        include_str!("../shaders/defaultmat.vs")
    }

    fn fragment_shader() -> &'static str {
        include_str!("../shaders/defaultmat.fs")
    }

    fn setup_pipeline(&self, pipeline: &GraphicsPipeline) {
        pipeline.set_vec3("color", &self.color);
    }

    fn process_pipeline(&self, _pipeline: &GraphicsPipeline) {
        
    }
}