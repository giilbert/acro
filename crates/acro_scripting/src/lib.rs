mod runtime;

use acro_ecs::{Application, Plugin};
use runtime::ScriptingRuntime;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&mut self, app: &mut Application) {
        let mut world = app.world();
        world.insert_resource(ScriptingRuntime::new(app.world_handle()));
    }
}
