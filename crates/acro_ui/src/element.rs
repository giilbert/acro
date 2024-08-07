use acro_render::RendererHandle;

use crate::{
    rect::{PositioningOptions, Rect},
    rendering::UiRenderContext,
};

pub trait UiElement {
    fn add_child_boxed(&mut self, child: Box<dyn UiElement>);

    fn get_rect(&self) -> &Rect;

    fn get_child(&self, index: usize) -> Option<&Box<dyn UiElement>>;
    fn get_child_mut(&mut self, index: usize) -> Option<&mut Box<dyn UiElement>>;

    fn add_child(&mut self, child: impl UiElement + 'static)
    where
        Self: Sized,
    {
        self.add_child_boxed(Box::new(child));
    }
    fn add(mut self, factory: impl UiElementFactory + 'static) -> Self
    where
        Self: Sized,
    {
        self.add_child_boxed(factory.create(self.get_rect().clone()));
        self
    }

    fn render(&self, ctx: &mut UiRenderContext);
}

pub trait UiElementFactory {
    fn create(self, parent_rect: Rect) -> Box<dyn UiElement>;
}

impl<T, F> UiElementFactory for F
where
    F: FnOnce(Rect) -> T,
    T: UiElement + 'static,
{
    fn create(self, parent_rect: Rect) -> Box<dyn UiElement> {
        Box::new(self(parent_rect))
    }
}
