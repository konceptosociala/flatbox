use gl::types::{GLuint, GLsizeiptr, GLint};

use crate::macros::gluint_wrapper;

gluint_wrapper! {
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

gluint_wrapper! {
    wrapper: BufferUsage,
    variants: [StreamDraw, StaticDraw, DynamicDraw]
}

pub struct Buffer {
    id: GLuint,
    target: GLuint,
    usage: GLuint,
}

impl Buffer {
    pub fn new(target: BufferTarget, usage: BufferUsage) -> Buffer {
        unsafe { Buffer::new_internal(target.into(), usage.into()) }
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

    unsafe fn new_internal(target: GLuint, usage: GLuint) -> Buffer {
        let mut id: GLuint = 0;
        gl::GenBuffers(1, &mut id);

        Buffer { id, target, usage }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, [self.id].as_ptr()) }
    }
}

pub struct VertexArray {
    id: GLuint,
}

impl VertexArray {
    pub fn new() -> VertexArray {
        unsafe { VertexArray::new_internal() }
    }

    pub fn bind(&self){
        unsafe { gl::BindVertexArray(self.id); }
    }

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

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, [self.id].as_ptr()) }
    }
}