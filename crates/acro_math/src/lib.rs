mod transform;
mod types;

pub use crate::{
    transform::{propagate_global_transform, Children, GlobalTransform, Parent, Root, Transform},
    types::*,
};

use acro_ecs::{schedule::Stage, Application, Plugin};
use acro_scripting::ScriptingRuntime;
use tracing::info;
use transform::register_components;

pub struct MathPlugin {
    pub scripting: bool,
}

impl Default for MathPlugin {
    fn default() -> Self {
        Self { scripting: true }
    }
}

impl Plugin for MathPlugin {
    fn build(&mut self, app: &mut Application) {
        {
            let mut world = app.world();
            register_components(&mut world);

            if self.scripting {
                let mut runtime = world.resources().get_mut::<ScriptingRuntime>();

                runtime
                    .register_component::<Transform>(&world, "Transform")
                    .expect("failed to register Transform component");

                runtime.add_op(op_get_property_vec2());
                runtime.add_op(op_get_property_vec3());
                runtime.add_op(op_get_property_vec4());

                runtime.add_op(op_set_property_vec2());
                runtime.add_op(op_set_property_vec3());
                runtime.add_op(op_set_property_vec4());
            }
        }

        app.add_system(Stage::PostUpdate, [], propagate_global_transform);
    }
}
