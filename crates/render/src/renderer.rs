use std::collections::hash_map::{HashMap, Entry};
use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use flatbox_core::{
    logger::{warn, error},
    math::transform::Transform,
};
use pretty_type_name::pretty_type_name;

#[cfg(feature = "context")]
use crate::context::Context;
use crate::glenum_wrapper;
use crate::pbr::texture::Order;
use crate::{
    error::RenderError,
    hal::shader::{GraphicsPipeline, Shader, ShaderType},
    pbr::{
        material::Material,
        model::Model,
        camera::Camera,
    },
};

#[allow(unused_imports)]
use crate::hal::buffer::VertexArray;

glenum_wrapper! {
    wrapper: Capability,
    variants: [
        ScissorTest,
        CullFace,
        DepthTest,
        Blend
    ]
}

glenum_wrapper! {
    wrapper: ColorBlendEquation,
    variants: [
        FuncAdd,
        FuncSubtract,
        FuncReverseSubtract
    ]
}

glenum_wrapper! {
    wrapper: ColorBlendMode,
    variants: [
        Zero,
        One,
        SrcColor,
        OneMinusSrcColor,
        SrcAlpha,
        OneMinusSrcAlpha,
        DstAlpha,
        OneMinusDstAlpha,
        DstColor,
        OneMinusDstColor,
        SrcAlphaSaturate,
        ConstantColor,
        OneMinusConstantColor,
        ConstantAlpha,
        OneMinusConstantAlpha
    ]
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct WindowExtent {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WindowExtent {
    pub fn new(width: f32, height: f32) -> WindowExtent {
        WindowExtent { x: 0.0, y: 0.0, width, height }
    }

    pub fn to_aspect(&self) -> f32 {
        self.width / self.height
    }
}

impl From<WindowExtent> for [u32; 2] {
    fn from(e: WindowExtent) -> Self {
        [e.width as u32, e.height as u32]
    }
}

pub type GraphicsPipelines = HashMap<TypeId, GraphicsPipeline>;

pub struct Renderer {
    graphics_pipelines: GraphicsPipelines,
    extent: WindowExtent,
    commands_history: RenderCommandsHistory,
}

#[cfg(not(feature = "context"))]
use crate::hal::GlInitFunction;

impl Renderer {
    #[cfg(not(feature = "context"))]
    pub fn init<F: GlInitFunction>(init_function: F) -> Renderer {
        gl::load_with(init_function);

        Renderer {
            graphics_pipelines: GraphicsPipelines::new(),
            extent: WindowExtent::new(800.0, 600.0),
            commands_history: RenderCommandsHistory::new(50),
        }
    }

    #[cfg(feature = "context")]
    pub fn init(context: &Context) -> Result<Renderer, RenderError> {
        gl::load_with(|addr| context.get_proc_address(addr));

        Ok(Renderer {
            graphics_pipelines: GraphicsPipelines::new(),
            extent: WindowExtent::new(800.0, 600.0),
            commands_history: RenderCommandsHistory::new(50),
        })
    }

    pub fn extent(&self) -> WindowExtent {
        self.extent
    }

    pub fn set_extent(&mut self, extent: WindowExtent) {
        self.extent = extent;
        unsafe { gl::Viewport(
            self.extent.x as i32, 
            self.extent.y as i32, 
            self.extent.width as i32, 
            self.extent.height as i32,
        ); }
    }

    pub fn get_pipeline<M: Material>(&self) -> Result<&GraphicsPipeline, RenderError> {
        self.graphics_pipelines.get(&TypeId::of::<M>()).ok_or(RenderError::MaterialNotBound(pretty_type_name::<M>().to_string()))
    }

    pub fn bind_material<M: Material>(&mut self) {
        let material_type = TypeId::of::<M>();
        
        if let Entry::Vacant(e) = self.graphics_pipelines.entry(material_type) {
            let vertex_shader = Shader::new_from_source(M::vertex_shader(), ShaderType::VertexShader)
                .expect("Cannot compile vertex shader");

            let fragment_shader = Shader::new_from_source(M::fragment_shader(), ShaderType::FragmentShader)
                .expect("Cannot compile fragment shader");

            let pipeline = GraphicsPipeline::new(&[vertex_shader, fragment_shader]).expect("Cannot initialize graphics pipeline");
            e.insert(pipeline);
        } else {
            error!("Material type `{}` is already bound", pretty_type_name::<M>());
        }
    }

    pub fn execute(&mut self, command: &mut dyn RenderCommand) -> Result<(), RenderError> {
        self.commands_history.push(command);
        command.execute(self)
    }

    pub fn history(&self) -> &RenderCommandsHistory {
        &self.commands_history
    }
}

#[derive(Clone)]
pub struct RenderCommandsHistory{
    cache: Vec<String>,
    max_capacity: usize,
}

impl RenderCommandsHistory {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            cache: Vec::new(),
            max_capacity,
        }
    }

    pub fn push(&mut self, command: &mut dyn RenderCommand) {
        if self.cache.len() >= self.max_capacity {
            self.cache.remove(0);
        }
        self.cache.push(command.name());
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.cache.get(index).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Debug for RenderCommandsHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(&self.cache)
            .finish()
    }
}

