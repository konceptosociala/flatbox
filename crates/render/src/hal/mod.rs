pub mod buffer;
pub mod shader;

pub trait GlInitFunction: FnMut(&'static str) -> *const std::ffi::c_void {}
impl<F> GlInitFunction for F
where 
    F: FnMut(&'static str) -> *const std::ffi::c_void {}