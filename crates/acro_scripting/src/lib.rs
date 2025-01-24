mod behavior;
mod events;
mod platform;
mod runtime;
mod source_file;

use std::any::Any;

pub use crate::{
    behavior::Behavior, events::*, runtime::ScriptingRuntime, source_file::SourceFile,
};

#[cfg(not(target_arch = "wasm32"))]
pub use crate::platform::deno_ops;

use acro_assets::{load_queued_assets, Assets};
use acro_ecs::{systems::SystemId, Application, Plugin, Stage, SystemSchedulingRequirement};
use runtime::{flush_events, init_behavior, late_init_scripting_runtime, update_behaviors};

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&mut self, app: &mut Application) {
        let world_handle = app.get_world_handle();
        app.init_component::<Behavior>()
            .insert_resource(EventListenerStore::default())
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
                late_init_scripting_runtime,
            )
            .add_system(Stage::Update, [], update_behaviors)
            .add_system(
                Stage::Update,
                [SystemSchedulingRequirement::RunAfter(SystemId::Native(
                    update_behaviors.type_id(),
                ))],
                flush_events,
            );
    }
}
