use std::collections::HashMap;

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

pub mod event;

pub type Schedules = HashMap<&'static str, ScheduleBuilder>;