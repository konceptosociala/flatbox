use std::time::Instant;

use anyhow::Result;
use flatbox_assets::resources::Resources;
use flatbox_core::catch::CatchError;
use flatbox_ecs::{
    Write, 
    Read, 
};
use flatbox_render::{
    renderer::{RenderCommand, Renderer}, 
    error::RenderError, 
    context::{Display, ControlFlow},
};

use crate::backend::EguiBackend;

pub struct DrawEguiCommand<'a> {
    egui: &'a mut EguiBackend,
}

impl<'a> DrawEguiCommand<'a> {
    pub fn new(egui: &'a mut EguiBackend) -> Self {
        DrawEguiCommand { egui }
    }
}

impl<'a> RenderCommand for DrawEguiCommand<'a> {
    fn execute(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {       
        self.egui.paint(renderer.extent()).catch();

        Ok(())
    }
}

pub fn render_egui(
    display: Read<Display>,
    control_flow: Read<ControlFlow>,
    mut renderer: Write<Renderer>,
    mut resources: Write<Resources>,
) -> Result<()> {
    let mut egui_backend = resources.get_resource_mut::<EguiBackend>().unwrap();
    let mut exit = false;

    let repaint_after = egui_backend.run((*display).clone(), |egui_ctx| {
        egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
            ui.heading("Hello World!");
            if ui.button("Quit").clicked() {
                exit = true;
            }
        });
    });

    if exit {
        control_flow.exit();
    } else if repaint_after.is_zero() {
        display.lock().window().request_redraw();
        control_flow.set_poll();
    } else if let Some(repaint_after_instant) = Instant::now().checked_add(repaint_after) {
        control_flow.set_wait_until(repaint_after_instant);
    }

    renderer.execute(&mut DrawEguiCommand::new(&mut egui_backend))?;

    Ok(())
}