use crate::{entity::EntityId, world::World};

pub trait Bundle {
    fn build(self, world: &mut World, entity: EntityId);
}

macro_rules! impl_bundle {
    ($($members:ident),+) => {
        impl<
            $($members: 'static),+,
        > Bundle for ($($members,)+) {
            #[allow(non_snake_case)]
            fn build(self, world: &mut World, entity: EntityId) {
                let ($($members,)*) = self;
                $(
                    world.insert(entity, $members);
                )*
            }
        }
    }
}

impl_bundle!(T1);
impl_bundle!(T1, T2);
impl_bundle!(T1, T2, T3);
impl_bundle!(T1, T2, T3, T4);
impl_bundle!(T1, T2, T3, T4, T5);
impl_bundle!(T1, T2, T3, T4, T5, T6);
impl_bundle!(T1, T2, T3, T4, T5, T6, T7);
impl_bundle!(T1, T2, T3, T4, T5, T6, T7, T8);
