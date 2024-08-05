mod behavior;
mod ops;
mod runtime;
mod source_file;

use std::any::Any;

pub use crate::{
    behavior::Behavior, ops::get_dyn_reflect, runtime::ScriptingRuntime, source_file::SourceFile,
};

use acro_assets::{load_queued_assets, Assets};
use acro_ecs::{systems::SystemId, Application, Plugin, Stage, SystemSchedulingRequirement};
use runtime::{init_behavior, init_scripting_runtime, update_behaviors};

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&mut self, app: &mut Application) {
        let world_handle = app.get_world_handle();
        app.init_component::<Behavior>()
            .insert_resource(ScriptingRuntime::new(world_handle))
            .with_resource::<Assets>(|mut assets| {
                assets.register_loader::<SourceFile>();
            })
            .add_system(Stage::PreUpdate, [], init_behavior)
            .add_system(
                Stage::PreUpdate,
                [SystemSchedulingRequirement::RunBefore(SystemId::Native(
                    load_queued_assets.type_id(),
                ))],
                init_scripting_runtime,
            )
            .add_system(Stage::Update, [], update_behaviors);
    }
}
