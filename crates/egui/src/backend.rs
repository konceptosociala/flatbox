use std::sync::Arc;
use parking_lot::Mutex;

use flatbox_render::{
    error::RenderError, 
    context::{Context, Display, WindowEvent},
    renderer::Renderer,
};
use crate::painter::Painter;

pub struct EguiBackend {
    pub egui_ctx: egui::Context,
    pub state: Arc<Mutex<egui_winit::State>>,
    pub painter: Painter,

    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
}

impl EguiBackend {
    pub fn new(context: &Context) -> Self {
        let painter = Painter::new().expect("Cannot initialize egui backend");

        let mut state = egui_winit::State::new(context.event_loop_target());
        state.set_max_texture_side(2048);

        let pixels_per_point = context.display().lock().window().scale_factor() as f32;
        state.set_pixels_per_point(pixels_per_point);

        Self {
            egui_ctx: egui::Context::default(),
            state: Arc::new(Mutex::new(state)),
            painter,
            shapes: Default::default(),
            textures_delta: Default::default(),
        }
    }

    pub fn context(&mut self) -> &egui::Context {
        &self.egui_ctx
    }

    pub fn on_event(&mut self, event: &WindowEvent<'_>) -> bool {
        self.state.lock().on_event(&self.egui_ctx, event)
    }

    pub fn run(
        &mut self,
        display: Display,
        run_ui: impl FnMut(&egui::Context),
    ) -> std::time::Duration {
        let raw_input = self.state.lock().take_egui_input(display.lock().window());
        let egui::FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = self.egui_ctx.run(raw_input, run_ui);

        self.state.lock()
            .handle_platform_output(display.lock().window(), &self.egui_ctx, platform_output);

        self.shapes = shapes;
        self.textures_delta.append(textures_delta);

        repaint_after
    }

    pub fn paint(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        let shapes = std::mem::take(&mut self.shapes);
        let textures_delta = std::mem::take(&mut self.textures_delta);
        let clipped_primitives = self.egui_ctx.tessellate(shapes);

        let pixels_per_point = self.egui_ctx.pixels_per_point();
        let screen_size_px = renderer.extent().into();

        self.painter.paint_and_update_textures(
            renderer,
            screen_size_px,
            pixels_per_point,
            &clipped_primitives,
            &textures_delta,
        )?;

        Ok(())
    }
}