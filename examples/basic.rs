use anyhow::Result;
use flatbox::{
    Flatbox,
    core::math::{
        transform::Transform,
        glm,
    },
    ecs::{Write, CommandBuffer},
    render::{
        pbr::{
            texture::Texture,
            material::DefaultMaterial, 
            model::Model,
            camera::{Camera, CameraType},
        },
        context::*,
    },
    extension::RenderGuiExtension,
};

fn main() {
    Flatbox::init(WindowBuilder {
        title:  "Flatbox basic example",
        width:  800,
        height: 600,
        ..Default::default()
    })
        .add_default_extensions() 
        .add_extension(RenderGuiExtension)   
        .add_setup_system(setup)
        .run();
}

fn setup(mut cmd: Write<CommandBuffer>) -> Result<()> {
    cmd.spawn((
        Model::cube(), 
        DefaultMaterial {
            diffuse_map: Texture::new("assets/crate.png", None)?,
            specular_map: Texture::new("assets/crate_spec.png", None)?,
            ..Default::default()
        },
        Transform::new_from_translation(glm::vec3(2.0, 0.0, 0.0)),
    ));

    cmd.spawn((
        Model::cube(), 
        DefaultMaterial {
            diffuse_map: Texture::new("assets/crate.png", None)?,
            specular_map: Texture::new("assets/crate_spec.png", None)?,
            ..Default::default()
        },
        Transform::new_from_translation(glm::vec3(-2.0, 0.0, 0.0)),
    ));

    cmd.spawn((
        Model::cube(), 
        DefaultMaterial {
            diffuse_map: Texture::new("assets/crate.png", None)?,
            specular_map: Texture::new("assets/crate_spec.png", None)?,
            ..Default::default()
        },
        Transform::new_from_translation(glm::vec3(0.0, 0.0, 2.0)),
    ));

    cmd.spawn((
        Model::cube(), 
        DefaultMaterial {
            diffuse_map: Texture::new("assets/crate.png", None)?,
            specular_map: Texture::new("assets/crate_spec.png", None)?,
            ..Default::default()
        },
        Transform::new_from_translation(glm::vec3(0.0, 0.0, -2.0)),
    ));

    cmd.spawn((
        Camera::builder()
            .camera_type(CameraType::FirstPerson)
            .is_active(true)
            .build(),
        Transform {
            translation: glm::vec3(3.0, -3.0, 3.0),
            rotation: glm::safe_quat_look_at(
                &glm::vec3(0.0, 0.0, 0.0), 
                &glm::vec3(3.0, -3.0, 3.0),
                &glm::Vec3::y_axis(), 
                &glm::Vec3::y_axis(),
            ),
            scale: 1.0,
        },
    ));

    Ok(())
}