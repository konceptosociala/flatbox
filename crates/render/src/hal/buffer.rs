use std::fmt::Debug;
use gl::types::{GLuint, GLsizeiptr, GLint};

use crate::macros::glenum_wrapper;

glenum_wrapper! {
    wrapper: BufferTarget,
    variants: [
        ArrayBuffer, AtomicCounterBuffer, CopyReadBuffer,
        CopyWriteBuffer, DispatchIndirectBuffer,
        DrawIndirectBuffer, ElementArrayBuffer,
        PixelPackBuffer, PixelUnpackBuffer, QueryBuffer,
        ShaderStorageBuffer, TextureBuffer,
        TransformFeedbackBuffer, UniformBuffer
    ]
}

glenum_wrapper! {
    wrapper: BufferUsage,
    variants: [StreamDraw, StaticDraw, DynamicDraw]
}

#[readonly::make]
pub struct Buffer {
    id: GLuint,
    target: GLuint,
    usage: GLuint,

    __debug_target: BufferTarget,
    __debug_usage: BufferUsage,
}

impl Buffer {
    pub fn new(target: BufferTarget, usage: BufferUsage) -> Buffer {
        unsafe { Buffer::new_internal(target, usage) }
    }

    pub fn fill<T: Sized>(
        &self,
        data: &[T],
    ){
        self.bind();
        let (_, bytes, _) = unsafe { data.align_to::<u8>() };
        unsafe {
            gl::BufferData(
                self.target,
                bytes.len() as GLsizeiptr,
                bytes.as_ptr() as *const _,
                self.usage,
            );
        }
    }

    fn bind(&self){
        unsafe { gl::BindBuffer(self.target, self.id); }
    }

    unsafe fn new_internal(target: BufferTarget, usage: BufferUsage) -> Buffer {
        let mut id: GLuint = 0;
        gl::GenBuffers(1, &mut id);

        Buffer {
            id, 
            target: target as u32, 
            usage: usage as u32,
            __debug_target: target,
            __debug_usage: usage,
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new(BufferTarget::ArrayBuffer, BufferUsage::StaticDraw)
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("id", &self.id)
            .field("target", &self.__debug_target)
            .field("usage", &self.__debug_usage)
            .finish()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, [self.id].as_ptr()) }
    }
}

#[readonly::make]
pub struct VertexArray {
    id: GLuint,
}

impl VertexArray {
    pub fn new() -> VertexArray {
        VertexArray::default()
    }

    pub fn bind(&self){
        unsafe { gl::BindVertexArray(self.id); }
    }

    /// set attribute
    /// 
    /// ## Safety
    /// 
    pub unsafe fn set_attribute<V: Sized>(
        &self,
        attrib_pos: u32,
        components: i32,
        offset: i32,
    ) {
        self.bind();
        gl::VertexAttribPointer(
            attrib_pos,
            components,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<V>() as GLint,
            offset as *const _,
        );
        gl::EnableVertexAttribArray(attrib_pos);
    }

    unsafe fn new_internal() -> VertexArray {
        let mut id: GLuint = 0;
        gl::GenVertexArrays(1, &mut id);

        VertexArray { id }
    }
}

impl Debug for VertexArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VertexArray")
            .field(&self.id)
            .finish()
    }
}

impl Default for VertexArray {
    fn default() -> Self {
        unsafe { VertexArray::new_internal() }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, [self.id].as_ptr()) }
    }
}