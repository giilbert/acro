use crate::{
    archetype::Archetype, entity::EntityId, pointer::change_detection::Mut, registry::ComponentId,
    systems::SystemRunContext,
};

/// A trait for types that can be fetched from a query.
/// 'w is the lifetime of the world or data borrowed from it.
pub trait QueryTransform {
    const IS_CREATE: bool = false;

    type InputOrCreate;
    type Output;

    #[inline]
    fn create(
        _ctx: &SystemRunContext,
        _current_archetype: &Archetype,
        _entity_index: usize,
    ) -> Self::Output {
        unimplemented!("create is not available for this type");
    }

    #[inline]
    fn transform_component(
        _ctx: &SystemRunContext,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        _input: Self::InputOrCreate,
    ) -> Self::Output {
        unimplemented!("create is not available for this type");
    }
}

impl<'v, T> QueryTransform for &'v T {
    type InputOrCreate = &'v T;
    type Output = Self::InputOrCreate;

    #[inline]
    fn transform_component(
        _ctx: &SystemRunContext,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        input: Self::InputOrCreate,
    ) -> Self::Output {
        input
    }
}

impl<'v, T> QueryTransform for Option<&'v T> {
    type InputOrCreate = Option<&'v T>;
    type Output = Self::InputOrCreate;

    #[inline]
    fn transform_component(
        _ctx: &SystemRunContext,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        input: Self::InputOrCreate,
    ) -> Self::Output {
        input
    }
}

impl<'v, T> QueryTransform for &'v mut T {
    type InputOrCreate = &'v mut T;
    type Output = Mut<'v, T>;

    #[inline]
    fn transform_component(
        ctx: &SystemRunContext,
        current_archetype: &Archetype,
        entity_index: usize,
        component: ComponentId,
        input: Self::InputOrCreate,
    ) -> Self::Output {
        let column = current_archetype
            .table
            .columns
            .get(&component)
            .expect("component not found");

        Mut::new(column.change_detection, ctx.tick, entity_index, input)
    }
}

impl<'v, T> QueryTransform for Option<&'v mut T> {
    type InputOrCreate = Option<&'v mut T>;
    type Output = Option<Mut<'v, T>>;

    #[inline]
    fn transform_component(
        ctx: &SystemRunContext,
        current_archetype: &Archetype,
        entity_index: usize,
        component: ComponentId,
        input: Self::InputOrCreate,
    ) -> Self::Output {
        input.map(|input| {
            let column = current_archetype
                .table
                .columns
                .get(&component)
                .expect("component not found");

            Mut::new(column.change_detection, ctx.tick, entity_index, input)
        })
    }
}

impl<'v> QueryTransform for EntityId {
    const IS_CREATE: bool = true;

    type InputOrCreate = EntityId;
    type Output = EntityId;

    #[inline]
    fn create(
        _ctx: &SystemRunContext,
        current_archetype: &Archetype,
        entity_index: usize,
    ) -> Self::Output {
        current_archetype.entities[entity_index]
    }
}
