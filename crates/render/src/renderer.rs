use std::collections::hash_map::{HashMap, Entry};
use std::any::TypeId;

use flatbox_core::math::transform::Transform;
use flatbox_core::{
    logger::error,
    math::glm,
};
use pretty_type_name::pretty_type_name;

use crate::{
    error::RenderError,
    hal::{
        shader::{GraphicsPipeline, Shader, ShaderType},
        GlInitFunction,
    },
    pbr::{
        material::Material,
        model::Model,
    },
};

pub type GraphicsPipelines = HashMap<TypeId, GraphicsPipeline>;

pub struct Renderer {
    graphics_pipelines: GraphicsPipelines,
    clear_color: glm::Vec3,
}

impl Renderer {
    pub fn init<F: GlInitFunction>(init_function: F) -> Renderer {
        gl::load_with(init_function);

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);  
        }

        Renderer {
            graphics_pipelines: GraphicsPipelines::new(),
            clear_color: glm::vec3(0.1, 0.1, 0.1),
        }
    }

    pub fn set_clear_color(&mut self, color: glm::Vec3) {
        self.clear_color = color;
    }

    pub fn get_pipeline<M: Material>(&self) -> Result<&GraphicsPipeline, RenderError> {
        self.graphics_pipelines.get(&TypeId::of::<M>()).ok_or(RenderError::MaterialNotBound(pretty_type_name::<M>().to_string()))
    }

    pub fn bind_material<M: Material>(&mut self) {
        let vertex_shader = Shader::new_from_source(M::vertex_shader(), ShaderType::VertexShader)
            .expect("Cannot compile vertex shader");

        let fragment_shader = Shader::new_from_source(M::fragment_shader(), ShaderType::FragmentShader)
            .expect("Cannot compile fragment shader");

        let material_type = TypeId::of::<M>();
        let pipeline = GraphicsPipeline::new(&[vertex_shader, fragment_shader]).expect("Cannot initialize graphics pipeline");

        if let Entry::Vacant(e) = self.graphics_pipelines.entry(material_type) {
            e.insert(pipeline);
        } else {
            error!("Material type `{}` is already bound", pretty_type_name::<M>());
        }
    }

    pub fn execute(&mut self, command: &mut dyn RenderCommand) -> Result<(), RenderError> {
        command.execute(self)
    }
}

pub trait RenderCommand {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError>;
}

#[derive(Debug)]
pub struct ClearCommand;

impl RenderCommand for ClearCommand {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        unsafe {
            gl::ClearColor(renderer.clear_color.x, renderer.clear_color.y, renderer.clear_color.z, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PrepareModelCommand<'a, M> {
    model: &'a mut Model,
    material: &'a M,
}

impl<'a, M: Material> PrepareModelCommand<'a, M> {
    pub fn new(model: &'a mut Model, material: &'a M) -> PrepareModelCommand<'a, M> {
        Self { model, material }
    }
}

impl<'a, M: Material> RenderCommand for PrepareModelCommand<'a, M> {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        let Some(ref mut mesh) = self.model.mesh else { return Ok(()) };

        let pipeline = renderer.get_pipeline::<M>()?;
        mesh.setup(pipeline);

        pipeline.apply();
        self.material.setup_pipeline(pipeline);

        mesh.prepared = true;

        Ok(())
    }
}

#[derive(Debug)]
pub struct DrawModelCommand<'a, M> {
    model: &'a Model,
    material: &'a M,
    transform: &'a Transform,
}

impl<'a, M: Material> DrawModelCommand<'a, M> {
    pub fn new(
        model: &'a Model, 
        material: &'a M,
        transform: &'a Transform,
    ) -> DrawModelCommand<'a, M> {
        Self { model, material, transform }
    }
}

impl<'a, M: Material> RenderCommand for DrawModelCommand<'a, M> {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        let Some(ref mesh) = self.model.mesh else { return Ok(()) };

        if !mesh.prepared {
            return Err(RenderError::ModelNotPrepared);
        }

        let pipeline = renderer.get_pipeline::<M>()?;

        self.material.process_pipeline(pipeline);
        
        let model = self.transform.to_matrix();
        let mut view = glm::Mat4::identity();
        view = glm::translate(&view, &glm::vec3(0.0, 0.0, -3.0));
        let projection = glm::perspective(45.0f32.to_radians(), 800.0 / 600.0, 0.1, 100.0);
        
        pipeline.apply();
        pipeline.set_mat4("model", &model);
        pipeline.set_mat4("view", &view);
        pipeline.set_mat4("projection", &projection);
    
        mesh.vertex_array.bind();

        unsafe {
            gl::DrawElements(gl::TRIANGLES, mesh.index_data.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
        }

        Ok(())
    }
}