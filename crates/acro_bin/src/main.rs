use std::any::Any;

use acro_assets::{Assets, AssetsPlugin};
use acro_ecs::{Application, Plugin, Query, Res, Stage, SystemRunContext, With};
use acro_log::LogPlugin;
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Root, Transform};
use acro_physics::PhysicsPlugin;
use acro_render::{Mesh, RenderPlugin, WindowState};
use acro_scene::{SceneManager, ScenePlugin};
use acro_scripting::{Behavior, ScriptingPlugin, SourceFile};
use acro_ui::UiPlugin;
use tracing::info;

fn update(
    ctx: SystemRunContext,
    query: Query<&mut Transform, With<Mesh>>,
    window: Res<WindowState>,
) {
    // let mut transform = query.single(&ctx);
    // transform.position.x = window.mouse_position.x / 100.0;
    // transform.position.y = window.mouse_position.y / 100.0;
}

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&mut self, app: &mut Application) {
        let world = app.world();

        world
            .resource_mut::<SceneManager>()
            .queue("examples/simple/main.scene");
    }
}

fn main() {
    Application::new()
        .add_plugin(LogPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(ScriptingPlugin)
        .add_plugin(MathPlugin::default())
        .add_plugin(ScenePlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(TestPlugin)
        .run();
}
