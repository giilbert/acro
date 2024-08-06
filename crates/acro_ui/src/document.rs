use acro_ecs::{Query, Res, SystemRunContext};
use acro_math::Vec2;
use acro_render::RendererHandle;

use crate::{
    element::UiElement,
    rect::{Rect, RootOptions},
    rendering::UiRenderContext,
};

#[derive(Default)]
pub struct UiDocument {
    rect: Rect,
    children: Vec<Box<dyn UiElement>>,
}

impl UiElement for UiDocument {
    fn create(
        _parent_rect: crate::rect::Rect,
        _positioning_options: crate::rect::PositioningOptions,
    ) -> Self
    where
        Self: Sized,
    {
        unimplemented!("construct UiDocument manually")
    }

    fn add_child_boxed(&mut self, child: Box<dyn UiElement>) {
        self.children.push(child);
    }

    fn get_child(&self, index: usize) -> Option<&Box<dyn UiElement>> {
        self.children.get(index)
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut Box<dyn UiElement>> {
        self.children.get_mut(index)
    }

    fn render(&self, ctx: &UiRenderContext) {
        self.children.iter().for_each(|child| child.render(ctx));
    }
}

pub struct ScreenUi {
    pub document: UiDocument,
}

impl ScreenUi {
    pub fn new() -> Self {
        Self {
            document: UiDocument::default(),
        }
    }
}

pub fn update_screen_ui_rect(
    ctx: SystemRunContext,
    screen_ui_query: Query<&mut ScreenUi>,
    renderer: Res<RendererHandle>,
) {
    let renderer_size = renderer.size.borrow();
    for screen_ui in screen_ui_query.over(&ctx) {
        let mut document_rect = screen_ui.document.rect.inner_mut();
        document_rect.size = Vec2::new(renderer_size.width as f32, renderer_size.height as f32);
        document_rect.recalculate();
    }
}

pub fn render_ui(
    ctx: SystemRunContext,
    screen_ui_query: Query<&ScreenUi>,
    renderer: Res<RendererHandle>,
) {
    for screen_ui in screen_ui_query.over(&ctx) {
        let render_ctx = UiRenderContext {};
        screen_ui.document.render(&render_ctx);
    }
}
