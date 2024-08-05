use acro_ecs::{Changed, Query, Res, ResMut, SystemRunContext};
use acro_math::Vec2;
use acro_reflect::Reflect;
use acro_render::{FrameState, RendererHandle};
use glyphon::{Attrs, Color, Family, Resolution, Shaping, TextArea, TextBounds};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::context::{UiContext, UiContextInner};

#[derive(Debug, Serialize, Deserialize, Reflect)]
pub struct Text {
    pub content: String,
    pub size: Vec2,
    pub font_size: f32,
    pub line_height: f32,
    #[reflect(skip)]
    #[serde(skip)]
    data: Option<TextData>,
}

#[derive(Debug)]
struct TextData {
    pub(crate) text_buffer: glyphon::Buffer,
}

pub fn init_text(
    ctx: SystemRunContext,
    query: Query<&mut Text, Changed<Text>>,
    renderer: Res<RendererHandle>,
    mut ui_context: ResMut<UiContext>,
) -> eyre::Result<()> {
    ui_context.ready(&renderer);

    for mut text in query.over(&ctx) {
        let mut text_buffer = glyphon::Buffer::new(
            &mut ui_context.font_system,
            glyphon::Metrics::new(text.font_size, text.line_height),
        );

        text_buffer.set_size(
            &mut ui_context.font_system,
            Some(text.size.x),
            Some(text.size.y),
        );
        text_buffer.set_text(
            &mut ui_context.font_system,
            &text.content,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut ui_context.font_system, false);

        text.data = Some(TextData { text_buffer });
    }

    Ok(())
}

pub fn draw_text(
    ctx: SystemRunContext,
    query: Query<&Text>,
    renderer: Res<RendererHandle>,
    mut ui_context: ResMut<UiContext>,
) -> eyre::Result<()> {
    let frame_state = renderer.frame_state();
    let mut encoder = frame_state.encoder.borrow_mut();

    // let font_system = &mut ui_context.font_system;
    let UiContextInner {
        font_system,
        swash_cache,
        ref mut viewport,
        atlas,
        ref mut text_renderer,
    } = &mut *ui_context.inner.as_mut().unwrap();

    for text in query.over(&ctx) {
        viewport.update(
            &renderer.queue,
            Resolution {
                width: renderer.config.borrow().width,
                height: renderer.config.borrow().height,
            },
        );

        text_renderer
            .prepare(
                &renderer.device,
                &renderer.queue,
                font_system,
                atlas,
                viewport,
                [TextArea {
                    buffer: &text.data.as_ref().unwrap().text_buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: Color::rgb(255, 255, 255),
                }],
                swash_cache,
            )
            .unwrap();

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
            text_renderer.render(&atlas, &viewport, &mut pass).unwrap();
        }
    }

    Ok(())
}
