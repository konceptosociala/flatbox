use as_any::AsAny;
use flatbox_core::math::transform::Transform;
use flatbox_ecs::{Component, EntityBuilder};

#[typetag::serde(tag = "component")]
pub trait SerializableComponent: Component + AsAny {
    fn add_into(&self, entity_builder: &mut EntityBuilder);
}

/// Macro for implementing [`SerializableComponent`] trait for multiple types, that implement [`Clone`] trait; for using in [`Scene`]'s. Use to avoid boilerplate
/// 
/// # Usage example
/// 
/// ```rust
/// #[derive(Clone)]
/// struct ComponentA;
/// 
/// #[derive(Clone)]
/// struct ComponentB;
/// 
/// #[derive(Clone)]
/// struct ComponentC;
/// 
/// impl_ser_component!(ComponentA, ComponentB, ComponentC);
/// 
/// ```
/// 
#[macro_export]
macro_rules! impl_ser_component {
    ($($comp:ty),+) => {
        $(
            #[typetag::serde]
            impl $crate::ser_component::SerializableComponent for $comp {
                fn add_into(&self, entity_builder: &mut ::flatbox_ecs::EntityBuilder) {
                    entity_builder.add(self.clone());
                }
            }
        )+
    }
}

impl_ser_component!(
    bool, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize,
    Transform
);