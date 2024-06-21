use std::{any::TypeId, cell::UnsafeCell, collections::HashMap, rc::Rc};

use fnv::FnvHashMap;

use crate::{
    archetype::Column,
    entity::EntityId,
    registry::{ComponentGroup, ComponentId, ComponentInfo, ComponentType},
};

use super::anyvec::AnyVec;

#[derive(Debug)]
pub struct Table {
    length: usize,
    components: ComponentGroup,
    pub columns: FnvHashMap<ComponentId, Rc<Column>>,
}

impl Table {
    pub fn new(components: ComponentGroup) -> Self {
        Self {
            length: 0,
            columns: components
                .iter()
                .map(|info| match info.component_type {
                    ComponentType::Native {
                        layout, dropper, ..
                    } => (
                        info.id,
                        Rc::new(Column::new(UnsafeCell::new(AnyVec::new(
                            layout, dropper, 1,
                        )))),
                    ),
                })
                .collect(),
            components,
        }
    }

    pub unsafe fn push_row(
        &mut self,
        component_data: impl Iterator<Item = (ComponentId, *const u8)>,
    ) {
        self.length += 1;
        for (component_id, data) in component_data {
            let column = self
                .columns
                .get_mut(&component_id)
                .expect("column not found");
            unsafe { (*column.data.get()).push_from_ptr(data) };
        }
    }

    /// Swap removes a row from the table, returning the index of the element which replaced the
    /// removed element if it's not the last element.
    pub fn remove_row(&mut self, index: usize) -> Option<usize> {
        let last_index = self.length - 1;

        for column in self.columns.values_mut() {
            unsafe {
                (*column.data.get()).swap_remove(index);
            }
        }

        self.length -= 1;

        // If the removed element was the last element, there is no replacement
        if last_index == index {
            None
        } else {
            Some(last_index)
        }
    }
}
