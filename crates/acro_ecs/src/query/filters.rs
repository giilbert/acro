use std::{any::Any, cell::Ref, fmt::Debug, marker::PhantomData};

use crate::{archetype::Archetype, registry::ComponentGroup, world::World};

use super::{
    info::{get_full_component_info, QueryComponentInfo},
    utils::QueryInfoUtils,
    ToQueryInfo,
};

pub trait QueryFilter<'w> {
    type Init: Debug + Any;

    // If true, this filter only acts on archetypes, and filter_test will never be called.
    const IS_STRICTLY_ARCHETYPAL: bool;

    fn init(world: &'w World) -> Self::Init;

    fn filter_archetype<'a>(
        world: &'w World,
        components: impl Iterator<Item = Ref<'a, Archetype>> + Clone,
    ) -> impl Iterator<Item = Ref<'a, Archetype>> + Clone;

    // If the filter is strictly archetypal, calling filter_test is unnecessary because the filter
    // will be applied to the archetypes directly.
    fn filter_test(_init: &Self::Init) -> bool {
        Self::IS_STRICTLY_ARCHETYPAL
    }
}

impl<'w> QueryFilter<'w> for () {
    type Init = ();

    const IS_STRICTLY_ARCHETYPAL: bool = true;

    fn init(_world: &'w World) -> Self::Init {}

    fn filter_archetype<'a>(
        _world: &'w World,
        components: impl Iterator<Item = Ref<'a, Archetype>> + Clone,
    ) -> impl Iterator<Item = Ref<'a, Archetype>> + Clone {
        components
    }
}

#[derive(Debug)]
pub struct With<T> {
    _phantom: PhantomData<T>,
}

impl<'w, T> QueryFilter<'w> for With<T>
where
    T: Any,
{
    type Init = ();
    const IS_STRICTLY_ARCHETYPAL: bool = true;

    fn init(_world: &'w World) -> Self::Init {}

    fn filter_archetype<'a>(
        world: &'w World,
        components: impl Iterator<Item = Ref<'a, Archetype>> + Clone,
    ) -> impl Iterator<Item = Ref<'a, Archetype>> + Clone {
        let full = get_full_component_info::<&T>(world);
        let component_info = full.component_info().clone();

        components.filter(move |archetype| archetype.components.contains(component_info.id))
    }
}

#[derive(Debug)]
pub struct Without<T> {
    _phantom: PhantomData<T>,
}

impl<'w, T> QueryFilter<'w> for Without<T>
where
    T: Any,
{
    type Init = ();
    const IS_STRICTLY_ARCHETYPAL: bool = true;

    fn init(_world: &'w World) -> Self::Init {}

    fn filter_archetype<'a>(
        world: &'w World,
        components: impl Iterator<Item = Ref<'a, Archetype>> + Clone,
    ) -> impl Iterator<Item = Ref<'a, Archetype>> + Clone {
        let full = get_full_component_info::<&T>(world);
        let component_info = full.component_info().clone();

        components.filter(move |archetype| !archetype.components.contains(component_info.id))
    }
}

// Generates a nested call to filter_archetype for each component in the tuple.
macro_rules! expand_filter_archetype {
    ($world:ident, $components:ident, $first:ident, $second:ident) => {
        $first::filter_archetype($world, $second::filter_archetype($world, $components))
    };
    ($world:ident, $components:ident, $head:ident, $($tail:ident),*) => {
        $head::filter_archetype($world,
            expand_filter_archetype!($world, $components, $($tail),+)
        )
    };
}

macro_rules! impl_to_filter_info_and {
    ($($members:ident),+) => {
        impl<'w, $($members: QueryFilter<'w>),*> QueryFilter<'w> for ($($members,)*)
        {
            // The Init type is a tuple of the Init types of each member.
            type Init = ($($members::Init,)*);

            const IS_STRICTLY_ARCHETYPAL: bool = $($members::IS_STRICTLY_ARCHETYPAL &&)+ true;

            fn init(world: &'w World) -> Self::Init {
                ($($members::init(world),)*)
            }

            fn filter_archetype<'a>(
                world: &'w World,
                components: impl Iterator<Item = Ref<'a, Archetype>> + Clone,
            ) -> impl Iterator<Item = Ref<'a, Archetype>> + Clone {
                expand_filter_archetype!(world, components, $($members),+)
            }


            fn filter_test(_init: &Self::Init) -> bool {
                todo!();
            }
        }
    }
}

impl_to_filter_info_and!(T1, T2);
impl_to_filter_info_and!(T1, T2, T3);
impl_to_filter_info_and!(T1, T2, T3, T4);
impl_to_filter_info_and!(T1, T2, T3, T4, T5);
impl_to_filter_info_and!(T1, T2, T3, T4, T5, T6);
impl_to_filter_info_and!(T1, T2, T3, T4, T5, T6, T7);
impl_to_filter_info_and!(T1, T2, T3, T4, T5, T6, T7, T8);

