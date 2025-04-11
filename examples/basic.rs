use anyhow::Result;
use flatbox::{
    core::math::{
        glm, transform::Transform
    }, 
    ecs::{CommandBuffer, Write}, 
    egui, 
    render::{
        context::*, pbr::{
            camera::{Camera, CameraType}, material::DefaultMaterial, model::Model, texture::Texture
        }
    }, 
    Flatbox
};
use flatbox_ecs::{query::Mut, Read, SubWorld, SystemStage::*};
use flatbox_egui::backend::EguiBackend;
use flatbox_render::pbr::model::ModelBundle;

fn main() {
    Flatbox::init(WindowBuilder {
        title:  "Flatbox basic example",
        width:  800,
        height: 600,
        ..Default::default()
    })
        .default_extensions() 
        .add_system(Setup, setup)
        .add_system(Render, set_ui)
        .run();
}

fn setup(mut cmd: Write<CommandBuffer>) -> Result<()> {
    use std::io::Write as _;
    // let model = Model::load_obj("/home/oleksandrhnutov/Стільниця/ico.obj")?.into_iter().next().unwrap();
    // let mut w = std::fs::File::create("ico").unwrap();
    // writeln!(&mut w, "{model:#?}").unwrap();
    
    // cmd.spawn(ModelBundle{
    //     model,
    //     material: DefaultMaterial {
    //         diffuse_map: Text`ure::new("assets/crate.png", None)?,
    //         specular_map: Texture::new("assets/crate_spec.png", None)?,
    //         ..Default::default()
    //     },
    //     transform: Transform::new_from_translation(glm::vec3(2.0, 0.0, 0.0)),
    // });

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

fn set_ui(
    mut control_flow: Write<ControlFlow>,
    display: Read<Display>,
    egui_world: SubWorld<&mut EguiBackend>,
    cam_world: SubWorld<(&Camera, &mut Transform)>,
){
    control_flow.set_repaint_after(
        egui_world
            .query::<&mut EguiBackend>()
            .iter()
            .map(|(_,b)| {b})
            .next()
            .unwrap()
            .run(&display, |ctx|{
                egui::Window::new("Ĉapitrelekta dialogo")
                    .collapsible(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("Hello World!");
                    });

                let mut trans = <TransformFunction>::None;

                if ctx.input().key_down(egui::Key::W) {
                    trans = Some(Box::new(|mut t: Mut<'_, Transform>| { t.translation.x += 1.0; println!("w pressed"); }));
                }
            
                if ctx.input().key_down(egui::Key::S) {
                    trans = Some(Box::new(|mut t: Mut<'_, Transform>| { t.translation.x -= 1.0; println!("s pressed"); }));
                }
            
                cam_world.query::<(&Camera, &mut Transform)>()
                    .into_iter()
                    .map(|(_, (_, t))| t)
                    .for_each(trans.unwrap_or(Box::new(|_|{})));  
            })
    );
}

type TransformFunction = Option<Box<dyn FnMut(Mut<'_, Transform>)>>;