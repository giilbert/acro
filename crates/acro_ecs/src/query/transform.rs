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
    type Input;
    type Output;

    #[inline]
    fn transform(
        world: &World,
        current_archetype: &Archetype,
        entity_index: usize,
        component: ComponentId,
        input: Self::Input,
    ) -> Self::Output;
}

impl<'w, 'v, T> QueryTransform<'w> for &'v T {
    type Input = &'v T;
    type Output = &'v T;

    #[inline]
    fn transform(
        _world: &World,
        _current_archetype: &Archetype,
        _entity_index: usize,
        _component: ComponentId,
        input: Self::Input,
    ) -> Self::Output {
        input
    }
}

impl<'w, 'v, T> QueryTransform<'w> for &'v mut T {
    type Input = &'v mut T;
    type Output = Mut<'v, T>;

    #[inline]
    fn transform(
        world: &World,
        current_archetype: &Archetype,
        entity_index: usize,
        component: ComponentId,
        input: Self::Input,
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
