use std::{cell::UnsafeCell, os::unix::thread};

use crate::{
    archetype::Archetype,
    entity::EntityId,
    pointer::change_detection::{self, ChangeDetectionContext, ChangeDetectionId, Mut},
    registry::ComponentId,
    world::World,
};

/// A trait for types that can be fetched from a query.
/// 'w is the lifetime of the world or data borrowed from it.
pub trait QueryTransform<'w> {
    const IS_CREATE: bool = false;

    type InputOrCreate;
    type Output;

    #[inline]
    fn create(
        _world: &World,
        _current_archetype: &Archetype,
        _entity_index: usize,
    ) -> Self::Output {
        unimplemented!("create is not available for this type");
    }

    #[inline]
    fn transform_component(
        _world: &World,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        _input: Self::InputOrCreate,
    ) -> Self::Output {
        unimplemented!("create is not available for this type");
    }
}

impl<'w, 'v, T> QueryTransform<'w> for &'v T {
    type InputOrCreate = &'v T;
    type Output = Self::InputOrCreate;

    #[inline]
    fn transform_component(
        _world: &World,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        input: Self::InputOrCreate,
    ) -> Self::Output {
        input
    }
}

impl<'w, 'v, T> QueryTransform<'w> for Option<&'v T> {
    type InputOrCreate = Option<&'v T>;
    type Output = Self::InputOrCreate;

    #[inline]
    fn transform_component(
        _world: &World,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        input: Self::InputOrCreate,
    ) -> Self::Output {
        input
    }
}

impl<'w, 'v, T> QueryTransform<'w> for &'v mut T {
    type InputOrCreate = &'v mut T;
    type Output = Mut<'v, T>;

    #[inline]
    fn transform_component(
        _world: &World,
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
        Mut::new(
            column.change_detection,
            ChangeDetectionId(entity_index),
            input,
        )
    }
}

impl<'w, 'v, T> QueryTransform<'w> for Option<&'v mut T> {
    type InputOrCreate = Option<&'v mut T>;
    type Output = Option<Mut<'v, T>>;

    #[inline]
    fn transform_component(
        _world: &World,
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

            Mut::new(
                column.change_detection,
                ChangeDetectionId(entity_index),
                input,
            )
        })
    }
}

impl<'w, 'v> QueryTransform<'w> for EntityId {
    const IS_CREATE: bool = true;

    type InputOrCreate = EntityId;
    type Output = EntityId;

    #[inline]
    fn create(_world: &World, current_archetype: &Archetype, entity_index: usize) -> Self::Output {
        current_archetype.entities[entity_index]
    }
}
