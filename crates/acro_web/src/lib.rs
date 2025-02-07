mod panic_hook;

use acro_assets::AssetsPlugin;
use acro_ecs::{Application, Plugin};
use acro_math::MathPlugin;
use acro_physics::PhysicsPlugin;
use acro_render::RenderPlugin;
use acro_scene::{SceneManager, ScenePlugin};
use acro_scripting::ScriptingPlugin;
use acro_ui::UiPlugin;
use wasm_bindgen::prelude::*;

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&mut self, app: &mut Application) {
        let world = app.world();

        world.resource_mut::<SceneManager>().queue("main.scene");
    }
}

#[wasm_bindgen]
pub fn init() {
    std::panic::set_hook(Box::new(panic_hook::panic_hook));
    tracing_wasm::set_as_global_default();

    tracing::info!("hello world from wasm!");
}

#[wasm_bindgen]
pub fn run() {
    tracing::info!("starting application..");

    acro_ecs::Application::new()
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
