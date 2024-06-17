use std::{
    cell::{Ref, RefCell, UnsafeCell},
    collections::HashSet,
    rc::{Rc, Weak},
};

use crate::{
    archetype::{Archetype, ArchetypeId, ArchetypeOperation},
    registry::ComponentId,
    storage::anyvec::AnyVec,
    world::World,
};

use super::{info::ToFilterInfo, Query, ToQueryInfo};

struct ExclusiveQueryState<'w> {
    pub current_entity_index: usize,
    pub current_archetype_index: usize,
    pub current_archetype: &'w RefCell<Archetype>,
    pub columns: Vec<Rc<UnsafeCell<AnyVec>>>,
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
            .get_archetype(query.info.archetypes[0])
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
                current_entity_index: 0,
                current_archetype_index: 0,
                columns: current_archetype.borrow().get_columns(&component_ids),
                current_archetype,
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

        // If the current archetype has ended, loop through the next archetype
        if self.state.current_entity_index == archetype.entities.len() {
            // If we are at the last archetype, return None to end the iterator
            if self.state.current_archetype_index == self.query.info.archetypes.len() - 1 {
                return None;
            }

            // Move to the next archetype
            self.state.current_archetype_index += 1;
            self.state.current_archetype = self
                .world
                .archetypes
                .get_archetype(self.query.info.archetypes[self.state.current_archetype_index])
                .expect("query archetype not found");

            // Update the columns information to be from the new archetype
            self.state.columns = self
                .state
                .current_archetype
                .borrow()
                .get_columns(&self.component_ids);

            self.state.current_entity_index = 0;
        };

        let index = self.state.current_entity_index;
        let current_columns = &self.state.columns;

        let ret = Some(unsafe {
            T::from_parts(current_columns.iter().map(|c| {
                (&*c.get())
                    .get_ptr(index)
                    .expect("column index out of range")
            }))
        });

        self.state.current_entity_index += 1;

        ret
    }
}