pub struct Or<T1, T2, T3 = (), T4 = (), T5 = (), T6 = (), T7 = (), T8 = ()>
where
    T1: for<'a> QueryFilter<'a>,
    T2: for<'a> QueryFilter<'a>,
    T3: for<'a> QueryFilter<'a>,
    T4: for<'a> QueryFilter<'a>,
    T5: for<'a> QueryFilter<'a>,
    T6: for<'a> QueryFilter<'a>,
    T7: for<'a> QueryFilter<'a>,
    T8: for<'a> QueryFilter<'a>,
{
    _phantom: PhantomData<(T1, T2, T3, T4, T5, T6, T7, T8)>,
}

impl<'w, T1, T2, T3, T4, T5, T6, T7, T8, I1, I2, I3, I4, I5, I6, I7, I8> QueryFilter<'w>
    for Or<T1, T2, T3, T4, T5, T6, T7, T8>
where
    T1: for<'a> QueryFilter<'a, Init = I1>,
    T2: for<'a> QueryFilter<'a, Init = I2>,
    T3: for<'a> QueryFilter<'a, Init = I3>,
    T4: for<'a> QueryFilter<'a, Init = I4>,
    T5: for<'a> QueryFilter<'a, Init = I5>,
    T6: for<'a> QueryFilter<'a, Init = I6>,
    T7: for<'a> QueryFilter<'a, Init = I7>,
    T8: for<'a> QueryFilter<'a, Init = I8>,
    I1: Debug + Any,
    I2: Debug + Any,
    I3: Debug + Any,
    I4: Debug + Any,
    I5: Debug + Any,
    I6: Debug + Any,
    I7: Debug + Any,
    I8: Debug + Any,
{
    type Init = (I1, I2, I3, I4, I5, I6, I7, I8);

    const IS_STRICTLY_ARCHETYPAL: bool = T1::IS_STRICTLY_ARCHETYPAL
        && T2::IS_STRICTLY_ARCHETYPAL
        && T3::IS_STRICTLY_ARCHETYPAL
        && T4::IS_STRICTLY_ARCHETYPAL
        && T5::IS_STRICTLY_ARCHETYPAL
        && T6::IS_STRICTLY_ARCHETYPAL
        && T7::IS_STRICTLY_ARCHETYPAL
        && T8::IS_STRICTLY_ARCHETYPAL;

    fn init(world: &'w World) -> Self::Init {
        (
            T1::init(world),
            T2::init(world),
            T3::init(world),
            T4::init(world),
            T5::init(world),
            T6::init(world),
            T7::init(world),
            T8::init(world),
        )
    }

    fn filter_archetype<'a>(
        world: &'w World,
        components: impl Iterator<Item = Ref<'a, Archetype>> + Clone,
    ) -> impl Iterator<Item = Ref<'a, Archetype>> + Clone {
        T1::filter_archetype(world, components.clone())
            .chain(T2::filter_archetype(world, components.clone()))
            .chain(T3::filter_archetype(world, components.clone()))
            .chain(T4::filter_archetype(world, components.clone()))
            .chain(T5::filter_archetype(world, components.clone()))
            .chain(T6::filter_archetype(world, components.clone()))
            .chain(T7::filter_archetype(world, components.clone()))
            .chain(T8::filter_archetype(world, components))
    }

    fn filter_test(init: &Self::Init) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use assert_unordered::assert_eq_unordered;

    use crate::{entity::EntityId, query::filters::Or, systems::SystemRunContext, world::World};

    use super::With;

    #[test]
    fn query_with_archetype_filter() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();
        world.init_component::<String>();

        let entity1 = world.spawn();
        world.insert(entity1, 42u32);
        world.insert(entity1, "hello".to_string());

        let entity2 = world.spawn();
        world.insert(entity2, 12u32);
        world.insert(entity2, "bye".to_string());
        world.insert(entity2, true);

        let entity3 = world.spawn();
        world.insert(entity3, 22u32);
        world.insert(entity3, false);

        let mut query = world.query::<EntityId, With<String>>();
        assert_eq_unordered!(
            &query.over(&world).collect::<Vec<_>>(),
            &vec![entity1, entity2]
        );

        let mut query = world.query::<EntityId, (With<String>, With<bool>)>();
        assert_eq_unordered!(&query.over(&world).collect::<Vec<_>>(), &vec![entity2]);

        let mut query = world.query::<EntityId, Or<With<String>, With<bool>>>();
        assert_eq_unordered!(
            &query
                .over(SystemRunContext::ignore_changes(&mut world))
                .collect::<Vec<_>>(),
            &vec![entity1, entity2, entity3]
        );

        let mut query = world.query::<EntityId, Or<With<String>, With<u32>>>();
        assert_eq_unordered!(
            &query
                .over(SystemRunContext::ignore_changes(&mut world))
                .collect::<Vec<_>>(),
            &vec![entity1, entity2, entity3]
        );
    }
}
