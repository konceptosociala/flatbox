use std::path::Path;

use flatbox_assets::{
    manager::Asset,
    typetag,
};
use gl::types::{GLuint, GLenum};
use image::EncodableLayout;
use serde::{Serialize, Deserialize};

use crate::{macros::glenum_wrapper, error::RenderError};

glenum_wrapper! {
    wrapper: Filter,
    variants: [
        Linear,
        Nearest
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

#[derive(Serialize, Deserialize)]
pub struct Texture {
    // TODO: add texture type and raw image data
    id: GLuint,
}

#[typetag::serde]
impl Asset for Texture {}

impl Texture {
    pub fn new<P: AsRef<Path>>(path: P, filter: Filter) -> Result<Texture, RenderError> {
        unsafe { Texture::new_internal(path, filter as u32) }
    }

    pub fn activate(&self, order: Order) {
        unsafe { gl::ActiveTexture(order as u32); }
        self.bind();
    }

    pub fn bind(&self){
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
    }

    unsafe fn new_internal<P: AsRef<Path>>(path: P, filter: GLenum) -> Result<Texture, RenderError> {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);

        let texture = Texture { id };
        texture.bind();

        let img = image::open(path)?.into_rgba8();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.as_bytes().as_ptr() as *const _,
        );

        Ok(texture)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, [self.id].as_ptr()); }
    }
}
