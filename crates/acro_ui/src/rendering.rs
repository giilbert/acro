use acro_render::RendererHandle;

use crate::box_renderer::BoxRenderer;

pub struct UiRenderContext<'a> {
    pub(crate) renderer: RendererHandle,
    pub(crate) box_renderer: &'a BoxRenderer,
}
