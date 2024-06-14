use std::{
    cell::{Ref, RefCell, UnsafeCell},
    collections::HashSet,
    rc::{Rc, Weak},
};

use fnv::FnvHashSet;

use crate::{
    archetype::{Archetype, ArchetypeId, ArchetypeOperation},
    registry::ComponentId,
    storage::anyvec::AnyVec,
    world::World,
};

use super::{info::ToFilterInfo, Query, ToQueryInfo};

struct ExclusiveQueryState<'w> {
    pub current_index: usize,
    pub current_archetype: &'w RefCell<Archetype>,
    pub columns: Vec<Rc<UnsafeCell<AnyVec>>>,
    pub searched: FnvHashSet<ArchetypeId>,
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
                searched: FnvHashSet::default(),
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

        // If the current archetype has ended, loop through the next archetype
        if self.state.current_index == archetype.entities.len() {
            self.state.searched.insert(archetype.id);
            self.state.to_query.extend(
                self.world
                    .archetypes
                    .edges
                    .get_insert_edges(archetype.id)
                    .filter(|id| !self.state.searched.contains(id)),
            );

            self.state.current_archetype = self
                .world
                .archetypes
                .get_archetype(self.state.to_query.pop()?)
                .expect("archetype should exist");

            self.state.columns = self
                .state
                .current_archetype
                .borrow()
                .get_columns(&self.component_ids);

            self.state.current_index = 0;
        };
        let index = self.state.current_index;

        let current_columns = &self.state.columns;

        let ret = Some(unsafe {
            T::from_parts(current_columns.iter().map(|c| {
                (&*c.get())
                    .get_ptr(index)
                    .expect("column index out of range")
            }))
        });

        self.state.current_index += 1;

        ret
    }
}
