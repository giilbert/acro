mod transform;
mod types;

pub use crate::{
    transform::{propagate_global_transform, Children, GlobalTransform, Parent, Root, Transform},
    types::{Mat4, UnitQuaternion},
};

use acro_ecs::{schedule::Stage, Application, Plugin};
use transform::register_components;

pub struct MathPlugin;

impl Plugin for MathPlugin {
    fn build(&mut self, app: &mut Application) {
        register_components(app.world());
        app.add_system(Stage::PostUpdate, [], propagate_global_transform);
    }
}
