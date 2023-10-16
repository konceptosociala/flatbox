use std::any::TypeId;
use flatbox_assets::manager::AssetManager;
use flatbox_core::logger::FlatboxLogger;
use flatbox_ecs::{World, Schedules, Schedule, System};
use flatbox_render::{
    renderer::Renderer,
    context::{Context, WindowBuilder, ContextEvent},
};

use extension::{Extension, Extensions};
use pretty_type_name::pretty_type_name;

pub mod error;
pub mod extension;
pub mod prelude;

pub mod assets {
    pub use flatbox_assets::*;
}

pub mod core {
    pub use flatbox_core::*;
}

pub mod ecs {
    pub use flatbox_ecs::*;
}

pub mod macros {
    pub use flatbox_macros::*;
}

pub mod render {
    pub use flatbox_render::*;
}

pub mod systems {
    pub use flatbox_systems::*;
}

pub struct Flatbox {
    pub assets: AssetManager,
    pub world: World,
    pub schedules: Schedules,
    pub extensions: Extensions,
    pub context: Context,
    pub renderer: Renderer,
    pub window_builder: WindowBuilder,
}

impl Flatbox {
    pub fn init(window_builder: WindowBuilder) -> Flatbox {
        FlatboxLogger::init_with_level(window_builder.logger_level);

        let context = Context::new(&window_builder);
        let renderer = Renderer::init(|addr| context.get_proc_address(addr));

        Flatbox {
            assets: AssetManager::new(),
            world: World::new(),
            schedules: Schedules::from([
                ("setup", Schedule::builder()),
                ("update", Schedule::builder()),
                ("render", Schedule::builder()),
            ]),
            extensions: Extensions::new(),
            context,
            renderer,
            window_builder,
        }
    }

    /// Add setup system to schedule
    pub fn add_setup_system<Args, Ret, S>(&mut self, system: S) -> &mut Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.get_mut("setup").unwrap().add_system(system);
        self
    }

    /// Add cyclical system to schedule
    pub fn add_system<Args, Ret, S>(&mut self, system: S) -> &mut Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.get_mut("update").unwrap().add_system(system);
        self
    }

    pub fn add_render_system<Args, Ret, S>(&mut self, system: S) -> &mut Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.get_mut("render").unwrap().add_system(system);
        self
    }

    pub fn flush_setup_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("setup").unwrap().flush();
        self
    }

    pub fn flush_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("update").unwrap().flush();
        self
    }

    pub fn flush_render_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("render").unwrap().flush();
        self
    }

    pub fn add_extension<E: Extension + 'static>(&mut self, extension: E) -> &mut Self {
        if self.extensions.insert(TypeId::of::<E>(), Box::new(extension)).is_some() {
            panic!("Extension `{}` is already added!", pretty_type_name::<E>());
        }

        self
    }

    pub fn run(&mut self){
        let extensions = std::mem::take(&mut self.extensions);

        for ext in extensions.values() {
            ext.apply(self);
        }

        let mut render_schedule = self.schedules.get_mut("render").unwrap().build();
        let mut setup_schedule = self.schedules.get_mut("setup").unwrap().build();
        let mut update_schedule = self.schedules.get_mut("update").unwrap().build();

        setup_schedule.execute((
            &mut self.world,
            &mut self.renderer,
            &mut self.assets,
        )).expect("Cannot execute setup systems");

        self.context.run(|event|{
            match event {
                ContextEvent::UpdateEvent => {
                    update_schedule.execute((
                        &mut self.world,
                        &mut self.renderer,
                        &mut self.assets,
                    )).expect("Cannot execute update systems");
                },
                ContextEvent::RenderEvent => {
                    render_schedule.execute_seq((
                        &mut self.world,
                        &mut self.renderer,
                        &mut self.assets,
                    )).expect("Cannot execute render systems");
                },
            }
        });
    }
}