use std::{cell::RefCell, rc::Rc};

use crate::{
    archetype::{Archetype, Column},
    world::World,
};

use super::{info::ToFilterInfo, Query, ToQueryInfo};

struct QueryState<'w> {
    pub current_entity_index: usize,
    pub current_archetype_index: usize,
    pub current_archetype: &'w RefCell<Archetype>,
    pub columns: Vec<Option<Rc<Column>>>,
}

pub struct QueryIter<'w, 'q, T, F>
where
    T: for<'a> ToQueryInfo<'a>,
    F: ToFilterInfo,
{
    pub(super) world: &'w World,
    pub(super) query: &'q Query<T, F>,
    state: QueryState<'w>,
}

impl<'w, 'q, T, F> QueryIter<'w, 'q, T, F>
where
    T: for<'a> ToQueryInfo<'a>,
    F: ToFilterInfo,
{
    pub fn new(world: &'w World, query: &'q Query<T, F>) -> Self {
        let current_archetype = world
            .archetypes
            .get_archetype(query.info.archetypes[0])
            .expect("query parent archetype not found");

        Self {
            world,
            query,
            state: QueryState {
                current_entity_index: 0,
                current_archetype_index: 0,
                columns: current_archetype.borrow().get_columns(&query.component_ids),
                current_archetype,
            },
        }
    }
}

impl<'w, T, F> Iterator for QueryIter<'w, '_, T, F>
where
    T: for<'a> ToQueryInfo<'a>,
    F: ToFilterInfo,
{
    type Item = <T as ToQueryInfo<'w>>::Output;

    #[inline]
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
                .get_columns(&self.query.component_ids);

            self.state.current_entity_index = 0;

            return self.next();
        };

        let index = self.state.current_entity_index;
        let current_columns = &self.state.columns;

        let ret =
            Some(unsafe {
                T::from_parts(
                    self.world,
                    &*self.state.current_archetype.borrow(),
                    self.state.current_entity_index,
                    self.query.component_ids.iter().zip(current_columns).map(
                        |(&component_id, c)| {
                            (
                                component_id,
                                c.as_ref()
                                    .map(|column| (&*column.data.get()).get_ptr(index))
                                    .flatten(),
                            )
                        },
                    ),
                )
            });

        self.state.current_entity_index += 1;

        ret
    }
}
