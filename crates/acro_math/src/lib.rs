mod ops;
mod transform;
mod tree;
mod types;

pub use crate::{
    transform::{
        propagate_global_transform, Children, GlobalTransform, Parent, Root, Transform,
        TransformBoundary,
    },
    types::*,
};

use acro_ecs::{schedule::Stage, Application, Plugin};
use acro_scripting::ScriptingRuntime;
use ops::op_get_entity_by_absolute_path;
use tree::TreeData;

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
        app.init_component::<Transform>()
            .init_component::<GlobalTransform>()
            .init_component::<Parent>()
            .init_component::<Children>()
            .init_component::<Root>()
            .init_component::<TransformBoundary>()
            .add_system(Stage::PostUpdate, [], propagate_global_transform);

        if self.scripting {
            app.with_resource::<ScriptingRuntime>(|mut runtime| {
                runtime.register_component::<Transform>("Transform");

                #[cfg(not(target_arch = "wasm32"))]
                {
                    runtime.native_add_op(op_get_property_vec2());
                    runtime.native_add_op(op_get_property_vec3());
                    runtime.native_add_op(op_get_property_vec4());

                    runtime.native_add_op(op_set_property_vec2());
                    runtime.native_add_op(op_set_property_vec3());
                    runtime.native_add_op(op_set_property_vec4());

                    runtime.native_add_op(op_get_entity_by_absolute_path());
                }
            });
        }

        let tree = TreeData::new(&*app.world());
        app.insert_resource(tree);
    }
}