pub trait RenderCommand {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError>;

    fn name(&self) -> String { pretty_type_name::<Self>() }
}

pub struct ClearCommand(pub f32, pub f32, pub f32);

impl RenderCommand for ClearCommand {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); }

        renderer.execute(&mut EnableCommand(Capability::Blend))?;
        renderer.execute(&mut EnableCommand(Capability::DepthTest))?;

        unsafe {
            gl::ClearColor(self.0, self.1, self.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        Ok(())
    }
}

pub struct EnableCommand(pub Capability);

impl RenderCommand for EnableCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::Enable(self.0 as u32); }
        Ok(())
    }
}

pub struct DisableCommand(pub Capability);

impl RenderCommand for DisableCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::Disable(self.0 as u32); }
        Ok(())
    }
}

pub struct BlendEquationSeparateCommand(pub ColorBlendEquation, pub ColorBlendEquation);

impl RenderCommand for BlendEquationSeparateCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::BlendEquationSeparate(self.0 as u32, self.1 as u32); }
        Ok(())
    }
}

pub struct BlendFuncSeparateCommand(pub ColorBlendMode, pub ColorBlendMode, pub ColorBlendMode, pub ColorBlendMode);

impl RenderCommand for BlendFuncSeparateCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::BlendFuncSeparate(
            self.0 as u32,
            self.1 as u32,
            self.2 as u32,
            self.3 as u32,
        ); }

        Ok(())
    }
}

pub struct ScissorCommand(pub WindowExtent);

impl RenderCommand for ScissorCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::Scissor(
            self.0.x as i32,
            self.0.y as i32,
            self.0.width as i32,
            self.0.height as i32,
        ); }
        Ok(())
    }
}

pub struct ColorMaskCommand(pub bool, pub bool, pub bool, pub bool);

impl RenderCommand for ColorMaskCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::ColorMask(
            self.0 as u8, 
            self.1 as u8, 
            self.2 as u8, 
            self.3 as u8
        ); }
        Ok(())
    }
}

pub struct ActivateTextureRawCommand(Order);

impl ActivateTextureRawCommand {
    ///
    /// # Safety
    /// [`GraphicsPipeline`]'s sampler with the given order must be set via [`GraphicsPipeline::set_int`] method
    pub unsafe fn new(order: Order) -> Self {
        ActivateTextureRawCommand(order)
    } 
}

impl RenderCommand for ActivateTextureRawCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::ActiveTexture(self.0 as u32); }
        Ok(())
    }
}

pub struct DrawTrianglesCommand(usize);

impl DrawTrianglesCommand {
    ///
    /// # Safety
    /// A valid [`VertexArray`] has to be bound
    /// Valid index and vertex buffers have to be bound
    pub unsafe fn new(indices_count: usize) -> Self {
        DrawTrianglesCommand(indices_count)
    }
}

impl RenderCommand for DrawTrianglesCommand {
    fn execute(&mut self, _: &mut Renderer) -> Result<(), RenderError> {
        unsafe { gl::DrawElements(
            gl::TRIANGLES, 
            self.0 as i32, 
            gl::UNSIGNED_INT, 
            std::ptr::null()
        ); }
        Ok(())
    }
} 

#[derive(Debug)]
pub struct RenderCameraCommand<'a, M: Material> {
    camera: &'a mut Camera,
    transform: &'a Transform,
    __phantom_data: PhantomData<M>,
}

impl<'a, M: Material> RenderCameraCommand<'a, M> {
    pub fn new(camera: &'a mut Camera, transform: &'a Transform) -> RenderCameraCommand<'a, M> {
        Self { camera, transform, __phantom_data: PhantomData }
    }
}

impl<'a, M: Material> RenderCommand for RenderCameraCommand<'a, M> {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        let pipeline = renderer.get_pipeline::<M>()?;

        if !self.camera.is_active() {
            warn!("Camera being rendered is not active");
        }

        self.camera.set_aspect(renderer.extent().to_aspect());
        self.camera.update_buffer(pipeline, self.transform);
                
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

        if mesh.prepared { return Ok(()); }

        println!("not prepaired");

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

        self.material.setup_pipeline(pipeline);
        
        let (model, inversed) = self.transform.to_matrices();
        
        pipeline.apply();
        pipeline.set_mat4("model", &model);
        pipeline.set_mat4("inversed", &inversed);
    
        mesh.vertex_array.bind();

        unsafe { renderer.execute(&mut DrawTrianglesCommand::new(mesh.index_data.len()))?; }

        Ok(())
    }
}