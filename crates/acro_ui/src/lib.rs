mod box_renderer;
mod context;
mod document;
mod element;
mod panel;
mod rect;
mod rendering;
mod text;

use acro_ecs::{Application, Plugin, Stage};
use acro_render::RendererHandle;
use acro_scene::ComponentLoaders;
use acro_scripting::ScriptingRuntime;
use context::UiContext;
use document::{render_ui, update_screen_ui_rect, ScreenUi};
use text::{draw_text, init_text, Text};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&mut self, app: &mut Application) {
        let ui_context = UiContext::default();

        app.init_component::<Text>()
            .init_component::<ScreenUi>()
            .insert_resource(ui_context)
            .with_resource::<ComponentLoaders>(|loaders| {
                loaders.register("Text", |world, entity, value| {
                    Ok(world.insert(entity, serde_yml::from_value::<Text>(value)?))
                });

                loaders.register("ScreenUi", |world, entity, _value| {
                    // TODO: actually load the component's value
                    Ok(world.insert(entity, ScreenUi::new()))
                });
            })
            .with_resource::<ScriptingRuntime>(|mut runtime| {
                runtime.register_component::<Text>("Text");
            })
            .add_system(Stage::PreRender, [], update_screen_ui_rect)
            .add_system(Stage::PreRender, [], init_text)
            .add_system(Stage::Render, [], draw_text)
            .add_system(Stage::Render, [], render_ui);
    }
}
