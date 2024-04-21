use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::storage::Storage;

#[derive(Debug, Default)]
pub struct ComponentRegistry {
    components: HashMap<TypeId, ComponentInfo>,
    storages: HashMap<TypeId, Rc<RefCell<Storage>>>,
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    name: String,
    id: TypeId,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::default(),
            storages: HashMap::default(),
        }
    }

    pub fn init_component<T: 'static>(&mut self, name: String) {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Rc::new(RefCell::new(Storage::new(TypeId::of::<T>()))));
        self.components.insert(
            TypeId::of::<T>(),
            ComponentInfo {
                name,
                id: TypeId::of::<T>(),
            },
        );
    }

    pub fn storage<T: 'static>(&self) -> Option<Rc<RefCell<Storage>>> {
        self.storages.get(&TypeId::of::<T>()).cloned()
    }
}
