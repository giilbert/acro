use std::{
    cell::{Ref, RefCell, UnsafeCell},
    rc::{Rc, Weak},
};

use crate::{
    archetype::{Archetype, ArchetypeId},
    registry::ComponentId,
    storage::anyvec::AnyVec,
    world::World,
};

use super::{info::ToFilterInfo, Query, ToQueryInfo};

struct ExclusiveQueryState<'w> {
    pub current_index: usize,
    pub current_archetype: &'w RefCell<Archetype>,
    pub columns: Vec<Rc<UnsafeCell<AnyVec>>>,
    pub to_query: Vec<ArchetypeId>,
}

pub struct ExclusiveQueryIter<'w, 'q, T, F>
where
    T: ToQueryInfo,
    F: ToFilterInfo,
{
    pub(super) world: &'w World,
    pub(super) query: &'q Query<T, F>,
    component_ids: Vec<ComponentId>,
    state: ExclusiveQueryState<'w>,
}

impl<'w, 'q, T, F> ExclusiveQueryIter<'w, 'q, T, F>
where
    T: ToQueryInfo,
    F: ToFilterInfo,
{
    pub fn new(world: &'w mut World, query: &'q Query<T, F>) -> Self {
        let current_archetype = world
            .archetypes
            .get_archetype(query.info.parent_archetype_id)
            .expect("query parent archetype not found");
        let component_ids = query
            .info
            .components
            .iter()
            .map(|c| c.component_info().id)
            .collect::<Vec<ComponentId>>();

        Self {
            world,
            query,
            state: ExclusiveQueryState {
                current_index: 0,
                columns: current_archetype.borrow().get_columns(&component_ids),
                current_archetype,
                to_query: vec![],
            },
            component_ids,
        }
    }
}

impl<T, F> Iterator for ExclusiveQueryIter<'_, '_, T, F>
where
    T: ToQueryInfo,
    F: ToFilterInfo,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let archetype = self.state.current_archetype.borrow();
        if self.state.current_index >= archetype.entities.len() {
            return None;
        }

        // TODO: search other archetypes

        let index = self.state.current_index;
        self.state.current_index += 1;

        Some(unsafe {
            T::from_parts(self.state.columns.iter().map(|c| {
                (&*c.get())
                    .get_ptr(index)
                    .expect("column index out of range")
            }))
        })
    }
}
