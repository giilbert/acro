mod box_renderer;
mod context;
mod document;
mod panel;
mod rect;
mod rendering;
mod text;

use acro_ecs::{Application, Plugin, Res, ResMut, Stage, SystemRunContext};
use acro_math::TransformBoundary;
use acro_render::RendererHandle;
use acro_scene::ComponentLoaders;
use acro_scripting::ScriptingRuntime;
use context::UiContext;
use document::{update_screen_ui_rect, ScreenUi};
use panel::{render_panel, Panel};
use rect::{PositioningOptions, Rect, RootOptions};
use text::{init_text, render_text, Text};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&mut self, app: &mut Application) {
        let ui_context = UiContext::default();

        app.init_component::<Rect>()
            .init_component::<Text>()
            .init_component::<ScreenUi>()
            .init_component::<Panel>()
            .insert_resource(ui_context)
            .with_resource::<ComponentLoaders>(|loaders| {
                loaders.register("ScreenUi", |world, entity, _value| {
                    world.insert(entity, TransformBoundary);
                    world.insert(entity, Rect::new_root(RootOptions::default()));
                    world.insert(entity, ScreenUi);
                    Ok(())
                });

                loaders.register("Rect", |world, entity, value| {
                    let position = serde_yml::from_value::<PositioningOptions>(value)?;
                    Ok(world.insert(entity, Rect::new(position)))
                });

                loaders.register("Text", |world, entity, value| {
                    Ok(world.insert(entity, serde_yml::from_value::<Text>(value)?))
                });

                loaders.register("Panel", |world, entity, value| {
                    let panel = serde_yml::from_value::<Panel>(value)?;
                    Ok(world.insert(entity, panel))
                });
            })
            .with_resource::<ScriptingRuntime>(|mut runtime| {
                runtime.register_component::<Text>("Text");
            })
            .add_system(Stage::PreRender, [], update_screen_ui_rect)
            .add_system(Stage::PreRender, [], init_text)
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
            .add_system(Stage::Render, [], render_panel)
            .add_system(Stage::Render, [], render_text);
    }
}
