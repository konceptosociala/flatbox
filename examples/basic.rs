use anyhow::Result;
use flatbox::{
    Flatbox,
    error::FlatboxResult,
    core::math::{
        transform::Transform,
        glm,
    },
    ecs::{Write, CommandBuffer, SubWorld, With},
    render::{
        hal::shader::*,
        pbr::{
            texture::{
                Texture,
                Filter, Order,
            },
            material::Material, 
            model::Model,
            camera::Camera,
        },
        context::*,
    },
    extension::RenderMaterialExtension,
};

fn main() {
    Flatbox::init(WindowBuilder {
        title:  "Learn OpenGL with Rust",
        width:  800,
        height: 600,
        ..Default::default()
    })
        .add_default_extensions() 
        .add_extension(RenderMaterialExtension::<MyMaterial>::new())      
        .add_setup_system(setup)
        .add_system(rotate)
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
        pipeline.set_int("material.rustTexture", 0);
        pipeline.set_int("material.wallTexture", 1);
    }
}

fn setup(mut cmd: Write<CommandBuffer>) -> Result<()> {
    cmd.spawn((
        Model::cube(), 
        MyMaterial::new("assets/rust.png", "assets/wall.jpg")?,
        Transform::identity(),
    ));

    cmd.spawn((
        Camera::builder()
            .is_active(true)
            .build(),
        Transform::new_from_translation(glm::vec3(0.0, 0.0, -3.0)),
    ));

    Ok(())
}

fn rotate(world: SubWorld<With<&mut Transform, &Model>>) {
    for (_, mut transform) in &mut world.query::<With<&mut Transform, &Model>>() {
        transform.rotation = glm::quat_rotate(&transform.rotation, 0.1f32.to_radians(), &glm::vec3(1.0, 1.0, 1.0))
    }
}