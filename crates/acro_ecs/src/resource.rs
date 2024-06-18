use std::any::{Any, TypeId};

use fnv::FnvHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub usize);

#[derive(Debug)]
pub struct ResourceRegistry {
    data: Vec<Box<dyn Any>>,
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
        self.data.push(Box::new(resource));
        self.types.insert(TypeId::of::<T>(), id);
        id
    }

    pub fn get<T: 'static>(&self) -> &T {
        self.data[self
            .types
            .get(&TypeId::of::<T>())
            .expect("resource not found")
            .0]
            .downcast_ref()
            .unwrap()
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.data[self
            .types
            .get(&TypeId::of::<T>())
            .expect("resource not found")
            .0]
            .downcast_mut()
            .unwrap()
    }
}
