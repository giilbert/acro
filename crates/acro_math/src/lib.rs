mod transform;
mod types;

pub use crate::{
    transform::{propagate_global_transform, Children, GlobalTransform, Parent, Root, Transform},
    types::*,
};

use acro_ecs::{schedule::Stage, Application, Plugin};
use acro_scripting::ScriptingRuntime;
use transform::register_components;

pub struct MathPlugin;

impl Plugin for MathPlugin {
    fn build(&mut self, app: &mut Application) {
        {
            let mut world = app.world();
            register_components(&mut world);

            let mut runtime = world.resources().get_mut::<ScriptingRuntime>();
            runtime
                .register_component::<Transform>(&world, "Transform")
                .expect("failed to register Transform component");
        }

        app.add_system(Stage::PostUpdate, [], propagate_global_transform);
    }
}
