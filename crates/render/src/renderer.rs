use super::hal::GlInitFunction;

pub struct Renderer {

}

impl Renderer {
    pub fn init<F: GlInitFunction>(mut init_function: F) -> Renderer {
        gl::load_with(|ptr| init_function(ptr) );

        Renderer {}
    }

    pub fn render(&self){
        unsafe {
            gl::ClearColor(1.0, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}