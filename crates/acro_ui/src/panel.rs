use acro_ecs::{Query, Res, ResMut, SystemRunContext};
use acro_render::{Color, RendererHandle};
use serde::{Deserialize, Serialize};

use crate::{box_renderer::BoxInstance, context::UiContext, rect::Rect};

#[derive(Serialize, Deserialize)]
pub struct Panel {
    pub(crate) color: Color,
}

impl Panel {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

pub fn render_panel(
    ctx: SystemRunContext,
    panel_query: Query<(&Rect, &Panel)>,
    ui_context: ResMut<UiContext>,
    renderer: Res<RendererHandle>,
) -> eyre::Result<()> {
    let box_renderer = &mut ui_context.inner_mut().box_renderer;

    for (panel_rect, panel) in panel_query.over(&ctx) {
        let panel_rect = panel_rect.inner();

        box_renderer.draw(BoxInstance {
            size: panel_rect.size,
            offset: panel_rect.offset,
            color: panel.color.to_srgba(),
        });
    }

    box_renderer.finish(&renderer)
}
