use std::{cell::RefCell, collections::HashMap, rc::Rc};

use acro_ecs::{Application, EntityId, Plugin, World};

mod scene;

use acro_math::{GlobalTransform, Transform};
use eyre::Result;
pub use ron;

pub type ComponentLoader = fn(&mut World, EntityId, ron::Value) -> Result<()>;

#[derive(Debug)]
pub struct ComponentLoaders {
    loaders: RefCell<Option<HashMap<String, ComponentLoader>>>,
}

impl Default for ComponentLoaders {
    fn default() -> Self {
        Self {
            loaders: RefCell::new(Some(HashMap::new())),
        }
    }
}

impl ComponentLoaders {
    pub fn register(&mut self, name: &str, loader: ComponentLoader) {
        self.loaders
            .borrow_mut()
            .as_mut()
            .expect("ComponentLoaders already taken")
            .insert(name.to_string(), loader);
    }

    pub fn take(&self) -> HashMap<String, ComponentLoader> {
        self.loaders
            .borrow_mut()
            .take()
            .expect("ComponentLoaders already taken")
    }
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&mut self, app: &mut Application) {
        let mut loaders = ComponentLoaders::default();
        loaders.register("Transform", |world, entity, serialized| {
            world.insert(entity, serialized.into_rust::<Transform>()?);
            world.insert(entity, GlobalTransform::default());
            Ok(())
        });
        app.world().insert_resource(loaders);
    }
}
