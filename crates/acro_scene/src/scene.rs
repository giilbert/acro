use std::any::Any;

use acro_ecs::{EntityId, Name, World};
use acro_math::{Children, Parent, Root};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Scene {
    entities: Vec<SerializedEntity>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerializedEntity {
    name: String,
    components: Vec<ron::Value>,
    children: Vec<SerializedEntity>,
}

impl Scene {
    pub fn load(&self, world: &mut World) {
        world.clear_all_entities();

        let root_entity = world.spawn((Name("Root".to_string()), Root));

        let mut root_children = vec![];
        for entity in &self.entities {
            let entity_id = Self::spawn_entity_with_parent(world, root_entity, entity);
            root_children.push(entity_id);
        }

        world.insert(root_entity, Children(root_children));
    }

    fn spawn_entity_with_parent(
        world: &mut World,
        parent: EntityId,
        entity: &SerializedEntity,
    ) -> EntityId {
        let entity_id = world.spawn((Name(entity.name.clone()),));

        let mut children = vec![];
        for child in &entity.children {
            let child_id = Self::spawn_entity_with_parent(world, entity_id, child);
            children.push(child_id);
        }

        world.insert(entity_id, Children(children));
        world.insert(entity_id, Parent(parent));

        // TODO: init the components of this entity
        println!("{:?}", entity.components);

        entity_id
    }
}

#[cfg(test)]
mod tests {
    use acro_ecs::{Application, Name, Query, World};
    use acro_math::{Children, MathPlugin};
    use tracing::info;

    use super::Scene;

    const TEST_SCENE: &str = r#"
Scene(
    entities: [
        SerializedEntity(
            name: "parent",
            components: [
                Transform(
                    position: Vec3(0.0, 0.0, 0.0),
                    rotation: Vec3(0.0, 0.0, 0.0),
                    scale: Vec3(1.0, 1.0, 1.0) 
                ),
            ],
            children: [
                SerializedEntity(
                    name: "child",
                    components: [
                        Transform(
                            position: Vec3(1.0, 0.0, 0.0),
                            rotation: Vec3(0.0, 0.0, 0.0),
                            scale: Vec3(1.0, 1.0, 1.0) 
                        ),
                    ],
                    children: []
                )
            ],
        )
    ]
)
"#;

    #[test]
    fn scene_loading() {
        let scene: Scene = ron::de::from_str(TEST_SCENE).unwrap();
        let app = Application::new().add_plugin(MathPlugin { scripting: false });
        let mut world = app.world();
        scene.load(&mut world);

        let query = world.query::<(&Name, &Children), ()>();
        for (name, children) in query.over(&*world) {
            println!("{name:?}: {children:?}");
        }

        panic!();
    }
}
