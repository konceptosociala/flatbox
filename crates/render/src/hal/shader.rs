use std::fs::read_to_string;
use std::path::Path;
use std::ptr;
use std::string::FromUtf8Error;
use std::ffi::{
    CString,
    NulError,
};

use thiserror::Error;
use gl::types::{GLuint, GLint};

use crate::macros::*;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Shader compilation error")]
    CompilationError(String),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Shader linking error")]
    LinkingError(String),
    #[error("Null byte error")]
    NullByteError(#[from] NulError),
    #[error("Can't convert raw pointer to UTF-8 string")]
    Utf8Error(#[from] FromUtf8Error),
}

gluint_wrapper! {
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
        unsafe { Shader::new_internal(source_code, shader_type.into()) }
    }

    unsafe fn new_internal(source_code: &str, shader_type: GLuint) -> Result<Shader, ShaderError> {
        let source_code = CString::new(source_code)?;
        let shader = Shader {
            id: gl::CreateShader(shader_type.into()),
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

pub struct ShaderProgram {
    id: GLuint,
}

impl ShaderProgram {
    pub fn new(shaders: &[Shader]) -> Result<ShaderProgram, ShaderError> {
        unsafe { ShaderProgram::new_internal(shaders) }
    }

    pub fn set_int_uniform(&self, name: &str, value: i32) -> Result<(), ShaderError> {
        self.apply();
        
        let uniform = CString::new(name)?;
        let location = unsafe { gl::GetUniformLocation(self.id, uniform.as_ptr()) };
        unsafe { gl::Uniform1i(location, value); }
        
        Ok(())
    }

    pub fn apply(&self){
        unsafe { gl::UseProgram(self.id); }
    }

    pub fn get_attribute_location(
        &self,
        attribute: &str,
    ) -> Result<u32, ShaderError> {
        let attribute = CString::new(attribute)?;
        Ok(unsafe { gl::GetAttribLocation(self.id, attribute.as_ptr()) as GLuint })
    }

    unsafe fn new_internal(shaders: &[Shader]) -> Result<ShaderProgram, ShaderError> {
        let program = ShaderProgram {
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