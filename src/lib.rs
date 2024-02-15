use std::any::TypeId;
use pretty_type_name::pretty_type_name;
use flatbox_assets::{manager::AssetManager, resources::{Resources, Resource}};
use flatbox_core::logger::FlatboxLogger;
use flatbox_ecs::{event::{AppExit, Event, EventHandler, Events}, Schedule, Schedules, System, World};
use flatbox_render::{
    renderer::Renderer,
    context::{Context, WindowBuilder, ContextEvent, WindowEvent}, 
    pbr::material::DefaultMaterial,
};

use crate::extension::{Extension, Extensions, RenderMaterialExtension, BaseRenderExtension};

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

pub mod egui {
    pub use flatbox_egui::*;
}

pub mod macros {
    // pub use flatbox_macros::*;
}

pub mod render {
    pub use flatbox_render::*;
}

pub mod systems {
    pub use flatbox_systems::*;
}

pub struct Flatbox {
    pub resources: Resources,
    pub assets: AssetManager,
    pub world: World,
    pub schedules: Schedules,
    pub events: Events,
    pub extensions: Extensions,
    pub context: Context,
    pub renderer: Renderer,
    pub window_builder: WindowBuilder,
    pub on_window_event: OnEventFn,
}

impl Flatbox {
    pub fn init(window_builder: WindowBuilder) -> Flatbox {
        FlatboxLogger::init_with_level(window_builder.logger_level);

        let context = Context::new(&window_builder);
        let renderer = Renderer::init(&context).expect("Cannot initialize renderer");

        Flatbox {
            resources: Resources::new(),
            assets: AssetManager::new(),
            world: World::new(),
            schedules: Schedules::from([
                ("setup", Schedule::builder()),
                ("update", Schedule::builder()),
                ("render", Schedule::builder()),
            ]),
            events: Events::new(),
            extensions: Extensions::new(),
            context,
            renderer,
            window_builder,
            on_window_event: Box::new(on_event_empty),
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

    pub fn add_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources.add_resource(resource);
        self
    }

    pub fn add_event_handler<E: Event>(&mut self) -> &mut Self {
        self.events.push_handler(EventHandler::<E>::new());
        self
    }

    pub fn set_on_window_event<F: Fn(&mut Resources, WindowEvent) -> bool + 'static>(&mut self, on_event: F) -> &mut Self {
        self.on_window_event = Box::new(on_event);
        self
    }

    pub fn add_extension<E: Extension + 'static>(&mut self, extension: E) -> &mut Self {
        if self.extensions.contains(&TypeId::of::<E>()) {
            panic!("Extension `{}` is already added!", pretty_type_name::<E>());
        } else {
            extension.apply(self);
        }

        self
    }

    pub fn add_default_extensions(&mut self) -> &mut Self {
        self
            .add_extension(BaseRenderExtension)
            .add_extension(RenderMaterialExtension::<DefaultMaterial>::new());

        self
    }

    pub fn run(&mut self){
        let on_window_event = std::mem::replace(&mut self.on_window_event, Box::new(on_event_empty));

        let mut render_schedule = self.schedules.get_mut("render").unwrap().build();
        let mut setup_schedule = self.schedules.get_mut("setup").unwrap().build();
        let mut update_schedule = self.schedules.get_mut("update").unwrap().build();

        self.events.push_handler(EventHandler::<AppExit>::new());

        setup_schedule.execute_seq((
            &mut self.events,
            &mut self.resources,
            &mut self.world,
            &mut self.renderer,
            &mut self.assets,
        )).expect("Cannot execute setup systems");

        self.context.run(|event|{
            match event {
                ContextEvent::ResizeEvent(extent) => {
                    self.renderer.set_extent(extent);
                },
                ContextEvent::UpdateEvent => {
                    update_schedule.execute((
                        &mut self.events,
                        &mut self.resources,
                        &mut self.world,
                        &mut self.renderer,
                        &mut self.assets,
                    )).expect("Cannot execute update systems");
                },
                ContextEvent::RenderEvent(mut display, mut control_flow) => {                  
                    render_schedule.execute_seq((
                        &mut display,
                        &mut control_flow,
                        &mut self.events,
                        &mut self.resources,
                        &mut self.world,
                        &mut self.renderer,
                        &mut self.assets,
                    )).expect("Cannot execute render systems");
                },
                ContextEvent::WindowEvent(display, event) => {
                    if on_window_event(&mut self.resources, event) {
                        display.lock().window().request_redraw();
                    }
                },
            }
        });
    }
}

pub type OnEventFn = Box<dyn Fn(&mut Resources, WindowEvent) -> bool>;

fn on_event_empty(_: &mut Resources, _: WindowEvent) -> bool { false }