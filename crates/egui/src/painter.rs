use std::collections::HashMap;
use egui::{
    emath::Rect,
    epaint::{Mesh, PaintCallbackInfo, Primitive, Vertex}, 
    TextureFilter, TextureId,
};
use flatbox_core::{
    logger::warn,
    math::glm,
};
use flatbox_render::{
    macros::set_vertex_attribute,
    hal::{
        shader::{GraphicsPipeline, Shader, ShaderType}, 
        buffer::{Buffer, BufferTarget, BufferUsage, VertexArray, AttributeType}
    }, 
    error::RenderError, 
    pbr::texture::{Filter, Texture, TextureDescriptor, WrapMode, ColorMode, ImageType, Order}, renderer::{Renderer, Capability, WindowExtent, EnableCommand, DisableCommand, ColorMaskCommand, BlendEquationSeparateCommand, ColorBlendMode, BlendFuncSeparateCommand, ColorBlendEquation, ScissorCommand, ActivateTextureRawCommand, DrawTrianglesCommand}
};

const VERT_SRC: &str = include_str!("shaders/egui.vs");
const FRAG_SRC: &str = include_str!("shaders/egui.fs");

pub trait ToNativeFilter {
    fn to_native(&self) -> Filter;
}

impl ToNativeFilter for TextureFilter {
    fn to_native(&self) -> Filter {
        match self {
            TextureFilter::Linear => Filter::Linear,
            TextureFilter::Nearest => Filter::Nearest,
        }
    }
}

pub struct CallbackFn {
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(PaintCallbackInfo, &Painter) + Sync + Send>,
}

impl CallbackFn {
    pub fn new<F: Fn(PaintCallbackInfo, &Painter) + Sync + Send + 'static>(callback: F) -> Self {
        let f = Box::new(callback);
        CallbackFn { f }
    }
}

pub struct Painter {
    max_texture_side: usize,
    pipeline: GraphicsPipeline,
    vertex_array: VertexArray,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    textures: HashMap<TextureId, Texture>,
    next_native_tex_id: u64,
    textures_to_destroy: Vec<Texture>,
}

impl Painter {
    pub fn new() -> Result<Painter, RenderError> {
        let vertex_shader = Shader::new_from_source(VERT_SRC, ShaderType::VertexShader)?;
        let fragment_shader = Shader::new_from_source(FRAG_SRC, ShaderType::FragmentShader)?;
        let pipeline = GraphicsPipeline::new(&[vertex_shader, fragment_shader])?;

        let vertex_array = VertexArray::new();
        let index_buffer = Buffer::new(BufferTarget::ElementArrayBuffer, BufferUsage::StreamDraw);
        let vertex_buffer = Buffer::new(BufferTarget::ArrayBuffer, BufferUsage::StreamDraw);
        
        vertex_buffer.bind();

        let a_pos_loc = pipeline.get_attribute_location("a_pos");
        let a_tc_loc = pipeline.get_attribute_location("a_tc");
        let a_srgba_loc = pipeline.get_attribute_location("a_srgba");
        
        set_vertex_attribute!(vertex_array, a_pos_loc, Vertex::pos, AttributeType::Float);
        set_vertex_attribute!(vertex_array, a_tc_loc, Vertex::uv, AttributeType::Float);
        set_vertex_attribute!(vertex_array, a_srgba_loc, Vertex::color, AttributeType::UnsignedByte);

        Ok(Painter {
            max_texture_side: 4096,
            pipeline,
            vertex_array,
            vertex_buffer,
            index_buffer,
            textures: HashMap::new(),
            next_native_tex_id: 1 << 32,
            textures_to_destroy: Vec::new(),
        })
    }

    pub fn max_texture_side(&self) -> usize {
        self.max_texture_side
    }

