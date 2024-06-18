use std::{any::Any, cell::UnsafeCell};

use crate::world::World;

struct System {
    pub name: String,
    pub run: Box<dyn Fn(&World, &mut dyn Any)>,
    pub parameters: Box<dyn Any>,
}

pub struct Application {
    world: World,
    systems: Vec<System>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            systems: Vec::new(),
        }
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn add_system<T: Any>(
        &mut self,
        system_init: impl FnOnce(&mut Application) -> T,
        system: impl Fn(&World, &mut T) -> () + 'static,
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
        for system in self.systems.iter_mut() {
            (system.run)(&self.world, system.parameters.as_mut());
        }
    }

    pub fn run(mut self) {
        self.run_once();
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
            |world: &World, (number_query, string_query)| {
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
