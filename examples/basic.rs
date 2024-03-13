use std::time::Instant;

use anyhow::Result;
use flatbox::{
    core::math::{
        glm, transform::Transform
    }, ecs::{CommandBuffer, Write}, egui, extension::RenderGuiExtension, render::{
        context::*, pbr::{
            camera::{Camera, CameraType}, material::DefaultMaterial, model::Model, texture::Texture
        }
    }, Flatbox
};
use flatbox_assets::resources::Resources;
use flatbox_ecs::{event::{AppExit, Events}, Read, SubWorld};
use flatbox_egui::{backend::EguiBackend, command::DrawEguiCommand};
use flatbox_render::renderer::Renderer;

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
        .add_render_system(render)
        .add_system(update)
        .run();
}

fn render(
    display: Read<Display>,
    control_flow: Read<ControlFlow>,
    events: Read<Events>,
    resources: Read<Resources>,
    mut renderer: Write<Renderer>,
){
    let mut egui_backend = resources.get_resource_mut::<EguiBackend>().unwrap();

    let repaint_after = egui_backend.run((*display).clone(), |egui_ctx|{
        egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
            ui.heading("Hello World!");
            if ui.button("Quit").clicked() {
                events.get_handler_mut::<AppExit>().unwrap().send(AppExit);
            }
        });
    });

    if events.get_handler::<AppExit>().unwrap().read().is_some() {
        control_flow.exit();
    } else if repaint_after.is_zero() {
        display.lock().window().request_redraw();
        control_flow.set_poll();
    } else if let Some(repaint_after_instant) = Instant::now().checked_add(repaint_after) {
        control_flow.set_wait_until(repaint_after_instant);
    }

    renderer.execute(&mut DrawEguiCommand::new(&mut egui_backend)).unwrap();
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

fn update(events: Read<Events>, cam_world: SubWorld<(&Camera, &mut Transform)>){
    if let Some(WindowEvent::KeyboardInput { input, .. }) = events.get_handler::<WindowEvent>().unwrap().read() {
        for (_, (_, mut t)) in &mut cam_world.query::<(&Camera, &mut Transform)>() {
            match input.virtual_keycode {
                Some(VirtualKeyCode::W) => t.translation.z -= 1.0,
                Some(VirtualKeyCode::S) => t.translation.z += 1.0,
                Some(VirtualKeyCode::A) => t.translation.x -= 1.0,
                Some(VirtualKeyCode::D) => t.translation.x += 1.0,
                _ => {},
            }
        }
    }
}