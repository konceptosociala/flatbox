use std::path::Path;

use gl::types::{GLuint, GLenum};
use image::{ImageError, EncodableLayout};

use crate::macros::gluint_wrapper;

gluint_wrapper! {
    wrapper: Filter,
    variants: [
        Linear,
        Nearest
    ]
}

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new<P: AsRef<Path>>(path: P, filter: Filter) -> Result<Texture, ImageError> {
        unsafe { Texture::new_internal(path, filter.into()) }
    }

    pub fn activate(&self, unit: GLuint) {
        unsafe { gl::ActiveTexture(unit); }
        self.bind();
    }

    pub fn bind(&self){
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
    }

    unsafe fn new_internal<P: AsRef<Path>>(path: P, filter: GLenum) -> Result<Texture, ImageError> {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);

        let texture = Texture { id };
        texture.bind();

        let img = image::open(path)?.into_rgba8();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
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
