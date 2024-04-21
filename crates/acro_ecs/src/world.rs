use std::{
    cell::RefCell,
    rc::Rc,
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

    pub fn storage<T: 'static>(&self) -> Rc<RefCell<Storage>> {
        self.components.storage::<T>().unwrap()
    }
}

#[cfg(test)]
mod tests {
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

        let storage_a = world.storage::<A>();
        let storage_b = world.storage::<B>();

        let entity = world.spawn();

        storage_a.borrow_mut().insert(
            entity,
            A {
                text: "hello".to_string(),
            },
        );
        storage_b.borrow_mut().insert(entity, B { number: 0 });
    }
}
