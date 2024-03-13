use std::path::Path;

use flatbox_assets::{
    manager::Asset,
    typetag,
};
use gl::types::GLuint;
use image::{EncodableLayout, ImageBuffer, Rgba};
use serde::{Serialize, Deserialize};

use crate::{
    macros::glenum_wrapper, 
    error::RenderError
};

glenum_wrapper! {
    wrapper: Filter,
    variants: [
        Linear,
        Nearest
    ]
}

glenum_wrapper! {
    wrapper: WrapMode,
    variants: [
        Repeat,
        ClampToEdge,
        MirroredRepeat
    ]
}

glenum_wrapper! {
    wrapper: ColorMode,
    variants: [
        Srgb8Alpha8,
        Rgba
    ]
}

glenum_wrapper! {
    wrapper: Order,
    variants: [
        Texture0, Texture1, Texture2, Texture3,
        Texture4, Texture5, Texture6, Texture7,
        Texture8, Texture9, Texture10, Texture11,
        Texture12, Texture13, Texture14, Texture15,
        Texture16, Texture17, Texture18, Texture19,
        Texture20, Texture21, Texture22, Texture23,
        Texture24, Texture25, Texture26, Texture27,
        Texture28, Texture29, Texture30, Texture31
    ]
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ImageType {
    Image2D,
    SubImage2D([usize; 2]),
}

pub struct TextureDescriptor {
    pub filter: Filter,
    pub wrap_mode: WrapMode,
    pub color_mode: ColorMode,
    pub image_type: ImageType,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        TextureDescriptor {
            filter: Filter::Linear,
            wrap_mode: WrapMode::Repeat,
            color_mode: ColorMode::Rgba,
            image_type: ImageType::Image2D,
        }
    }
}

// FIXME: texture serde, clone, debug
#[derive(Clone, Debug)]
pub struct Texture {
    id: GLuint,
}

impl Serialize for Texture {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        unimplemented!("serialize texture");
    }
}

impl<'de> Deserialize<'de> for Texture {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        unimplemented!("serialize texture");
    }
}

#[typetag::serde]
impl Asset for Texture {}

impl Texture {
    pub fn new<P: AsRef<Path>>(path: P, descr: Option<TextureDescriptor>) -> Result<Texture, RenderError> {
        let img = image::open(path)?.into_rgba8();
        Texture::new_from_raw(img.as_bytes(), img.width(), img.height(), descr)
    }

    pub fn new_from_raw(
        buf: &[u8], 
        width: u32, 
        height: u32, 
        descr: Option<TextureDescriptor>,
    ) -> Result<Texture, RenderError> {
        unsafe { Texture::new_internal(buf, width, height, descr) }
    }

    pub fn activate(&self, order: Order) {
        unsafe { gl::ActiveTexture(order as u32); }
        self.bind();
    }

    pub fn bind(&self){
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
    }

    unsafe fn new_internal(
        buf: &[u8], 
        width: u32, 
        height: u32, 
        descr: Option<TextureDescriptor>,
    ) -> Result<Texture, RenderError> {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);

        let texture = Texture { id };
        texture.bind();

        let descr = descr.unwrap_or_default();

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, descr.filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, descr.filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, descr.wrap_mode as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, descr.wrap_mode as i32);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

        match descr.image_type {
            ImageType::Image2D => gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                descr.color_mode as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_ptr() as *const _,
            ),
            ImageType::SubImage2D([x, y]) => gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x as _,
                y as _,
                width as _,
                height as _,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_ptr() as *const _,
            )
        };

        Ok(texture)
    }
}

impl Default for Texture {
    fn default() -> Self {
        let img = ImageBuffer::from_fn(16, 16, |_, _| Rgba::<u8>([255, 255, 255, 255])).into_raw();

        Texture::new_from_raw(&img, 16, 16, Some(TextureDescriptor {
            filter: Filter::Nearest,
            ..Default::default()
        })).unwrap()
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, [self.id].as_ptr()); }
    }
}

pub fn load_image_from_memory(buf: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    match image::load_from_memory(buf) {
        Ok(img) => {
            let img = img.into_rgba8();
            let width = img.width();
            let height = img.height();
            
            Some((img.into_raw(), width, height))
        },
        _ => None,
    }
}

#[macro_export]
macro_rules! include_texture {
    ($file:expr $(,)?) => {
        {
            let buf = include_bytes!($file);
            let img = $crate::pbr::texture::load_image_from_memory(buf)
                .expect(format!("Cannot load `{}`", $file).as_str());
            
            Texture::new_from_raw(&img.0, img.1, img.2, None)
                .expect(format!("Cannot create texture from `{}`", $file).as_str())
        }
    };
}