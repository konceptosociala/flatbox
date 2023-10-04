use std::ptr;

use flatbox_core::{
    math::*,
    logger::*,
};

use flatbox_render::{
    hal::shader::*,
    pbr::{
        texture::{
            Texture,
            Filter, Order,
        },
        mesh::Mesh,
    },
    renderer::*, 
    context::*,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    FlatboxLogger::init();

    let context = Context::new(&WindowBuilder {
        title: "Learn OpenGL with Rust",
        width: 800,
        height: 800,
        ..Default::default()
    });

    let _renderer = Renderer::init(|ptr| context.get_proc_address(ptr));

    let program = GraphicsPipeline::new(&[
        Shader::new_from_source(include_str!("../src/shaders/basic.vs"), ShaderType::VertexShader)?,
        Shader::new_from_source(include_str!("../src/shaders/basic.fs"), ShaderType::FragmentShader)?,
    ])?;

    let mut mesh = Mesh::cube();
    mesh.setup(&program);

    let rust = Texture::new("../../assets/rust.png", Filter::Linear)?;
    program.set_int_uniform("rustTexture", 0);

    let wall = Texture::new("../../assets/wall.jpg", Filter::Linear)?;
    program.set_int_uniform("wallTexture", 1);

    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
        gl::Enable(gl::DEPTH_TEST);  
    }

    context.run(move ||{
        unsafe {
            gl::ClearColor(1.0, 0.55, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        rust.activate(Order::Texture0);
        wall.activate(Order::Texture1);
        
        let mut model = glm::Mat4::identity();
        model = glm::rotate(&model, (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis() - 1696149300000) as f32 / 513.0, &glm::vec3(1.0, 1.0, 1.0));
        let mut view = glm::Mat4::identity();
        view = glm::translate(&view, &glm::vec3(0.0, 0.0, -3.0));
        let projection;
        projection = glm::perspective(45.0f32.to_radians(), 800.0 / 600.0, 0.1, 100.0);
        
        program.apply();

        let model_loc = program.get_uniform_location("model");
        let view_loc = program.get_uniform_location("view");
        let projection_loc = program.get_uniform_location("projection");
        unsafe { gl::UniformMatrix4fv(model_loc as i32, 1, gl::FALSE, glm::value_ptr(&model).as_ptr()); }
        unsafe { gl::UniformMatrix4fv(view_loc as i32, 1, gl::FALSE, glm::value_ptr(&view).as_ptr()); }
        unsafe { gl::UniformMatrix4fv(projection_loc as i32, 1, gl::FALSE, glm::value_ptr(&projection).as_ptr()); }

        mesh.vertex_array.bind();

        unsafe {
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, ptr::null());
        }
    });

}