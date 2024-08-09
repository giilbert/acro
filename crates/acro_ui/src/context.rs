use std::{
    cell::{RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use acro_render::RendererHandle;
use glyphon::{
    Attrs, Buffer, Cache, Family, FontSystem, Metrics, Shaping, SwashCache, TextAtlas, TextRenderer,
};
use wgpu::MultisampleState;

use crate::box_renderer::BoxRenderer;

#[derive(Default, Clone)]
pub struct UiContext {
    pub(crate) inner: Rc<RefCell<Option<UiContextInner>>>,
}

pub struct UiContextInner {
    pub(crate) renderer: RendererHandle,
    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,
    pub(crate) viewport: glyphon::Viewport,
    pub(crate) atlas: glyphon::TextAtlas,
    pub(crate) text_renderer: glyphon::TextRenderer,
    pub(crate) box_renderer: BoxRenderer,
}

impl UiContext {
    pub fn ready(&mut self, renderer: &RendererHandle) {
        if self.inner.borrow().is_none() {
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

            *self.inner.borrow_mut() = Some(UiContextInner {
                renderer: renderer.clone(),
                font_system,
                swash_cache,
                viewport,
                atlas,
                text_renderer,
                box_renderer: BoxRenderer::new(&renderer),
            });
        }
    }

    pub fn inner_mut(&self) -> RefMut<UiContextInner> {
        RefMut::map(self.inner.borrow_mut(), |inner| {
            inner.as_mut().expect("UiContextInner not initialized")
        })
    }
}
