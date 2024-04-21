use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::entity::EntityId;

#[derive(Debug)]
pub struct Storage {
    id: TypeId,
    map: HashMap<EntityId, Box<dyn Any>, fnv::FnvBuildHasher>,
}

impl Storage {
    pub fn new(id: TypeId) -> Self {
        Self {
            id,
            map: HashMap::default(),
        }
    }

    pub fn insert<T: 'static>(&mut self, entity: EntityId, component: T) {
        assert!(TypeId::of::<T>() == self.id, "type mismatch");
        self.map.insert(entity, Box::new(component));
    }

    pub fn get<T: Any>(&self, entity: EntityId) -> Option<&T> {
        self.map
            .get(&entity)
            .map(|c| c.as_ref().downcast_ref::<T>())
            .flatten()
    }

    pub unsafe fn get_unchecked<T: Any>(&self, entity: EntityId) -> Option<&T> {
        self.map
            .get(&entity)
            .map(|inner| unsafe { &*(inner.as_ref() as *const dyn Any as *const T) })
    }

    pub fn downcast<T: Any>(&mut self) -> Option<DowncastedStorage<T>> {
        if self.id == TypeId::of::<T>() {
            Some(DowncastedStorage::new(self))
        } else {
            None
        }
    }
}

pub struct DowncastedStorage<'s, T: Any> {
    pub storage: &'s mut Storage,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Any> DowncastedStorage<'a, T> {
    pub fn new(storage: &'a mut Storage) -> Self {
        Self {
            storage,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn insert(&mut self, entity: EntityId, component: T) {
        self.storage.insert(entity, Box::new(component));
    }

    pub fn get(&self, entity: EntityId) -> Option<&T> {
        unsafe { self.storage.get_unchecked(entity) }
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::entity::EntityId;

    use super::Storage;

    #[test]
    pub fn storage_test() {
        let mut storage = Storage::new(TypeId::of::<String>());
        storage.insert(EntityId(0), "hello".to_string());
        storage.insert(EntityId(1), "bye".to_string());
        let downcasted = storage.downcast::<String>().expect("downcast failed");

        assert_eq!(downcasted.get(EntityId(0)).unwrap(), "hello");
    }
}
