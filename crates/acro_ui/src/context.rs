use std::ops::{Deref, DerefMut};

use acro_render::RendererHandle;
use glyphon::{
    Attrs, Buffer, Cache, Family, FontSystem, Metrics, Shaping, SwashCache, TextAtlas, TextRenderer,
};
use wgpu::MultisampleState;

#[derive(Default)]
pub struct UiContext {
    pub(crate) inner: Option<UiContextInner>,
}

pub struct UiContextInner {
    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,
    pub(crate) viewport: glyphon::Viewport,
    pub(crate) atlas: glyphon::TextAtlas,
    pub(crate) text_renderer: glyphon::TextRenderer,
}

impl UiContext {
    pub fn ready(&mut self, renderer: &RendererHandle) {
        if self.inner.is_none() {
            let font_system = FontSystem::new();
            let swash_cache = SwashCache::new();
            let cache = Cache::new(&renderer.device);
            let viewport = glyphon::Viewport::new(&renderer.device, &cache);
            let mut atlas = TextAtlas::new(
                &renderer.device,
                &renderer.queue,
                &cache,
                renderer.config.borrow().format,
            );
            let text_renderer = TextRenderer::new(
                &mut atlas,
                &renderer.device,
                MultisampleState::default(),
                None,
            );

            self.inner = Some(UiContextInner {
                font_system,
                swash_cache,
                viewport,
                atlas,
                text_renderer,
            });
        }
    }
}

impl Deref for UiContext {
    type Target = UiContextInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("UiContext not ready")
    }
}

impl DerefMut for UiContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("UiContext not ready")
    }
}
