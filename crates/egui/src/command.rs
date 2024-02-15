use anyhow::Result;
use flatbox_core::catch::CatchError;
use flatbox_render::{
    renderer::{RenderCommand, Renderer}, 
    error::RenderError, 
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
        self.egui.paint(renderer).catch();

        Ok(())
    }
}