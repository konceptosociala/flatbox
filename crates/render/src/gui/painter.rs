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

use crate::{
    macros::set_vertex_attribute,
    hal::{
        shader::{GraphicsPipeline, Shader, ShaderType}, 
        buffer::{Buffer, BufferTarget, BufferUsage, VertexArray, AttributeType}
    }, 
    error::RenderError, 
    pbr::texture::{Filter, Texture}
};

const VERT_SRC: &str = include_str!("../shaders/egui.vs");
const FRAG_SRC: &str = include_str!("../shaders/egui.fs");

impl From<TextureFilter> for Filter {
    fn from(f: TextureFilter) -> Self {
        match f {
            TextureFilter::Linear => Filter::Linear,
            TextureFilter::Nearest => Filter::Nearest,
        }
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

impl Painter {
    pub fn new() -> Result<Painter, RenderError> {
        let max_texture_side = 4096;

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
            max_texture_side,
            pipeline,
            vertex_array,
            vertex_buffer,
            index_buffer,
            textures: Default::default(),
            next_native_tex_id: 1 << 32,
            textures_to_destroy: Vec::new(),
        })
    }

    pub fn max_texture_side(&self) -> usize {
        self.max_texture_side
    }

    unsafe fn prepare_painting(
        &mut self,
        [width_in_pixels, height_in_pixels]: [u32; 2],
        pixels_per_point: f32,
    ) -> (u32, u32) {
        gl::Enable(gl::SCISSOR_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
        gl::Enable(gl::BLEND);
        gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
        gl::BlendFuncSeparate(
            gl::ONE,
            gl::ONE_MINUS_SRC_ALPHA,
            gl::ONE_MINUS_DST_ALPHA,
            gl::ONE,
        );

        let width_in_points = width_in_pixels as f32 / pixels_per_point;
        let height_in_points = height_in_pixels as f32 / pixels_per_point;

        gl::Viewport(0, 0, width_in_pixels as i32, height_in_pixels as i32);
        self.pipeline.apply();
        self.pipeline.set_vec2("u_screen_size", &glm::vec2(width_in_points, height_in_points));
        self.pipeline.set_int("u_sampler", 0);

        gl::ActiveTexture(gl::TEXTURE0); // FIXME: Activate texture

        self.vertex_array.bind();
        self.index_buffer.bind();

        (width_in_pixels, height_in_pixels)
    }

    /// You are expected to have cleared the color buffer before calling this.
    pub fn paint_and_update_textures(
        &mut self,
        screen_size_px: [u32; 2],
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.set_texture(*id, image_delta);
        }

        self.paint_primitives(screen_size_px, pixels_per_point, clipped_primitives);

        for &id in &textures_delta.free {
            self.free_texture(id);
        }
    }

    pub fn paint_primitives(
        &mut self,
        screen_size_px: [u32; 2],
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
    ) {
        let size_in_pixels = unsafe { self.prepare_painting(screen_size_px, pixels_per_point) };

        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in clipped_primitives
        {
            set_clip_rect(size_in_pixels, pixels_per_point, *clip_rect);

            match primitive {
                Primitive::Mesh(mesh) => {
                    self.paint_mesh(mesh);
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

                        unsafe {
                            gl::Viewport(
                                rect_min_x,
                                size_in_pixels.1 as i32 - rect_max_y,
                                rect_max_x - rect_min_x,
                                rect_max_y - rect_min_y,
                            );
                        }

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

                        // Restore state:
                        unsafe { self.prepare_painting(screen_size_px, pixels_per_point) };
                    }
                }
            }
        }

