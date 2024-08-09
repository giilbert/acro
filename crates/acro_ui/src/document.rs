use acro_ecs::{Query, Res, ResMut, SystemRunContext};
use acro_math::Vec2;
use acro_render::RendererHandle;

use crate::{
    context::UiContext,
    element::UiElement,
    panel::Panel,
    rect::{Dim, Dir, FlexOptions, PositioningOptions, Rect, RootOptions},
    rendering::UiRenderContext,
};

pub struct UiDocument {
    rect: Rect,
    children: Vec<Box<dyn UiElement>>,
}

impl UiDocument {
    pub fn new() -> Self {
        UiDocument {
            rect: Rect::new_root(RootOptions {
                size: Vec2::new(1.0, 1.0),
                ..Default::default()
            }),
            children: Vec::new(),
        }
    }
}

impl UiElement for UiDocument {
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
        self.children.iter().for_each(|child| child.render(ctx));
    }
}

pub struct ScreenUi {
    pub document: UiDocument,
}

impl ScreenUi {
    pub fn new() -> Self {
        let document = UiDocument::new().add(|p| {
            Panel::new(
                p,
                PositioningOptions {
                    margin: Dir::all(Dim::Px(20.0)),
                    width: Dim::Percent(1.0),
                    height: Dim::Px(200.0),
                    ..Default::default()
                },
            )
        });

        Self { document }
    }
}

pub fn update_screen_ui_rect(
    ctx: SystemRunContext,
    screen_ui_query: Query<&mut ScreenUi>,
    renderer: Res<RendererHandle>,
) {
    let renderer_size = renderer.size.borrow();
    for screen_ui in screen_ui_query.over(&ctx) {
        {
            let mut document_rect = screen_ui.document.rect.inner_mut();
            document_rect.options = PositioningOptions {
                width: Dim::Px(renderer_size.width as f32),
                height: Dim::Px(renderer_size.height as f32),
                ..Default::default()
            };
            document_rect.size = Vec2::new(renderer_size.width as f32, renderer_size.height as f32);
        }
        screen_ui.document.rect.recalculate();
    }
}

pub fn render_ui(
    ctx: SystemRunContext,
    screen_ui_query: Query<&ScreenUi>,
    mut ui_context: ResMut<UiContext>,
    renderer: Res<RendererHandle>,
) -> eyre::Result<()> {
    for screen_ui in screen_ui_query.over(&ctx) {
        let mut render_ctx = UiRenderContext {
            box_renderer: &mut ui_context.box_renderer,
            renderer: renderer.clone(),
        };
        screen_ui.document.render(&mut render_ctx);
    }

    ui_context.box_renderer.finish(&renderer)
}
