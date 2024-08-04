use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
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

    pub fn get<T: 'static>(&self) -> Res<T> {
        Res {
            inner: Ref::map(
                self.data[self
                    .types
                    .get(&TypeId::of::<T>())
                    .unwrap_or_else(|| panic!("resource {} not found", std::any::type_name::<T>()))
                    .0]
                    .borrow(),
                |r| r.downcast_ref().unwrap(),
            ),
        }
    }

    pub fn get_mut<T: 'static>(&self) -> ResMut<T> {
        ResMut {
            inner: RefMut::map(
                self.data[self
                    .types
                    .get(&TypeId::of::<T>())
                    .unwrap_or_else(|| panic!("resource {} not found", std::any::type_name::<T>()))
                    .0]
                    .borrow_mut(),
                |r| r.downcast_mut().unwrap(),
            ),
        }
    }
}

pub struct Res<'b, T> {
    pub(crate) inner: Ref<'b, T>,
}

impl<T> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ResMut<'b, T> {
    inner: RefMut<'b, T>,
}

impl<T> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
