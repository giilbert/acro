use crate::{
    element::UiElement,
    rect::{PositioningOptions, Rect},
    rendering::UiRenderContext,
};

pub struct Panel {
    rect: Rect,
    parent_rect: Rect,

    children: Vec<Box<dyn UiElement>>,
}

impl Panel {
    pub fn new(parent_rect: Rect, options: PositioningOptions) -> Self {
        let rect = parent_rect.new_child(options);

        Self {
            rect,
            parent_rect,
            children: Vec::new(),
        }
    }
}

impl UiElement for Panel {
    fn get_rect(&self) -> &Rect {
        &self.rect
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
        // TODO: batch render panels
    }
}
