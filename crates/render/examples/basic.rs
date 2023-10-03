use std::ptr;

use glutin::{
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    ContextBuilder,
    GlRequest,
    Api, event::{Event, WindowEvent}, dpi::{Size, LogicalSize},
};

use flatbox_core::{
    math::*,
    logger::*,
};

use flatbox_render::{
    hal::{
        shader::*,
        buffer::*,
    },
    pbr::{
        texture::{
            Texture,
            Filter,
        },
        mesh::Vertex,
    },
    macros::*,
    renderer::*,
};

const VERTICES: [Vertex; 24] = [
    Vertex { position: [-0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
    Vertex { position: [-0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
    Vertex { position: [0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
    Vertex { position: [0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

    Vertex { position: [-0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
    Vertex { position: [-0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
    Vertex { position: [0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
    Vertex { position: [0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

    Vertex { position: [0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
    Vertex { position: [0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
    Vertex { position: [0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
    Vertex { position: [0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

    Vertex { position: [-0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
    Vertex { position: [-0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
    Vertex { position: [-0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
    Vertex { position: [-0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

    Vertex { position: [-0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
    Vertex { position: [-0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
    Vertex { position: [0.5,0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
    Vertex { position: [0.5,0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },

    Vertex { position: [-0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 0.0] },
    Vertex { position: [-0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [0.0, 1.0] },
    Vertex { position: [0.5,-0.5,-0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 1.0] },
    Vertex { position: [0.5,-0.5,0.5], normal: [0.0, 0.0, 0.0], texcoord: [1.0, 0.0] },
];

const INDICES: [u32; 36] = [
    0,1,3,
    3,1,2,
    4,5,7,
    7,5,6,
    8,9,11,
    11,9,10,
    12,13,15,
    15,13,14,
    16,17,19,
    19,17,18,
    20,21,23,
    23,21,22
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    FlatboxLogger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(Size::from(LogicalSize::new(800, 800)))
        .with_title("Learn OpenGL with Rust");

    let gl_context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 6)))
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
    let program = GraphicsPipeline::new(&[vertex_shader, fragment_shader])?;

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
        gl::Enable(gl::DEPTH_TEST);  
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
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                rust.activate(gl::TEXTURE0);
                wall.activate(gl::TEXTURE1);
                
                let mut model = glm::Mat4::identity();
                model = glm::rotate(&model, (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis() - 1696149300000) as f32 / 513.0, &glm::vec3(1.0, 1.0, 1.0));
                let mut view = glm::Mat4::identity();
                view = glm::translate(&view, &glm::vec3(0.0, 0.0, -3.0));
                let projection;
                projection = glm::perspective(45.0f32.to_radians(), 800.0 / 600.0, 0.1, 100.0);
                
                program.apply();

                let model_loc = program.get_uniform_location("model").unwrap();
                let view_loc = program.get_uniform_location("view").unwrap();
                let projection_loc = program.get_uniform_location("projection").unwrap();
                unsafe { gl::UniformMatrix4fv(model_loc as i32, 1, gl::FALSE, glm::value_ptr(&model).as_ptr()); }
                unsafe { gl::UniformMatrix4fv(view_loc as i32, 1, gl::FALSE, glm::value_ptr(&view).as_ptr()); }
                unsafe { gl::UniformMatrix4fv(projection_loc as i32, 1, gl::FALSE, glm::value_ptr(&projection).as_ptr()); }

                vertex_array.bind();

                unsafe {
                    gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, ptr::null());
                }

                gl_context.swap_buffers().unwrap();
                gl_context.window().request_redraw();
            },
            _ => {},
        }
    });
}