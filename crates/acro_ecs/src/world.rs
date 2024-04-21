use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{entity::EntityId, registry::ComponentRegistry, storage::Storage};

#[derive(Default)]
pub struct World {
    next_id: AtomicUsize,
    components: ComponentRegistry,
}

impl World {
    pub fn init_component<T: 'static>(&mut self, name: impl ToString) {
        self.components.init_component::<T>(name.to_string());
    }

    pub fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_id.fetch_add(1, Ordering::Relaxed));
        id
    }

    pub fn storage<T: 'static>(&mut self) -> &RefCell<Storage> {
        self.components.storage::<T>().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use super::World;

    pub struct A {
        text: String,
    }

    pub struct B {
        number: u32,
    }

    #[test]
    pub fn create_world_with_entities() {
        let mut world = World::default();
        world.init_component::<A>("A");
        world.init_component::<B>("B");
        let entity = world.spawn();
        world.storage::<A>().borrow_mut().insert(
            entity,
            A {
                text: "hello".to_string(),
            },
        );
        world
            .storage::<B>()
            .borrow_mut()
            .insert(entity, B { number: 0 });
    }
}
