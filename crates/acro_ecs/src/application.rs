use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use chrono::Utc;
use tracing::info;

use crate::{
    plugin::Plugin,
    pointer::change_detection::Tick,
    schedule::{Schedule, Stage, SystemSchedulingRequirement},
    systems::{IntoSystem, SystemData, SystemId, SystemRunContext},
    world::World,
    EntityId, Res, ResMut,
};

pub struct Application {
    world: Rc<RefCell<World>>,
    current_tick: Tick,
    // systems: Vec<SystemData>,
    schedule: Schedule,
    runner: Box<dyn FnOnce(Application)>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            world: Rc::new(RefCell::new(World::new())),
            current_tick: Tick::new(0),
            schedule: Schedule::new(),
            runner: Box::new(|_app| panic!("no runner set!")),
        }
    }

    pub fn add_plugin(mut self, mut plugin: impl Plugin) -> Self {
        plugin.build(&mut self);
        self
    }

    pub fn world(&self) -> RefMut<World> {
        self.world.borrow_mut()
    }

    pub fn insert_resource<T: Any>(&mut self, resource: T) -> &mut Self {
        self.world.borrow_mut().resources.insert(resource);
        self
    }

    pub fn with_resource<T: Any>(&mut self, f: impl FnOnce(ResMut<T>) -> ()) -> &mut Self {
        f(self.world.borrow().resources.get_mut::<T>());
        self
    }

    pub fn init_component<T: 'static>(&mut self) -> &mut Self {
        self.world.borrow_mut().init_component::<T>();
        self
    }

    pub fn get_world_handle(&self) -> Rc<RefCell<World>> {
        Rc::clone(&self.world)
    }

    pub fn add_system<I, P>(
        &mut self,
        stage: Stage,
        scheduling_requirements: impl IntoIterator<Item = SystemSchedulingRequirement>,
        system: I,
    ) -> &mut Self
    where
        I: IntoSystem<P> + 'static,
        P: 'static,
    {
        let parameters = I::init(&self.world.borrow_mut());
        self.schedule
            .add_system(
                stage,
                SystemData {
                    id: SystemId::Native(system.type_id()),
                    name: std::any::type_name_of_val(&system).to_string(),
                    run: system.into_system(),
                    last_run_tick: Tick::new(0),
                    parameters,
                    scheduling_requirements: scheduling_requirements.into_iter().collect(),
                },
            )
            .expect("add system failed");

        self
    }

    pub fn run_once(&mut self) {
        // let start = Utc::now();
        self.schedule.run_once(&self.world);

        // let elapsed = Utc::now().signed_duration_since(start);
        // tracing::info!("run once took {:?}", elapsed);
    }

    pub fn set_runner(&mut self, runner: impl FnOnce(Application) + 'static) {
        self.runner = Box::new(runner);
    }

    pub fn run(mut self) {
        for system in &self.schedule.stages[&Stage::PreUpdate] {
            info!("{}", system.name);
        }

        let runner = std::mem::replace(
            &mut self.runner,
            Box::new(|_app| panic!("runner replaced!")),
        );
        runner(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::query::Query;

    use super::*;

    #[test]
    fn test_application() {
        let mut app = Application::new();

        {
            let mut world = app.world();

            world.init_component::<u32>();
            world.init_component::<String>();
            world.insert_resource(4u32);

            let entity1 = world.spawn_empty();
            world.insert(entity1, 42u32);

            let entity2 = world.spawn_empty();
            world.insert(entity2, "hello".to_string());
        }

        app.add_system(
            Stage::Update,
            [],
            |ctx: SystemRunContext, number_query: Query<&u32>, string_query: Query<&String>| {
                *ctx.world.resource_mut::<u32>() += 1;

                for value in number_query.over(&ctx) {
                    assert_eq!(value, &42);
                }

                for value in string_query.over(&ctx) {
                    assert_eq!(value, &"hello".to_string());
                }
            },
        );

        app.run_once();
    }
}
