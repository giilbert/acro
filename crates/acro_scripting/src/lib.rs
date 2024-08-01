mod behavior;
mod runtime;
mod source_file;

use std::any::Any;

pub use crate::{behavior::Behavior, runtime::ScriptingRuntime, source_file::SourceFile};

use acro_assets::{load_queued_assets, Assets};
use acro_ecs::{systems::SystemId, Application, Plugin, Stage, SystemSchedulingRequirement};
use runtime::init_behavior;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&mut self, app: &mut Application) {
        app.world().init_component::<Behavior>();

        app.add_system(
            Stage::PreUpdate,
            [SystemSchedulingRequirement::RunAfter(SystemId::Native(
                load_queued_assets.type_id(),
            ))],
            init_behavior,
        );

        let mut world = app.world();
        world.insert_resource(ScriptingRuntime::new(app.world_handle()));

        let mut assets = world.resources().get_mut::<Assets>();
        assets.register_loader::<SourceFile>();
    }
}
