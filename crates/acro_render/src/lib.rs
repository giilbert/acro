mod camera;
mod mesh;
mod shader;
mod state;
mod window;

use std::cell::RefCell;

pub use crate::{
    camera::{Camera, CameraType, MainCamera},
    mesh::{Mesh, Vertex},
    shader::Shaders,
};

use acro_ecs::{Application, Plugin, Res, Stage, SystemRunContext};
use mesh::{render_mesh_system, upload_mesh_system};
use state::{FrameState, RendererHandle};
use window::Window;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&mut self, app: &mut Application) {
        app.world().init_component::<Mesh>();
        app.world().init_component::<Camera>();
        app.world().init_component::<MainCamera>();

        let window = Window::new();
        app.set_runner(move |app| {
            window.run(app);
        });

        app.add_system(Stage::PreRender, [], start_render_system);
        app.add_system(Stage::PreRender, [], upload_mesh_system);
        app.add_system(Stage::Render, [], render_mesh_system);
        app.add_system(Stage::PostRender, [], end_render_system);
    }
}

fn start_render_system(_ctx: SystemRunContext, renderer: Res<RendererHandle>) {
    let output = renderer.surface.get_current_texture().unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let _clear_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.05,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    *renderer.frame_state.borrow_mut() = Some(FrameState {
        encoder: RefCell::new(encoder),
        view,
        output,
    });
}

fn end_render_system(_ctx: SystemRunContext, renderer: Res<RendererHandle>) {
    let frame_state = renderer.take_frame_state().expect("frame already ended");
    let encoder = frame_state.encoder.into_inner();

    let commands = encoder.finish();
    renderer.queue.submit(std::iter::once(commands));

    frame_state.output.present();
}
