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

pub type ComponentLoader = fn(&mut World, EntityId, serde_yml::Value) -> Result<()>;

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
        let loaders = ComponentLoaders::default();
        loaders.register("Transform", |world, entity, serialized| {
            world.insert(entity, serde_yml::from_value::<Transform>(serialized)?);
            world.insert(entity, GlobalTransform::default());
            Ok(())
        });
        loaders.register("Behavior", |world, entity, serialized| {
            let behavior = serde_yml::from_value::<Behavior>(serialized)?;
            world
                .resources()
                .get::<Assets>()
                .queue::<SourceFile>(&behavior.source);
            world.insert(entity, behavior);
            Ok(())
        });

        app.insert_resource(loaders)
            .insert_resource(SceneManager::default())
            .add_system(
                Stage::PreUpdate,
                [SystemSchedulingRequirement::RunBefore(SystemId::Native(
                    load_queued_assets.type_id(),
                ))],
                load_queued_scene,
            );
    }
}
