use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use acro_assets::{load_queued_assets, Assets};
use acro_ecs::{
    systems::SystemId, Application, EntityId, Plugin, Stage, SystemSchedulingRequirement, World,
};

mod manager;
mod scene;

use acro_math::{GlobalTransform, Transform};
use acro_scripting::{Behavior, SourceFile};
use eyre::Result;
use manager::load_queued_scene;
pub use manager::SceneManager;
pub use ron;
use tracing::info;

pub type ComponentLoader = fn(&mut World, EntityId, ron::Value) -> Result<()>;

#[derive(Debug)]
pub struct ComponentLoaders {
    pub(crate) loaders: Rc<RefCell<HashMap<String, ComponentLoader>>>,
}

impl Default for ComponentLoaders {
    fn default() -> Self {
        Self {
            loaders: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl ComponentLoaders {
    pub fn register(&self, name: &str, loader: ComponentLoader) {
        self.loaders.borrow_mut().insert(name.to_string(), loader);
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

        let loaders = ComponentLoaders::default();
        loaders.register("Transform", |world, entity, serialized| {
            world.insert(entity, serialized.into_rust::<Transform>()?);
            world.insert(entity, GlobalTransform::default());
            Ok(())
        });
        loaders.register("Behavior", |world, entity, serialized| {
            let behavior = serialized.into_rust::<Behavior>()?;
            world
                .resources()
                .get::<Assets>()
                .queue::<SourceFile>(&behavior.source);
            world.insert(entity, behavior);
            Ok(())
        });
        app.world().insert_resource(loaders);
        app.world().insert_resource(SceneManager::default());
    }
}