        unsafe {
            self.vertex_array.unbind();
            self.index_buffer.unbind();
            gl::Disable(gl::SCISSOR_TEST);
        }
    }

    #[inline(never)]
    fn paint_mesh(&mut self, mesh: &Mesh) {
        debug_assert!(mesh.is_valid());
        if let Some(texture) = self.texture(mesh.texture_id) {
            self.vertex_buffer.fill(&mesh.vertices);
            self.index_buffer.fill(&mesh.indices);
            texture.bind();

            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
        } else {
            warn!("Failed to find texture {:?}", mesh.texture_id);
        }
    }

    pub fn set_texture(&mut self, tex_id: egui::TextureId, delta: &egui::epaint::ImageDelta) {
        let texture = self.textures.entry(tex_id).or_insert_with(|| unsafe {
            let mut handle = 0;
            gl::GenTextures(1, &mut handle);
            Texture::from_raw(handle)
        });
        
        texture.bind();

        match &delta.image {
            egui::ImageData::Color(image) => {
                assert_eq!(
                    image.width() * image.height(),
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );

                let data: &[u8] = bytemuck::cast_slice(image.pixels.as_ref());

                self.upload_texture_srgb(delta.pos, image.size, delta.filter, data);
            }
            egui::ImageData::Font(image) => {
                assert_eq!(
                    image.width() * image.height(),
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );

                let gamma = 1.0;
                let data: Vec<u8> = image
                    .srgba_pixels(gamma)
                    .flat_map(|a| a.to_array())
                    .collect();

                self.upload_texture_srgb(delta.pos, image.size, delta.filter, &data);
            }
        };
    }

    fn upload_texture_srgb(
        &mut self,
        pos: Option<[usize; 2]>,
        [w, h]: [usize; 2],
        texture_filter: TextureFilter,
        data: &[u8],
    ) {
        assert_eq!(data.len(), w * h * 4);
        assert!(
            w <= self.max_texture_side && h <= self.max_texture_side,
            "Got a texture image of size {}x{}, but the maximum supported texture side is only {}",
            w,
            h,
            self.max_texture_side
        );

        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                Filter::from(texture_filter) as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                Filter::from(texture_filter) as i32,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            let (internal_format, src_format) = (gl::SRGB8_ALPHA8, gl::RGBA);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            let level = 0;
            if let Some([x, y]) = pos {
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    level,
                    x as _,
                    y as _,
                    w as _,
                    h as _,
                    src_format,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr().cast(),
                );
            } else {
                let border = 0;
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    level,
                    internal_format as _,
                    w as _,
                    h as _,
                    border,
                    src_format,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr().cast(),
                );
            }
        }
    }

    pub fn free_texture(&mut self, tex_id: TextureId) {
        if let Some(old_tex) = self.textures.remove(&tex_id) {
            drop(old_tex);
        }
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
    size_in_pixels: (u32, u32), 
    pixels_per_point: f32, 
    clip_rect: Rect,
){
    // Transform clip rect to physical pixels:
    let clip_min_x = pixels_per_point * clip_rect.min.x;
    let clip_min_y = pixels_per_point * clip_rect.min.y;
    let clip_max_x = pixels_per_point * clip_rect.max.x;
    let clip_max_y = pixels_per_point * clip_rect.max.y;

    // Round to integer:
    let clip_min_x = clip_min_x.round() as i32;
    let clip_min_y = clip_min_y.round() as i32;
    let clip_max_x = clip_max_x.round() as i32;
    let clip_max_y = clip_max_y.round() as i32;

    // Clamp:
    let clip_min_x = clip_min_x.clamp(0, size_in_pixels.0 as i32);
    let clip_min_y = clip_min_y.clamp(0, size_in_pixels.1 as i32);
    let clip_max_x = clip_max_x.clamp(clip_min_x, size_in_pixels.0 as i32);
    let clip_max_y = clip_max_y.clamp(clip_min_y, size_in_pixels.1 as i32);

    unsafe {
        gl::Scissor(
            clip_min_x,
            size_in_pixels.1 as i32 - clip_max_y,
            clip_max_x - clip_min_x,
            clip_max_y - clip_min_y,
        );
    }
}

