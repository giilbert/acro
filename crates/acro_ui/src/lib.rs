mod box_renderer;
mod button;
mod context;
mod panel;
mod positioning_options;
mod rect;
mod screen_ui;
mod text;
mod ui_element_state;

use std::any::Any;

use acro_ecs::{
    systems::SystemId, Application, Plugin, Res, ResMut, Stage, SystemRunContext,
    SystemSchedulingRequirement,
};
use acro_math::TransformBoundary;
use acro_render::RendererHandle;
use acro_scene::ComponentLoaders;
use acro_scripting::ScriptingRuntime;
use button::{poll_button_interaction, Button};
use context::UiContext;
use panel::{render_panel, Panel};
use positioning_options::{Dim, FlexOptions, PositioningOptions};
use rect::{Rect, RootOptions};
use screen_ui::{update_screen_ui_rect, ScreenUi};
use text::{init_text, render_text, Text};
use ui_element_state::{poll_ui_element_state, UiElementState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&mut self, app: &mut Application) {
        let ui_context = UiContext::default();

        app.init_component::<Rect>()
            .init_component::<UiElementState>()
            .init_component::<Text>()
            .init_component::<ScreenUi>()
            .init_component::<Panel>()
            .init_component::<Button>()
            .insert_resource(ui_context)
            .with_resource::<ComponentLoaders>(|loaders| {
                loaders.register("ScreenUi", |world, entity, value| {
                    world.insert(entity, TransformBoundary);
                    world.insert(entity, Rect::new_root(serde_yml::from_value(value)?));
                    world.insert(entity, ScreenUi);

                    Ok(())
                });

                loaders.register("Rect", |world, entity, value| {
                    let position = serde_yml::from_value::<PositioningOptions>(value)?;
                    world.insert(entity, UiElementState::default());
                    Ok(world.insert(entity, Rect::new(position)))
                });

                loaders.register("Text", |world, entity, value| {
                    Ok(world.insert(entity, serde_yml::from_value::<Text>(value)?))
                });

                loaders.register("Panel", |world, entity, value| {
                    let panel = serde_yml::from_value::<Panel>(value)?;
                    Ok(world.insert(entity, panel))
                });

                loaders.register("Button", |world, entity, _value| {
                    Ok(world.insert(entity, Button::default()))
                });
            })
            .with_resource::<ScriptingRuntime>(|mut runtime| {
                runtime.register_component::<Text>("Text");
                runtime.register_component::<Button>("Button");
            })
            .add_system(Stage::PreUpdate, [], poll_ui_element_state)
            .add_system(
                Stage::Update,
                [SystemSchedulingRequirement::RunAfter(SystemId::Native(
                    poll_button_interaction.type_id(),
                ))],
                poll_button_interaction,
            )
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
            .add_system(Stage::Render, [], render_panel)
            .add_system(Stage::PreRender, [], init_text)
            .add_system(Stage::Render, [], render_text);
    }
}
