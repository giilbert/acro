use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChangeDetectionId(pub usize);

#[derive(Debug, Default)]
pub struct ChangeDetectionContext {
    pub changes: Vec<ChangeDetectionId>,
}

#[derive(Debug)]
pub struct Mut<'v, T> {
    id: ChangeDetectionId,
    context: &'static UnsafeCell<ChangeDetectionContext>,
    value: &'v mut T,
}

impl<'v, T> Mut<'v, T> {
    pub fn new(
        context: &'static UnsafeCell<ChangeDetectionContext>,
        id: ChangeDetectionId,
        value: &'v mut T,
    ) -> Self {
        Self { id, context, value }
    }
}

impl<T: PartialEq<T>> PartialEq<T> for Mut<'_, T> {
    fn eq(&self, other: &T) -> bool {
        self.value == other
    }
}

impl<'v, T> Deref for Mut<'v, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'v, T> DerefMut for Mut<'v, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let context = &mut *self.context.get();
            context.changes.push(self.id);
        }
        self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        archetype::ArchetypeId, pointer::change_detection::ChangeDetectionId,
        registry::ComponentId, world::World, Application,
    };

    #[test]
    fn test_change_detection() {
        let mut app = Application::new();

        app.world().init_component::<u32>();

        let entity1 = app.world().spawn();
        app.world().insert(entity1, 40u32);
        let entity2 = app.world().spawn();
        app.world().insert(entity2, 42u32);

        app.add_system(
            |app| (app.world().query::<&mut u32, ()>(),),
            |world: &mut World, (number_query,)| {
                for (mut value,) in number_query.over(world) {
                    if value == 42 {
                        *value = 20;
                    }
                }
            },
        );

        app.run_once();

        assert_eq!(
            unsafe {
                &*app
                    .world()
                    .archetypes
                    .get_archetype(ArchetypeId(1))
                    .unwrap()
                    .borrow()
                    .table
                    .columns
                    .get(&ComponentId(0))
                    .unwrap()
                    .change_detection
                    .get()
            }
            .changes,
            vec![ChangeDetectionId(1)]
        );
    }
}
