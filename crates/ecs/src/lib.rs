pub mod event;

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

pub type Schedules = std::collections::HashMap<&'static str, ScheduleBuilder>;