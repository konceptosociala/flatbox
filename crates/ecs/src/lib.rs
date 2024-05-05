use std::collections::HashMap;

pub use paste::paste;

pub use hecs::{
    *,
    serialize::column::{
        *,
        deserialize as deserialize_world,
        serialize as serialize_world,
    },
};
pub use hecs_schedule::{
    *, 
    borrow::*,
    Access, Batch, CommandBuffer, QueryOne,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SystemStage {
    Setup,
    Update,
    PreRender,
    Render,
    PostRender,
}

pub struct Schedules {
    schedules: HashMap<SystemStage, ScheduleBuilder>,
}

impl Default for Schedules {
    fn default() -> Self {
        Schedules {
            schedules: HashMap::from([
                (SystemStage::Setup, Schedule::builder()),
                (SystemStage::Update, Schedule::builder()),
                (SystemStage::PreRender, Schedule::builder()),
                (SystemStage::Render, Schedule::builder()),
                (SystemStage::PostRender, Schedule::builder()),
            ]),
        }
    }
}

impl Schedules {
    pub fn new() -> Self { 
        Self::default() 
    }

    pub fn add_system<Args, Ret, S>(&mut self, system_stage: SystemStage, system: S)
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.get_mut(&system_stage).unwrap().add_system(system);
    }

    pub fn get_systems(&mut self, system_stage: SystemStage) -> Option<&mut ScheduleBuilder> {
        self.schedules.get_mut(&system_stage)
    }

    pub fn flush_systems(&mut self, system_stage: SystemStage) {
        self.schedules.get_mut(&system_stage).unwrap().flush();
    }
}

#[macro_export]
macro_rules! single {
    ($world:ident, $state:ident => $query:ty) => {
        $crate::paste! {
            let mut [<$state _query>] = $world.query::<$query>();
            
            let count = [<$state _query>].iter().len();
            if count != 1 {
                panic!("Entity `{}` is not single! Count is `{}`", stringify!($query), count);
            }

            let mut $state = [<$state _query>]
                .iter()
                .map(|(_, s)| {s})
                .next()
                .unwrap();
        }
    };
}