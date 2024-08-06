use crate::{element::UiElement, rect::Rect, rendering::UiRenderContext};

pub struct UiDocument {
    rect: Rect,
    children: Vec<Box<dyn UiElement>>,
}

impl UiElement for UiDocument {
    fn create(
        parent_rect: crate::rect::Rect,
        positioning_options: crate::rect::PositioningOptions,
    ) -> Self
    where
        Self: Sized,
    {
        todo!();
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
        todo!();
    }
}
