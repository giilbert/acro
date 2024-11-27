use std::any::Any;

use acro_assets::{Assets, AssetsPlugin};
use acro_ecs::{Application, Plugin, Query, Res, Stage, SystemRunContext, With};
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Root, Transform};
use acro_physics::PhysicsPlugin;
use acro_render::{Mesh, RenderPlugin, WindowState};
use acro_scene::{SceneManager, ScenePlugin};
use acro_scripting::{Behavior, ScriptingPlugin, SourceFile};
use acro_ui::UiPlugin;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt as _, EnvFilter};

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
    let subscriber = tracing_subscriber::FmtSubscriber::new().with(EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
    acro_build::web::get_esbuild_binary_or_download().unwrap();
    acro_build::web::build_javascript_bundle("examples/simple").unwrap();

    Application::new()
        .add_plugin(AssetsPlugin)
        .add_plugin(ScriptingPlugin)
        .add_plugin(MathPlugin::default())
        .add_plugin(ScenePlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(TestPlugin);
    // .run();
}