    fn prepare_painting(
        &mut self,
        renderer: &mut Renderer,
        [width_in_pixels, height_in_pixels]: [u32; 2],
        pixels_per_point: f32,
    ) -> Result<(u32, u32), RenderError> {
        renderer.execute(&mut EnableCommand(Capability::ScissorTest))?;
        renderer.execute(&mut DisableCommand(Capability::CullFace))?;
        renderer.execute(&mut DisableCommand(Capability::DepthTest))?;
        renderer.execute(&mut ColorMaskCommand(true, true, true, true))?;
        renderer.execute(&mut EnableCommand(Capability::Blend))?;
        renderer.execute(&mut BlendEquationSeparateCommand(ColorBlendEquation::FuncAdd, ColorBlendEquation::FuncAdd))?;
        renderer.execute(&mut BlendFuncSeparateCommand(
            ColorBlendMode::One,
            ColorBlendMode::OneMinusSrcAlpha,
            ColorBlendMode::OneMinusDstAlpha,
            ColorBlendMode::One,
        ))?;

        let width_in_points = width_in_pixels as f32 / pixels_per_point;
        let height_in_points = height_in_pixels as f32 / pixels_per_point;

        self.pipeline.apply();
        self.pipeline.set_vec2("u_screen_size", &glm::vec2(width_in_points, height_in_points));
        self.pipeline.set_int("u_sampler", 0);

        unsafe { renderer.execute(&mut ActivateTextureRawCommand::new(Order::Texture0))?; }

        self.vertex_array.bind();
        self.index_buffer.bind();

        Ok((width_in_pixels, height_in_pixels))
    }

    pub fn paint_and_update_textures(
        &mut self,
        renderer: &mut Renderer,
        screen_size_px: [u32; 2],
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
    ) -> Result<(), RenderError> {
        for (id, image_delta) in &textures_delta.set {
            self.set_texture(*id, image_delta)?;
        }

        self.paint_primitives(renderer, screen_size_px, pixels_per_point, clipped_primitives)?;

        for &id in &textures_delta.free {
            self.textures.remove(&id);
        }

        Ok(())
    }

    pub fn paint_primitives(
        &mut self,
        renderer: &mut Renderer,
        screen_size_px: [u32; 2],
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
    ) -> Result<(), RenderError> {
        let size_in_pixels = self.prepare_painting(renderer, screen_size_px, pixels_per_point)?;

        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in clipped_primitives
        {
            set_clip_rect(renderer, size_in_pixels, pixels_per_point, *clip_rect)?;

            match primitive {
                Primitive::Mesh(mesh) => {
                    self.paint_mesh(renderer, mesh)?;
                }
                Primitive::Callback(callback) => {
                    if callback.rect.is_positive() {
                        // Transform callback rect to physical pixels:
                        let rect_min_x = pixels_per_point * callback.rect.min.x;
                        let rect_min_y = pixels_per_point * callback.rect.min.y;
                        let rect_max_x = pixels_per_point * callback.rect.max.x;
                        let rect_max_y = pixels_per_point * callback.rect.max.y;

                        let rect_min_x = rect_min_x.round() as i32;
                        let rect_min_y = rect_min_y.round() as i32;
                        let rect_max_x = rect_max_x.round() as i32;
                        let rect_max_y = rect_max_y.round() as i32;

                        renderer.set_extent(WindowExtent {
                            x: rect_min_x as f32,
                            y: (size_in_pixels.1 as i32 - rect_max_y) as f32,
                            width: (rect_max_x - rect_min_x) as f32,
                            height: (rect_max_y - rect_min_y) as f32,
                        });

                        let info = egui::PaintCallbackInfo {
                            viewport: callback.rect,
                            clip_rect: *clip_rect,
                            pixels_per_point,
                            screen_size_px,
                        };

                        if let Some(callback) = callback.callback.downcast_ref::<CallbackFn>() {
                            (callback.f)(info, self);
                        } else {
                            warn!("Warning: Unsupported render callback. Expected egui_gl::CallbackFn");
                        }

                        self.prepare_painting(renderer, screen_size_px, pixels_per_point)?;
                    }
                }
            }
        }

        self.vertex_array.unbind();
        self.index_buffer.unbind();
        renderer.execute(&mut DisableCommand(Capability::ScissorTest))?;

        Ok(())
    }

    #[inline(never)]
    fn paint_mesh(
        &mut self, 
        renderer: &mut Renderer, 
        mesh: &Mesh
    ) -> Result<(), RenderError> {
        debug_assert!(mesh.is_valid());
        if let Some(texture) = self.texture(mesh.texture_id) {
            self.vertex_buffer.fill(&mesh.vertices);
            self.index_buffer.fill(&mesh.indices);
            texture.bind();

            unsafe { renderer.execute(&mut DrawTrianglesCommand::new(mesh.indices.len()))?; }
        } else {
            warn!("Failed to find texture {:?}", mesh.texture_id);
        }

        Ok(())
    }

