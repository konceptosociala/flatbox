use std::marker::PhantomData;
use std::any::TypeId;
use std::fmt::Debug;
use flatbox_render::pbr::material::Material;
#[cfg(feature = "egui")]
use flatbox_egui::backend::EguiBackend;
use flatbox_systems::rendering::{render_material, clear_screen, bind_material};

use crate::Flatbox;
 
pub trait Extension: Debug {
    fn apply(&self, app: &mut Flatbox);
}

pub type Extensions = Vec<TypeId>;

#[derive(Default, Debug)]
pub struct BaseRenderExtension;

impl Extension for BaseRenderExtension {
    fn apply(&self, app: &mut Flatbox) {
        app
            .add_render_system(clear_screen);
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
            .add_setup_system(bind_material::<M>)
            .add_render_system(render_material::<M>);
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
            .add_resource(EguiBackend::new(&app.context))
            .set_on_window_event(|resources, event| {
                resources.get_resource_mut::<EguiBackend>().unwrap().on_event(&event)
            });
    }
}
