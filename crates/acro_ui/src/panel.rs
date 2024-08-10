use acro_render::Color;

use crate::{
    box_renderer::BoxInstance,
    context::UiContext,
    element::UiElement,
    rect::{PositioningOptions, Rect},
    rendering::UiRenderContext,
};

pub struct Panel {
    ctx: UiContext,
    color: Color,

    rect: Rect,
    parent_rect: Rect,

    children: Vec<Box<dyn UiElement>>,
}

impl Panel {
    pub fn new(
        ctx: UiContext,
        parent_rect: Rect,
        options: PositioningOptions,
        color: Color,
    ) -> Self {
        // let rect = parent_rect.new_child(options);
        todo!();

        // Self {
        //     ctx,
        //     color,
        //     rect,
        //     parent_rect,
        //     children: Vec::new(),
        // }
    }
}

impl UiElement for Panel {
    fn get_ctx(&self) -> &crate::context::UiContext {
        &self.ctx
    }

    fn get_rect(&self) -> &Rect {
        &self.rect
    }

    fn add_child_boxed(&mut self, child: Box<dyn UiElement>) {
        self.children.push(child);
        self.rect.recalculate();
    }

    fn get_child(&self, index: usize) -> Option<&Box<dyn UiElement>> {
        self.children.get(index)
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut Box<dyn UiElement>> {
        self.children.get_mut(index)
    }

    fn render(&self, ctx: &mut UiRenderContext) {
        let rect = self.rect.inner();
        self.ctx.inner_mut().box_renderer.draw(BoxInstance {
            offset: rect.offset,
            size: rect.size,
            color: self.color.to_srgba(),
        });
        // TODO: add z indexing so no hacky sorting is needed
        self.ctx
            .inner_mut()
            .box_renderer
            .finish(&ctx.renderer)
            .expect("Failed to render box");

        self.children.iter().for_each(|child| child.render(ctx));
    }
}
