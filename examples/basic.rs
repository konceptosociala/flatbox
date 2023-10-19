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
            camera::{Camera, CameraType},
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
        .add_system(camera)
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

struct CircleAngle(f32);

fn setup(mut cmd: Write<CommandBuffer>) -> Result<()> {
    cmd.spawn((
        Model::cube(), 
        MyMaterial::new("assets/rust.png", "assets/wall.jpg")?,
        Transform::identity(),
    ));

    cmd.spawn((
        Camera::builder()
            .camera_type(CameraType::FirstPerson)
            .is_active(true)
            .build(),
        Transform::new_from_translation(glm::vec3(0.0, 0.0, -3.0)),
        CircleAngle(0.0),
    ));

    Ok(())
}

fn rotate(world: SubWorld<With<&mut Transform, &Model>>) {
    for (_, mut transform) in &mut world.query::<With<&mut Transform, &Model>>() {
        transform.rotation = glm::quat_rotate(&transform.rotation, 1.0f32.to_radians(), &glm::vec3(1.0, 1.0, 1.0));
    }
}

fn camera(world: SubWorld<With<(&mut Transform, &mut CircleAngle), &Camera>>) {
    for (_, (mut transform, mut angle)) in &mut world.query::<With<(&mut Transform, &mut CircleAngle), &Camera>>() {
        let r = 1.0;
        let x = r * f32::cos(angle.0);
        let y = r * f32::sin(angle.0);

        let point = glm::vec3(
            transform.translation.x + x,
            transform.translation.y + y,
            transform.translation.z,
        );

        transform.rotation = safe_quat_look_at(
            &glm::vec3(0.0, 0.0, 0.0),
            &point,
            &glm::Vec3::y(),
            &glm::Vec3::y(),
        );

        angle.0 += 0.01;
    }
}

fn safe_quat_look_at(
    look_from: &glm::Vec3,
    look_to: &glm::Vec3,
    up: &glm::Vec3,
    alternative_up: &glm::Vec3,
) -> glm::Quat {
    let mut direction: glm::Vec3 = look_to - look_from;
    let direction_length = glm::length(&direction);

    if direction_length <= 0.0001 {
        return glm::quat(1.0, 0.0, 0.0, 0.0);
    }

    direction /= direction_length;

    let dot = glm::dot(&direction, up);
    let abs = if dot < 0.0 { -dot } else { dot };
    if abs > 0.9999 {
        glm::quat_look_at(&direction, alternative_up)
    }
    else {
        glm::quat_look_at(&direction, up)
    }
}
