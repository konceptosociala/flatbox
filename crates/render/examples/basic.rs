use std::ptr;

use glutin::{
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    ContextBuilder,
    GlRequest,
    Api, event::{Event, WindowEvent}, dpi::{Size, LogicalSize},
};

use flatbox_render::{
    hal::{
        shader::*,
        buffer::*,
    },
    pbr::texture::{
        Texture,
        Filter,
    },
    macros::*,
    renderer::*,
};

#[repr(C)]
pub struct Vertex {
    position: [f32; 2],
    texcoord: [f32; 2],
}

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-0.5, -0.5], 
        texcoord: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5], 
        texcoord: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5], 
        texcoord: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        texcoord: [0.0, 0.0],
    }
];

const INDICES: [i32; 6] = [
    0, 1, 2,
    2, 3, 0,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(Size::from(LogicalSize::new(800, 800)))
        .with_title("Learn OpenGL with Rust");

    let gl_context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(window, &event_loop)
        .expect("Cannot create windowed context");

    let gl_context = unsafe {
        gl_context
            .make_current()
            .expect("Failed to make context current")
    };

    Renderer::init(|ptr| gl_context.get_proc_address(ptr) as *const _);

    let vertex_shader = Shader::new_from_source(include_str!("../src/shaders/basic.vs"), ShaderType::VertexShader)?;
    let fragment_shader = Shader::new_from_source(include_str!("../src/shaders/basic.fs"), ShaderType::FragmentShader)?;
    let program = ShaderProgram::new(&[vertex_shader, fragment_shader])?;
    
    let vertex_array = VertexArray::new();
    vertex_array.bind();

    let vertex_buffer = Buffer::new(BufferTarget::ArrayBuffer, BufferUsage::StaticDraw);
    vertex_buffer.fill(&VERTICES);

    let index_buffer = Buffer::new(BufferTarget::ElementArrayBuffer, BufferUsage::StaticDraw);
    index_buffer.fill(&INDICES);

    let position_attribute = program.get_attribute_location("position")?;
    let texcoord_attribute = program.get_attribute_location("coordinate")?;
    set_vertex_attribute!(vertex_array, position_attribute, Vertex::position);
    set_vertex_attribute!(vertex_array, texcoord_attribute, Vertex::texcoord);

    let rust = Texture::new("../../assets/rust.png", Filter::Linear)?;
    program.set_int_uniform("rustTexture", 0)?;

    let wall = Texture::new("../../assets/wall.jpg", Filter::Linear)?;
    program.set_int_uniform("wallTexture", 1)?;

    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
    }

    event_loop.run(move |event, _, control_flow|{
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => gl_context.resize(physical_size),
                _ => {},
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(1.0, 0.55, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                rust.activate(gl::TEXTURE0);
                wall.activate(gl::TEXTURE1);
                
                program.apply();
                vertex_array.bind();

                unsafe {
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                }

                gl_context.swap_buffers().unwrap();
            },
            _ => {},
        }
    });
}