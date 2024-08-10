use std::{any::Any, collections::HashMap};

use acro_ecs::{EntityId, Name, World};
use acro_math::{Children, GlobalTransform, Parent, Root, Transform};
use tracing::warn;

use crate::{ComponentLoader, ComponentLoaders};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    entities: Vec<Entity>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Entity {
    name: String,
    #[serde(default)]
    components: Vec<Component>,
    #[serde(default)]
    children: Vec<Entity>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub name: String,
    #[serde(flatten)]
    data: serde_yml::Value,
}

impl Scene {
    pub fn load(self, world: &mut World) {
        world.clear_all_entities();

        let root_entity = world.spawn((
            Name("Root".to_string()),
            Root,
            Transform::default(),
            GlobalTransform::default(),
        ));

        let component_loaders = world.resources().get::<ComponentLoaders>().loaders.clone();
        let component_loaders = &*component_loaders.borrow();

        let mut root_children = vec![];
        for entity in self.entities.into_iter() {
            let entity_id =
                Self::spawn_entity_with_parent(world, root_entity, entity, component_loaders);
            root_children.push(entity_id);
        }

        world.insert(root_entity, Children(root_children));
    }

    fn spawn_entity_with_parent(
        world: &mut World,
        parent: EntityId,
        entity: Entity,
        component_loaders: &HashMap<String, ComponentLoader>,
    ) -> EntityId {
        let entity_id = world.spawn((Name(entity.name.clone()),));

        let mut children = vec![];
        for child in entity.children.into_iter() {
            let child_id =
                Self::spawn_entity_with_parent(world, entity_id, child, component_loaders);
            children.push(child_id);
        }

        world.insert(entity_id, Children(children));
        world.insert(entity_id, Parent(parent));

        // println!("{:?}", entity.components);

        for component in entity.components.into_iter() {
            if let Some(loader) = component_loaders.get(&component.name) {
                let result = loader(world, entity_id, component.data);
                if let Err(err) = result {
                    warn!("Failed to load component `{}`: {:?}", component.name, err);
                }
            } else {
                warn!("No loader for component `{}`. Ignoring..", component.name);
            }
        }

        entity_id
    }
}

#[cfg(test)]
mod tests {
    use acro_ecs::{Application, Name, Query, World};
    use acro_math::{Children, MathPlugin};
    use tracing::info;

    use crate::ScenePlugin;

    use super::Scene;

    const TEST_SCENE: &str = r#"
    entities:
      - name: parent
        components:
          - name: Transform
            position: [0.0, 0.0, 0.0]
            rotation: [0.0, 0.0, 0.0]
            scale: [1.0, 1.0, 1.0]
        children:
          - name: child
            components:
              - name: Transform
                position: [1.0, 0.0, 0.0]
                rotation: [0.0, 0.0, 0.0]
                scale: [1.0, 1.0, 1.0]
            children: []
"#;

    #[test]
    fn scene_loading() {
        let scene: Scene = serde_yml::from_str(TEST_SCENE).unwrap();
        let app = Application::new()
            .add_plugin(ScenePlugin)
            .add_plugin(MathPlugin { scripting: false });
        let mut world = app.world();
        scene.load(&mut world);

        let query = world.query::<(&Name, &Children), ()>();
        for (name, children) in query.over(&*world) {
            println!("{name:?}: {children:?}");
        }
    }
}
