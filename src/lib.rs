use flatbox_assets::manager::AssetManager;
use flatbox_core::logger::FlatboxLogger;
use flatbox_ecs::{World, Schedules, Schedule, System};
use flatbox_render::{
    renderer::Renderer,
    context::{Context, WindowBuilder},
};

pub mod error;
pub mod prelude;

pub struct Flatbox {
    pub assets: AssetManager,
    pub world: World,
    pub schedules: Schedules<&'static str>,
    pub context: Context,
    pub renderer: Renderer,
    pub window_builder: WindowBuilder,
}

impl Flatbox {
    pub fn init(window_builder: WindowBuilder) -> Flatbox {
        if window_builder.init_logger {
            FlatboxLogger::init();
        }

        let context = Context::new(&window_builder);
        let renderer = Renderer::init(|addr| context.get_proc_address(addr));

        Flatbox {
            assets: AssetManager::new(),
            world: World::new(),
            schedules: Schedules::from([
                ("setup", Schedule::builder()),
                ("update", Schedule::builder()),
            ]),
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

    pub fn flush_setup_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("setup").unwrap().flush();
        self
    }

    pub fn flush_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("update").unwrap().flush();
        self
    }

    pub fn run(&mut self){
        let mut setup_schedule = self.schedules.get_mut("setup").unwrap().build();
        let mut update_schedule = self.schedules.get_mut("update").unwrap().build();

        setup_schedule.execute((
            &mut self.world,
            &mut self.renderer,
            &mut self.assets,
        )).expect("Cannot execute setup systems");

        self.context.run(||{
            update_schedule.execute((
                &mut self.world,
                &mut self.renderer,
                &mut self.assets,
            )).expect("Cannot execute update systems");
        });
    }
}