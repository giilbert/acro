use acro_ecs::{Query, Res, ResMut, SystemRunContext};
use acro_math::Vec2;
use acro_render::{Color, RendererHandle, Srgba};

use crate::{
    context::UiContext,
    element::UiElement,
    panel::Panel,
    rect::{Dim, Dir, FlexOptions, PositioningOptions, Rect, RootOptions},
    rendering::UiRenderContext,
    text::Text,
};

pub struct UiDocument {
    ctx: UiContext,
    rect: Rect,
    children: Vec<Box<dyn UiElement>>,
}

impl UiDocument {
    pub fn new(ctx: UiContext) -> Self {
        UiDocument {
            ctx,
            rect: Rect::new_root(RootOptions {
                size: Vec2::new(1.0, 1.0),
                flex: FlexOptions {
                    gap: Dim::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            children: Vec::new(),
        }
    }
}

impl UiElement for UiDocument {
    fn get_ctx(&self) -> &UiContext {
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
        self.children.iter().for_each(|child| child.render(ctx));
    }
}

pub struct ScreenUi {
    pub document: UiDocument,
}

impl ScreenUi {
    pub fn new(ctx: UiContext) -> Self {
        let document = UiDocument::new(ctx).add(|ctx, p| {
            Panel::new(
                ctx,
                p,
                PositioningOptions {
                    margin: Dir::all(Dim::Px(20.0)),
                    padding: Dir::all(Dim::Px(10.0)),
                    width: Dim::Percent(1.0),
                    height: Dim::Px(200.0),
                    ..Default::default()
                },
                Color::Srgba(Srgba::new(0.03, 0.03, 0.03, 0.5)),
            )
            .add(|ctx, p| Text::new(ctx, p))
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
                flex: document_rect.options.flex,
                margin: document_rect.options.margin,
                padding: document_rect.options.padding,
            };
            document_rect.size = Vec2::new(renderer_size.width as f32, renderer_size.height as f32);
        }
        screen_ui.document.rect.recalculate();
        // println!("{}", screen_ui.document.rect.get_tree_string());
    }
}

pub fn render_ui(
    ctx: SystemRunContext,
    screen_ui_query: Query<&ScreenUi>,
    ui_context: ResMut<UiContext>,
    renderer: Res<RendererHandle>,
) -> eyre::Result<()> {
    for screen_ui in screen_ui_query.over(&ctx) {
        let mut render_ctx = UiRenderContext {
            renderer: renderer.clone(),
        };
        screen_ui.document.render(&mut render_ctx);
    }

    ui_context.inner_mut().box_renderer.finish(&renderer)
}
