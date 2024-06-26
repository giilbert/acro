use std::{any::Any, cell::UnsafeCell};

use crate::{plugin::Plugin, world::World};

struct System {
    pub name: String,
    pub run: Box<dyn Fn(&mut World, &mut dyn Any)>,
    pub parameters: Box<dyn Any>,
}

pub struct Application {
    world: World,
    systems: Vec<System>,
    runner: Box<dyn FnOnce(Application)>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            systems: Vec::new(),
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

    pub fn add_system<T: Any>(
        &mut self,
        system_init: impl FnOnce(&mut Application) -> T,
        system: impl Fn(&mut World, &mut T) -> () + 'static,
    ) {
        let parameters = system_init(self);
        self.systems.push(System {
            name: std::any::type_name_of_val(&system).to_string(),
            run: Box::new(move |world, parameters| {
                system(world, parameters.downcast_mut().unwrap())
            }),
            parameters: Box::new(parameters),
        });
    }

    pub fn run_once(&mut self) {
        // let now = std::time::Instant::now();
        for system in self.systems.iter_mut() {
            (system.run)(&mut self.world, system.parameters.as_mut());
        }
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
    use super::*;

    #[test]
    fn test_application() {
        let mut app = Application::new();
        app.world().init_component::<u32>();
        app.world().init_component::<String>();

        app.world().resources.insert(4u32);

        let entity1 = app.world().spawn();
        app.world().insert(entity1, 42u32);

        let entity2 = app.world().spawn();
        app.world().insert(entity2, "hello".to_string());

        app.add_system(
            |app| {
                (
                    app.world.query::<(&u32,), ()>(),
                    app.world.query::<(&String,), ()>(),
                )
            },
            |world: &mut World, (number_query, string_query)| {
                *world.resources.get_mut::<u32>() += 1;

                for (value,) in number_query.over(world) {
                    assert_eq!(value, &42);
                }

                for (value,) in string_query.over(world) {
                    assert_eq!(value, &"hello".to_string());
                }
            },
        );

        app.run_once();
    }
}
