use std::fs::read_to_string;
use std::path::Path;
use std::ptr;
use std::string::FromUtf8Error;

use thiserror::Error;
use gl::types::{GLuint, GLint};
use flatbox_core::math::glm;

use crate::macros::*;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Shader compilation error")]
    CompilationError(String),
    #[error("Shader I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Shader linking error")]
    LinkingError(String),
    #[error("Can't convert raw pointer to UTF-8 string")]
    Utf8Error(#[from] FromUtf8Error),
}

glenum_wrapper! {
    wrapper: ShaderType,
    variants: [
        VertexShader,
        FragmentShader
    ]
}

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(path: impl AsRef<Path>, shader_type: ShaderType) -> Result<Shader, ShaderError> {
        let source_code = read_to_string(path)?;
        
        Shader::new_from_source(&source_code, shader_type)
    }

    pub fn new_from_source(source_code: &str, shader_type: ShaderType) -> Result<Shader, ShaderError> {
        unsafe { Shader::new_internal(source_code, shader_type as u32) }
    }

    unsafe fn new_internal(source_code: &str, shader_type: GLuint) -> Result<Shader, ShaderError> {
        let source_code = c_string!(source_code);
        let shader = Shader {
            id: gl::CreateShader(shader_type),
        };

        gl::ShaderSource(shader.id, 1, &source_code.as_ptr(), ptr::null());
        gl::CompileShader(shader.id);

        let mut success: GLint = 0;
        gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);
        if success == 1 {
            Ok(shader)
        } else {
            let mut error_log_size: GLint = 0;
            gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut error_log_size);

            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetShaderInfoLog(
                shader.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            );

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;

            Err(ShaderError::CompilationError(log))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

pub struct GraphicsPipeline {
    pub id: GLuint,
}

impl GraphicsPipeline {
    pub fn new(shaders: &[Shader]) -> Result<GraphicsPipeline, ShaderError> {
        unsafe { GraphicsPipeline::new_internal(shaders) }
    }

    pub fn apply(&self){
        unsafe { gl::UseProgram(self.id); }
    }

    pub fn set_bool(&self, name: &str, value: bool) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::Uniform1i(location, value as i32); }
    }

    pub fn set_int(&self, name: &str, value: i32) {
        self.apply();
        let location = self.get_uniform_location(name);
        unsafe { gl::Uniform1i(location, value); }
    }

    pub fn set_float(&self, name: &str, value: f32) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::Uniform1f(location, value); }
    }

    pub fn set_vec2(&self, name: &str, value: &glm::Vec2) {
        let location = self.get_uniform_location(name);
        unsafe { gl::Uniform2fv(location, 1, glm::value_ptr(value).as_ptr()); }
    }

    pub fn set_vec3(&self, name: &str, value: &glm::Vec3) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::Uniform3fv(location, 1, glm::value_ptr(value).as_ptr()); }
    }

    pub fn set_vec4(&self, name: &str, value: &glm::Vec4) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::Uniform4fv(location, 1, glm::value_ptr(value).as_ptr()); }
    }

    pub fn set_mat2(&self, name: &str, value: &glm::Mat2) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::UniformMatrix2fv(location, 1, gl::FALSE, glm::value_ptr(value).as_ptr()); }
    }

    pub fn set_mat3(&self, name: &str, value: &glm::Mat3) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::UniformMatrix3fv(location, 1, gl::FALSE, glm::value_ptr(value).as_ptr()); }
    }

    pub fn set_mat4(&self, name: &str, value: &glm::Mat4) {        
        let location = self.get_uniform_location(name);
        unsafe { gl::UniformMatrix4fv(location, 1, gl::FALSE, glm::value_ptr(value).as_ptr()); }
    }

    pub fn get_attribute_location(&self, attribute: &str) -> u32 {
        let attribute = c_string!(attribute);
        unsafe { gl::GetAttribLocation(self.id, attribute.as_ptr()) as GLuint }
    }

    pub fn get_uniform_location(&self, uniform: &str) -> i32 {
        let uniform = c_string!(uniform);
        unsafe { gl::GetUniformLocation(self.id, uniform.as_ptr()) as GLint }
    }

    unsafe fn new_internal(shaders: &[Shader]) -> Result<GraphicsPipeline, ShaderError> {
        let program = GraphicsPipeline {
            id: gl::CreateProgram()
        };

        for shader in shaders {
            gl::AttachShader(program.id, shader.id);
        }

        gl::LinkProgram(program.id);

        let mut success: GLint = 0;
        gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);

        if success == 1 {
            Ok(program)
        } else {
            let mut error_log_size: GLint = 0;
            gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut error_log_size);

            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetProgramInfoLog(
                program.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            );

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;

            Err(ShaderError::LinkingError(log))
        }
    }
}