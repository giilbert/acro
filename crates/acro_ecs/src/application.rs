
use crate::{
    plugin::Plugin,
    pointer::change_detection::Tick,
    schedule::{Schedule, Stage, SystemSchedulingRequirement},
    systems::{IntoSystem, SystemData, SystemId, SystemRunContext},
    world::World,
};

pub struct Application {
    world: World,
    current_tick: Tick,
    // systems: Vec<SystemData>,
    schedule: Schedule,
    runner: Box<dyn FnOnce(Application)>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            current_tick: Tick::new(0),
            schedule: Schedule::new(),
            runner: Box::new(|_app| panic!("no runner set!")),
        }
    }

    pub fn add_plugin(mut self, mut plugin: impl Plugin) -> Self {
        plugin.build(&mut self);
        self
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn add_system<I, P>(
        &mut self,
        stage: Stage,
        scheduling_requirements: impl IntoIterator<Item = SystemSchedulingRequirement>,
        system: I,
    ) where
        I: IntoSystem<P>,
        P: 'static,
    {
        let parameters = I::init(&self.world);
        self.schedule
            .add_system(
                stage,
                SystemData {
                    id: SystemId::Native(std::any::TypeId::of::<P>()),
                    name: std::any::type_name_of_val(&system).to_string(),
                    run: system.into_system(),
                    last_run_tick: Tick::new(0),
                    parameters,
                    scheduling_requirements: scheduling_requirements.into_iter().collect(),
                },
            )
            .expect("add system failed");
    }

    pub fn run_once(&mut self) {
        // let now = std::time::Instant::now();
        self.schedule.run_once(&mut self.world);
        // let elapsed = now.elapsed();
        // println!("run once took {:?}", elapsed);
    }

    pub fn set_runner(&mut self, runner: impl FnOnce(Application) + 'static) {
        self.runner = Box::new(runner);
    }

    pub fn run(mut self) {
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
        app.world().init_component::<u32>();
        app.world().init_component::<String>();

        app.world().resources.insert(4u32);

        let entity1 = app.world().spawn_empty();
        app.world().insert(entity1, 42u32);

        let entity2 = app.world().spawn_empty();
        app.world().insert(entity2, "hello".to_string());

        app.add_system(
            Stage::Update,
            [],
            |ctx: SystemRunContext,
             number_query: Query<&u32>,
             string_query: Query<&String>| {
                *ctx.world.resources.get_mut::<u32>() += 1;

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
