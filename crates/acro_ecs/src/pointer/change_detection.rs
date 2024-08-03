use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tick(u32);

impl Tick {
    pub fn new(tick: u32) -> Self {
        Self(tick)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn is_newer_than(&self, other: &Self) -> bool {
        self.0 > other.0
    }
}

#[derive(Debug, Default)]
pub struct ChangeDetectionContext {
    pub changed_ticks: Vec<Tick>,
}

#[derive(Debug)]
pub struct Mut<'v, T> {
    current_tick: Tick,
    index: usize,
    context: &'static UnsafeCell<ChangeDetectionContext>,
    value: &'v mut T,
}

impl<'v, T> Mut<'v, T> {
    pub fn new(
        context: &'static UnsafeCell<ChangeDetectionContext>,
        current_tick: Tick,
        index: usize,
        value: &'v mut T,
    ) -> Self {
        Self {
            current_tick,
            index,
            context,
            value,
        }
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
            context.changed_ticks[self.index] = self.current_tick;
        }
        self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        archetype::ArchetypeId, pointer::change_detection::Tick, query::Query,
        registry::ComponentId, schedule::Stage, systems::SystemRunContext, Application,
    };

    #[test]
    fn test_change_detection() {
        let mut app = Application::new();

        let component_id = app.world().init_component::<u32>().id;

        app.world().spawn((40u32,));
        app.world().spawn((42u32,));

        app.add_system(
            Stage::Update,
            [],
            |ctx: SystemRunContext, number_query: Query<&mut u32>| {
                for mut value in number_query.over(&ctx) {
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
                    .get(&component_id)
                    .unwrap()
                    .change_detection
                    .get()
            }
            .changed_ticks,
            vec![Tick(1), Tick(2)]
        );
    }
}