    pub fn set_texture(
        &mut self, 
        tex_id: egui::TextureId, 
        delta: &egui::epaint::ImageDelta
    ) -> Result<(), RenderError> {
        let texture = match &delta.image {
            egui::ImageData::Color(image) => {
                let (w, h) = (image.width(), image.height());

                let data: &[u8] = bytemuck::cast_slice(image.pixels.as_ref());

                assert_eq!(
                    w * h,
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );
                assert_eq!(data.len(), w * h * 4);
                assert!(
                    w <= self.max_texture_side && h <= self.max_texture_side,
                    "Got a texture image of size {}x{}, but the maximum supported texture side is only {}",
                    w,
                    h,
                    self.max_texture_side
                );

                Texture::new_from_raw(
                    image.width() as u32, 
                    image.height() as u32, 
                    data,
                    Some(TextureDescriptor {
                        filter: delta.filter.to_native(),
                        wrap_mode: WrapMode::ClampToEdge,
                        color_mode: ColorMode::Srgb8Alpha8,
                        image_type: match delta.pos {
                            Some([x, y]) => ImageType::SubImage2D([x, y]),
                            None => ImageType::Image2D,
                        },
                    })
                )?

            }
            egui::ImageData::Font(image) => {
                let (w, h) = (image.width(), image.height());

                let gamma = 1.0;
                let data: Vec<u8> = image
                    .srgba_pixels(gamma)
                    .flat_map(|a| a.to_array())
                    .collect();

                assert_eq!(
                    w * h,
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );
                assert_eq!(data.len(), w * h * 4);
                assert!(
                    w <= self.max_texture_side && h <= self.max_texture_side,
                    "Got a texture image of size {}x{}, but the maximum supported texture side is only {}",
                    w,
                    h,
                    self.max_texture_side
                );

                Texture::new_from_raw(
                    image.width() as u32, 
                    image.height() as u32, 
                    &data,
                    Some(TextureDescriptor {
                        filter: delta.filter.to_native(),
                        wrap_mode: WrapMode::ClampToEdge,
                        color_mode: ColorMode::Srgb8Alpha8,
                        image_type: match delta.pos {
                            Some(coords) => ImageType::SubImage2D(coords),
                            None => ImageType::Image2D,
                        },
                    })
                )?
            }
        };

        self.textures.insert(tex_id, texture);

        Ok(())
    }

    pub fn texture(&self, texture_id: TextureId) -> Option<&Texture> {
        self.textures.get(&texture_id)
    }

    pub fn register_native_texture(&mut self, native: Texture) -> egui::TextureId {
        let id = egui::TextureId::User(self.next_native_tex_id);
        self.next_native_tex_id += 1;
        self.textures.insert(id, native);
        id
    }

    pub fn replace_native_texture(&mut self, id: TextureId, replacing: Texture) {
        if let Some(old_tex) = self.textures.insert(id, replacing) {
            self.textures_to_destroy.push(old_tex);
        }
    }
}

fn set_clip_rect(
    renderer: &mut Renderer,
    size_in_pixels: (u32, u32), 
    pixels_per_point: f32, 
    clip_rect: Rect,
) -> Result<(), RenderError> {
    // Transform clip rect to physical pixels:
    let clip_min_x = pixels_per_point * clip_rect.min.x;
    let clip_min_y = pixels_per_point * clip_rect.min.y;
    let clip_max_x = pixels_per_point * clip_rect.max.x;
    let clip_max_y = pixels_per_point * clip_rect.max.y;

    // Round to integer:
    let clip_min_x = clip_min_x.round();
    let clip_min_y = clip_min_y.round();
    let clip_max_x = clip_max_x.round();
    let clip_max_y = clip_max_y.round();

    // Clamp:
    let clip_min_x = clip_min_x.clamp(0.0, size_in_pixels.0 as f32);
    let clip_min_y = clip_min_y.clamp(0.0, size_in_pixels.1 as f32);
    let clip_max_x = clip_max_x.clamp(clip_min_x, size_in_pixels.0 as f32);
    let clip_max_y = clip_max_y.clamp(clip_min_y, size_in_pixels.1 as f32);

    renderer.execute(&mut ScissorCommand(WindowExtent { 
        x:      clip_min_x, 
        y:      size_in_pixels.1 as f32 - clip_max_y, 
        width:  clip_max_x - clip_min_x, 
        height: clip_max_y - clip_min_y, 
    }))?;

    Ok(())
}

