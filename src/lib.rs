use std::any::TypeId;
use extension::RenderGuiExtension;
use flatbox_egui::backend::EguiBackend;
use pretty_type_name::pretty_type_name;
use flatbox_core::logger::FlatboxLogger;
use flatbox_ecs::{Schedules, System, SystemStage::{self, *}, World};
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

pub mod physics {
    // pub use flatbox_physics::*;
}

pub mod render {
    pub use flatbox_render::*;
}

pub mod systems {
    pub use flatbox_systems::*;
}

pub struct Flatbox {
    pub world: World,
    pub schedules: Schedules,
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
            world: World::new(),
            schedules: Schedules::new(),
            extensions: Extensions::new(),
            context,
            renderer,
            window_builder,
            on_window_event: Box::new(on_event_empty),
        }
    }

    pub fn add_system<Args, Ret, S>(&mut self, system_stage: SystemStage, system: S) -> &mut Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.add_system(system_stage, system);
        self
    }

    pub fn flush_systems(&mut self, system_stage: SystemStage) -> &mut Self {
        self.schedules.flush_systems(system_stage);
        self
    }

    pub fn set_on_window_event<F: Fn(&mut World, WindowEvent) -> bool + 'static>(&mut self, on_event: F) -> &mut Self {
        self.on_window_event = Box::new(on_event);
        self
    }

    pub fn apply_extension<E: Extension + 'static>(&mut self, extension: E) -> &mut Self {
        if self.extensions.contains(&TypeId::of::<E>()) {
            panic!("Extension `{}` is already added!", pretty_type_name::<E>());
        } else {
            self.extensions.push(TypeId::of::<E>());
            extension.apply(self);
        }

        self
    }

    pub fn default_extensions(&mut self) -> &mut Self {
        self
            .apply_extension(BaseRenderExtension)
            .apply_extension(RenderMaterialExtension::<DefaultMaterial>::new())
            .apply_extension(RenderGuiExtension);

        self
    }

    pub fn run(&mut self){
        let on_window_event = std::mem::replace(&mut self.on_window_event, Box::new(on_event_empty));
        let mut setup_schedule = self.schedules.get_systems(Setup).unwrap().build();
        let mut update_schedule = self.schedules.get_systems(Update).unwrap().build();
        let mut pre_render_schedule = self.schedules.get_systems(PreRender).unwrap().build();
        let mut render_schedule = self.schedules.get_systems(Render).unwrap().build();
        let mut post_render_schedule = self.schedules.get_systems(PostRender).unwrap().build();

        #[cfg(feature = "egui")]
        self.world.spawn((EguiBackend::new(&self.context),));

        self.context.run(|event|{
            match event {
                ContextEvent::Setup(mut display) => {
                    setup_schedule.execute_seq((
                        &mut display,
                        &mut self.world,
                        &mut self.renderer,
                    )).expect("Cannot execute set-up systems");
                },
                ContextEvent::Resize(extent) => {
                    self.renderer.set_extent(extent);
                },
                ContextEvent::Update => {
                    update_schedule.execute((
                        &mut self.world,
                    )).expect("Cannot execute update systems");
                },
                ContextEvent::Render(mut display, mut control_flow) => { 
                    pre_render_schedule.execute_seq((
                        &mut display,
                        &mut control_flow,
                        &mut self.world,
                        &mut self.renderer,
                    )).expect("Cannot execute pre-render systems");

                    render_schedule.execute_seq((
                        &mut display,
                        &mut control_flow,
                        &mut self.world,
                        &mut self.renderer,
                    )).expect("Cannot execute render systems");

                    post_render_schedule.execute_seq((
                        &mut display,
                        &mut control_flow,
                        &mut self.world,
                        &mut self.renderer,
                    )).expect("Cannot execute post-render systems");
                },
                ContextEvent::Window(display, event) => {
                    if on_window_event(&mut self.world, event) {
                        display.lock().window().request_redraw();
                    }
                },
            }
        });
    }
}

pub type OnEventFn = Box<dyn Fn(&mut World, WindowEvent) -> bool>;

fn on_event_empty(_: &mut World, _: WindowEvent) -> bool { false }