use std::cell::RefCell;

use acro_ecs::{Changed, Query, Res, ResMut, SystemRunContext};
use acro_math::Vec2;
use acro_reflect::Reflect;
use acro_render::{FrameState, RendererHandle};
use glyphon::{
    cosmic_text::CacheKeyFlags, Attrs, Family, Resolution, Shaping, Style, TextArea, TextBounds,
    Weight,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    context::{UiContext, UiContextInner},
    rect::Rect,
};

#[derive(Reflect, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
    pub font_size: f32,
    pub line_height: f32,

    // TODO: impl reflect
    // pub color: Color,
    #[serde(default = "get_default_font_weight")]
    pub weight: u16,
    #[serde(default = "get_default_italic")]
    pub italic: bool,

    #[reflect(skip)]
    #[serde(skip)]
    pub(crate) data: Option<TextData>,
}

fn get_default_font_weight() -> u16 {
    400
}

fn get_default_italic() -> bool {
    false
}

impl Text {
    pub fn create_attrs(&self) -> Attrs {
        Attrs {
            color_opt: None,
            family: Family::SansSerif,
            stretch: glyphon::Stretch::Normal,
            style: if self.italic {
                Style::Italic
            } else {
                Style::Normal
            },
            weight: Weight(self.weight),
            metadata: 0,
            cache_key_flags: CacheKeyFlags::empty(),
            metrics_opt: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TextData {
    pub(crate) text_buffer: RefCell<Box<glyphon::Buffer>>,
}

pub fn init_text(
    ctx: SystemRunContext,
    text_query: Query<(&Rect, &mut Text), Changed<Text>>,
    ui_context: ResMut<UiContext>,
) -> eyre::Result<()> {
    let UiContextInner {
        ref mut font_system,
        ..
    } = &mut *ui_context.inner_mut();

    for (text_rect, mut text) in text_query.over(&ctx) {
        let text_rect = text_rect.inner();

        let mut text_buffer = glyphon::Buffer::new(
            font_system,
            glyphon::Metrics::new(text.font_size, text.line_height),
        );

        text_buffer.set_size(font_system, Some(text_rect.size.x), Some(text_rect.size.y));
        text_buffer.set_text(
            font_system,
            &text.content,
            text.create_attrs(),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(font_system, false);

        let text_data = TextData {
            text_buffer: RefCell::new(Box::new(text_buffer)),
        };

        text.data = Some(text_data);
    }

    Ok(())
}

pub fn render_text(
    ctx: SystemRunContext,
    text_query: Query<(&Rect, &Text)>,
    ui_context: ResMut<UiContext>,
    renderer: Res<RendererHandle>,
) -> eyre::Result<()> {
    let UiContextInner {
        ref mut font_system,
        swash_cache,
        ref mut viewport,
        atlas,
        ref mut text_renderer,
        ..
    } = &mut *ui_context.inner_mut();

    let frame_state = renderer.frame_state();
    let mut encoder = frame_state.encoder.borrow_mut();

    viewport.update(
        &renderer.queue,
        Resolution {
            width: renderer.config.borrow().width,
            height: renderer.config.borrow().height,
        },
    );

    let text_areas = text_query.over(&ctx).map(|(rect, text)| {
        let rect = rect.inner();
        let data = text
            .data
            .as_ref()
            .expect("text data not initialized")
            .text_buffer
            .borrow();

        TextArea {
            // TODO: fork glyphon to use a better way of passing the buffer
            buffer: unsafe { std::mem::transmute(data.as_ref()) },
            left: rect.offset.x,
            top: rect.offset.y,
            scale: 1.0,
            bounds: TextBounds {
                left: rect.offset.x as i32,
                top: rect.offset.y as i32,
                right: (rect.offset.x + rect.size.x) as i32,
                bottom: (rect.offset.y + rect.size.y) as i32,
            },
            default_color: glyphon::Color::rgb(255, 255, 255),
        }
    });

    text_renderer.prepare(
        &renderer.device,
        &renderer.queue,
        font_system,
        atlas,
        viewport,
        text_areas,
        swash_cache,
    )?;

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame_state.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        text_renderer.render(&atlas, &viewport, &mut pass)?;
    }

    Ok(())
}
