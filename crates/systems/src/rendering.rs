use std::time::{Duration, Instant};

use anyhow::Result;
// use flatbox_assets::resources::Resources;
use flatbox_core::{math::transform::Transform, AppExit};
use flatbox_ecs::*;
use flatbox_egui::{backend::EguiBackend, command::DrawEguiCommand};
use flatbox_render::{
    context::{ControlFlow, Display}, error::RenderError, pbr::{
        camera::Camera, material::Material, model::Model
    }, renderer::{ClearCommand, DrawModelCommand, PrepareModelCommand, RenderCameraCommand, Renderer}
};

pub fn clear_screen(mut renderer: Write<Renderer>) -> Result<()> {
    renderer.execute(&mut ClearCommand(0.1, 0.1, 0.1))?;
    
    Ok(())
}

pub fn bind_material<M: Material>(mut renderer: Write<Renderer>) {
    renderer.bind_material::<M>();
}

pub fn render_material<M: Material>(
    model_world: SubWorld<(&mut Model, &M, &Transform)>,
    camera_world: SubWorld<(&mut Camera, &Transform)>,
    mut renderer: Write<Renderer>,
) -> Result<()> {
    let mut found_active_camera = false;

    for (_, (mut camera, transform)) in &mut camera_world.query::<(&mut Camera, &Transform)>() {
        if camera.is_active() {
            if found_active_camera {
                Err(RenderError::MultipleActiveCameras)?;
            } else {
                found_active_camera = true;

                renderer.execute(&mut RenderCameraCommand::<M>::new(&mut camera, transform))?;
                for (_, (mut model, material, transform)) in &mut model_world.query::<(&mut Model, &M, &Transform)>() {
                    renderer.execute(&mut PrepareModelCommand::new(&mut model, material))?;
                    renderer.execute(&mut DrawModelCommand::new(&model, material, transform))?;
                }
            }
        }
    }

    Ok(())
}

pub fn run_egui_backend(
    egui_world: SubWorld<&mut EguiBackend>,
    display: Read<Display>,
    mut control_flow: Write<ControlFlow>,
){
    control_flow.set_repaint_after(
        egui_world
            .query::<&mut EguiBackend>()
            .iter()
            .map(|(_,b)| {b})
            .next()
            .unwrap()
            .run((*display).clone(), |_|{})
    );
}

pub fn draw_ui(
    app_exit: SubWorld<&AppExit>,
    egui_world: SubWorld<&mut EguiBackend>,
    display: Read<Display>,
    mut control_flow: Write<ControlFlow>,
    mut renderer: Write<Renderer>,
){
    let mut egui_backend_query = egui_world.query::<&mut EguiBackend>();
    let mut egui_backend = egui_backend_query
        .iter()
        .map(|(_,b)| {b})
        .next()
        .unwrap();

    if app_exit.query::<&AppExit>().iter().len() > 0 {
        control_flow.exit();
    } else if control_flow.repaint_after().is_zero() {
        display.lock().window().request_redraw();
        control_flow.set_poll();
    } else if let Some(repaint_after_instant) = Instant::now().checked_add(control_flow.repaint_after()) {
        control_flow.set_wait_until(repaint_after_instant);
        control_flow.set_repaint_after(Duration::ZERO);
    }

    renderer.execute(&mut DrawEguiCommand::new(&mut egui_backend)).unwrap();    
}