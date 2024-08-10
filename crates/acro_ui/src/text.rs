use std::cell::RefCell;

use acro_ecs::{Changed, Query, Res, ResMut, SystemRunContext};
use acro_math::Vec2;
use acro_reflect::Reflect;
use acro_render::{FrameState, RendererHandle};
use glyphon::{Attrs, Color, Family, Resolution, Shaping, TextArea, TextBounds};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    context::{UiContext, UiContextInner},
    element::UiElement,
    rect::{Dim, PositioningOptions, Rect},
    rendering::UiRenderContext,
};

#[derive(Reflect)]
pub struct Text {
    #[reflect(skip)]
    pub(crate) ctx: UiContext,
    #[reflect(skip)]
    pub(crate) rect: Rect,
    #[reflect(skip)]
    pub(crate) parent_rect: Rect,
    #[reflect(skip)]
    pub(crate) children: Vec<Box<dyn UiElement>>,
    #[reflect(skip)]
    pub(crate) data: TextData,

    options: TextOptions,
}

#[derive(Reflect)]
pub struct TextOptions {
    pub content: String,
    pub font_size: f32,
    pub line_height: f32,
}

#[derive(Debug)]
pub(crate) struct TextData {
    pub(crate) last_size: RefCell<Vec2>,
    pub(crate) text_buffer: RefCell<glyphon::Buffer>,
}

impl Text {
    pub fn new(ctx: UiContext, parent_rect: Rect) -> Self {
        todo!();
        // TODO: not hard code this
        // let options = TextOptions {
        //     content: "text content blah blah".to_string(),
        //     font_size: 20.0,
        //     line_height: 30.0,
        // };

        // let text_buffer = glyphon::Buffer::new_empty(glyphon::Metrics::new(
        //     options.font_size,
        //     options.line_height,
        // ));

        // let rect = parent_rect.new_child(PositioningOptions {
        //     width: Dim::Percent(1.0),
        //     height: Dim::Percent(1.0),
        //     ..Default::default()
        // });

        // Text {
        //     data: TextData {
        //         last_size: RefCell::new(Vec2::zeros()),
        //         text_buffer: RefCell::new(text_buffer),
        //     },
        //     ctx,
        //     children: Vec::new(),
        //     rect,
        //     parent_rect,

        //     options,
        // }
    }
}

// impl UiElement for Text {
//     fn get_ctx(&self) -> &UiContext {
//         &self.ctx
//     }

//     fn get_rect(&self) -> &Rect {
//         &self.rect
//     }

//     fn add_child_boxed(&mut self, child: Box<dyn UiElement>) {
//         self.children.push(child);
//         self.rect.recalculate();
//     }

//     fn get_child(&self, index: usize) -> Option<&Box<dyn UiElement>> {
//         self.children.get(index)
//     }

//     fn get_child_mut(&mut self, index: usize) -> Option<&mut Box<dyn UiElement>> {
//         self.children.get_mut(index)
//     }

//     fn render(&self, ctx: &mut UiRenderContext) {
//         let UiContextInner {
//             ref mut font_system,
//             swash_cache,
//             ref mut viewport,
//             atlas,
//             ref mut text_renderer,
//             ..
//         } = &mut *self.ctx.inner_mut();
//         let rect = self.rect.inner();

//         if *self.data.last_size.borrow() != rect.size {
//             *self.data.last_size.borrow_mut() = rect.size;

//             let mut text_buffer = glyphon::Buffer::new(
//                 font_system,
//                 glyphon::Metrics::new(self.options.font_size, self.options.line_height),
//             );

//             text_buffer.set_size(font_system, Some(rect.size.x), Some(rect.size.y));
//             text_buffer.set_text(
//                 font_system,
//                 &self.options.content,
//                 Attrs::new().family(Family::SansSerif),
//                 Shaping::Advanced,
//             );
//             text_buffer.shape_until_scroll(font_system, false);

//             self.data.text_buffer.replace(text_buffer);
//         }

//         let renderer = &ctx.renderer;
//         let frame_state = renderer.frame_state();
//         let mut encoder = frame_state.encoder.borrow_mut();

//         viewport.update(
//             &renderer.queue,
//             Resolution {
//                 width: renderer.config.borrow().width,
//                 height: renderer.config.borrow().height,
//             },
//         );

//         text_renderer
//             .prepare(
//                 &renderer.device,
//                 &renderer.queue,
//                 font_system,
//                 atlas,
//                 viewport,
//                 [TextArea {
//                     buffer: &self.data.text_buffer.borrow(),
//                     left: rect.offset.x,
//                     top: rect.offset.y,
//                     scale: 1.0,
//                     bounds: TextBounds {
//                         left: 0,
//                         top: 0,
//                         right: (rect.offset.x + rect.size.x) as i32,
//                         bottom: (rect.offset.y + rect.size.y) as i32,
//                     },
//                     default_color: Color::rgb(255, 255, 255),
//                 }],
//                 swash_cache,
//             )
//             .unwrap();

//         {
//             let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: None,
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &frame_state.view,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Load,
//                         store: wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: None,
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//             });
//             text_renderer.render(&atlas, &viewport, &mut pass).unwrap();
//             // info!("render text");
//         }
//     }
// }
