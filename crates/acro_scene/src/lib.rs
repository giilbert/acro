use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use acro_assets::load_queued_assets;
use acro_ecs::{
    systems::SystemId, Application, EntityId, Plugin, Stage, SystemSchedulingRequirement, World,
};

mod manager;
mod scene;

use acro_math::{GlobalTransform, Transform};
use eyre::Result;
use manager::load_queued_scene;
pub use manager::SceneManager;
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
        app.add_system(
            Stage::PreUpdate,
            [SystemSchedulingRequirement::RunBefore(SystemId::Native(
                load_queued_assets.type_id(),
            ))],
            load_queued_scene,
        );

        let mut loaders = ComponentLoaders::default();
        loaders.register("Transform", |world, entity, serialized| {
            world.insert(entity, serialized.into_rust::<Transform>()?);
            world.insert(entity, GlobalTransform::default());
            Ok(())
        });
        app.world().insert_resource(loaders);
        app.world().insert_resource(SceneManager::default());
    }
}
