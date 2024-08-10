use acro_ecs::{EntityId, Query, Res, SystemRunContext};
use acro_math::{Children, Parent, Vec2};
use acro_render::RendererHandle;
use tracing::info;

use crate::{
    positioning_options::{Dim, PositioningOptions},
    rect::{Rect, RectQueries},
};

pub struct ScreenUi;

pub fn update_screen_ui_rect(
    ctx: SystemRunContext,
    screen_ui_query: Query<(EntityId, &mut ScreenUi, &Rect)>,
    children_query: Query<&Children>,
    parent_query: Query<&Parent>,
    rect_query: Query<&Rect>,
    renderer: Res<RendererHandle>,
) {
    let renderer_size = renderer.size.borrow();
    for (entity_id, _screen_ui, rect) in screen_ui_query.over(&ctx) {
        {
            let mut document_rect = rect.inner_mut();
            document_rect.options = PositioningOptions {
                width: Dim::Px(renderer_size.width as f32),
                height: Dim::Px(renderer_size.height as f32),
                min_width: Some(Dim::Px(renderer_size.width as f32)),
                min_height: Some(Dim::Px(renderer_size.height as f32)),
                flex: document_rect.options.flex,
                margin: document_rect.options.margin,
                padding: document_rect.options.padding,
            };
            document_rect.size = Vec2::new(renderer_size.width as f32, renderer_size.height as f32);
        }

        rect.recalculate(
            entity_id,
            &RectQueries {
                ctx: &ctx,
                children_query: &children_query,
                parent_query: &parent_query,
                rect_query: &rect_query,
            },
        );
    }
}
