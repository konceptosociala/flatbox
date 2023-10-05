use flatbox_assets::manager::AssetManager;
use flatbox_ecs::{World, Schedules};
use flatbox_render::{
    renderer::Renderer,
    context::{Context, WindowBuilder},
};

pub mod error;
pub mod prelude;

pub struct Flatbox {
    pub window_builder: WindowBuilder,
    pub world: World,
    pub schedules: Schedules<&'static str>,
    pub assets: AssetManager,
    pub context: Context,
    pub renderer: Renderer,
}

impl Flatbox {
    
}