use acro_ecs::{Query, Res, ResMut, SystemRunContext};
use acro_math::{Children, Vec2};
use acro_render::RendererHandle;

use crate::{
    context::UiContext,
    element::UiElement,
    rect::{Dim, PositioningOptions, Rect, RootOptions},
    rendering::UiRenderContext,
};

pub struct ScreenUi {
    ctx: UiContext,
}

impl ScreenUi {
    pub fn new(ctx: UiContext) -> Self {
        Self { ctx }
    }
}

pub fn update_screen_ui_rect(
    ctx: SystemRunContext,
    screen_ui_query: Query<(&mut ScreenUi, &Rect)>,
    children_query: Query<(&Rect, &Children)>,
    renderer: Res<RendererHandle>,
) {
    let renderer_size = renderer.size.borrow();
    for (screen_ui, rect) in screen_ui_query.over(&ctx) {
        {
            let mut document_rect = rect.inner_mut();
            document_rect.options = PositioningOptions {
                width: Dim::Px(renderer_size.width as f32),
                height: Dim::Px(renderer_size.height as f32),
                flex: document_rect.options.flex,
                margin: document_rect.options.margin,
                padding: document_rect.options.padding,
            };
            document_rect.size = Vec2::new(renderer_size.width as f32, renderer_size.height as f32);
        }
        rect.recalculate();
        // println!("{}", screen_ui.document.rect.get_tree_string());
    }
}

pub fn render_ui(
    ctx: SystemRunContext,
    screen_ui_query: Query<&ScreenUi>,
    ui_context: ResMut<UiContext>,
    renderer: Res<RendererHandle>,
) -> eyre::Result<()> {
    for screen_ui in screen_ui_query.over(&ctx) {
        let mut render_ctx = UiRenderContext {
            renderer: renderer.clone(),
        };
        screen_ui.render(&mut render_ctx);
    }

    ui_context.inner_mut().box_renderer.finish(&renderer)
}

pub fn update_rect(
    ctx: SystemRunContext,
    screen_ui_query: Query<&ScreenUi>,
    children_query: Query<(&Rect, &Children)>,
) {
}
