use acro_ecs::{Application, Plugin};
use acro_render::RenderPlugin;

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&mut self, app: &mut Application) {}
}

fn main() {
    Application::new()
        .add_plugin(TestPlugin)
        .add_plugin(RenderPlugin)
        .run();
}
