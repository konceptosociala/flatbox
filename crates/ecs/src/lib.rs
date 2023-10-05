pub mod event;

pub use hecs::{
    *,
    serialize::column::*,
};
pub use hecs_schedule::{
    *, 
    borrow::*,
    Access, Batch, CommandBuffer, QueryOne,
};

pub type Schedules<K> = std::collections::HashMap<K, ScheduleBuilder>;