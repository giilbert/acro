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

impl UiElement for Panel {
    fn create(mut parent_rect: Rect, positioning_options: PositioningOptions) -> Self
    where
        Self: Sized,
    {
        Panel {
            rect: parent_rect.new_child(positioning_options),
            parent_rect,
            children: Vec::new(),
        }
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
