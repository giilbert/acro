use acro_render::RendererHandle;

use crate::{
    rect::{PositioningOptions, Rect},
    rendering::UiRenderContext,
};

pub trait UiElement {
    fn create(parent_rect: Rect, positioning_options: PositioningOptions) -> Self
    where
        Self: Sized;

    fn add_child_boxed(&mut self, child: Box<dyn UiElement>);

    fn get_child(&self, index: usize) -> Option<&Box<dyn UiElement>>;
    fn get_child_mut(&mut self, index: usize) -> Option<&mut Box<dyn UiElement>>;

    fn add_child(&mut self, child: impl UiElement + 'static)
    where
        Self: Sized,
    {
        self.add_child_boxed(Box::new(child));
    }

    fn render(&self, ctx: &UiRenderContext);
}
