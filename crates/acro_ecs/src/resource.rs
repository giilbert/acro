use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
};

use fnv::FnvHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub usize);

#[derive(Debug)]
pub struct ResourceRegistry {
    data: Vec<RefCell<Box<dyn Any>>>,
    types: FnvHashMap<TypeId, ResourceId>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            data: vec![],
            types: FnvHashMap::default(),
        }
    }

    pub fn insert<T: 'static>(&mut self, resource: T) -> ResourceId {
        let id = ResourceId(self.data.len());
        self.data.push(RefCell::new(Box::new(resource)));
        self.types.insert(TypeId::of::<T>(), id);
        id
    }

    pub fn get<T: 'static>(&self) -> Ref<T> {
        Ref::map(
            self.data[self
                .types
                .get(&TypeId::of::<T>())
                .expect("resource not found")
                .0]
                .borrow(),
            |r| r.downcast_ref().unwrap(),
        )
    }

    pub fn get_mut<T: 'static>(&self) -> RefMut<T> {
        RefMut::map(
            self.data[self
                .types
                .get(&TypeId::of::<T>())
                .expect("resource not found")
                .0]
                .borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        )
    }
}
