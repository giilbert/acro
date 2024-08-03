use std::any::Any;

use acro_ecs::{world::World, Changed, EntityId, Query, SystemRunContext};
use acro_reflect::{Reflect, ReflectPath, ReflectSetError};
use nalgebra::UnitQuaternion;

use crate::types::{Mat4, Quaternion, Vec3};

#[derive(Debug, Clone, Copy, Reflect, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn get_matrix(&self) -> Mat4 {
        Mat4::new_translation(&self.position)
            * Mat4::new_rotation(self.rotation)
            * Mat4::new_nonuniform_scaling(&self.scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0].into(),
            rotation: [0.0, 0.0, 0.0].into(),
            scale: [1.0, 1.0, 1.0].into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::identity(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Parent(pub EntityId);

#[derive(Debug, Clone)]
pub struct Children(pub Vec<EntityId>);

#[derive(Debug, Clone)]
pub struct Root;

pub fn propagate_global_transform(
    ctx: SystemRunContext,
    transform_query: Query<(EntityId, &Transform, &Children, &Parent), Changed<Transform>>,
    global_transform_query: Query<(EntityId, &mut GlobalTransform, &Children)>,
) {
    for (entity, transform, _children, parent) in transform_query.over(&ctx) {
        recurse_propagate(
            &ctx,
            entity,
            parent.0,
            transform,
            &transform_query,
            &global_transform_query,
        );
    }
}

fn recurse_propagate(
    ctx: &SystemRunContext,
    current_entity: EntityId,
    parent: EntityId,
    transform: &Transform,
    transform_query: &Query<(EntityId, &Transform, &Children, &Parent), Changed<Transform>>,
    global_transform_query: &Query<(EntityId, &mut GlobalTransform, &Children)>,
) {
    // Update the global transform of the current entity
    let (_parent_id, parent_global_transform, _parent_children) = global_transform_query
        .get(&ctx, parent)
        .expect("Invalid tree structure: parent entity does not have a global transform!");
    let (_, mut this_global_transform, children) = global_transform_query
        .get(&ctx, current_entity)
        .expect("Invalid tree structure: entity does not have a global transform!");
    this_global_transform.matrix = parent_global_transform.matrix * transform.get_matrix();

    // Propagate the global transform of this entity to its children
    for child in children.0.iter() {
        recurse_propagate(
            ctx,
            *child,
            current_entity,
            transform_query
                .get(&ctx, *child)
                .expect("Invalid tree structure: child entity does not have a transform!")
                .1,
            transform_query,
            global_transform_query,
        );
    }
}

pub fn register_components(world: &mut World) {
    world.init_component::<Transform>();
    world.init_component::<GlobalTransform>();
    world.init_component::<Parent>();
    world.init_component::<Children>();
    world.init_component::<Root>();
}

#[cfg(test)]
mod tests {
    use acro_ecs::{pointer::change_detection::Tick, world::World};

    use crate::{
        transform::Root,
        types::{Mat4, Quaternion},
    };

    use super::{
        propagate_global_transform, register_components, Children, GlobalTransform, Parent,
        Transform,
    };

    #[test]
    fn transform_propagation() {
        let mut world = World::new();
        register_components(&mut world);

        let root = world.spawn_empty();
        world.insert(
            root,
            Transform {
                position: [0.0, 0.0, 0.0].into(),
                rotation: [0.0, 0.0, 0.0].into(),
                scale: [1.0, 1.0, 1.0].into(),
            },
        );
        world.insert(
            root,
            GlobalTransform {
                matrix: Mat4::identity(),
            },
        );
        world.insert(root, Root);

        let child_1 = world.spawn_empty();
        world.insert(
            child_1,
            Transform {
                position: [0.0, -2.0, 0.0].into(),
                rotation: [0.0, 0.0, 0.0].into(),
                scale: [1.0, 1.0, 1.0].into(),
            },
        );
        world.insert(
            child_1,
            GlobalTransform {
                matrix: Mat4::identity(),
            },
        );
        world.insert(child_1, Parent(root));

        let child_of_child_1 = world.spawn_empty();
        world.insert(
            child_of_child_1,
            Transform {
                position: [0.0, 2.0, 0.0].into(),
                rotation: [0.0, 0.0, 0.0].into(),
                scale: [1.0, 1.0, 1.0].into(),
            },
        );
        world.insert(
            child_of_child_1,
            GlobalTransform {
                matrix: Mat4::identity(),
            },
        );
        world.insert(child_of_child_1, Parent(child_1));
        world.insert(child_of_child_1, Children(vec![]));

        world.insert(root, Children(vec![child_1]));
        world.insert(child_1, Children(vec![child_of_child_1]));

        world.run_system(propagate_global_transform, Tick::new(1));

        let child_1_global_transform = world.get::<GlobalTransform>(child_1).unwrap();
        assert_eq!(
            child_1_global_transform.matrix,
            Transform {
                position: [0.0, -2.0, 0.0].into(),
                rotation: [0.0, 0.0, 0.0].into(),
                scale: [1.0, 1.0, 1.0].into(),
            }
            .get_matrix()
        );

        let child_of_child_1_global_transform =
            world.get::<GlobalTransform>(child_of_child_1).unwrap();
        assert_eq!(
            child_of_child_1_global_transform.matrix,
            Transform {
                position: [0.0, 0.0, 0.0].into(),
                rotation: [0.0, 0.0, 0.0].into(),
                scale: [1.0, 1.0, 1.0].into(),
            }
            .get_matrix()
        );
    }
}
