mod box_renderer;
mod context;
mod document;
mod element;
mod panel;
mod rect;
mod rendering;
mod text;

use acro_ecs::{Application, Plugin, Res, ResMut, Stage, SystemRunContext};
use acro_render::RendererHandle;
use acro_scene::ComponentLoaders;
use acro_scripting::ScriptingRuntime;
use context::UiContext;
use document::{render_ui, update_screen_ui_rect, ScreenUi};
use text::Text;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&mut self, app: &mut Application) {
        let ui_context = UiContext::default();

        app.init_component::<Text>()
            .init_component::<ScreenUi>()
            .insert_resource(ui_context)
            .with_resource::<ComponentLoaders>(|loaders| {
                loaders.register("ScreenUi", |world, entity, _value| {
                    let ui_context = world.resource::<UiContext>().clone();
                    // TODO: actually load the component's value
                    Ok(world.insert(entity, ScreenUi::new(ui_context)))
                });
            })
            .with_resource::<ScriptingRuntime>(|mut runtime| {
                runtime.register_component::<Text>("Text");
            })
            .add_system(Stage::PreRender, [], update_screen_ui_rect)
            .add_system(
                Stage::PreRender,
                [],
                |_: SystemRunContext,
                 renderer: Res<RendererHandle>,
                 mut ui_context: ResMut<UiContext>| {
                    ui_context.ready(&renderer);
                },
            )
            // .add_system(Stage::Render, [], draw_text)
            .add_system(Stage::Render, [], render_ui);
    }
}
