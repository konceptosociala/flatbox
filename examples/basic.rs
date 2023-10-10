use flatbox::{
    Flatbox, 
    error::FlatboxResult
};
use flatbox_core::math::transform::Transform;
use flatbox_ecs::{Write, CommandBuffer};
use flatbox_render::{
    hal::shader::*,
    pbr::{
        texture::{
            Texture,
            Filter, Order,
        },
        material::Material, model::Model,
    },
    renderer::*, 
    context::*,
};
use flatbox_systems::rendering::*;

fn main() {
    Flatbox::init(WindowBuilder {
        title:  "Learn OpenGL with Rust",
        width:  800,
        height: 800,
        ..Default::default()
    })
        .add_setup_system(setup)
        .add_system(clear_screen)
        .add_system(render_material::<MyMaterial>)
        .run();

}

pub struct MyMaterial {
    rust_texture: Texture,
    wall_texture: Texture,
}

impl MyMaterial {
    pub fn new(rust_texture: &str, wall_texture: &str) -> FlatboxResult<MyMaterial> {
        Ok(MyMaterial {
            rust_texture: Texture::new(rust_texture, Filter::Linear)?,
            wall_texture: Texture::new(wall_texture, Filter::Linear)?,
        })
    }
}

impl Material for MyMaterial {
    fn vertex_shader() -> &'static str {
        include_str!("../crates/render/src/shaders/basic.vs")
    }

    fn fragment_shader() -> &'static str {
        include_str!("../crates/render/src/shaders/basic.fs")
    }

    fn process_pipeline(&self, _: &GraphicsPipeline) {
        self.rust_texture.activate(Order::Texture0);
        self.wall_texture.activate(Order::Texture1);
    }

    fn setup_pipeline(&self, pipeline: &GraphicsPipeline) {
        pipeline.set_int("rustTexture", 0);
        pipeline.set_int("wallTexture", 1);   
    }
}

fn setup(
    mut renderer: Write<Renderer>,
    mut cmd: Write<CommandBuffer>,
) -> FlatboxResult<()> {
    renderer.bind_material::<MyMaterial>();

    cmd.spawn((
        Model::cube(), 
        MyMaterial::new("assets/rust.png", "assets/wall.jpg")?,
        Transform::identity(),
    ));

    Ok(())
}