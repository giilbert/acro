use acro_ecs::{EntityId, Name, Query, With, World};

use crate::{Children, Root};

pub struct TreeData {
    root_query: Query<(EntityId, &'static Children), With<Root>>,
    children_query: Query<(EntityId, &'static Name, &'static Children)>,
}

impl TreeData {
    pub fn new(world: &World) -> Self {
        Self {
            root_query: world.query::<(EntityId, &Children), With<Root>>(),
            children_query: world.query::<(EntityId, &Name, &Children), ()>(),
        }
    }

    pub fn get_entity_by_path(&self, world: &World, path: &str) -> Option<EntityId> {
        let (mut current_entity, mut current_entity_children) = self.root_query.single(world);

        for part in path.trim_matches('/').split('/') {
            if part == "" {
                continue;
            }

            let (next_entity_id, next_children) =
                current_entity_children.0.iter().find_map(|&children_id| {
                    let (entity_id, name, children) = self
                        .children_query
                        .get(world, children_id)
                        .expect("children not found");

                    if name.0 == part {
                        Some((entity_id, children))
                    } else {
                        None
                    }
                })?;

            current_entity = next_entity_id;
            current_entity_children = next_children;
        }

        Some(current_entity)
    }
}

pub trait WorldTreeExt {
    fn get_entity_by_absolute_path(&self, path: &str) -> Option<EntityId>;
}

impl WorldTreeExt for World {
    fn get_entity_by_absolute_path(&self, path: &str) -> Option<EntityId> {
        self.resource::<TreeData>().get_entity_by_path(self, path)
    }
}

#[cfg(test)]
mod tests {
    use acro_ecs::{Name, World};

    use crate::{tree::TreeData, Children, GlobalTransform, Parent, Root, Transform};

    use super::WorldTreeExt;

    #[test]
    fn tree_test() {
        let mut world = World::new();
        world.init_component::<Name>();
        world.init_component::<Transform>();
        world.init_component::<GlobalTransform>();
        world.init_component::<Root>();
        world.init_component::<Parent>();
        world.init_component::<Children>();
        world.insert_resource(TreeData::new(&world));

        let root = world.spawn((
            Name("root".to_string()),
            Transform::default(),
            GlobalTransform::default(),
            Root,
        ));

        let child_1 = world.spawn((
            Name("child_1".to_string()),
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        let child_2 = world.spawn((
            Name("child_2".to_string()),
            Transform::default(),
            GlobalTransform::default(),
            Parent(child_1),
            Children(vec![]),
        ));

        world.insert(child_1, Children(vec![child_2]));
        world.insert(root, Children(vec![child_1]));

        assert_eq!(
            world
                .get_entity_by_absolute_path("/")
                .expect("entity not found"),
            root
        );
        assert_eq!(
            world
                .get_entity_by_absolute_path("/child_1")
                .expect("entity not found"),
            child_1
        );
        assert_eq!(
            world
                .get_entity_by_absolute_path("/child_1/child_2")
                .expect("entity not found"),
            child_2
        );
    }
}
