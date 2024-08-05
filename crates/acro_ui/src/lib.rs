mod context;
mod text;

use acro_ecs::{Application, Plugin, Stage};
use acro_render::RendererHandle;
use acro_scene::ComponentLoaders;
use context::UiContext;
use text::{draw_text, init_text, Text};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&mut self, app: &mut Application) {
        let ui_context = UiContext::default();

        app.init_component::<Text>()
            .insert_resource(ui_context)
            .with_resource::<ComponentLoaders>(|loaders| {
                loaders.register("Text", |world, entity, value| {
                    Ok(world.insert(entity, serde_yml::from_value::<Text>(value)?))
                });
            })
            .add_system(Stage::PreRender, [], init_text)
            .add_system(Stage::Render, [], draw_text);
    }
}
