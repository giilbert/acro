use acro_ecs::{
    entity::EntityId,
    query::{Changed, Query},
    systems::SystemRunContext,
    world::World,
};

use crate::types::{Mat4, UnitQuaternion, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: UnitQuaternion,
    pub scale: Vec3,
}

impl Transform {
    pub fn get_matrix(&self) -> Mat4 {
        Mat4::new_translation(&self.position)
            * Mat4::from(self.rotation)
            * Mat4::new_nonuniform_scaling(&self.scale)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

#[derive(Debug, Clone, Copy)]
pub struct Parent(pub EntityId);

#[derive(Debug, Clone)]
pub struct Children(pub Vec<EntityId>);

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

    dbg!(this_global_transform.matrix);

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

fn setup_transform(world: &mut World) {
    world.init_component::<Transform>();
    world.init_component::<GlobalTransform>();
    world.init_component::<Parent>();
    world.init_component::<Children>();
}

#[cfg(test)]
mod tests {
    use acro_ecs::{pointer::change_detection::Tick, world::World};

    use crate::types::{Mat4, UnitQuaternion};

    use super::{
        propagate_global_transform, setup_transform, Children, GlobalTransform, Parent, Transform,
    };

    #[test]
    fn transform_propagation() {
        let mut world = World::new();
        setup_transform(&mut world);

        let root = world.spawn();
        world.insert(
            root,
            Transform {
                position: [0.0, 0.0, 0.0].into(),
                rotation: UnitQuaternion::identity(),
                scale: [1.0, 1.0, 1.0].into(),
            },
        );
        world.insert(
            root,
            GlobalTransform {
                matrix: Mat4::identity(),
            },
        );

        let child1 = world.spawn();
        world.insert(
            child1,
            Transform {
                position: [0.0, 0.0, 0.0].into(),
                rotation: UnitQuaternion::identity(),
                scale: [1.0, 1.0, 1.0].into(),
            },
        );
        world.insert(
            child1,
            GlobalTransform {
                matrix: Mat4::identity(),
            },
        );
        world.insert(child1, Parent(root));
        world.insert(child1, Children(vec![]));

        world.insert(root, Children(vec![child1]));

        world.run_system(propagate_global_transform, Tick::new(1));

        let global_transform = world.get::<GlobalTransform>(child1).unwrap();
        assert_eq!(global_transform.matrix, Mat4::identity());
    }
}
