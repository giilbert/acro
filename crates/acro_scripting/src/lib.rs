mod behavior;
mod reflect;
mod runtime;
mod source_file;

use std::any::Any;

pub use crate::{
    behavior::Behavior, reflect::get_dyn_reflect, runtime::ScriptingRuntime,
    source_file::SourceFile,
};

use acro_assets::{load_queued_assets, Assets};
use acro_ecs::{systems::SystemId, Application, Plugin, Stage, SystemSchedulingRequirement};
use runtime::{init_behavior, init_scripting_runtime, update_behaviors};

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&mut self, app: &mut Application) {
        app.world().init_component::<Behavior>();

        app.add_system(Stage::PreUpdate, [], init_behavior);
        app.add_system(
            Stage::PreUpdate,
            [SystemSchedulingRequirement::RunBefore(SystemId::Native(
                load_queued_assets.type_id(),
            ))],
            init_scripting_runtime,
        );
        app.add_system(Stage::Update, [], update_behaviors);

        let mut world = app.world();
        world.insert_resource(ScriptingRuntime::new(app.get_world_handle()));

        let mut assets = world.resources().get_mut::<Assets>();
        assets.register_loader::<SourceFile>();
    }
}
