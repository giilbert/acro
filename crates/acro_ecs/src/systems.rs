use crate::{pointer::change_detection::Tick, world::World};

#[derive(Debug, Clone)]
pub struct SystemRunContext<'w> {
    pub world: &'w World,
    pub tick: Tick,
}

impl SystemRunContext<'_> {
    pub fn new(world: &World, tick: Tick) -> SystemRunContext {
        SystemRunContext { world, tick }
    }

    pub fn ignore_changes(world: &World) -> SystemRunContext {
        SystemRunContext::new(world, Tick::new(0))
    }
}

pub trait IntoSystemRunContext<'w> {
    fn into_system_run_context(self) -> SystemRunContext<'w>;
}

impl<'w> IntoSystemRunContext<'w> for &'w World {
    fn into_system_run_context(self) -> SystemRunContext<'w> {
        SystemRunContext::new(self, Tick::new(0))
    }
}

impl<'w> IntoSystemRunContext<'w> for SystemRunContext<'w> {
    fn into_system_run_context(self) -> SystemRunContext<'w> {
        self
    }
}

impl<'w> IntoSystemRunContext<'w> for &'w SystemRunContext<'w> {
    fn into_system_run_context(self) -> SystemRunContext<'w> {
        self.clone()
    }
}
