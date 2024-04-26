use std::marker::PhantomData;
use std::any::TypeId;
use std::fmt::Debug;
use flatbox_render::pbr::material::Material;
use flatbox_systems::rendering::{bind_material, clear_screen, draw_ui, render_material, run_egui_backend};

#[cfg(feature = "egui")]
use flatbox_egui::backend::EguiBackend;

use crate::Flatbox;

use flatbox_ecs::SystemStage::*;
 
pub trait Extension: Debug {
    fn apply(&self, app: &mut Flatbox);
}

pub type Extensions = Vec<TypeId>;

#[derive(Default, Debug)]
pub struct BaseRenderExtension;

impl Extension for BaseRenderExtension {
    fn apply(&self, app: &mut Flatbox) {
        app
            .add_system(Render, clear_screen);
    }
}

pub struct RenderMaterialExtension<M>(PhantomData<M>);

impl<M> Debug for RenderMaterialExtension<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RenderMaterialExtension")
    }
}

impl<M: Material> RenderMaterialExtension<M> {
    pub fn new() -> Self {
        RenderMaterialExtension::default()
    }
}

impl<M: Material> Extension for RenderMaterialExtension<M> {
    fn apply(&self, app: &mut Flatbox) {
        app
            .add_system(Setup, bind_material::<M>)
            .add_system(Render, render_material::<M>);
    }
}

impl<M: Material> Default for RenderMaterialExtension<M> {
    fn default() -> Self {
        RenderMaterialExtension(PhantomData)
    }
}

#[cfg(feature = "egui")]
#[derive(Debug)]
pub struct RenderGuiExtension;

#[cfg(feature = "egui")]
impl Extension for RenderGuiExtension {
    fn apply(&self, app: &mut Flatbox) {
        app
            .add_system(Render, run_egui_backend)
            .add_system(PostRender, draw_ui)
            .set_on_window_event(|world, event| {
                world
                    .query::<&mut EguiBackend>()
                    .iter()
                    .map(|(_, b)| {b})
                    .next().unwrap().on_event(&event)
            });
    }
}
